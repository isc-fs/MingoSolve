//! Text commands shared by the `fsq` binary and the tests: find, show, solve (or just `<formula> k=v`), chain,
//! calc, tools, ex, vars. Same syntax and output shape as the Python engine. `@var=unit` picks a display unit.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::engine::{self, conflicts};
use crate::format::g6;
use crate::registry::registry;
use crate::tools::{tool, tools};
use crate::units::{self, Dims, Quantity};

pub const HELP: &str = "\
  find <words>                       search formulas (key, title, tags, variable descriptions)
  show <formula>                     equations and variables of a formula
  <formula> k=v ... [@x=unit]        solve a formula for whatever is missing (also: solve <formula> ...)
  chain <target> k=v ... [@x=unit] [only=tag,tag]
                                     chain all formulas until <target> is found
  calc <expr> [-> unit]              unit-aware calculator: calc 0.5*300kg*(100km/h)**2 -> kJ
  tools [words]                      list procedural tools (scoring, rule tables, circuits...)
  <tool> args | k=v ...              run a tool: event_score skidpad t_team=5.6 t_min=5.1 rules=legacy
  ex <words>                         past quiz questions solved with fsq (how to enter them)
  vars [words]                       list variable names, units and meaning
values take units: v=100km/h  n=8000rpm  m=280kg  (bare numbers are in the variable's SI unit)";

#[derive(Deserialize)]
pub struct Example {
    pub id: u32,
    pub what: String,
    pub cmd: String,
    pub answer: f64,
}

#[derive(Deserialize)]
struct Examples {
    example: Vec<Example>,
}

pub fn examples() -> Vec<Example> {
    toml::from_str::<Examples>(crate::data::EXAMPLES)
        .expect("data/examples.toml")
        .example
}

/// `m/s**2` -> `m/s²`, `ohm` -> `Ω`, `N*m` -> `N·m`, `dimensionless` -> ``.
pub fn pretty_unit(u: &str) -> String {
    if u == "dimensionless" {
        return String::new();
    }
    u.replace("**2", "²")
        .replace("**3", "³")
        .replace("**4", "⁴")
        .replace("**-1", "⁻¹")
        .replace("ohm", "Ω")
        .replace('*', "·")
}

/// A variable's value (in its own unit) as text, optionally converted to `display`.
pub fn fmt_var(name: &str, value: f64, display: Option<&str>) -> Result<String, String> {
    let v = registry().var(name);
    let Some(target) = display else {
        return Ok(format!("{} {}", g6(value), pretty_unit(&v.unit))
            .trim_end()
            .to_string());
    };
    let from = units::unit_of(&v.unit).map_err(|e| e.0)?;
    let to = units::unit_of(target).map_err(|e| e.0)?;
    if from.dims != to.dims {
        return Err(format!("cannot show {name} ({}) in {target}", v.unit));
    }
    Ok(format!(
        "{} {}",
        g6(value * from.value / to.value),
        pretty_unit(target)
    )
    .trim_end()
    .to_string())
}

const NAMED: &[(&str, Dims)] = &[
    ("m", [1, 0, 0, 0, 0, 0, 0]),
    ("kg", [0, 1, 0, 0, 0, 0, 0]),
    ("s", [0, 0, 1, 0, 0, 0, 0]),
    ("A", [0, 0, 0, 1, 0, 0, 0]),
    ("K", [0, 0, 0, 0, 1, 0, 0]),
    ("N", [1, 1, -2, 0, 0, 0, 0]),
    ("Pa", [-1, 1, -2, 0, 0, 0, 0]),
    ("J", [2, 1, -2, 0, 0, 0, 0]),
    ("W", [2, 1, -3, 0, 0, 0, 0]),
    ("V", [2, 1, -3, -1, 0, 0, 0]),
    ("Ω", [2, 1, -3, -2, 0, 0, 0]),
    ("C", [0, 0, 1, 1, 0, 0, 0]),
    ("F", [-2, -1, 4, 2, 0, 0, 0]),
    ("H", [2, 1, -2, -2, 0, 0, 0]),
    ("Hz", [0, 0, -1, 0, 0, 0, 0]),
    ("m/s", [1, 0, -1, 0, 0, 0, 0]),
    ("m/s²", [1, 0, -2, 0, 0, 0, 0]),
    ("m²", [2, 0, 0, 0, 0, 0, 0]),
    ("m³", [3, 0, 0, 0, 0, 0, 0]),
    ("N·m", [2, 1, -2, 0, 0, 0, 0]),
];

fn show_quantity(q: Quantity) -> String {
    match NAMED.iter().find(|(_, d)| *d == q.dims) {
        Some((n, _)) => format!("{} {n}", g6(q.value)),
        None if q.dims == units::DIMENSIONLESS => g6(q.value),
        None => format!("{} {}", g6(q.value), units::dims_str(q.dims)),
    }
}

struct Parsed<'a> {
    given: Vec<(&'a str, &'a str)>,
    display: HashMap<&'a str, &'a str>,
    only: Option<HashSet<String>>,
}

