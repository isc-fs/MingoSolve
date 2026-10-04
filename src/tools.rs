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
        let out = (self.run)(&Args(map))?;
        if out
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .any(|w| matches!(w, "NaN" | "inf" | "-inf"))
        {
            return Err(units::NOT_FINITE.into());
        }
        Ok(out)
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
        .ok_or("above 600 V: TS max is 600 V (EV 4.1.1)")?;
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
    let (value, series) = (a.num("value")?, a.num("series")?);
    if value <= 0.0 {
        return Err("value must be positive".into());
    }
    if series.fract() != 0.0 || !(1.0..=1000.0).contains(&series) {
        return Err("series must be a whole number from 1 to 1000".into());
    }
    let series = series as usize;
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
    let (x, bits) = (a.num("x")?, a.num("bits")?);
    if bits.fract() != 0.0 || !(1.0..=64.0).contains(&bits) {
        return Err("bits must be a whole number from 1 to 64".into());
    }
    let (x, bits) = (x as i64, bits as u32);
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
    Ok(g6(frame_bits(a)?))
}

fn frame_bits(a: &Args) -> Result<f64, String> {
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
    Ok(base + extra + a.num("ifs")?.floor())
}

fn can_transfer(a: &Args) -> Result<String, String> {
    let fb = frame_bits(a)?;
    let frames = (a.num("bits")? / (8.0 * a.num("data_bytes")?.floor()) - 1e-9).ceil();
    Ok(format!(
        "{} frames x {} bit  time {} s",
        g6(frames),
        g6(fb),
        g6(frames * fb / a.num("bitrate")?)
    ))
}

fn charge_time(a: &Args) -> Result<String, String> {
    let (p_ch, p_i) = (a.num("p_charger")?, a.num("i_set")? * a.num("v_batt")?);
    let p = p_ch.min(p_i);
    let t = (a.num("soc1")? - a.num("soc0")?) * a.num("e_nom")? / p;
    let limit = if p_ch <= p_i {
        "charger power"
    } else {
        "current setpoint"
    };
    Ok(format!(
        "power {} W ({limit} limit)  time {} s = {} min = {} h",
        g6(p),
        g6(t),
        g6(t / 60.0),
        g6(t / 3600.0)
    ))
}

fn packs_needed(a: &Args) -> Result<String, String> {
    let q = |k: &str| units::parse_quantity(a.str(k)).map_err(|e| format!("{k}: {e}"));
    let (need, per) = (q("need")?, q("per_pack")?);
    if need.dims != per.dims {
        return Err("need and per_pack must have the same unit (both Ah or both kWh)".into());
    }
    let x = need.value / (per.value * a.num("usable")?);
    Ok(format!("{} packs -> {}", g6(x), g6((x - 1e-9).ceil())))
}

fn cable_fuse(a: &Args) -> Result<String, String> {
    let (area, t_ins, t_max) = (a.num("a")?, a.num("t_ins")?, a.num("t_max")?);
    let rho_t = a.num("rho")? * (1.0 + a.num("alpha")? * (t_max - a.num("t0")?));
    let d_avg = (4.0 * area / std::f64::consts::PI).sqrt() + t_ins;
    let q = a.num("lam")? * std::f64::consts::PI * d_avg * (t_max - a.num("t_amb")?) / t_ins;
    let i = (q * area / rho_t).sqrt();
    let step = a.num("step")?;
    Ok(format!(
        "I_max {} A -> fuse {} A",
        g6(i),
        g6((i / step + 1e-9).floor() * step)
    ))
}

fn ts_breaker(a: &Args) -> Result<String, String> {
    let (v0, r_i) = (a.num("v0")?, a.num("r_i")?);
    let r_w = a.num("k_wire")? * 2.0 * a.num("l")? / (a.num("kappa")? * a.num("a")?);
    let i_min = v0 / (r_i + r_w);
    let i_max = v0 / r_i;
    let (s_in, s_cn) = (a.num("step_in")?, a.num("step_icn")?);
    let i_n = (i_min / a.num("k_trip")? / s_in + 1e-9).floor() * s_in;
    let i_cn = (i_max / s_cn - 1e-9).ceil() * s_cn;
    Ok(format!(
        "R_wire {} ohm  I_sc min {} A -> In {} A  I_sc max {} A -> Icn {} A",
        g6(r_w),
        g6(i_min),
        g6(i_n),
        g6(i_max),
        g6(i_cn)
    ))
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
    if s.len() > 200 {
        return Err("number too long".into());
    }
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i32>().map_err(|_| format!("bad number {s}"))?),
        None => (s, 0),
    };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits: BigInt = format!("{int}{frac}")
        .parse()
        .map_err(|_| format!("bad number {s}"))?;
    if exp.abs() > 300 {
        return Err(format!("exponent of {s} is out of range"));
    }
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

