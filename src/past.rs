//! Past-question recognition: a pasted question is fingerprinted (hashes of its word 3-grams, numbers masked) and
//! compared with `data/past_questions.toml`, which holds the same fingerprints for the whole FS-Quiz bank and no
//! text. Numbers are masked so a question that came back with other numbers still matches; `same_numbers` says
//! whether the official answer applies as is. The tokenizer and hashes mirror `legacy/python/scripts/export_past.py`.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::answer::known_keys;
use crate::cli::examples;
use crate::data::PAST_QUESTIONS;

/// Below this a bank question is not reported (chosen from the leave-one-out scores of the bank, see tests/past_bank.rs).
pub const MIN_SIMILARITY: f64 = 0.6;
/// From here a match is stated as fact; between the two it is "probably".
pub const STRONG_SIMILARITY: f64 = 0.85;
/// A runner-up this close to the best match makes the answer ambiguous.
const AMBIGUITY_MARGIN: f64 = 0.05;
/// The pasted text may be this many times longer than the bank question before the extra words dilute the score.
const SLACK: f64 = 1.5;
const MIN_SHARED: usize = 3;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fingerprint {
    shingles: Vec<u32>,
    nums: Vec<u32>,
}

impl Fingerprint {
    /// Same words, whatever the numbers.
    pub fn masked_eq(&self, other: &Fingerprint) -> bool {
        self.shingles == other.shingles
    }
}

#[derive(Debug, Clone)]
pub struct PastQuestion {
    pub id: u32,
    pub quizzes: Vec<String>,
    pub answer: Option<String>,
    pub fp: Fingerprint,
}

