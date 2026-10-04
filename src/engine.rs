//! Formula-level solving, ported from the Python engine: `solve_step` (everything one formula determines),
//! `solve` (with defaults such as g = 9.81), `chain` (apply formulas until a target is known, keep only the
//! steps it depends on) and `conflicts` (formulas the final values violate).

use std::collections::{HashMap, HashSet};

use crate::expr::Expr;
use crate::registry::{registry, Formula};
use crate::solve::{solve1, solve_system};
use crate::units::{self, UnitError};

pub type Values = HashMap<String, f64>;

/// Variables found, in the order they were determined; several roots are sorted ascending and aligned
/// across variables solved together. The first root is the one carried forward.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Found(pub Vec<(String, Vec<f64>)>);

impl Found {
    pub fn get(&self, name: &str) -> Option<&[f64]> {
        self.0
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_slice())
    }

    fn set(&mut self, name: &str, vals: Vec<f64>) {
        match self.0.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = vals,
            None => self.0.push((name.to_string(), vals)),
        }
    }
}

fn positive(name: &str) -> bool {
    !registry().var(name).signed
}

fn subsets(n: usize, k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if n < k {
        return vec![];
    }
    let mut out = subsets(n - 1, k);
    for mut s in subsets(n - 1, k - 1) {
        s.push(n - 1);
        out.push(s);
    }
    out
}

/// Keep only the candidate solutions (tuples over `xs`) that let the rest of the formula be solved without
/// contradiction, and say whether any did. Matters when a signed variable appears squared (I^2 R gives +-I and
/// the given V = I R_T rules out -I). If none can be completed, all candidates stay, in ascending order.
fn consistent(
    f: &Formula,
    work: &Values,
    xs: &[String],
    sols: Vec<Vec<f64>>,
) -> (Vec<Vec<f64>>, bool) {
    let mut scored: Vec<(bool, Vec<f64>)> = sols
        .into_iter()
        .map(|t| {
            let mut w = work.clone();
            for (x, v) in xs.iter().zip(&t) {
                w.insert(x.clone(), *v);
            }
            for (n, vals) in solve_step(f, &w).0 {
                w.insert(n, vals[0]);
            }
            let complete = f.names.iter().all(|n| w.contains_key(n));
            (complete && conflicts_in(f, &w, 1e-6).is_empty(), t)
        })
        .collect();
    let any = scored.iter().any(|(ok, _)| *ok);
    if any {
        scored.retain(|(ok, _)| *ok);
    }
    (scored.into_iter().map(|(_, t)| t).collect(), any)
}

pub fn solve_step(f: &Formula, known: &Values) -> Found {
    let mut found = Found::default();
    let mut work = known.clone();
    loop {
        let pending: Vec<Expr> = f
            .eqs
            .iter()
            .map(|e| e.substitute(&work))
            .filter(|e| !matches!(e, Expr::Num(_)))
            .collect();
        let unknowns = |e: &Expr| {
            let mut v = Vec::new();
            e.vars(&mut v);
            v
        };
        let mut progress = false;
        // prefer an equation whose roots the rest of the formula agrees with; else the first that gives roots
        let mut fallback: Option<(String, Vec<f64>)> = None;
        for e in &pending {
            let u = unknowns(e);
            if u.len() != 1 {
                continue;
            }
            let raw: Vec<Vec<f64>> = solve1(e, &u[0], positive(&u[0]))
                .into_iter()
                .map(|r| vec![r])
                .collect();
            if raw.is_empty() {
                continue;
            }
            let (sols, ok) = consistent(f, &work, &u[0..1], raw);
            let vals: Vec<f64> = sols.into_iter().map(|t| t[0]).collect();
            if ok {
                fallback = Some((u[0].clone(), vals));
                break;
            }
            fallback.get_or_insert((u[0].clone(), vals));
        }
        if let Some((x, vals)) = fallback {
            work.insert(x.clone(), vals[0]);
            found.set(&x, vals);
            progress = true;
        }
        if progress {
            continue;
        }
        'outer: for size in 2..=pending.len() {
            for idx in subsets(pending.len(), size) {
                let group: Vec<Expr> = idx.iter().map(|&i| pending[i].clone()).collect();
                let mut xs: Vec<String> = Vec::new();
                for g in &group {
                    for v in unknowns(g) {
                        if !xs.contains(&v) {
                            xs.push(v);
                        }
                    }
                }
                if xs.len() != size {
                    continue;
                }
                xs.sort();
                let (sols, _) = consistent(f, &work, &xs, solve_system(&group, &xs, &positive));
                if let Some(first) = sols.first() {
                    for (i, x) in xs.iter().enumerate() {
                        work.insert(x.clone(), first[i]);
                        found.set(x, sols.iter().map(|s| s[i]).collect());
                    }
                    progress = true;
                    break 'outer;
                }
            }
        }
        if !progress {
            return found;
        }
    }
}

