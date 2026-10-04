//! Procedural tools (ported from the Python `tools.py`): rules scoring for legacy/2026/2027 read from the
//! embedded `data/rules`, rule tables, exact nodal circuit analysis, electronics, digital and mechanics helpers.
//! Each tool declares typed parameters so the CLI and the app call them the same way.

use std::collections::HashMap;
use std::sync::OnceLock;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

use crate::expr;
use crate::format::g6;
use crate::solve::solve1;
use crate::units;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Num,
    Str,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: &'static str,
    pub kind: Kind,
    pub default: Option<&'static str>,
}

pub struct Tool {
    pub name: &'static str,
    pub doc: &'static str,
    pub params: Vec<Param>,
    run: fn(&Args) -> Result<String, String>,
}

pub struct Args(HashMap<&'static str, String>);

impl Args {
    pub fn num(&self, k: &str) -> Result<f64, String> {
        let v = &self.0[k];
        units::parse_quantity(v)
            .map(|q| q.value)
            .map_err(|e| format!("{k}: {e}"))
    }

    pub fn str(&self, k: &str) -> &str {
        &self.0[k]
    }
}

fn p(spec: &'static str) -> Vec<Param> {
    // "name:n" / "name:s" / "name:n=default"
    spec.split_whitespace()
        .map(|s| {
            let (head, default) = match s.split_once('=') {
                Some((h, d)) => (h, Some(d)),
                None => (s, None),
            };
            let (name, kind) = head.split_once(':').unwrap();
            Param {
                name,
                kind: if kind == "s" { Kind::Str } else { Kind::Num },
                default,
            }
        })
        .collect()
}

impl Tool {
    /// Call with positional values and/or `name=value` pairs.
    pub fn call(&self, args: &[&str]) -> Result<String, String> {
        let mut map = HashMap::new();
        let mut pos = 0;
        for a in args {
            match a.split_once('=') {
                Some((k, v)) if self.params.iter().any(|p| p.name == k) => {
                    let name = self.params.iter().find(|p| p.name == k).unwrap().name;
                    map.insert(name, v.to_string());
                }
                _ => {
                    let param = self
                        .params
                        .get(pos)
                        .ok_or_else(|| format!("{}: too many arguments", self.name))?;
                    map.insert(param.name, a.to_string());
                    pos += 1;
                }
            }
            while self
                .params
                .get(pos)
                .is_some_and(|p| map.contains_key(p.name))
            {
                pos += 1;
            }
        }
        for prm in &self.params {
            if !map.contains_key(prm.name) {
                let d = prm
                    .default
                    .ok_or_else(|| format!("{}: missing {}", self.name, prm.name))?;
                map.insert(prm.name, d.to_string());
            }
        }
        (self.run)(&Args(map))
    }

    pub fn signature(&self) -> String {
        let ps: Vec<String> = self
            .params
            .iter()
            .map(|p| match p.default {
                Some(d) => format!("{}={}", p.name, if d.is_empty() { "\"\"" } else { d }),
                None => p.name.to_string(),
            })
            .collect();
        format!("{}({})", self.name, ps.join(", "))
    }
}

// ------------------------------------------------------------------ rules data

/// Every embedded rule set (legacy, 2026, 2027...) as parsed TOML.
pub fn rule_sets() -> &'static HashMap<String, toml::Table> {
    static R: OnceLock<HashMap<String, toml::Table>> = OnceLock::new();
    R.get_or_init(|| {
        crate::data::RULES
            .iter()
            .map(|(y, s)| (y.to_string(), s.parse().expect("rules toml")))
            .collect()
    })
}

/// Shared penalties (D 10.1.7, IN 12.1.4...) as parsed TOML.
pub fn penalties() -> &'static toml::Table {
    static P: OnceLock<toml::Table> = OnceLock::new();
    P.get_or_init(|| crate::data::PENALTIES.parse().expect("penalties toml"))
}

pub fn current_rules() -> &'static str {
    rule_sets()
        .keys()
        .filter(|y| y.chars().all(|c| c.is_ascii_digit()))
        .max()
        .unwrap()
}

fn num(v: &toml::Value) -> f64 {
    v.as_float()
        .or_else(|| v.as_integer().map(|i| i as f64))
        .expect("number in rules data")
}

fn rule<'a>(year: &str, path: &[&str]) -> Result<&'a toml::Value, String> {
    let mut v = rule_sets()
        .get(year)
        .ok_or_else(|| format!("unknown rules {year:?} (have legacy, 2026, 2027)"))?;
    let mut cur: Option<&toml::Value> = None;
    for (i, k) in path.iter().enumerate() {
        let t = if i == 0 {
            Some(v)
        } else {
            cur.and_then(|c| c.as_table())
        };
        cur = t.and_then(|t| t.get(*k));
        if cur.is_none() {
            return Err(format!("rules {year}: no {}", path.join(".")));
        }
        if let Some(t) = cur.and_then(|c| c.as_table()) {
            v = t;
        }
    }
    Ok(cur.unwrap())
}

