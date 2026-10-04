//! The finder's pre-fill against the real FS-Quiz bank: for every past question that has a formula example, each
//! pre-filled variable must agree with the example's official command (given value, or the value solved from the
//! command's givens). A blank is always fine; a wrong value is a failure.
//! Bank: `FSQ_BANK` or `../IFS-Tests/data/fsquiz/bank.json`; missing locally = skipped, missing in CI = failure.

use std::collections::HashMap;
use std::path::PathBuf;

use fsq::cli::examples;
use fsq::engine::{solve, to_si};
use fsq::finder::find;
use fsq::registry::registry;

fn bank_texts() -> Option<HashMap<u32, String>> {
    let path = std::env::var("FSQ_BANK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../IFS-Tests/data/fsquiz/bank.json")
        });
    let Ok(raw) = std::fs::read_to_string(&path) else {
        assert!(
            std::env::var_os("CI").is_none(),
            "bank required in CI: {} not found",
            path.display()
        );
        eprintln!("prefill_bank: skipped, no bank at {}", path.display());
        return None;
    };
    let json: serde_json::Value = serde_json::from_str(&raw).expect("bank.json");
    Some(
        json["questions"]
            .as_array()
            .expect("questions")
            .iter()
            .filter_map(|q| {
                Some((
                    q["question_id"].as_u64()? as u32,
                    q["text"].as_str()?.to_string(),
                ))
            })
            .collect(),
    )
}

fn split(line: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted) = (Vec::new(), String::new(), false);
    for c in line.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 0.005 * b.abs() + 1e-12
}

#[test]
fn prefill_never_contradicts_the_official_command() {
    let Some(bank) = bank_texts() else { return };
    let (mut checked, mut wrong, mut top1, mut miss8, mut total) = (0, Vec::new(), 0, 0, 0);
    for e in examples() {
        let parts = split(&e.cmd);
        let key = &parts[0];
        let Some(f) = registry().formula(key) else {
            continue;
        };
        let Some(text) = bank.get(&e.id) else {
            continue;
        };
        total += 1;
        let hits = find(text, 24);
        match hits
            .iter()
            .position(|h| h.kind == "formula" && h.id == *key)
        {
            Some(0) => top1 += 1,
            Some(p) if p < 8 => {}
            _ => miss8 += 1,
        }
        let Some(hit) = hits.iter().find(|h| h.kind == "formula" && h.id == *key) else {
            continue;
        };
        let given: Vec<(&str, &str)> = parts[1..]
            .iter()
            .filter(|p| !p.starts_with('@'))
            .filter_map(|p| p.split_once('='))
            .filter(|(n, _)| f.names.iter().any(|x| x == n))
            .collect();
        let solved = solve(key, &given).expect("example solves");
        for (var, text_value) in &hit.prefill {
            checked += 1;
            let got = to_si(var, text_value).unwrap_or(f64::NAN);
            let expected: Vec<f64> = if let Some((_, v)) = given.iter().find(|(n, _)| n == var) {
                vec![to_si(var, v).expect("example value")]
            } else if let Some(d) = solved.defaults.get(var) {
                vec![*d]
            } else {
                solved
                    .found
                    .get(var)
                    .map(<[f64]>::to_vec)
                    .unwrap_or_default()
            };
            if !expected.iter().any(|x| close(got, *x)) {
                wrong.push(format!(
                    "Q{} {key} {var}: prefilled {text_value} ({got}), expected {expected:?}",
                    e.id
                ));
            }
        }
    }
    eprintln!(
        "prefill_bank: {total} formula examples, {top1} with their formula as top hit, {miss8} outside the top 8; \
         {checked} pre-filled values checked, {} wrong",
        wrong.len()
    );
    assert!(
        wrong.is_empty(),
        "{} wrong pre-fills:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