#[derive(Debug, Clone, Serialize)]
pub struct PastExample {
    pub cmd: String,
    pub answer: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PastMatch {
    pub id: u32,
    pub quizzes: Vec<String>,
    /// The official answer when it is a number with a unit.
    pub answer: Option<String>,
    pub similarity: f64,
    pub runner_up: f64,
    /// True when the match is not clear-cut: weak similarity or another question scores as well.
    pub probable: bool,
    /// The numbers of the bank question all appear in the pasted text, so its official answer applies as is.
    pub same_numbers: bool,
    pub example: Option<PastExample>,
    /// Why the official key is known to be wrong, when it is.
    pub known_key: Option<String>,
}

fn fnv1a(s: &str) -> u32 {
    s.bytes().fold(0x811C_9DC5_u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// ASCII letter runs and digit runs (with inner `.` or `,` between digits); anything else separates tokens.
fn tokens(text: &str) -> Vec<&str> {
    let b = text.as_bytes();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < b.len() {
        let start = i;
        if b[i].is_ascii_alphabetic() {
            while i < b.len() && b[i].is_ascii_alphabetic() {
                i += 1;
            }
        } else if b[i].is_ascii_digit() {
            while i < b.len() {
                if b[i].is_ascii_digit() {
                    i += 1;
                } else if matches!(b[i], b'.' | b',')
                    && b.get(i + 1).is_some_and(u8::is_ascii_digit)
                {
                    i += 2;
                } else {
                    break;
                }
            }
        } else {
            i += 1;
            continue;
        }
        out.push(&text[start..i]);
    }
    out
}

fn canonical_number(tok: &str) -> String {
    let tok = tok.replace(',', ".");
    let (whole, frac) = tok.split_once('.').unwrap_or((&tok, ""));
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{frac}")
    }
}

pub fn fingerprint(text: &str) -> Fingerprint {
    let (mut words, mut nums) = (Vec::new(), Vec::new());
    for tok in tokens(text) {
        if tok.as_bytes()[0].is_ascii_digit() {
            words.push("#".to_string());
            nums.push(fnv1a(&canonical_number(tok)));
        } else {
            words.push(tok.to_ascii_lowercase());
        }
    }
    let mut shingles: Vec<u32> = if words.len() < 3 {
        vec![fnv1a(&words.join(" "))]
    } else {
        words.windows(3).map(|w| fnv1a(&w.join(" "))).collect()
    };
    for v in [&mut shingles, &mut nums] {
        v.sort_unstable();
        v.dedup();
    }
    Fingerprint { shingles, nums }
}

fn b64_u32s(s: &str) -> Vec<u32> {
    let val = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("data/past_questions.toml: bad base64"),
    };
    let (mut bytes, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for c in s.bytes() {
        acc = (acc << 6) | u32::from(val(c));
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[derive(Deserialize)]
struct Row {
    id: u32,
    quizzes: Vec<String>,
    answer: Option<String>,
    nums: String,
    fp: String,
}

#[derive(Deserialize)]
struct Rows {
    q: Vec<Row>,
}

/// Every bank question's fingerprint, parsed once from the embedded `data/past_questions.toml`.
pub fn bank() -> &'static [PastQuestion] {
    static BANK: OnceLock<Vec<PastQuestion>> = OnceLock::new();
    BANK.get_or_init(|| {
        toml::from_str::<Rows>(PAST_QUESTIONS)
            .expect("data/past_questions.toml")
            .q
            .into_iter()
            .map(|r| PastQuestion {
                id: r.id,
                quizzes: r.quizzes,
                answer: r.answer,
                fp: Fingerprint {
                    shingles: b64_u32s(&r.fp),
                    nums: b64_u32s(&r.nums),
                },
            })
            .collect()
    })
}

fn shared(a: &[u32], b: &[u32]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

/// Containment of the bank question in the pasted text; words beyond `SLACK` times the bank question dilute it.
fn similarity(q: &Fingerprint, pasted: &Fingerprint) -> (f64, usize) {
    let n = shared(&q.shingles, &pasted.shingles);
    let denom = (q.shingles.len() as f64).max(pasted.shingles.len() as f64 / SLACK);
    (n as f64 / denom, n)
}

fn jaccard(a: &[u32], b: &[u32]) -> f64 {
    let n = shared(a, b);
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    n as f64 / (a.len() + b.len() - n) as f64
}

fn same_numbers(q: &Fingerprint, pasted: &Fingerprint) -> bool {
    shared(&q.nums, &pasted.nums) == q.nums.len()
}

#[derive(Debug, Clone, Copy)]
pub struct Ranked {
    /// Index into the bank.
    pub index: usize,
    pub similarity: f64,
    pub same_numbers: bool,
    jaccard: f64,
    nums_jaccard: f64,
}

/// Score of every bank question against the pasted text, best first: similarity, then same numbers, then the
/// closeness in length and in numbers (so a short generic question contained in a long one does not tie with it).
pub fn rank(bank: &[PastQuestion], text: &str) -> Vec<Ranked> {
    let pasted = fingerprint(text);
    let mut scored: Vec<_> = bank
        .iter()
        .enumerate()
        .filter_map(|(index, q)| {
            let (similarity, n) = similarity(&q.fp, &pasted);
            (n >= MIN_SHARED.min(q.fp.shingles.len())).then(|| Ranked {
                index,
                similarity,
                same_numbers: same_numbers(&q.fp, &pasted),
                jaccard: jaccard(&q.fp.shingles, &pasted.shingles),
                nums_jaccard: jaccard(&q.fp.nums, &pasted.nums),
            })
        })
        .collect();
    scored.sort_by(|a, b| {
        b.similarity
            .total_cmp(&a.similarity)
            .then(b.same_numbers.cmp(&a.same_numbers))
            .then(b.jaccard.total_cmp(&a.jaccard))
            .then(b.nums_jaccard.total_cmp(&a.nums_jaccard))
    });
    scored
}

pub fn recognise(text: &str) -> Option<PastMatch> {
    recognise_in(bank(), text)
}

pub fn recognise_in(bank: &[PastQuestion], text: &str) -> Option<PastMatch> {
    let ranked = rank(bank, text);
    let best = *ranked.first()?;
    let (similarity, same) = (best.similarity, best.same_numbers);
    if similarity < MIN_SIMILARITY {
        return None;
    }
    let q = &bank[best.index];
    let runner = ranked.get(1).copied();
    let runner_up = runner.map_or(0.0, |r| r.similarity);
    // a sibling that differs only by its numbers is told apart by them
    let separated = same && runner.is_some_and(|r| !r.same_numbers);
    let probable = similarity < STRONG_SIMILARITY
        || (runner_up >= similarity - AMBIGUITY_MARGIN && !separated);
    Some(PastMatch {
        id: q.id,
        quizzes: q.quizzes.clone(),
        answer: q.answer.clone(),
        similarity,
        runner_up,
        probable,
        same_numbers: same,
        example: lookups().examples.get(&q.id).cloned(),
        known_key: lookups().keys.get(&q.id).cloned(),
    })
}

struct Lookups {
    examples: HashMap<u32, PastExample>,
    keys: HashMap<u32, String>,
}

fn lookups() -> &'static Lookups {
    static L: OnceLock<Lookups> = OnceLock::new();
    L.get_or_init(|| Lookups {
        examples: examples()
            .into_iter()
            .map(|e| {
                (
                    e.id,
                    PastExample {
                        cmd: e.cmd,
                        answer: e.answer,
                    },
                )
            })
            .collect(),
        keys: known_keys().into_iter().map(|k| (k.id, k.note)).collect(),
    })
}

impl PastQuestion {
    /// A bank row built from text, for tests (the shipped rows come from the export script).
    pub fn from_text(id: u32, text: &str, quizzes: &[&str], answer: Option<&str>) -> Self {
        PastQuestion {
            id,
            quizzes: quizzes.iter().map(|s| s.to_string()).collect(),
            answer: answer.map(str::to_string),
            fp: fingerprint(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q: &str =
        "A flywheel of 12.5 kg rotates at 3000 rpm. How much kinetic energy does it store if its \
                     moment of inertia is 0.04 kg m²?";

    #[test]
    fn fingerprint_ignores_case_spacing_and_punctuation() {
        let messy =
            "a  FLYWHEEL of 12.5 KG rotates, at 3000 rpm -- How much kinetic energy does it store \
                     if its\n moment of inertia is 0.04 kg m²???";
        assert_eq!(fingerprint(Q), fingerprint(messy));
    }

    #[test]
    fn fingerprint_masks_numbers_but_tracks_them() {
        let other = Q.replace("12.5", "7").replace("3000", "1500");
        let (a, b) = (fingerprint(Q), fingerprint(&other));
        assert_eq!(a.shingles, b.shingles);
        assert_ne!(a.nums, b.nums);
        // decimal comma and trailing zeros are the same number
        assert_eq!(fingerprint("x 12,50 y").nums, fingerprint("x 12.5 y").nums);
    }

    #[test]
    fn fnv1a_matches_the_reference_vectors() {
        assert_eq!(fnv1a(""), 0x811C_9DC5);
        assert_eq!(fnv1a("a"), 0xE40C_292C);
        assert_eq!(fnv1a("foobar"), 0xBF9C_F968);
    }

    #[test]
    fn base64_round_trips_little_endian_words() {
        // "AQAAAP///38" is [1, 0x7FFF_FFFF] little-endian, unpadded
        assert_eq!(b64_u32s("AQAAAP///38"), vec![1, 0x7FFF_FFFF]);
        assert!(b64_u32s("").is_empty());
    }

    fn bank_of_two() -> Vec<PastQuestion> {
        vec![
            PastQuestion::from_text(1, Q, &["FSG 2023 EV"], Some("236 J")),
            PastQuestion::from_text(
                2,
                "Which of the following statements about brake discs is correct for a lightweight car?",
                &["FSA 2024 EV"],
                None,
            ),
        ]
    }

    #[test]
    fn recognises_the_same_question_with_other_numbers() {
        let bank = bank_of_two();
        let same = recognise_in(&bank, Q).unwrap();
        assert_eq!(
            (same.id, same.same_numbers, same.probable),
            (1, true, false)
        );
        let changed = Q.replace("12.5", "20").replace("0.04", "0.09");
        let m = recognise_in(&bank, &changed).unwrap();
        assert_eq!((m.id, m.same_numbers), (1, false));
        assert!(m.similarity > STRONG_SIMILARITY);
    }

    #[test]
    fn unrelated_text_does_not_match() {
        let bank = bank_of_two();
        let other = "A car accelerates from rest to 100 km/h in 3.2 s. What is the average power if the mass \
                     is 280 kg?";
        assert!(recognise_in(&bank, other).is_none());
    }

    #[test]
    fn siblings_that_differ_only_by_numbers_are_told_apart_by_them() {
        let a = Q.replace("12.5", "20");
        let bank = vec![
            PastQuestion::from_text(10, Q, &[], None),
            PastQuestion::from_text(11, &a, &[], None),
        ];
        let m = recognise_in(&bank, &a).unwrap();
        assert_eq!((m.id, m.same_numbers, m.probable), (11, true, false));
        let third = Q.replace("12.5", "30");
        let m = recognise_in(&bank, &third).unwrap();
        assert!(m.probable && !m.same_numbers, "no id fits the numbers");
    }

    #[test]
    fn known_keys_are_attached_by_id() {
        // Q125 is a real row of data/known_keys.toml
        let bank = vec![PastQuestion::from_text(125, Q, &[], None)];
        let m = recognise_in(&bank, Q).unwrap();
        assert!(m.known_key.unwrap().contains("tetrahedron"));
        let bank = vec![PastQuestion::from_text(124, Q, &[], None)];
        assert!(recognise_in(&bank, Q).unwrap().known_key.is_none());
    }

    #[test]
    fn embedded_bank_is_consistent_with_examples_and_known_keys() {
        let ids: std::collections::HashSet<u32> = bank().iter().map(|q| q.id).collect();
        assert_eq!(ids.len(), bank().len(), "duplicate ids");
        for e in examples() {
            assert!(ids.contains(&e.id), "example Q{} is not in the bank", e.id);
        }
        for k in known_keys() {
            assert!(
                ids.contains(&k.id),
                "known key Q{} is not in the bank",
                k.id
            );
        }
        assert!(bank().iter().all(|q| !q.fp.shingles.is_empty()));
    }
}