fn pen(path: &[&str]) -> Option<f64> {
    let mut cur = penalties().get(path[0])?;
    for k in &path[1..] {
        cur = cur.as_table()?.get(*k)?;
    }
    Some(num(cur))
}

fn rules_arg(a: &Args) -> String {
    let r = a.str("rules");
    if r.is_empty() {
        current_rules().to_string()
    } else {
        r.to_string()
    }
}

fn pmax_of(event: &str, year: &str) -> Result<f64, String> {
    let points_key = rule(year, &["tmax_pmin", event])
        .ok()
        .and_then(|t| t.get("points"))
        .and_then(|v| v.as_str())
        .unwrap_or(event)
        .to_string();
    Ok(num(rule(year, &["points", &points_key])?))
}

/// Score expression in `t` (as text for the solver) and Tmax.
fn score_expr(
    event: &str,
    t_min: f64,
    year: &str,
    pmax: Option<f64>,
    finish: bool,
) -> Result<(String, f64), String> {
    let formula = rule(year, &["dynamic_formula"])?
        .as_str()
        .unwrap_or("squared");
    if formula == "legacy" {
        let c = rule(year, &["legacy_scoring", event])?;
        let g = |k: &str| num(&c[k]);
        let t_max = g("tmax_factor") * t_min;
        let floor = if finish || event != "endurance" {
            g("floor")
        } else {
            0.0
        };
        return Ok((
            format!(
                "{}*(({t_max}/t)**{} - 1)/{} + {floor}",
                g("scale"),
                g("exponent"),
                g("divisor")
            ),
            t_max,
        ));
    }
    let c = rule(year, &["tmax_pmin", event])?;
    let p = match pmax {
        Some(p) => p,
        None => pmax_of(event, year)?,
    };
    let t_max = num(&c["tmax"]) * t_min;
    let pmin = num(&c["pmin"]) * p;
    Ok((
        format!("({p} - {pmin})*(({t_max} - t)/({t_max} - {t_min}))**2 + {pmin}"),
        t_max,
    ))
}

fn eval_t(src: &str, t: f64) -> f64 {
    expr::parse(src)
        .unwrap()
        .eval(&|v| (v == "t").then_some(t))
        .unwrap()
}

// ------------------------------------------------------------------ tool bodies

fn corrected_time(a: &Args) -> Result<String, String> {
    let event = a.str("event");
    let doo = pen(&["doo_oc", event, "doo"]).ok_or_else(|| format!("unknown event {event}"))?;
    let oc = a.num("oc")?;
    let oc_pen = pen(&["doo_oc", event, "oc"]);
    if oc > 0.0 && oc_pen.is_none() {
        return Err(format!("an OC in {event} is a DQ"));
    }
    let post = a.str("post");
    let post_pen = if post.is_empty() {
        0.0
    } else {
        pen(&["post_inspection", post, event]).ok_or("post must be A or B")?
    };
    let t = a.num("t_raw")?
        + a.num("doo")? * doo
        + oc * oc_pen.unwrap_or(0.0)
        + pen(&["flag"]).unwrap() * a.num("flags")?
        + pen(&["out_of_order"]).unwrap() * a.num("out_of_order")?
        + post_pen;
    Ok(g6(t))
}

fn event_score(a: &Args) -> Result<String, String> {
    let (event, year) = (a.str("event"), rules_arg(a));
    let pmax = Some(a.num("pmax")?).filter(|p| *p > 0.0);
    let (src, t_max) = score_expr(event, a.num("t_min")?, &year, pmax, a.num("finish")? != 0.0)?;
    let s = eval_t(&src, a.num("t_team")?.min(t_max));
    if year == "legacy" {
        return Ok(g6(s));
    }
    let cap = match pmax {
        Some(p) => p,
        None => pmax_of(event, &year)?,
    };
    Ok(g6(s.clamp(0.0, cap)))
}