/// Exact rational elimination is cubic in this; quiz circuits have a handful.
const MAX_NODAL_UNKNOWNS: usize = 60;

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
    if size > MAX_NODAL_UNKNOWNS {
        return Err(format!(
            "netlist too large (more than {MAX_NODAL_UNKNOWNS} unknowns)"
        ));
    }
    let zero = BigRational::zero();
    let mut m = vec![vec![zero.clone(); size + 1]; size];
    let idx = |n: &str| nodes.iter().position(|x| x == n);
    for (k, e) in elems.iter().enumerate() {
        let val = exact(&e[3])?;
        let (na, nb) = (idx(&e[1]), idx(&e[2]));
        match e[0].to_uppercase().chars().next().unwrap() {
            'R' => {
                if val.is_zero() {
                    return Err(format!(
                        "{} has zero resistance: use a 0 V source for a short",
                        e[0]
                    ));
                }
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

fn wheel_slip(a: &Args) -> Result<String, String> {
    let deg = 180.0 / std::f64::consts::PI;
    let (vx, half_f, a_cg, b_cg, delta) = (
        a.num("vx")?,
        a.num("t")? / 2.0,
        a.num("a")?,
        a.num("b")?,
        a.num("delta")?,
    );
    let half_r = match a.num("t_r")? {
        x if x > 0.0 => x / 2.0,
        _ => half_f,
    };
    let (toe_f, toe_r) = (a.num("toe_f")?, a.num("toe_r")?);
    let yaw = match a.num("R")? {
        r if r != 0.0 => vx / r,
        _ => a.num("yaw")?,
    };
    // name, x ahead of CG, y left of CG, road-wheel angle (toe-in turns a left wheel right)
    let wheels = [
        ("FL", a_cg, half_f, delta - toe_f),
        ("FR", a_cg, -half_f, delta + toe_f),
        ("RL", -b_cg, half_r, -toe_r),
        ("RR", -b_cg, -half_r, toe_r),
    ];
    let mut vy = a.num("vy")?;
    let known = a.str("known").to_uppercase();
    if !known.is_empty() {
        let &(_, x, y, d) = wheels
            .iter()
            .find(|w| w.0 == known)
            .ok_or("known: FL, FR, RL or RR")?;
        vy = (vx - yaw * y) * (d - a.num("alpha")?).tan() - yaw * x;
    }
    let mut out = vec![format!("yaw {} rad/s  vy(CG) {} m/s", g6(yaw), g6(vy))];
    let mut drift = HashMap::new();
    for (name, x, y, d) in wheels {
        let (vxw, vyw) = (vx - yaw * y, vy + yaw * x);
        let beta = vyw.atan2(vxw);
        drift.insert(name, beta);
        out.push(format!(
            "{name}  vx {} m/s  vy {} m/s  wheel angle {} deg  slip {} deg",
            g6(vxw),
            g6(vyw),
            g6(d * deg),
            g6((d - beta) * deg)
        ));
    }
    out.push(format!(
        "toe for equal front slips {} deg, rear {} deg (toe-in +)",
        g6((drift["FR"] - drift["FL"]) / 2.0 * deg),
        g6((drift["RR"] - drift["RL"]) / 2.0 * deg)
    ));
    Ok(out.join("\n  "))
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

fn qty(s: &str) -> Result<f64, String> {
    units::parse_quantity(s)
        .map(|q| q.value)
        .map_err(|e| format!("{s}: {e}"))
}

fn items(s: &str) -> Vec<Vec<&str>> {
    s.split([';', '\n'])
        .map(|e| e.split_whitespace().collect::<Vec<_>>())
        .filter(|e| !e.is_empty())
        .collect()
}

/// Solve a dense square system by Gaussian elimination with partial pivoting.
fn solve_dense(mut m: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    let scale = m.iter().flatten().fold(0.0f64, |a, x| a.max(x.abs()));
    for c in 0..n {
        let p = (c..n).max_by(|&i, &j| m[i][c].abs().total_cmp(&m[j][c].abs()))?;
        if m[p][c].abs() <= 1e-12 * scale {
            return None;
        }
        m.swap(c, p);
        rhs.swap(c, p);
        let pivot = m[c].clone();
        for r in 0..n {
            let k = m[r][c] / pivot[c];
            if r != c && k != 0.0 {
                for (x, pv) in m[r][c..].iter_mut().zip(&pivot[c..]) {
                    *x -= k * pv;
                }
                rhs[r] -= k * rhs[c];
            }
        }
    }
    Some((0..n).map(|i| rhs[i] / m[i][i]).collect())
}

fn truss(a: &Args) -> Result<String, String> {
    let mut names: Vec<&str> = Vec::new();
    let mut xy: Vec<(f64, f64)> = Vec::new();
    for n in items(a.str("nodes")) {
        let [name, x, y] = n[..] else {
            return Err("nodes: <name> <x> <y>; ...".into());
        };
        names.push(name);
        xy.push((qty(x)?, qty(y)?));
    }
    let node = |s: &str| {
        names
            .iter()
            .position(|n| *n == s)
            .ok_or_else(|| format!("unknown node {s}"))
    };
    let mut members = Vec::new();
    for m in a
        .str("members")
        .split([',', ';', ' ', '\n'])
        .filter(|m| !m.is_empty())
    {
        let (i, j) = m.split_once('-').ok_or("members like A-B C-D")?;
        let (i, j) = (node(i)?, node(j)?);
        let (dx, dy) = (xy[j].0 - xy[i].0, xy[j].1 - xy[i].1);
        let len = dx.hypot(dy);
        if len == 0.0 {
            return Err(format!("member {m} has zero length"));
        }
        members.push((m, i, j, dx / len, dy / len));
    }
    // reaction components: (node, ux, uy)
    let mut reactions = Vec::new();
    for s in items(a.str("supports")) {
        let [n, kind] = s[..] else {
            return Err("supports: <node> pin|roller|rollerx; ...".into());
        };
        let k = node(n)?;
        match kind {
            "pin" => reactions.extend([(k, 1.0, 0.0), (k, 0.0, 1.0)]),
            "roller" | "rollery" => reactions.push((k, 0.0, 1.0)),
            "rollerx" => reactions.push((k, 1.0, 0.0)),
            _ => {
                return Err(format!(
                    "support {kind}: use pin, roller (vertical reaction) or rollerx"
                ))
            }
        }
    }
    let nn = names.len();
    let mut rhs = vec![0.0; 2 * nn];
    for l in items(a.str("loads")) {
        let (k, fx, fy) = match l[..] {
            [n, polar] if polar.contains('@') => {
                let (mag, ang) = polar.split_once('@').unwrap();
                let (mag, ang) = (qty(mag)?, qty(ang)?.to_radians());
                (node(n)?, mag * ang.cos(), mag * ang.sin())
            }
            [n, fx, fy] => (node(n)?, qty(fx)?, qty(fy)?),
            _ => return Err("loads: <node> <Fx> <Fy> or <node> <F>@<deg from +x>; ...".into()),
        };
        rhs[2 * k] -= fx;
        rhs[2 * k + 1] -= fy;
    }
    let unknowns = members.len() + reactions.len();
    if unknowns != 2 * nn {
        return Err(format!(
            "{} members + {} reactions != 2 x {nn} joints: not statically determinate",
            members.len(),
            reactions.len()
        ));
    }
    // joint equilibrium: each member pulls its end joints towards each other when in tension
    let mut m = vec![vec![0.0; unknowns]; 2 * nn];
    for (c, &(_, i, j, ux, uy)) in members.iter().enumerate() {
        m[2 * i][c] += ux;
        m[2 * i + 1][c] += uy;
        m[2 * j][c] -= ux;
        m[2 * j + 1][c] -= uy;
    }
    for (r, &(k, ux, uy)) in reactions.iter().enumerate() {
        let c = members.len() + r;
        m[2 * k][c] += ux;
        m[2 * k + 1][c] += uy;
    }
    let f = solve_dense(m, rhs)
        .ok_or("singular: the truss is a mechanism (or supports are collinear)")?;
    let clean = |x: f64| {
        if x.abs() < 1e-9 * (1.0 + f.iter().fold(0.0f64, |a, v| a.max(v.abs()))) {
            0.0
        } else {
            x
        }
    };
    let mut out: Vec<String> = members
        .iter()
        .zip(&f)
        .map(|(mb, &v)| {
            let v = clean(v);
            let tag = if v > 0.0 {
                "tension"
            } else if v < 0.0 {
                "compression"
            } else {
                "zero-force"
            };
            format!("{} = {}  ({tag})", mb.0, g6(v))
        })
        .collect();
    for (r, &(k, ux, _)) in reactions.iter().enumerate() {
        let dir = if ux != 0.0 { "x" } else { "y" };
        out.push(format!(
            "R_{}{dir} = {}",
            names[k],
            g6(clean(f[members.len() + r]))
        ));
    }
    Ok(out.join("\n  "))
}

fn endurance_energy(a: &Args) -> Result<String, String> {
    let (m, g, rho, v) = (a.num("m")?, a.num("g")?, a.num("rho")?, a.num("v")?);
    let dist = a.num("laps")? * a.num("lap")?;
    let q = rho * v * v / 2.0;
    let f_drag = q * a.num("cda")?.abs();
    let normal = m * g + q * a.num("cla")?.abs();
    let f_roll = a.num("mu_r")? * normal;
    let mut e_brake_lap = 0.0;
    for ev in a
        .str("brakes")
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
    {
        let bad =
            || format!("brake event {ev:?}: use <count>x<v_from>-<v_to> in km/h, e.g. 3x75-40");
        let (n, speeds) = ev.split_once(['x', '*']).ok_or_else(bad)?;
        let (v1, v2) = speeds.split_once('-').ok_or_else(bad)?;
        let p = |s: &str| s.trim().parse::<f64>().map_err(|_| bad());
        let (n, v1, v2) = (p(n)?, p(v1)? / 3.6, p(v2)? / 3.6);
        e_brake_lap += n * m * (v1 * v1 - v2 * v2) / 2.0;
    }
    let e_road = dist * (f_drag + f_roll);
    let e_wheel = e_road + a.num("laps")? * e_brake_lap;
    let eta = a.num("eta")?;
    let e_src = e_wheel / eta;
    let kwh = |e: f64| g6(e / 3.6e6);
    let mut out = vec![
        format!(
            "drag {} N, normal load {} N, rolling {} N",
            g6(f_drag),
            g6(normal),
            g6(f_roll)
        ),
        format!(
            "road load {} MJ + braking {} kJ/lap",
            g6(e_road / 1e6),
            g6(e_brake_lap / 1e3)
        ),
        format!(
            "energy at the wheels {} MJ = {} kWh",
            g6(e_wheel / 1e6),
            kwh(e_wheel)
        ),
        format!("from the source (/eta) {} kWh", kwh(e_src)),
    ];
    let soc_min = a.num("soc_min")?;
    if soc_min > 0.0 {
        out.push(format!(
            "battery at the start (/(1 - soc_min)) {} kWh",
            kwh(e_src / (1.0 - soc_min))
        ));
    }
    let e_fuel = a.num("e_fuel")?;
    if e_fuel > 0.0 {
        let litres = (e_src / e_fuel + a.num("reserve")?) * 1e3;
        out.push(format!("fuel at the start (+ reserve) {} l", g6(litres)));
    }
    Ok(out.join("\n  "))
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
            t("can_transfer", "Classic CAN frames and time to move a payload of `bits` (best case: full 8-byte frames, no stuffing).",
              "bits:n bitrate:n data_bytes:n=8 extended:n=0 ifs:n=3 stuffing:n=0", can_transfer),
            t("charge_time", "Charging time: power = min(charger power, current setpoint x battery voltage), t = (soc1 - soc0) x E_nom / power. SoC as 0..1.",
              "e_nom:n soc0:n soc1:n p_charger:n i_set:n v_batt:n", charge_time),
            t("packs_needed", "Whole packs needed: ceil(need / (per_pack x usable)). Type need as 4.2A*8h or 15kWh; usable = depth of discharge (to 14 % -> 0.86).",
              "need:n per_pack:n usable:n=1", packs_needed),
            t("cable_fuse", "Cable ampacity (steady state, heat through the insulation): I^2 rho(T_max)/A = lam pi (d + t_ins) (T_max - T_amb)/t_ins, d = sqrt(4A/pi); fuse floored to the step. rho given at t0.",
              "a:n t_ins:n t_amb:n t_max:n lam:n rho:n alpha:n t0:n=20degC step:n=1", cable_fuse),
            t("ts_breaker", "Main breaker for a TS cable: R_wire = k_wire 2 l/(kappa A) (go + return); In = floor(V0/(Ri + R_wire)/k_trip) to step_in; Icn = ceil(V0/Ri) to step_icn. kappa in S/m (56 m/(ohm mm2) = 56e6).",
              "v0:n r_i:n l:n a:n kappa:n=56e6 k_wire:n=1.3 k_trip:n=10 step_in:n=10 step_icn:n=10000", ts_breaker),
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
            t("wheel_slip", "Velocity and slip angle of all four wheels (rigid body, x forward, y left, yaw > 0 = left turn). Angles in deg (type 10deg). Give vy, or known=RL alpha=4.6deg to back out vy from one wheel's slip. R = corner radius sets yaw = vx/R. Also prints the toe that equalises the slips on each axle.",
              "vx:n a:n b:n t:n delta:n=0 yaw:n=0 R:n=0 vy:n=0 toe_f:n=0 toe_r:n=0 t_r:n=0 known:s= alpha:n=0", wheel_slip),
            t("truss", "2D pin-jointed truss by the method of joints. nodes \"A 0 0; B 3 0; C 1.5 2\", members \"A-B B-C A-C\", supports \"A pin; B roller\" (roller = vertical reaction, rollerx = horizontal), loads \"C 0 -10kN\" or \"C 10kN@-90\" (deg from +x). Member forces: + tension, - compression.",
              "nodes:s members:s supports:s loads:s=", truss),
            t("endurance_energy", "Endurance energy budget: laps x lap at constant v for drag (cda = cd A) and rolling (mu_r on m g + downforce, cla = |cl| A), plus braking losses per lap brakes=\"1x90-30,3x75-40\" (count x km/h from-to, all dissipated). /eta to the source; battery /(1 - soc_min); fuel /(eta e_fuel) + reserve.",
              "m:n laps:n lap:n v:n cda:n cla:n=0 mu_r:n=0 brakes:s= eta:n=1 soc_min:n=0 e_fuel:n=0 reserve:n=0 rho:n=1.225 g:n=9.81", endurance_energy),
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
    fn electrical_tools_reproduce_quiz_keys() {
        // Q609 key 40 min: 12 A x 510 V = 6.12 kW > 5.5 kW, so 0.6 x 6.1 kWh / 5.5 kW = 0.6655 h = 39.93 min
        assert!(run(
            "charge_time",
            &["6.1kWh", "0.2", "0.8", "5.5kW", "12A", "510V"]
        )
        .contains("39.9273 min"));
        // Q954 key 4: 4.2 A x 8 h / (10 Ah x 0.86) = 3.907; Q997 key 3: 2.5 kW/0.63 x 235 min / 6.5 kWh = 2.39
        assert!(run("packs_needed", &["4.2A*8h", "10Ah", "0.86"]).ends_with("-> 4"));
        assert!(run("packs_needed", &["2.5kW/0.63*235min", "6.5kWh"]).ends_with("-> 3"));
        assert!(tool("packs_needed")
            .unwrap()
            .call(&["10Ah", "1kWh"])
            .is_err());
        // Q71 key 11 A: d = 1.596 mm, q = 6e-3 pi 1.956e-3 x 20/0.36e-3 = 2.048 W/m,
        // rho(70 degC) = 26.4n x 1.205, I = sqrt(2.048 x 2e-6/3.181e-8) = 11.35 A
        let out = run(
            "cable_fuse",
            &[
                "2mm**2",
                "0.36mm",
                "50degC",
                "70degC",
                "6mW/(K*m)",
                "26.4e-9",
                "0.0041",
            ],
        );
        assert!(
            out.starts_with("I_max 11.34") && out.ends_with("fuse 11 A"),
            "{out}"
        );
        // Q617 key "560, 30": R_w = 1.3 x 20/(56e6 x 6e-6) = 77.4 mOhm, 600/0.1059 = 5667 A -> 560 A,
        // 600/0.0285 = 21053 A -> 30 kA
        let out = run("ts_breaker", &["600", "28.5mohm", "10m", "6mm**2"]);
        assert!(
            out.contains("In 560 A") && out.ends_with("Icn 30000 A"),
            "{out}"
        );
        // Q675 key "580000, 129": 290 s x 128 kbit/s / 64 bit per frame, 111 bit frames at 500 kbit/s
        assert_eq!(
            run("can_transfer", &["290*128000", "500000"]),
            "580000 frames x 111 bit  time 128.76 s"
        );
    }

    #[test]
    fn wheel_slip_matches_official_answers() {
        // Q92 (official 8.9-9.3 deg): FL at x = 1.53/2, y = 0.6: vx 11.667 - 1.3*0.6 = 10.887,
        // vy -0.833 + 1.3*0.765 = 0.1612, slip 10 - atan(0.1612/10.887) = 9.152 deg
        let out = run(
            "wheel_slip",
            &[
                "vx=42km/h",
                "a=0.765",
                "b=0.765",
                "t=1.2",
                "delta=10deg",
                "yaw=1.3",
                "vy=-3km/h",
            ],
        );
        assert!(
            out.contains(
                "FL  vx 10.8867 m/s  vy 0.161167 m/s  wheel angle 10 deg  slip 9.15185 deg"
            ),
            "{out}"
        );
        // Q1001 (official -0.46): rear inside slip 4.6 deg with 0.5 deg toe-in, R = 8 m, L = 1.525 m
        let out = run(
            "wheel_slip",
            &[
                "vx=10.2",
                "R=8",
                "a=0.7625",
                "b=0.7625",
                "t=1.2",
                "delta=12deg",
                "toe_r=0.5deg",
                "known=RL",
                "alpha=4.6deg",
            ],
        );
        assert!(out.contains("RL  vx 9.435 m/s"), "{out}");
        assert!(out.contains("slip 4.6 deg"), "{out}");
        assert!(out.contains("toe for equal front slips -0.46"), "{out}");
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
    /// Parse "name = value" lines of the truss output.
    fn truss_forces(out: &str) -> HashMap<String, f64> {
        out.lines()
            .filter_map(|l| {
                let (k, v) = l.trim().split_once(" = ")?;
                Some((k.to_string(), v.split_whitespace().next()?.parse().ok()?))
            })
            .collect()
    }

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn truss_symmetric_triangle() {
        // A(0,0) pin, B(2,0) roller, apex C(1,1) with P = 10 down. Symmetry: R_A = R_B = P/2 = 5.
        // Joint A, y: R_Ay + f_AC sin45 = 0 -> f_AC = -5 sqrt2 = -7.0711 (compression);
        // x: f_AB + f_AC cos45 = 0 -> f_AB = +5 (tension tie).
        let f = truss_forces(&run(
            "truss",
            &[
                "A 0 0; B 2 0; C 1 1",
                "A-B B-C A-C",
                "A pin; B roller",
                "C 0 -10",
            ],
        ));
        assert!(close(f["A-B"], 5.0, 1e-9));
        assert!(close(f["A-C"], -50f64.sqrt(), 1e-4));
        assert!(close(f["B-C"], -50f64.sqrt(), 1e-4));
        assert!(
            close(f["R_Ay"], 5.0, 1e-9)
                && close(f["R_By"], 5.0, 1e-9)
                && close(f["R_Ax"], 0.0, 1e-9)
        );
    }

    #[test]
    fn truss_cantilever_two_bays() {
        // Wall pins at 1(0,0) and 2(0,1); bottom chord 1-3-5, top chord 2-4, verticals/diagonals 3-4, 2-3, 4-5;
        // P = 1 down at the tip 5(2,0).
        // Joint 5: y: f45/sqrt2 = P -> f45 = sqrt2 P; x: -f35 - f45/sqrt2 = 0 -> f35 = -P.
        // Joint 4: x: -f24 + f45/sqrt2 = 0 -> f24 = P; y: -f34 - f45/sqrt2 = 0 -> f34 = -P.
        // Joint 3: y: f34 + f23/sqrt2 = 0 -> f23 = sqrt2 P; x: -f13 - f23/sqrt2 + f35 = 0 -> f13 = -2P.
        // Wall: R_1x = 2P (push), R_2x = -2P (pull), R_2y = P, R_1y = 0; moment about 1: 2P x 1 = P x 2.
        let f = truss_forces(&run(
            "truss",
            &[
                "n1 0 0; n2 0 1; n3 1 0; n4 1 1; n5 2 0",
                "n1-n3 n2-n4 n3-n4 n2-n3 n3-n5 n4-n5",
                "n1 pin; n2 pin",
                "n5 0 -1",
            ],
        ));
        let r2 = 2f64.sqrt();
        for (k, v) in [
            ("n4-n5", r2),
            ("n3-n5", -1.0),
            ("n2-n4", 1.0),
            ("n3-n4", -1.0),
            ("n2-n3", r2),
            ("n1-n3", -2.0),
            ("R_n1x", 2.0),
            ("R_n1y", 0.0),
            ("R_n2x", -2.0),
            ("R_n2y", 1.0),
        ] {
            assert!(close(f[k], v, 1e-5), "{k}: {} vs {v}", f[k]);
        }
    }

    #[test]
    fn truss_reproduces_quiz_keys() {
        // Q1000 (geometry from its figure, L = 1, K = 20 N): official f_HG = -17.322 N.
        let f = truss_forces(&run(
            "truss",
            &[
                "A 0 0; C 1 0; E 3 0; G 4 0; B 0 1; D 1 2; F 3 2; H 4 1; M 2 1",
                "A-B A-C B-C B-D C-D C-M D-M D-F F-M M-E E-F F-H E-H H-G E-G",
                "A pin; G roller",
                "B 200 0; D 100@-135; F 0 20",
            ],
        ));
        assert!(close(f["H-G"], -17.322, 5e-4), "{}", f["H-G"]);
        // Q746 (figure, 3 m grid): f_AC = -25 kN fixes the pin reaction A_x = 25 kN, so sum Fx gives
        // R = 100 cos45 - 25 = 45.71 kN at H. Official solution: R_Gy = 35.355 kN, f_HG = -35.4 kN.
        let f = truss_forces(&run(
            "truss",
            &[
                "A 0 0; C 3 0; E 9 0; G 12 0; B 0 3; D 3 6; F 9 6; H 12 3; M 6 3",
                "A-B A-C B-C B-D C-D C-M D-M D-F F-M M-E E-F F-H E-H H-G E-G",
                "A pin; G roller",
                "D 0 -25; F 100@-135; H 45.71 0",
            ],
        ));
        assert!(close(f["A-C"], -25.0, 0.01), "{}", f["A-C"]);
        assert!(close(f["H-G"], -35.355, 0.01), "{}", f["H-G"]);
        assert!(close(f["R_Gy"], 35.355, 0.01));
        assert!(tool("truss")
            .unwrap()
            .call(&["A 0 0; B 1 0; C 0 1", "A-B B-C", "A pin; B roller", ""])
            .unwrap_err()
            .contains("not statically determinate"));
    }

    #[test]
    fn endurance_energy_reproduces_quiz_keys() {
        // Q1006 / Q1085 official solution: E_wheel = 15.4986 MJ; /0.69/0.9 -> 6.93 kWh; /(0.22 x 34 MJ/l) + 0.2 -> 2.27 l.
        let common = [
            "m=268",
            "laps=22",
            "lap=1000",
            "v=55km/h",
            "cda=1.3*1.1",
            "cla=-4.5*1.1",
            "mu_r=0.05",
            "brakes=1x90-30,3x75-40,4x55-25,1x65-20",
            "rho=1.2",
        ];
        let bat = run(
            "endurance_energy",
            &[&common[..], &["eta=0.69", "soc_min=0.1"]].concat(),
        );
        assert!(bat.contains("energy at the wheels 15.4986 MJ"), "{bat}");
        assert!(
            bat.contains("battery at the start (/(1 - soc_min)) 6.93265 kWh"),
            "{bat}"
        );
        let fuel = run(
            "endurance_energy",
            &[&common[..], &["eta=0.22", "e_fuel=34MJ/l", "reserve=0.2l"]].concat(),
        );
        assert!(
            fuel.contains("fuel at the start (+ reserve) 2.27201 l"),
            "{fuel}"
        );
    }
}
