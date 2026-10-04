//! Structured comparison of the Rust CLI against the Python CLI snapshot (tests/golden/cli_snapshot.json, from
//! legacy/python/scripts/cli_snapshot.py). Both outputs are parsed the same way: defaults, solved variables with
//! all roots (compared as sets: Rust orders consistent roots first), chain steps, target, conflicts, other numbers.

use std::collections::BTreeMap;

use fsq::cli::run;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq)]
struct Parsed {
    defaults: BTreeMap<String, Option<f64>>,
    solved: BTreeMap<String, Vec<Option<f64>>>,
    steps: Vec<(String, String, Option<f64>)>,
    target: Option<(String, Option<f64>)>,
    conflicts: Vec<String>,
    numbers: Vec<f64>,
    error: bool,
}

#[derive(Deserialize)]
struct Entry {
    cmd: String,
    python: Parsed,
    raw: String,
}

#[derive(Deserialize)]
struct Known {
    #[serde(default)]
    cli: BTreeMap<String, String>,
    #[serde(default)]
    rust_only_tools: Vec<String>,
}

fn parse(out: &str) -> Parsed {
    let num = Regex::new(r"-?\d+(?:\.\d+)?(?:e[-+]?\d+)?").unwrap();
    let first = |s: &str| num.find(s).map(|m| m.as_str().parse::<f64>().unwrap());
    let default = Regex::new(r"^\(default (\w+) = (.*)\)$").unwrap();
    let step = Regex::new(r"^\[(\w+)\] (\w+) = (.*)$").unwrap();
    let target = Regex::new(r"^=> (\w+) = (.*)$").unwrap();
    let solved = Regex::new(r"^(\w+) = (.*?)(?:   <- several roots)?   \(.*\)$").unwrap();
    let mut r = Parsed::default();
    for raw in out.lines() {
        let line = raw.trim();
        if let Some(c) = default.captures(line) {
            r.defaults.insert(c[1].into(), first(&c[2]));
        } else if let Some(c) = step.captures(line) {
            r.steps.push((c[1].into(), c[2].into(), first(&c[3])));
        } else if let Some(c) = target.captures(line) {
            r.target = Some((c[1].into(), first(&c[2])));
        } else if let Some(rest) = line.strip_prefix("! inconsistent:") {
            r.conflicts
                .push(rest.split(':').next().unwrap().trim().into());
        } else if line.starts_with("error:") {
            r.error = true;
        } else if let Some(c) = solved.captures(line) {
            r.solved
                .insert(c[1].into(), c[2].split(" | ").map(first).collect());
        } else {
            r.numbers.extend(
                num.find_iter(line)
                    .map(|m| m.as_str().parse::<f64>().unwrap()),
            );
        }
    }
    r
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-5 * a.abs().max(b.abs()) + 1e-12
}

fn close_opt(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => close(a, b),
        (a, b) => a == b,
    }
}

fn same_set(a: &[Option<f64>], b: &[Option<f64>]) -> bool {
    let sort = |v: &[Option<f64>]| {
        let mut v: Vec<f64> = v.iter().map(|x| x.unwrap_or(f64::NAN)).collect();
        v.sort_by(f64::total_cmp);
        v
    };
    let (a, b) = (sort(a), sort(b));
    a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| close(*x, *y))
}

fn diff(py: &Parsed, rs: &Parsed) -> Vec<String> {
    let mut d = Vec::new();
    if py.error != rs.error {
        d.push(format!("error: python {} rust {}", py.error, rs.error));
    }
    if py.defaults.keys().ne(rs.defaults.keys())
        || py
            .defaults
            .iter()
            .zip(&rs.defaults)
            .any(|((_, a), (_, b))| !close_opt(*a, *b))
    {
        d.push(format!(
            "defaults: python {:?} rust {:?}",
            py.defaults, rs.defaults
        ));
    }
    if py.solved.keys().ne(rs.solved.keys())
        || py
            .solved
            .iter()
            .zip(&rs.solved)
            .any(|((_, a), (_, b))| !same_set(a, b))
    {
        d.push(format!(
            "solved: python {:?} rust {:?}",
            py.solved, rs.solved
        ));
    }
    if py.steps.len() != rs.steps.len()
        || py
            .steps
            .iter()
            .zip(&rs.steps)
            .any(|(a, b)| a.0 != b.0 || a.1 != b.1 || !close_opt(a.2, b.2))
    {
        d.push(format!("steps: python {:?} rust {:?}", py.steps, rs.steps));
    }
    let same_target = match (&py.target, &rs.target) {
        (Some(a), Some(b)) => a.0 == b.0 && close_opt(a.1, b.1),
        (a, b) => a == b,
    };
    if !same_target {
        d.push(format!(
            "target: python {:?} rust {:?}",
            py.target, rs.target
        ));
    }
    if py.conflicts != rs.conflicts {
        d.push(format!(
            "conflicts: python {:?} rust {:?}",
            py.conflicts, rs.conflicts
        ));
    }
    if py.numbers.len() != rs.numbers.len()
        || py
            .numbers
            .iter()
            .zip(&rs.numbers)
            .any(|(a, b)| !close(*a, *b))
    {
        d.push(format!(
            "numbers: python {:?} rust {:?}",
            py.numbers, rs.numbers
        ));
    }
    d
}

#[test]
fn rust_cli_matches_python_snapshot() {
    let snap: Vec<Entry> = serde_json::from_str(include_str!("golden/cli_snapshot.json"))
        .expect("run legacy/python/scripts/cli_snapshot.py");
    let known: Known = toml::from_str(include_str!("golden/known_differences.toml")).unwrap();
    let mut failures = Vec::new();
    for e in &snap {
        let head = e.cmd.split_whitespace().next().unwrap_or("");
        if known.rust_only_tools.iter().any(|t| t == head) {
            // Python can't run it; make sure it says so rather than silently answering something else
            assert!(
                e.raw.contains("unknown command"),
                "{} is listed as Rust-only but Python ran it",
                e.cmd
            );
            continue;
        }
        let d = diff(&e.python, &parse(&run(&e.cmd)));
        if !d.is_empty() && !known.cli.contains_key(&e.cmd) {
            failures.push(format!("{}\n    {}", e.cmd, d.join("\n    ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} commands differ:\n{}",
        failures.len(),
        snap.len(),
        failures.join("\n")
    );
}