fn parse_args<'a>(args: &[&'a str]) -> Parsed<'a> {
    let mut p = Parsed {
        given: vec![],
        display: HashMap::new(),
        only: None,
    };
    for a in args {
        let (k, v) = a.split_once('=').unwrap_or((a, ""));
        if k == "only" {
            p.only = Some(v.split(',').map(String::from).collect());
        } else if let Some(k) = k.strip_prefix('@') {
            p.display.insert(k, v);
        } else if !v.is_empty() && v != "?" {
            p.given.push((k, v));
        }
    }
    p
}

/// Split like a shell: whitespace separates, double quotes group.
fn split(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut any = false;
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            c if c.is_whitespace() && !quoted => {
                if any || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    any = false;
                }
            }
            c => cur.push(c),
        }
    }
    if any || !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn show(key: &str) -> Result<String, String> {
    let f = registry()
        .formula(key)
        .ok_or_else(|| format!("no formula {key}"))?;
    let mut out = vec![format!("{}: {}", f.key, f.title)];
    out.extend(f.src.iter().map(|s| format!("    {s}")));
    for n in &f.names {
        let v = registry().var(n);
        out.push(format!("  {n:<10} [{}] {}", v.unit, v.desc));
    }
    if !f.notes.is_empty() {
        out.push(format!("  note: {}", f.notes));
    }
    Ok(out.join("\n"))
}

fn solve_cmd(key: &str, args: &[&str]) -> Result<String, String> {
    let p = parse_args(args);
    let r = engine::solve(key, &p.given)?;
    let mut out = Vec::new();
    let mut defaults: Vec<_> = r.defaults.iter().collect();
    defaults.sort_by(|a, b| a.0.cmp(b.0));
    for (n, v) in defaults {
        out.push(format!("  (default {n} = {})", fmt_var(n, *v, None)?));
    }
    if r.found.0.is_empty() {
        out.push("  nothing determinable from these inputs:".into());
        out.push(show(key)?);
    }
    for (n, vals) in &r.found.0 {
        let shown: Result<Vec<String>, String> = vals
            .iter()
            .map(|x| fmt_var(n, *x, p.display.get(n.as_str()).copied()))
            .collect();
        let flag = if vals.len() > 1 {
            "   <- several roots"
        } else {
            ""
        };
        out.push(format!(
            "  {n} = {}{flag}   ({})",
            shown?.join(" | "),
            registry().var(n).desc
        ));
    }
    Ok(out.join("\n"))
}

