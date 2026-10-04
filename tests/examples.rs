//! Every row of data/examples.toml (a past FS-Quiz question with its official answer) must reproduce:
//! the answer appears in the command output within 0.5 %.

use fsq::cli::{examples, run};

#[test]
fn past_questions_reproduce() {
    let num = |s: &str| s.parse::<f64>().ok();
    let mut failures = Vec::new();
    let all = examples();
    for e in &all {
        let out = run(&e.cmd);
        let found = out
            .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == '+'))
            .filter_map(num)
            .any(|n| (n - e.answer).abs() <= 0.005 * e.answer.abs() + 1e-9);
        if !found {
            failures.push(format!(
                "Q{} {}: expected {}\n  {}\n{out}",
                e.id, e.what, e.answer, e.cmd
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} failed:\n{}",
        failures.len(),
        all.len(),
        failures.join("\n")
    );
}