fn cones_from_score(a: &Args) -> Result<String, String> {
    let (event, year) = (a.str("event"), rules_arg(a));
    let (src, t_max) = score_expr(event, a.num("t_min")?, &year, None, true)?;
    let e = expr::parse(&format!("{src} - ({})", a.num("score")?)).map_err(|e| e.0)?;
    let doo = pen(&["doo_oc", event, "doo"]).ok_or("unknown event")?;
    let t_raw = a.num("t_raw")?;
    let sols: Vec<String> = solve1(&e, "t", true)
        .into_iter()
        .filter(|&t| t > 0.0 && t <= t_max * (1.0 + 1e-9))
        .map(|t| {
            format!(
                "T_team={} s -> {} DOO",
                crate::format::g(t, 4),
                crate::format::g((t - t_raw) / doo, 3)
            )
        })
        .collect();
    Ok(sols.join("; "))
}

fn dv_rank_score(a: &Args) -> Result<String, String> {
    let (r, n) = (a.num("rank")?, a.num("n_all")?);
    Ok(g6(a.num("pmax")? * (n + 1.0 - r) / n))
}

fn dc_autocross_score(a: &Args) -> Result<String, String> {
    let (t_max, t_min, pmax) = (a.num("t_max")?, a.num("t_min")?, a.num("pmax")?);
    let t1 = a.num("t1")?.min(t_max);
    let tt = t1.min((t1 + a.num("t2")?.min(t_max)) / 2.0);
    Ok(g6(0.9 * pmax * (t_max - tt) / (t_max - t_min) + 0.1 * pmax))
}

fn trackdrive_score(a: &Args) -> Result<String, String> {
    let (pmax, t_max) = (a.num("pmax")?, 2.0 * a.num("t_fastest")?);
    Ok(g6(0.75
        * pmax
        * (t_max / a.num("t_team")?.min(t_max) - 1.0)
        + 0.025 * pmax * a.num("laps")?))
}

fn efficiency_score(a: &Args) -> Result<String, String> {
    let year = rules_arg(a);
    let p = Some(a.num("pmax")?)
        .filter(|p| *p > 0.0)
        .unwrap_or(num(rule(&year, &["points", "efficiency"])?));
    let (t, e, ef_min) = (a.num("t")?, a.num("e")?, a.num("ef_min")?);
    if year == "legacy" {
        let ef = (a.num("t_min")? / t) * (a.num("e_min")? / e);
        return Ok(g6(p * (ef - ef_min) / (a.num("ef_max")? - ef_min)));
    }
    let (ef, ef_max) = (t * t * e, 2.0 * ef_min);
    Ok(g6(p * ((ef_max - ef).max(0.0) / (ef_max - ef_min)).powi(2)))
}

fn static_nonfinalist(a: &Args) -> Result<String, String> {
    let (year, event) = (rules_arg(a), a.str("event"));
    let base = num(rule(&year, &["static_nonfinalist", event])?);
    let minus = rule(&year, &["static_nonfinalist", "bpp_minus_finalists"])
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let k = base
        - if event == "bpp" && minus {
            a.num("n_finalists")?
        } else {
            0.0
        };
    Ok(g6(k * a.num("p_team")? / a.num("p_best")?))
}

fn max_points(a: &Args) -> Result<String, String> {
    let year = rules_arg(a);
    let mut total = 0.0;
    for e in a.str("events").split(',') {
        total += num(rule(&year, &["points", e.trim()])?);
    }
    Ok(g6(total))
}