fn chain_cmd(target: &str, args: &[&str]) -> Result<String, String> {
    let p = parse_args(args);
    let c = engine::chain(target, &p.given, p.only.as_ref())?;
    let mut out = Vec::new();
    let mut defaults: Vec<_> = c.defaults.iter().collect();
    defaults.sort_by(|a, b| a.0.cmp(b.0));
    for (n, v) in defaults {
        out.push(format!("  (default {n} = {})", fmt_var(n, *v, None)?));
    }
    for s in &c.steps {
        out.push(format!(
            "  [{}] {} = {}",
            s.formula,
            s.var,
            fmt_var(&s.var, s.values[0], p.display.get(s.var.as_str()).copied())?
        ));
    }
    if c.reached {
        out.push(format!(
            "  => {target} = {}",
            fmt_var(target, c.known[target], p.display.get(target).copied())?
        ));
    } else {
        let mut k: Vec<&String> = c.known.keys().collect();
        k.sort();
        out.push(format!(
            "  could not reach {target}; known: {}",
            k.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    for x in conflicts(&c.known) {
        out.push(format!("  ! inconsistent: {x}"));
    }
    Ok(out.join("\n"))
}

fn calc(expr: &str) -> Result<String, String> {
    let (e, unit) = match expr.split_once("->") {
        Some((e, u)) => (e.trim(), Some(u.trim())),
        None => (expr.trim(), None),
    };
    let q = units::parse_quantity(e).map_err(|e| e.0)?;
    match unit {
        Some(u) => {
            let to = units::unit_of(u).map_err(|e| e.0)?;
            if to.dims != q.dims {
                return Err(format!("result is {}, not {u}", units::dims_str(q.dims)));
            }
            Ok(format!("  {} {}", g6(q.value / to.value), pretty_unit(u)))
        }
        None => Ok(format!("  {}", show_quantity(q))),
    }
}

/// Run one command line and return its output (errors as text, like the Python REPL).
pub fn run(line: &str) -> String {
    match run_inner(line) {
        Ok(s) => s,
        Err(e) => format!("  error: {e}"),
    }
}

fn run_inner(line: &str) -> Result<String, String> {
    let parts = split(line);
    let Some((cmd, rest)) = parts.split_first() else {
        return Ok(String::new());
    };
    let args: Vec<&str> = rest.iter().map(String::as_str).collect();
    let r = registry();
    match cmd.as_str() {
        "help" | "?" => Ok(HELP.into()),
        "find" => Ok(r
            .search(&args.join(" "))
            .iter()
            .map(|f| format!("  {:<28} {}   ({})", f.key, f.title, f.names.join(", ")))
            .collect::<Vec<_>>()
            .join("\n")),
        "show" => show(args.first().ok_or("show <formula>")?),
        "vars" => {
            let w = args.join(" ").to_lowercase();
            let mut v: Vec<_> = r
                .vars
                .values()
                .filter(|v| format!("{} {}", v.name, v.desc).to_lowercase().contains(&w))
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(v.iter()
                .map(|v| format!("  {:<12} [{}] {}", v.name, v.unit, v.desc))
                .collect::<Vec<_>>()
                .join("\n"))
        }
        "solve" => solve_cmd(args.first().ok_or("solve <formula> k=v ...")?, &args[1..]),
        "chain" => chain_cmd(args.first().ok_or("chain <target> k=v ...")?, &args[1..]),
        "calc" => calc(&args.join(" ")),
        "tools" => {
            let w = args.join(" ").to_lowercase();
            Ok(tools()
                .iter()
                .filter(|t| format!("{} {}", t.name, t.doc).to_lowercase().contains(&w))
                .map(|t| format!("  {}\n      {}", t.signature(), t.doc))
                .collect::<Vec<_>>()
                .join("\n"))
        }
        "ex" => {
            let words: Vec<String> = args.iter().map(|w| w.to_lowercase()).collect();
            Ok(examples()
                .iter()
                .filter(|e| {
                    let hay = format!("q{} {} {}", e.id, e.what, e.cmd).to_lowercase();
                    words.iter().all(|w| hay.contains(w.as_str()))
                })
                .map(|e| {
                    format!(
                        "  Q{}: {}  -> {}\n      {}",
                        e.id,
                        e.what,
                        g6(e.answer),
                        e.cmd
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
        name if r.formula(name).is_some() => solve_cmd(name, &args),
        name => match tool(name) {
            Some(t) => t.call(&args).map(|s| format!("  {s}")),
            None => Err(format!("unknown command {name:?}; try help")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_like_a_shell() {
        assert_eq!(
            split(r#"nodal "R1 1 2 3; R2 2 0 3" ask=x"#),
            ["nodal", "R1 1 2 3; R2 2 0 3", "ask=x"]
        );
        assert_eq!(
            split(r#"nodal "R1 a 0 3" ask="""#),
            ["nodal", "R1 a 0 3", "ask="]
        );
    }

    #[test]
    fn output_shapes() {
        assert_eq!(
            run("speed s=75m t=3.8s @v_avg=km/h"),
            "  v_avg = 71.0526 km/h   (average speed over a distance)"
        );
        assert_eq!(run("calc 0.5*300kg*(100km/h)**2 -> kJ"), "  115.741 kJ");
        assert_eq!(run("calc 0.5*300kg*(100km/h)**2"), "  115741 J");
        assert!(run("chain t v0=0 s=75 v=100km/h").ends_with("=> t = 5.4 s"));
        assert!(
            run("battery_load N_s=103 V_cell=3.8V R_pack=0.08ohm P=30kW")
                .contains("<- several roots")
        );
        assert!(run("bogus").contains("unknown command"));
    }
}
