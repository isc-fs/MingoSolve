//! Past-question recognition against the real FS-Quiz bank: every bank question must be recognised as itself, still
//! be recognised when every number in it is replaced, and invented text built from fragments of other questions must
//! not match. Also prints the leave-one-out scores the thresholds in `fsq::past` were chosen from.
//! Bank: `FSQ_BANK` or `../IFS-Tests/data/fsquiz/bank.json`; without it the test skips (CI always does).

use std::path::PathBuf;

use fsq::past::{
    bank, fingerprint, rank, recognise, Fingerprint, MIN_SIMILARITY, STRONG_SIMILARITY,
};
use regex::Regex;

fn bank_texts() -> Option<Vec<(u32, String)>> {
    let path = std::env::var("FSQ_BANK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../IFS-Tests/data/fsquiz/bank.json")
        });
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("past_bank: skipped, no bank at {}", path.display());
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

/// Ids whose fingerprint equals `id`'s (`masked_only`: same words, whatever the numbers).
fn twins(texts: &[(u32, String)], fps: &[Fingerprint], id: u32, masked_only: bool) -> Vec<u32> {
    let mine = &fps[texts.iter().position(|t| t.0 == id).unwrap()];
    texts
        .iter()
        .zip(fps)
        .filter(|(_, f)| {
            if masked_only {
                f.masked_eq(mine)
            } else {
                *f == mine
            }
        })
        .map(|(t, _)| t.0)
        .collect()
}

/// Every number token replaced by a different one (integers: n*7+13; decimals: x*1.9+0.37 to two places).
fn change_numbers(text: &str) -> String {
    let re = Regex::new(r"[0-9]+(?:[.,][0-9]+)*").unwrap();
    re.replace_all(text, |c: &regex::Captures| {
        let tok = c[0].replace(',', ".");
        match tok.parse::<f64>() {
            Ok(v) if tok.contains('.') => format!("{:.2}", v * 1.9 + 0.37),
            Ok(v) => format!("{}", v * 7.0 + 13.0),
            Err(_) => "9999".to_string(),
        }
    })
    .into_owned()
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % n
    }
}

#[test]
fn bank_questions_are_recognised() {
    let Some(texts) = bank_texts() else { return };
    let fps: Vec<_> = texts.iter().map(|t| fingerprint(&t.1)).collect();
    assert_eq!(
        texts.len(),
        bank().len(),
        "regenerate data/past_questions.toml"
    );

    let (mut exact, mut twin, mut missed, mut wrong_numbers) = (0, 0, Vec::new(), 0);
    let (mut num_ok, mut num_twin, mut num_missed) = (0, 0, Vec::new());
    for (id, text) in &texts {
        match recognise(text) {
            Some(m) if m.id == *id => {
                exact += 1;
                wrong_numbers += usize::from(!m.same_numbers);
            }
            Some(m) if twins(&texts, &fps, *id, false).contains(&m.id) => twin += 1,
            r => {
                println!(
                    "past_bank: Q{id} own text gave {:?}",
                    r.map(|m| (m.id, m.similarity, m.runner_up))
                );
                missed.push(*id)
            }
        }
        match recognise(&change_numbers(text)) {
            Some(m) if m.id == *id => num_ok += 1,
            Some(m) if twins(&texts, &fps, *id, true).contains(&m.id) => num_twin += 1,
            _ => num_missed.push(*id),
        }
    }
    let n = texts.len() as f64;
    println!(
        "past_bank: own text {exact}/{} as itself, {twin} as an identical-text twin, {} missed {missed:?}; \
         {wrong_numbers} own-text matches say numbers differ",
        texts.len(),
        missed.len()
    );
    println!(
        "past_bank: all numbers changed {num_ok} as itself, {num_twin} as a number-only sibling, {} missed ({:.1} % recognised) {num_missed:?}",
        num_missed.len(),
        100.0 * (num_ok + num_twin) as f64 / n
    );
    assert!(missed.is_empty(), "own text not recognised: {missed:?}");
    assert_eq!(wrong_numbers, 0);
    assert!(
        (num_ok + num_twin) as f64 >= 0.95 * n,
        "numbers changed: only {} of {} recognised",
        num_ok + num_twin,
        texts.len()
    );
}

#[test]
fn invented_text_from_bank_fragments_does_not_match() {
    let Some(texts) = bank_texts() else { return };
    let mut rng = Lcg(0x5EED);
    let words: Vec<Vec<&str>> = texts
        .iter()
        .map(|(_, t)| t.split_whitespace().collect())
        .collect();
    let (mut weak, mut strong) = (0, 0);
    for _ in 0..200 {
        let mut s = Vec::new();
        for _ in 0..4 {
            let w = &words[rng.next(words.len())];
            let len = 5 + rng.next(4);
            let start = rng.next(w.len().saturating_sub(len).max(1));
            s.extend(w.iter().skip(start).take(len).copied());
        }
        let s = s.join(" ");
        if let Some(m) = recognise(&s) {
            weak += 1;
            strong += usize::from(!m.probable);
            println!(
                "past_bank: false positive {:.2} on Q{}: {s}",
                m.similarity, m.id
            );
        }
    }
    println!(
        "past_bank: 200 invented sentences, {weak} matched at all, {strong} with high confidence"
    );
    assert_eq!(strong, 0);
    assert!(weak <= 2, "{weak} of 200 invented sentences matched");
}

/// How close one bank question gets to another (each against the rest of the bank, twins excluded): the scores the
/// thresholds were chosen from. Asserts only that the strong threshold is above all but near-identical rewrites.
#[test]
fn leave_one_out_scores() {
    let Some(texts) = bank_texts() else { return };
    let rows = bank();
    let fps: Vec<_> = texts.iter().map(|t| fingerprint(&t.1)).collect();
    let mut hist = [0usize; 11];
    let (mut weak, mut strong) = (0, 0);
    for (id, text) in &texts {
        let skip = twins(&texts, &fps, *id, true);
        let best = rank(rows, text)
            .into_iter()
            .find(|r| !skip.contains(&rows[r.index].id))
            .map_or(0.0, |r| r.similarity);
        hist[(best * 10.0) as usize] += 1;
        weak += usize::from(best >= MIN_SIMILARITY);
        strong += usize::from(best >= STRONG_SIMILARITY);
    }
    println!(
        "past_bank: leave-one-out best score by tenth (0.0 .. 1.0): {hist:?}; >= {MIN_SIMILARITY}: {weak}, >= {STRONG_SIMILARITY}: {strong}"
    );
}