fn pcb_spacing(a: &Args) -> Result<String, String> {
    let year = rules_arg(a);
    let t = rule(&year, &["pcb_spacing"])?;
    let modes: Vec<&str> = t["modes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m.as_str())
        .collect();
    let mode = a.str("mode");
    let col = modes
        .iter()
        .position(|m| *m == mode)
        .ok_or_else(|| format!("mode for {year}: {}", modes.join(" | ")))?
        + 1;
    let v = a.num("v")?;
    for row in t["rows"].as_array().unwrap() {
        let row = row.as_array().unwrap();
        if v <= num(&row[0]) {
            return Ok(g6(num(&row[col])));
        }
    }
    Err("above 600 V: TS max is 600 V (EV 4.1.1)".into())
}

fn ts_rules(a: &Args) -> Result<String, String> {
    let year = rules_arg(a);
    let ts = rule(&year, &["ts"])?;
    let v_max = a.num("v_max")?;
    let test = ts["insulation_tiers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_array().unwrap())
        .find(|r| v_max <= num(&r[0]))
        .map(|r| num(&r[1]))
        .unwrap();
    let nominal = ts["tsal_uses"].as_str() == Some("nominal");
    let v_nom = a.num("v_nom")?;
    let tsal_v = if v_nom > 0.0 { v_nom } else { v_max };
    let note = if nominal && v_nom == 0.0 {
        " (pass v_nom: this rule year uses NOMINAL voltage)"
    } else {
        ""
    };
    Ok(format!(
        "insulation test {} V, pass >= {} kOhm (500 Ohm/V); TSAL red threshold min(60, V/2) = {} V{note}; \
         section <= 120 V, <= 6 MJ (max cell V x nominal Ah), <= 12 kg; TS <= 600 V; discharge <= 60 V in 5 s",
        g6(test),
        g6(500.0 * v_max / 1e3),
        g6((tsal_v / 2.0).min(60.0))
    ))
}

fn weight_penalty(a: &Args) -> Result<String, String> {
    let over = (a.num("delta_kg")?.abs() - pen(&["weight_tolerance_kg"]).unwrap()).max(0.0);
    Ok(g6(
        pen(&["weight_per_kg"]).unwrap() * (over - 1e-9).ceil().max(0.0)
    ))
}

fn segment_max_cells(a: &Args) -> Result<String, String> {
    let by_v = (a.num("v_lim")? / a.num("v_cell_max")? + 1e-9).floor();
    let by_e = (a.num("e_lim")? / a.num("e_cell")? + 1e-9).floor();
    Ok(g6(by_v.min(by_e)))
}

const E6: [f64; 6] = [1.0, 1.5, 2.2, 3.3, 4.7, 6.8];
const E12: [f64; 12] = [1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2];
const E24: [f64; 24] = [
    1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0, 3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6,
    6.2, 6.8, 7.5, 8.2, 9.1,
];

fn e_series(a: &Args) -> Result<String, String> {
    let (value, series) = (a.num("value")?, a.num("series")? as usize);
    let base: Vec<f64> = match series {
        6 => E6.to_vec(),
        12 => E12.to_vec(),
        24 => E24.to_vec(),
        n => (0..n)
            .map(|i| (10f64.powf(i as f64 / n as f64) * 100.0).round() / 100.0)
            .collect(),
    };
    let dec = 10f64.powf(value.log10().floor());
    let mut cands: Vec<f64> = base
        .iter()
        .flat_map(|b| [0.1, 1.0, 10.0].map(|k| b * dec * k))
        .collect();
    cands.sort_by(f64::total_cmp);
    let below = cands
        .iter()
        .copied()
        .filter(|c| *c <= value * (1.0 + 1e-9))
        .fold(f64::NAN, f64::max);
    let above = cands
        .iter()
        .copied()
        .filter(|c| *c >= value * (1.0 - 1e-9))
        .fold(f64::NAN, f64::min);
    let near = cands
        .iter()
        .copied()
        .min_by(|x, y| (x / value).ln().abs().total_cmp(&(y / value).ln().abs()))
        .unwrap();
    let f4 = |x: f64| crate::format::g(x, 4);
    Ok(format!(
        "below {}  nearest {}  above {}",
        f4(below),
        f4(near),
        f4(above)
    ))
}

fn gamma(a: &Args) -> Result<String, String> {
    let (re, im, z0) = (a.num("z_load_re")?, a.num("z_load_im")?, a.num("z0")?);
    // |(zl - z0)/(zl + z0)|
    let m = ((re - z0).powi(2) + im * im).sqrt() / ((re + z0).powi(2) + im * im).sqrt();
    Ok(format!(
        "|Gamma| {}  VSWR {}  transmitted {:.4}%",
        crate::format::g(m, 5),
        crate::format::g((1.0 + m) / (1.0 - m), 5),
        (1.0 - m * m) * 100.0
    ))
}

fn alias(a: &Args) -> Result<String, String> {
    let (f, fs) = (a.num("f")?, a.num("fs")?);
    Ok(g6((f - (f / fs).round() * fs).abs()))
}

fn twos(a: &Args) -> Result<String, String> {
    let (x, bits) = (a.num("x")? as i64, a.num("bits")? as u32);
    let v = if x < 0 {
        (1i128 << bits) + x as i128
    } else {
        x as i128
    };
    Ok(format!("{v:0width$b}", width = bits as usize))
}

fn uart_time(a: &Args) -> Result<String, String> {
    let bits = 1.0 + a.num("data")? + a.num("parity")? + a.num("stop")?;
    Ok(g6(a.num("n_bytes")? * bits / a.num("baud")?))
}

fn can_frame_bits(a: &Args) -> Result<String, String> {
    let base = if a.num("extended")? != 0.0 {
        64.0
    } else {
        44.0
    } + 8.0 * a.num("data_bytes")?.floor();
    let extra = if a.num("stuffing")? != 0.0 {
        ((base - 13.0 - 1.0) / 4.0).floor()
    } else {
        0.0
    };
    Ok(g6(base + extra + a.num("ifs")?.floor()))
}

fn db_sum(a: &Args) -> Result<String, String> {
    let mut total = 0.0;
    for x in a.str("levels").split(',') {
        let l: f64 = x.trim().parse().map_err(|_| format!("bad level {x}"))?;
        total += 10f64.powf(l / 10.0);
    }
    Ok(g6(10.0 * total.log10()))
}

fn exact(s: &str) -> Result<BigRational, String> {
    let s = s.trim();
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i32>().map_err(|_| format!("bad number {s}"))?),
        None => (s, 0),
    };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits: BigInt = format!("{int}{frac}")
        .parse()
        .map_err(|_| format!("bad number {s}"))?;
    let scale = exp - frac.len() as i32;
    let ten = BigInt::from(10);
    Ok(if scale >= 0 {
        BigRational::from_integer(digits * num_traits::pow(ten, scale as usize))
    } else {
        BigRational::new(digits, num_traits::pow(ten, (-scale) as usize))
    })
}

