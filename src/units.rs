//! Runtime units: quantities as (SI value, dimension exponents over m kg s A K mol cd), parsed from strings
//! like `100 km/h` or `4.2V*20Ah` with the unit table generated from pint (`data/units.toml`).
//! Offset temperatures only as a single literal (`60 degC` -> 333.15 K); anywhere else degC is a difference.

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::expr::{self, Expr, Op};

/// Shown instead of a NaN or infinite value (0**-1, 10**400/10**400, a unit factor of zero).
pub const NOT_FINITE: &str = "no finite answer (division by zero or overflow)";

pub type Dims = [i32; 7];
pub const DIMENSIONLESS: Dims = [0; 7];
const DIM_NAMES: [&str; 7] = ["m", "kg", "s", "A", "K", "mol", "cd"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub dims: Dims,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitError(pub String);

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UnitError {}

impl From<expr::ParseError> for UnitError {
    fn from(e: expr::ParseError) -> Self {
        Self(e.0)
    }
}

#[derive(Deserialize)]
struct UnitRow {
    name: String,
    dims: Dims,
    factor: f64,
    #[serde(default)]
    offset: f64,
}

#[derive(Deserialize)]
struct UnitFile {
    prefixes: HashMap<String, f64>,
    prefixable: Vec<String>,
    unit: Vec<UnitRow>,
}

struct Table {
    units: HashMap<String, (f64, Dims, f64)>,
    prefixes: HashMap<String, f64>,
    prefixable: Vec<String>,
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| {
        let f: UnitFile = toml::from_str(crate::data::UNITS).expect("data/units.toml");
        Table {
            units: f
                .unit
                .into_iter()
                .map(|u| (u.name, (u.factor, u.dims, u.offset)))
                .collect(),
            prefixes: f.prefixes,
            prefixable: f.prefixable,
        }
    })
}

/// (factor to SI, dims, offset) of a single unit name, trying the exact name before prefix + unit.
fn lookup(name: &str) -> Option<(f64, Dims, f64)> {
    let t = table();
    if let Some(u) = t.units.get(name) {
        return Some(*u);
    }
    for (p, k) in &t.prefixes {
        if let Some(rest) = name.strip_prefix(p.as_str()) {
            if t.prefixable.iter().any(|b| b == rest) {
                let (f, d, _) = t.units[rest];
                return Some((f * k, d, 0.0));
            }
        }
    }
    None
}

fn combine(a: Dims, b: Dims, sign: i32) -> Dims {
    std::array::from_fn(|i| a[i].saturating_add(b[i].saturating_mul(sign)))
}

fn eval(e: &Expr) -> Result<Quantity, UnitError> {
    Ok(match e {
        Expr::Num(n) => Quantity {
            value: *n,
            dims: DIMENSIONLESS,
        },
        Expr::Var(name) => {
            let (f, dims, _) =
                lookup(name).ok_or_else(|| UnitError(format!("unknown unit {name:?}")))?;
            Quantity { value: f, dims }
        }
        Expr::Neg(x) => {
            let q = eval(x)?;
            Quantity {
                value: -q.value,
                ..q
            }
        }
        Expr::Call(f, x) => {
            let q = eval(x)?;
            if q.dims != DIMENSIONLESS {
                return Err(UnitError(format!(
                    "{}() needs a dimensionless argument",
                    f.name()
                )));
            }
            Quantity {
                value: f.apply(q.value),
                dims: DIMENSIONLESS,
            }
        }
        Expr::Bin(op, a, b) => {
            let (a, b) = (eval(a)?, eval(b)?);
            match op {
                Op::Add | Op::Sub => {
                    if a.dims != b.dims {
                        return Err(UnitError(format!(
                            "cannot add {} and {}",
                            dims_str(a.dims),
                            dims_str(b.dims)
                        )));
                    }
                    let v = if *op == Op::Add {
                        a.value + b.value
                    } else {
                        a.value - b.value
                    };
                    Quantity {
                        value: v,
                        dims: a.dims,
                    }
                }
                Op::Mul => Quantity {
                    value: a.value * b.value,
                    dims: combine(a.dims, b.dims, 1),
                },
                Op::Div => Quantity {
                    value: a.value / b.value,
                    dims: combine(a.dims, b.dims, -1),
                },
                Op::Pow => {
                    if b.dims != DIMENSIONLESS {
                        return Err(UnitError("exponent must be dimensionless".into()));
                    }
                    let p = b.value;
                    let dims = a.dims.map(|d| d as f64 * p);
                    if dims.iter().any(|d| d.fract().abs() > 1e-9) {
                        return Err(UnitError("fractional power of a unit".into()));
                    }
                    Quantity {
                        value: a.value.powf(p),
                        dims: dims.map(|d| d.round() as i32),
                    }
                }
            }
        }
    })
}

