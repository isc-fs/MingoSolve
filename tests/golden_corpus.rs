//! Golden corpus: for every case where the Python engine found the hidden truth, Rust must find it too,
//! and every root Rust returns must satisfy the formula. Reviewed exceptions live in known_differences.toml.

use std::collections::{HashMap, HashSet};

use fsq::engine::{conflicts_in, solve_step};
use fsq::registry::registry;
use serde::Deserialize;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    formula: String,
    given: HashMap<String, f64>,
    hidden: Vec<String>,
    truth: HashMap<String, f64>,
    python_finds_truth: bool,
}

#[derive(Deserialize)]
struct Known {
    differences: HashMap<String, String>,
}

#[test]
fn rust_finds_every_truth_python_found() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/solve_corpus.json"
    );
    let Ok(text) = std::fs::read_to_string(path) else {
        panic!("missing {path}: run legacy/python/scripts/golden_corpus.py");
    };
    let corpus: Corpus = serde_json::from_str(&text).unwrap();
    let known: Known = toml::from_str(include_str!("golden/known_differences.toml")).unwrap();
    let allowed: HashSet<&String> = known.differences.keys().collect();
    let mut misses = Vec::new();
    let mut bad_roots = Vec::new();
    let mut rust_better = 0;
    let mut degenerate = 0;
    for c in &corpus.cases {
        let f = registry().formula(&c.formula).unwrap();
        // a case is only meaningful if its truth point satisfies the formula and nothing underflowed: sampling is
        // log-uniform, so an exact 0 (V_c = V_0 exp(-20800)) means any small C or large t "solves" it numerically
        let mut point = c.given.clone();
        point.extend(c.truth.clone());
        if point.values().any(|v| *v == 0.0) || !conflicts_in(f, &point, 1e-6).is_empty() {
            degenerate += 1;
            continue;
        }
        // as the app and the CLI do: variables with a default (g, mu_0...) take it when the case doesn't give them
        let mut given = c.given.clone();
        for n in &f.names {
            if !given.contains_key(n) && !c.hidden.contains(n) {
                if let Some(d) = registry().var(n).default {
                    given.insert(n.clone(), d);
                }
            }
        }
        let found = solve_step(f, &given);
        let hits = c.hidden.iter().all(|n| {
            found.get(n).is_some_and(|rs| {
                rs.iter()
                    .any(|r| (r - c.truth[n]).abs() <= 1e-6 * c.truth[n].abs().max(1.0))
            })
        });
        let key = format!("{}:{}", c.formula, c.hidden.join("+"));
        if c.python_finds_truth && !hits && !allowed.contains(&key) {
            misses.push(format!("{key} truth={:?} rust={:?}", c.truth, found.0));
        }
        if !c.python_finds_truth && hits {
            rust_better += 1;
        }
        // every carried-forward root must satisfy the whole formula
        let mut vals = given.clone();
        for (n, rs) in &found.0 {
            vals.insert(n.clone(), rs[0]);
        }
        if f.names.iter().all(|n| vals.contains_key(n)) && !conflicts_in(f, &vals, 1e-6).is_empty()
        {
            bad_roots.push(format!("{key}: {:?}", conflicts_in(f, &vals, 1e-6)));
        }
    }
    eprintln!(
        "{} cases ({degenerate} degenerate skipped), {} misses, {} bad roots, {rust_better} where Rust finds a truth Python missed",
        corpus.cases.len(),
        misses.len(),
        bad_roots.len()
    );
    assert!(
        misses.is_empty(),
        "Rust misses truths Python found:\n{}",
        misses.join("\n")
    );
    assert!(
        bad_roots.is_empty(),
        "roots that violate the formula:\n{}",
        bad_roots.join("\n")
    );
}