fn show_exact(r: &BigRational) -> String {
    let f = r.numer().to_f64().unwrap_or(f64::NAN) / r.denom().to_f64().unwrap_or(f64::NAN);
    if r.denom() == &BigInt::from(1) {
        format!("{} = {}", r.numer(), g6(f))
    } else {
        format!("{}/{} = {}", r.numer(), r.denom(), g6(f))
    }
}

fn nodal(a: &Args) -> Result<String, String> {
    let ask = a.str("ask");
    let mut elems: Vec<Vec<String>> = a
        .str("netlist")
        .replace('\n', ";")
        .split(';')
        .map(|e| e.split_whitespace().map(String::from).collect::<Vec<_>>())
        .filter(|e| !e.is_empty())
        .collect();
    if elems.iter().any(|e| e.len() != 4) {
        return Err("each element: <name> <node> <node> <value>".into());
    }
    let rab = ask
        .strip_prefix("Rab:")
        .map(|s| s.split_once(',').ok_or("ask=Rab:a,b"));
    if let Some(r) = rab {
        let (na, nb) = r?;
        elems.retain(|e| e[0].to_uppercase().starts_with('R'));
        elems.push(vec!["Itest".into(), nb.into(), na.into(), "1".into()]);
    }
    let mut nodes: Vec<String> = elems
        .iter()
        .flat_map(|e| [e[1].clone(), e[2].clone()])
        .filter(|n| n != "0")
        .collect();
    nodes.sort();
    nodes.dedup();
    let vsrc: Vec<usize> = (0..elems.len())
        .filter(|&i| elems[i][0].to_uppercase().starts_with('V'))
        .collect();
    let nn = nodes.len();
    let size = nn + vsrc.len();
    let zero = BigRational::zero();
    let mut m = vec![vec![zero.clone(); size + 1]; size];
    let idx = |n: &str| nodes.iter().position(|x| x == n);
    for (k, e) in elems.iter().enumerate() {
        let val = exact(&e[3])?;
        let (na, nb) = (idx(&e[1]), idx(&e[2]));
        match e[0].to_uppercase().chars().next().unwrap() {
            'R' => {
                let gval = BigRational::from_integer(1.into()) / val;
                for (x, y, s) in [(na, na, 1), (nb, nb, 1), (na, nb, -1), (nb, na, -1)] {
                    if let (Some(x), Some(y)) = (x, y) {
                        let term = gval.clone() * BigRational::from_integer(s.into());
                        m[x][y] += term;
                    }
                }
            }
            'I' => {
                // current leaves node a, enters node b (KCL rows are "sum leaving = 0", rhs = -injections)
                if let Some(x) = na {
                    m[x][size] -= val.clone();
                }
                if let Some(y) = nb {
                    m[y][size] += val;
                }
            }
            'V' => {
                let row = nn + vsrc.iter().position(|&i| i == k).unwrap();
                if let Some(x) = na {
                    m[x][row] += BigRational::from_integer(1.into());
                    m[row][x] += BigRational::from_integer(1.into());
                }
                if let Some(y) = nb {
                    m[y][row] -= BigRational::from_integer(1.into());
                    m[row][y] -= BigRational::from_integer(1.into());
                }
                m[row][size] = val;
            }
            c => return Err(format!("unknown element type {c} (use R, V or I)")),
        }
    }
    // Gauss-Jordan over exact rationals
    for c in 0..size {
        let p = (c..size)
            .find(|&r| !m[r][c].is_zero())
            .ok_or("singular circuit (floating node?)")?;
        m.swap(c, p);
        let piv = m[c][c].clone();
        for x in &mut m[c][c..] {
            *x = x.clone() / piv.clone();
        }
        let pivot_row = m[c].clone();
        for (r, row) in m.iter_mut().enumerate() {
            if r != c && !row[c].is_zero() {
                let k = row[c].clone();
                for (x, pv) in row[c..].iter_mut().zip(&pivot_row[c..]) {
                    *x -= k.clone() * pv.clone();
                }
            }
        }
    }
    let volt = |n: &str| idx(n).map_or_else(BigRational::zero, |i| m[i][size].clone());
    if let Some(r) = rab {
        let (na, nb) = r?;
        let rr = volt(na) - volt(nb);
        return Ok(format!("R_{na}{nb} = {} ohm", show_exact(&rr)));
    }
    let mut out: Vec<String> = nodes
        .iter()
        .map(|n| format!("V({n}) = {} V", show_exact(&volt(n))))
        .collect();
    for e in elems
        .iter()
        .filter(|e| e[0].to_uppercase().starts_with('R'))
    {
        let i = (volt(&e[1]) - volt(&e[2])) / exact(&e[3])?;
        out.push(format!(
            "I({}, {}->{}) = {} A",
            e[0],
            e[1],
            e[2],
            show_exact(&i)
        ));
    }
    for (k, &i) in vsrc.iter().enumerate() {
        // MNA unknown = current leaving the + node through the source, as in the Python engine
        let cur = m[nn + k][size].to_f64().unwrap_or(f64::NAN);
        out.push(format!(
            "I({}, into + terminal) = {} A",
            elems[i][0],
            g6(cur)
        ));
    }
    Ok(out.join("\n  "))
}