pub fn to_si(name: &str, value: &str) -> Result<f64, UnitError> {
    let v = registry()
        .vars
        .get(name)
        .ok_or_else(|| UnitError(format!("unknown variable {name}")))?;
    units::convert(value, &v.unit)
}

fn defaults(names: &[String], given: &Values) -> Values {
    names
        .iter()
        .filter(|n| !given.contains_key(*n))
        .filter_map(|n| registry().var(n).default.map(|d| (n.clone(), d)))
        .collect()
}

#[derive(Debug)]
pub struct Solved {
    pub found: Found,
    pub defaults: Values,
}

/// Solve one formula from typed values (`"100 km/h"`, `"3"`). Defaults fill variables not given.
pub fn solve(key: &str, given: &[(&str, &str)]) -> Result<Solved, String> {
    let f = registry()
        .formula(key)
        .ok_or_else(|| format!("no formula {key}"))?;
    let mut known = Values::new();
    for (n, v) in given {
        if !f.names.iter().any(|x| x == n) {
            return Err(format!(
                "{key} has no variable {n}; it uses {}",
                f.names.join(", ")
            ));
        }
        known.insert(n.to_string(), to_si(n, v).map_err(|e| e.0)?);
    }
    let used = defaults(&f.names, &known);
    known.extend(used.clone());
    Ok(Solved {
        found: solve_step(f, &known),
        defaults: used,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub formula: String,
    pub var: String,
    pub values: Vec<f64>,
}

#[derive(Debug)]
pub struct Chained {
    pub steps: Vec<Step>,
    pub known: Values,
    pub defaults: Values,
    pub reached: bool,
}

/// Apply formulas (all, or those whose key or tags are in `only`) until `target` is known.
pub fn chain(
    target: &str,
    given: &[(&str, &str)],
    only: Option<&HashSet<String>>,
) -> Result<Chained, String> {
    let r = registry();
    let mut known = Values::new();
    for (n, v) in given {
        known.insert(n.to_string(), to_si(n, v).map_err(|e| e.0)?);
    }
    let all: Vec<String> = r.vars.keys().cloned().collect();
    let used = defaults(&all, &known);
    known.extend(used.clone());
    let pool: Vec<&Formula> = r
        .formulas
        .iter()
        .filter(|f| only.is_none_or(|o| o.contains(&f.key) || f.tags.iter().any(|t| o.contains(t))))
        .collect();
    let mut steps = Vec::new();
    let mut deps: HashMap<String, (String, Vec<String>)> = HashMap::new();
    while !known.contains_key(target) {
        let mut progress = false;
        for f in &pool {
            let new = solve_step(f, &known);
            for (n, vals) in new.0 {
                if known.contains_key(&n) {
                    continue;
                }
                known.insert(n.clone(), vals[0]);
                deps.insert(
                    n.clone(),
                    (
                        f.key.clone(),
                        f.names.iter().filter(|x| **x != n).cloned().collect(),
                    ),
                );
                steps.push(Step {
                    formula: f.key.clone(),
                    var: n,
                    values: vals,
                });
                progress = true;
            }
            if known.contains_key(target) {
                break;
            }
        }
        if !progress {
            break;
        }
    }
    if !known.contains_key(target) {
        return Ok(Chained {
            steps,
            known,
            defaults: Values::new(),
            reached: false,
        });
    }
    let mut need = HashSet::new();
    let mut todo = vec![target.to_string()];
    while let Some(n) = todo.pop() {
        if let Some((_, inputs)) = deps.get(&n) {
            if need.insert(n.clone()) {
                todo.extend(inputs.iter().cloned());
            }
        }
    }
    steps.retain(|s| need.contains(&s.var));
    let used: Values = used
        .into_iter()
        .filter(|(n, _)| {
            steps
                .iter()
                .any(|s| r.formula(&s.formula).unwrap().names.contains(n))
        })
        .collect();
    Ok(Chained {
        steps,
        known,
        defaults: used,
        reached: true,
    })
}

/// Equations of `f` that `known` violates (relative tolerance `tol`); all its variables must be known.
pub fn conflicts_in(f: &Formula, known: &Values, tol: f64) -> Vec<String> {
    f.eqs
        .iter()
        .zip(&f.src)
        .filter_map(|(e, src)| {
            let (lhs, rhs) = match e {
                Expr::Bin(_, a, b) => (a.eval_map(known)?, b.eval_map(known)?),
                _ => return None,
            };
            (lhs - rhs)
                .abs()
                .gt(&(tol * lhs.abs().max(rhs.abs()).max(1e-12)))
                .then(|| format!("{}: {src}  ({lhs:.6} != {rhs:.6})", f.key))
        })
        .collect()
}

pub fn conflicts(known: &Values) -> Vec<String> {
    registry()
        .formulas
        .iter()
        .filter(|f| f.names.iter().all(|n| known.contains_key(n)))
        .flat_map(|f| conflicts_in(f, known, 1e-3))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * b.abs().max(1e-12)
    }

    #[test]
    fn solves_any_unknown_with_units() {
        let r = solve("speed", &[("s", "75 m"), ("t", "3.5 s")]).unwrap();
        assert!(approx(r.found.get("v_avg").unwrap()[0], 21.428_571, 1e-6));
        let r = solve("speed", &[("v_avg", "100 km/h"), ("t", "3")]).unwrap();
        assert!(approx(r.found.get("s").unwrap()[0], 83.333_333, 1e-6));
    }

    #[test]
    fn systems_and_defaults() {
        let r = solve(
            "uniform_accel",
            &[("v0", "0"), ("s", "75"), ("v", "100 km/h")],
        )
        .unwrap();
        assert!(approx(r.found.get("t").unwrap()[0], 5.4, 1e-9));
        assert!(approx(r.found.get("a").unwrap()[0], 5.144_032_9, 1e-6));
        let r = solve(
            "cornering_downforce",
            &[
                ("mu", "1.4"),
                ("m", "240kg"),
                ("ClA", "3.2"),
                ("rho_air", "1.1"),
                ("R_c", "9.125m"),
            ],
        )
        .unwrap();
        assert_eq!(r.defaults.get("g"), Some(&9.81));
        assert!(approx(r.found.get("v").unwrap()[0], 11.759_1, 1e-4));
    }

    #[test]
    fn battery_roots_paired() {
        let r = solve(
            "battery_load",
            &[
                ("N_s", "103"),
                ("V_cell", "3.8V"),
                ("R_pack", "0.08ohm"),
                ("P", "30kW"),
            ],
        )
        .unwrap();
        let (i, vt) = (r.found.get("I").unwrap(), r.found.get("V_t").unwrap());
        assert_eq!(i.len(), 2);
        for k in 0..2 {
            assert!(approx(vt[k], 391.4 - i[k] * 0.08, 1e-9));
        }
        assert!(approx(i[0], 77.8879, 1e-5));
    }

    #[test]
    fn chain_and_conflicts() {
        let c = chain("t", &[("v0", "0"), ("s", "75"), ("v", "100 km/h")], None).unwrap();
        assert!(c.reached);
        assert!(approx(c.known["t"], 5.4, 1e-9));
        assert!(conflicts(&c.known).is_empty());
        let bad: Values = [
            ("v0", 0.0),
            ("s", 75.0),
            ("v", 27.78),
            ("t", 2.7),
            ("a", 10.29),
        ]
        .map(|(k, v)| (k.to_string(), v))
        .into();
        assert!(!conflicts(&bad).is_empty());
        let c = chain(
            "V",
            &[
                ("R_0", "1ohm"),
                ("alpha", "10ppm/K"),
                ("R_th", "1K/W"),
                ("T_amb", "25degC"),
                ("T_0", "25degC"),
                ("I", "13A"),
            ],
            None,
        )
        .unwrap();
        assert!(approx(c.known["V"], 13.022, 1e-4), "{}", c.known["V"]);
    }
}
