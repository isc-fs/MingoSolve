//! The format hint against the real FS-Quiz bank. For every past question that has a formula example and a hint:
//! opening the example with the hinted display unit and rounding must give the official answer exactly as the quiz
//! wants it typed (string equal after normalising the decimal comma). Also reports how many bank questions carry
//! a rounding or unit instruction and how many the parser reads.
//! Bank: `FSQ_BANK` or `../IFS-Tests/data/fsquiz/bank.json`; without it the test skips (CI always does).

use std::path::PathBuf;

use fsq::answer::{format_answer, Precision};
use fsq::cli::examples;
use fsq::engine::solve;
use fsq::format_hint::{dims_key, parse};
use fsq::registry::registry;
use fsq::units::unit_of;

struct BankQuestion {
    id: u32,
    kind: String,
    text: String,
    key: Option<String>,
}

fn bank() -> Option<Vec<BankQuestion>> {
    let path = std::env::var("FSQ_BANK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../IFS-Tests/data/fsquiz/bank.json")
        });
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("format_bank: skipped, no bank at {}", path.display());
        return None;
    };
    let json: serde_json::Value = serde_json::from_str(&raw).expect("bank.json");
    Some(
        json["questions"]
            .as_array()
            .expect("questions")
            .iter()
            .filter_map(|q| {
                let key = q["answers"]
                    .as_array()
                    .and_then(|a| a.iter().find(|x| x["is_correct"].as_bool() == Some(true)))
                    .and_then(|x| x["text"].as_str())
                    .map(str::to_string);
                Some(BankQuestion {
                    id: q["question_id"].as_u64()? as u32,
                    kind: q["type"].as_str()?.to_string(),
                    text: q["text"].as_str()?.to_string(),
                    key,
                })
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

/// Decimal comma as point, no leading plus, and a zero key written bare ("0") equals "0.00".
fn normal(s: &str) -> String {
    let s = s.trim().replace(',', ".");
    let s = s.trim_start_matches('+');
    match s.parse::<f64>() {
        Ok(v) if v == 0.0 => "0".into(),
        _ => s.to_string(),
    }
}

fn has_any(text: &str, needles: &[&str]) -> bool {
    let lc = text.to_lowercase();
    needles.iter().any(|n| lc.contains(n))
}

#[test]
fn hint_reproduces_official_answers_and_covers_the_bank() {
    let Some(bank) = bank() else { return };

    // how many questions carry each kind of instruction, and how many the parser reads
    let rounding_words = [
        "decimal",
        "significant dig",
        "significant fig",
        "nearest",
        "round to",
        "rounded to",
    ];
    let unit_words = [
        "answer in",
        "solution in",
        "result in",
        "results in",
        "calculated in",
        "given in",
        "answer [",
    ];
    let (mut r_words, mut r_read, mut u_words, mut u_read, mut any_read) = (0, 0, 0, 0, 0);
    for q in &bank {
        let hint = parse(&q.text);
        let by_words = has_any(&q.text, &rounding_words);
        let by_unit = has_any(&q.text, &unit_words);
        r_words += usize::from(by_words);
        u_words += usize::from(by_unit);
        if let Some(h) = &hint {
            any_read += 1;
            r_read += usize::from(h.rounding.is_some());
            u_read += usize::from(h.unit.is_some());
            if let Some(u) = &h.unit {
                assert!(unit_of(u).is_ok(), "Q{}: {u} does not parse", q.id);
                assert_eq!(h.dims, dims_key(u));
            }
        }
    }
    eprintln!(
        "format_bank: {} questions; rounding wording in {r_words}, parsed {r_read}; \
         unit wording in {u_words}, parsed {u_read}; {any_read} carry a hint",
        bank.len()
    );

    let (mut with_hint, mut exact, mut numeric, mut mismatches) = (0, 0, 0, Vec::new());
    for e in examples() {
        let parts = split(&e.cmd);
        let key = &parts[0];
        let Some(f) = registry().formula(key) else {
            continue;
        };
        let Some(q) = bank.iter().find(|q| q.id == e.id) else {
            continue;
        };
        let Some(hint) = parse(&q.text) else { continue };
        let Some(official) = q.key.as_deref().filter(|_| q.kind.starts_with("input")) else {
            continue;
        };
        let given: Vec<(&str, &str)> = parts[1..]
            .iter()
            .filter(|p| !p.starts_with('@'))
            .filter_map(|p| p.split_once('='))
            .filter(|(n, _)| f.names.iter().any(|x| x == n))
            .collect();
        let solved = solve(key, &given).expect("example solves");
        let official_value = official.replace(',', ".").parse::<f64>().ok();
        let Some(official_value) = official_value else {
            continue;
        };
        with_hint += 1;

        // the variable the official answer belongs to, read in the unit the question asks for
        let example_unit = parts[1..]
            .iter()
            .find_map(|p| p.strip_prefix('@')?.split_once('='))
            .map(|(v, u)| (v.to_string(), u.to_string()));
        // 0.5 % or half a rounding step, whichever is wider (the key is itself rounded)
        let half_step = match hint.rounding {
            Some(Precision::Decimals(d)) => 0.5 * 10f64.powi(-(d as i32)),
            _ => 0.0,
        };
        let tolerance = (0.005 * official_value.abs()).max(half_step) + 1e-9;
        let mut shown: Option<(f64, String)> = None;
        for (var, values) in &solved.found.0 {
            let var_unit = &registry().var(var).unit;
            let hinted = hint
                .unit
                .as_ref()
                .filter(|u| dims_key(u) == dims_key(var_unit));
            let display = hinted.cloned().or_else(|| {
                example_unit
                    .as_ref()
                    .filter(|(v, _)| v == var)
                    .map(|(_, u)| u.clone())
            });
            let scale = |v: f64| match &display {
                Some(u) => v * unit_of(var_unit).unwrap().value / unit_of(u).unwrap().value,
                None => v,
            };
            if let Some(v) = values
                .iter()
                .map(|v| scale(*v))
                .find(|v| (v - official_value).abs() <= tolerance)
            {
                shown = Some((v, var.to_string()));
                break;
            }
        }
        let Some((value, var)) = shown else {
            mismatches.push(format!("Q{}: no solved variable reads {official}", e.id));
            continue;
        };
        match hint.rounding {
            Some(p) => {
                let got = format_answer(value, p, false).expect("format");
                if normal(&got) == normal(official) {
                    exact += 1;
                } else {
                    mismatches.push(format!(
                        "Q{} {key}.{var}: formatted {got}, official {official} (hint {:?} {:?})",
                        e.id, hint.rounding, hint.unit
                    ));
                }
            }
            None => numeric += 1,
        }
    }
    eprintln!(
        "format_bank: {with_hint} examples with a hint and a numeric official answer: {exact} reproduced exactly with the \
         hinted rounding, {numeric} unit-only hints agree numerically, {} mismatches",
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