fn two_dof(a: &Args) -> Result<String, String> {
    let (m1, m2, k1, k2) = (a.num("m1")?, a.num("m2")?, a.num("k1")?, a.num("k2")?);
    let r = roots::find_roots_quadratic(
        m1 * m2,
        -(k1 * m2 + (k1 + k2) * m1),
        k1 * (k1 + k2) - k1 * k1,
    );
    let mut fs: Vec<f64> = r
        .as_ref()
        .iter()
        .map(|w2| w2.sqrt() / std::f64::consts::TAU)
        .collect();
    fs.sort_by(f64::total_cmp);
    Ok(fs
        .iter()
        .map(|f| format!("{} Hz", crate::format::g(*f, 4)))
        .collect::<Vec<_>>()
        .join("  "))
}

fn gauge(a: &Args) -> Result<String, String> {
    let (pv, atm) = (a.num("p")?, a.num("p_atm")?);
    Ok(g6(if a.str("to") == "abs" {
        pv + atm
    } else {
        pv - atm
    }))
}

fn gear_train(a: &Args) -> Result<String, String> {
    let mut r = 1.0;
    for pair in a.str("teeth").split(',') {
        let (x, y) = pair.split_once('/').ok_or("teeth like 20/45,24/55")?;
        let (x, y): (f64, f64) = (
            x.trim().parse().map_err(|_| "bad tooth count")?,
            y.trim().parse().map_err(|_| "bad tooth count")?,
        );
        r *= x / y;
    }
    Ok(g6(a.num("n_in")? * r))
}

fn accel_then_cruise(a: &Args) -> Result<String, String> {
    let (s, acc, vmax) = (a.num("s")?, a.num("a")?, a.num("v_max")?);
    let s_acc = vmax * vmax / (2.0 * acc);
    Ok(g6(if s <= s_acc {
        (2.0 * s / acc).sqrt()
    } else {
        vmax / acc + (s - s_acc) / vmax
    }))
}

fn round_to(a: &Args) -> Result<String, String> {
    let (x, step) = (a.num("x")?, a.num("step")?);
    let q = x / step;
    let r = match a.str("mode") {
        "down" => q.floor(),
        "up" => q.ceil(),
        _ => q.round_ties_even(),
    };
    Ok(g6(r * step))
}