/// Parse a quantity string (decimal comma allowed when it is the only comma).
pub fn parse_quantity(src: &str) -> Result<Quantity, UnitError> {
    let s = if src.matches(',').count() == 1 {
        src.replace(',', ".")
    } else {
        src.to_string()
    };
    let s = s.trim();
    for unit in ["degC", "degF"] {
        if let Some(num) = s.strip_suffix(unit) {
            if let Ok(x) = num.trim().parse::<f64>() {
                let (f, dims, off) = lookup(unit).expect("temperature unit in table");
                return Ok(Quantity {
                    value: x * f + off,
                    dims,
                });
            }
        }
    }
    finite(eval(&expr::parse(s)?)?)
}

fn finite(q: Quantity) -> Result<Quantity, UnitError> {
    if q.value.is_finite() {
        Ok(q)
    } else {
        Err(UnitError(NOT_FINITE.into()))
    }
}

/// Factor and dims of a variable's unit string (`m/s**2`, `J/(kg*K)`, `dimensionless`).
pub fn unit_of(unit: &str) -> Result<Quantity, UnitError> {
    if unit == "dimensionless" {
        return Ok(Quantity {
            value: 1.0,
            dims: DIMENSIONLESS,
        });
    }
    finite(eval(&expr::parse(unit)?)?)
}

/// Convert a typed value to the magnitude in `unit`. Bare dimensionless input is taken as already in `unit`
/// (as the Python engine does), so `t=3` means 3 s.
pub fn convert(input: &str, unit: &str) -> Result<f64, UnitError> {
    let q = parse_quantity(input)?;
    let target = unit_of(unit)?;
    if q.dims == DIMENSIONLESS && target.dims != DIMENSIONLESS {
        return Ok(q.value);
    }
    if q.dims != target.dims {
        return Err(UnitError(format!(
            "{input} is {}, expected {unit} ({})",
            dims_str(q.dims),
            dims_str(target.dims)
        )));
    }
    let v = q.value / target.value;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(UnitError(NOT_FINITE.into()))
    }
}

pub fn dims_str(d: Dims) -> String {
    let parts: Vec<String> = d
        .iter()
        .zip(DIM_NAMES)
        .filter(|(e, _)| **e != 0)
        .map(|(e, n)| {
            if *e == 1 {
                n.to_string()
            } else {
                format!("{n}^{e}")
            }
        })
        .collect();
    if parts.is_empty() {
        "dimensionless".into()
    } else {
        parts.join("·")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Golden {
        case: Vec<Case>,
    }

    #[derive(Deserialize)]
    struct Case {
        input: String,
        si: f64,
    }

    #[test]
    fn matches_pint_on_the_golden_table() {
        let g: Golden = toml::from_str(crate::data::UNITS_GOLDEN).unwrap();
        for c in g.case {
            let q = parse_quantity(&c.input).unwrap_or_else(|e| panic!("{}: {e}", c.input));
            assert!(
                (q.value - c.si).abs() <= 1e-12 * c.si.abs().max(1e-300),
                "{}: {} vs pint {}",
                c.input,
                q.value,
                c.si
            );
        }
    }

    #[test]
    fn conversions_used_by_the_quiz() {
        assert!((convert("100 km/h", "m/s").unwrap() - 27.777_777_777_8).abs() < 1e-9);
        assert!((convert("8000rpm", "rad/s").unwrap() - 837.758_040_957).abs() < 1e-6);
        assert!(
            (convert("50Hz", "rad/s").unwrap() - 50.0).abs() < 1e-12,
            "rad is dimensionless, as in pint"
        );
        assert_eq!(convert("3", "s").unwrap(), 3.0);
        assert_eq!(convert("1,5 km", "m").unwrap(), 1500.0);
        assert!((convert("60 degC", "K").unwrap() - 333.15).abs() < 1e-9);
        assert!((convert("20 Ah", "C").unwrap() - 72000.0).abs() < 1e-9);
        assert!((convert("5 %", "dimensionless").unwrap() - 0.05).abs() < 1e-15);
        assert!((convert("2 bar", "Pa").unwrap() - 2e5).abs() < 1e-6);
        assert!((convert("10 ppm/K", "1/K").unwrap() - 1e-5).abs() < 1e-18);
        assert!((convert("1.5g0", "m/s**2").unwrap() - 14.715).abs() < 1e-9);
        assert!(
            (convert("5 g", "kg").unwrap() - 0.005).abs() < 1e-15,
            "g is gram, g0 is gravity"
        );
        assert!((convert("2 min", "s").unwrap() - 120.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_wrong_dimensions_and_unknown_units() {
        assert!(convert("3 kg", "m").is_err());
        assert!(convert("3 furlongs", "m").is_err());
        assert!(parse_quantity("3 m + 2 s").is_err());
    }
}