pub fn tools() -> &'static [Tool] {
    static T: OnceLock<Vec<Tool>> = OnceLock::new();
    T.get_or_init(|| {
        let t = |name, doc, spec, run| Tool { name, doc, params: p(spec), run };
        vec![
            t("corrected_time", "Raw time + DOO/OC (per event) + 60 s per disobeyed flag + 120 s out of order + post-inspection group A/B.",
              "t_raw:n doo:n=0 oc:n=0 event:s=autocross flags:n=0 out_of_order:n=0 post:s=", corrected_time),
            t("event_score", "Manual dynamic event score (2026/2027 D 9.1.1). event: skidpad accel autocross endurance, dc_skidpad dc_accel. Times already corrected. rules=legacy reproduces older quiz keys (endurance finish=0 drops the +25, as the Q171 key does).",
              "event:s t_team:n t_min:n rules:s= pmax:n=0 finish:n=1", event_score),
            t("cones_from_score", "Inverse: which corrected time (and how many DOO) gives this score.",
              "event:s score:n t_raw:n t_min:n rules:s=", cones_from_score),
            t("dv_rank_score", "2026/2027 D 9.2.2 autonomous-mode skidpad/accel: Pmax (N_all + 1 - R) / N_all. Runs > 25 s raw are DQ.",
              "rank:n n_all:n pmax:n=75", dv_rank_score),
            t("dc_autocross_score", "2026/2027 D 9.3.2. t_max = lap length / 6 m/s. DNF/DQ runs: pass t_max.",
              "t1:n t2:n t_min:n t_max:n pmax:n=100", dc_autocross_score),
            t("trackdrive_score", "2026/2027 D 9.3.3 + D 9.3.4: 0.75 Pmax (Tmax/Tteam - 1) with Tmax = 2 T_fastest, + 2.5 % Pmax per completed lap.",
              "t_team:n t_fastest:n laps:n=10 pmax:n=200", trackdrive_score),
            t("efficiency_score", "2026/2027 D 9.4: EF = T^2 E, score = Pmax ((EFmax - EF)/(EFmax - EFmin))^2, EFmax = 2 EFmin, EF clamped. legacy: EF = (Tmin/T)(Emin/E), linear (pass t_min, e_min, ef_min, ef_max).",
              "t:n e:n ef_min:n rules:s= pmax:n=0 t_min:n=0 e_min:n=0 ef_max:n=0", efficiency_score),
            t("static_nonfinalist", "Non-finalist static score scaled to the best NON-finalist. 2027: BPP 65, cost 80 cap. 2026: BPP 70. legacy: BPP (75 - n_finalists), cost 95.",
              "p_team:n p_best:n event:s=bpp rules:s= n_finalists:n=0", static_nonfinalist),
            t("max_points", "Sum of maximum points, e.g. events=bpp,cost,skidpad,accel,efficiency.",
              "events:s rules:s=", max_points),
            t("pcb_spacing", "TS-LV PCB spacing [mm]. 2026/2027 modes: clearance | creepage | coated. legacy: surface | air | coated.",
              "v:n mode:s=creepage rules:s=", pcb_spacing),
            t("ts_rules", "Voltage-dependent TS rule values for a rule year.", "v_max:n v_nom:n=0 rules:s=", ts_rules),
            t("weight_penalty", "IN 12.1.7: 20 points per started kg beyond +-5 kg vs the technical inspection weight (6.2 kg -> 40).",
              "delta_kg:n", weight_penalty),
            t("segment_max_cells", "Max cells per section: min(floor(V_lim/V_cell), floor(E_lim/E_cell)). E_cell = MAX cell V x nominal capacity (EV 5.1.2): type 4.2V*20Ah. Mass limit 12 kg too.",
              "v_cell_max:n e_cell:n v_lim:n=120 e_lim:n=6e6", segment_max_cells),
            t("e_series", "Nearest standard values (below, nearest, above) in E6/E12/E24 (E96 computed).",
              "value:n series:n=12", e_series),
            t("gamma", "Reflection coefficient and VSWR of a complex load.", "z_load_re:n z_load_im:n=0 z0:n=50", gamma),
            t("alias", "Apparent frequency after sampling f at fs.", "f:n fs:n", alias),
            t("twos", "Two's complement representation.", "x:n bits:n", twos),
            t("uart_time", "Seconds to send n bytes (start + data + parity + stop bits).",
              "n_bytes:n baud:n data:n=8 parity:n=0 stop:n=1", uart_time),
            t("can_frame_bits", "Bits per classic CAN frame (incl. IFS). stuffing=1 adds worst-case stuff bits.",
              "data_bytes:n=8 extended:n=0 ifs:n=3 stuffing:n=0", can_frame_bits),
            t("db_sum", "Incoherent sum of sound levels: db_sum 90,92,85.", "levels:s", db_sum),
            t("nodal", "Exact DC nodal analysis. Elements separated by ';': R<name> a b ohms | V<name> plus minus volts | I<name> from to amps. Node 0 is ground. ask=Rab:a,b gives the equivalent resistance. Example: nodal \"R1 1 2 3; R2 2 0 3; R3 1 0 3; V1 1 0 10\".",
              "netlist:s ask:s=", nodal),
            t("two_dof", "Quarter car: sprung m1 on k1 (spring) over unsprung m2 on k2 (tyre). Natural frequencies [Hz].",
              "m1:n m2:n k1:n k2:n", two_dof),
            t("gauge", "Convert pressure between gauge and absolute (Pa in, Pa out; type 2bar).", "p:n to:s=abs p_atm:n=101325", gauge),
            t("gear_train", "Output speed of a gear train: teeth = driver/driven,driver/driven (e.g. 20/45,24/55). Same unit as n_in.",
              "n_in:n teeth:s", gear_train),
            t("accel_then_cruise", "Time to cover s from rest accelerating at a up to v_max, then constant v_max.",
              "s:n a:n v_max:n", accel_then_cruise),
            t("round_to", "Round to a step: mode nearest | down | up (fuses/limits round down, 'at least' rounds up).",
              "x:n step:n=1 mode:s=nearest", round_to),
        ]
    })
}

pub fn tool(name: &str) -> Option<&'static Tool> {
    tools().iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(name: &str, args: &[&str]) -> String {
        tool(name).unwrap().call(args).unwrap()
    }

    #[test]
    fn scoring_reproduces_quiz_keys() {
        assert_eq!(
            run("event_score", &["skidpad", "5.6", "5.1", "legacy"]),
            "41.117"
        );
        assert_eq!(
            run("event_score", &["autocross", "74.2", "62.5", "legacy"]),
            "24.7069"
        );
        assert_eq!(
            run(
                "event_score",
                &["endurance", "1672", "1345", "legacy", "finish=0"]
            ),
            "65.1348"
        );
        assert!(run(
            "cones_from_score",
            &["autocross", "73.1", "86.3", "87.1", "legacy"]
        )
        .contains("-> 3 DOO"));
        assert_eq!(
            run(
                "corrected_time",
                &["1760", "event=endurance", "out_of_order=1", "post=B"]
            ),
            "2000"
        );
        assert_eq!(
            run(
                "static_nonfinalist",
                &["59", "67", "bpp", "legacy", "n_finalists=6"]
            ),
            "60.7612"
        );
        assert_eq!(
            run(
                "max_points",
                &["bpp,cost,skidpad,accel,efficiency", "legacy"]
            ),
            "425"
        );
    }

    #[test]
    fn rules_2027() {
        assert_eq!(run("event_score", &["autocross", "72", "60"]), "32.5");
        assert_eq!(run("event_score", &["dc_accel", "50", "50"]), "75");
        assert_eq!(
            run("static_nonfinalist", &["55", "61"]),
            g6(65.0 * 55.0 / 61.0)
        );
        assert_eq!(run("weight_penalty", &["6.2"]), "40");
        assert!(run("ts_rules", &["550"]).starts_with("insulation test 1000 V"));
        assert!(run("ts_rules", &["550", "rules=2026"]).starts_with("insulation test 500 V"));
        assert_eq!(run("pcb_spacing", &["400"]), "20");
        assert_eq!(run("pcb_spacing", &["237", "surface", "legacy"]), "9.5");
        assert_eq!(run("trackdrive_score", &["100", "100"]), "200");
        assert_eq!(
            run("efficiency_score", &["t=1000", "e=10", "ef_min=1e6"]),
            "0"
        );
    }

    #[test]
    fn helpers() {
        assert_eq!(run("segment_max_cells", &["4.2", "4.2V*20Ah"]), "19");
        assert_eq!(
            run("e_series", &["2411"]),
            "below 2200  nearest 2200  above 2700"
        );
        assert_eq!(run("twos", &["-20", "15"]), "111111111101100");
        assert_eq!(run("uart_time", &["200", "115200"]), "0.0173611");
        assert_eq!(run("alias", &["400", "340"]), "60");
        assert_eq!(
            run("two_dof", &["800", "60", "17000", "180000"]),
            "0.7011 Hz  9.122 Hz"
        );
        assert_eq!(
            run("gear_train", &["3600", "20/45,24/55,30/43,15/53"]),
            "137.86"
        );
        assert_eq!(run("accel_then_cruise", &["75", "4", "56km/h"]), "6.76587");
        assert!(run("gamma", &["50.13", "-40.84"]).contains("VSWR 2.2137"));
    }

    #[test]
    fn nodal_is_exact() {
        let out = run("nodal", &["R1 1 2 3; R2 2 0 3; R3 1 0 3; V1 1 0 10"]);
        assert!(out.contains("V(2) = 5 = 5 V"), "{out}");
        assert!(out.contains("I(R1, 1->2) = 5/3 = 1.66667 A"), "{out}");
        assert!(out.contains("I(V1, into + terminal) = -5 A"), "{out}");
        assert_eq!(
            run("nodal", &["R1 a b 3; R2 b 0 3; R3 a 0 3", "ask=Rab:a,0"]),
            "R_a0 = 2 = 2 ohm"
        );
        // 6/7 style fraction survives (bridge-like ladder)
        let out = run("nodal", &["R1 a b 1; R2 b 0 2; R3 a 0 4; V1 a 0 1", ""]);
        assert!(out.contains("V(b) = 2/3"), "{out}");
    }
}
