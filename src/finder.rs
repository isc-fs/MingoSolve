//! Question finder: paste a quiz question, get the formulas, tools and past examples that fit, ranked by word
//! overlap and by the physical dimensions of the quantities in the text ("280 kg ... 100 km/h ... 4 s" = mass,
//! speed, time). Formula hits come with a pre-fill: each quantity mapped to the variable of that dimension whose
//! description best matches the words around it.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::answer::known_keys;
use crate::cli::examples;
use crate::registry::registry;
use crate::tools::tools;
use crate::units::{self, Dims, DIMENSIONLESS};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Quantity {
    pub text: String,
    pub dims: Dims,
    /// Words just around the quantity ("cells in series at" for "3.8 V"), used to pick between variables.
    pub context: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub kind: &'static str,
    pub id: String,
    pub title: String,
    pub score: f64,
    /// (variable, value as typed in the question) for formula hits.
    pub prefill: Vec<(String, String)>,
    pub warning: Option<String>,
}

const STOP: &[&str] = &[
    "the",
    "and",
    "for",
    "with",
    "what",
    "which",
    "how",
    "are",
    "is",
    "its",
    "that",
    "this",
    "from",
    "into",
    "your",
    "you",
    "team",
    "car",
    "vehicle",
    "given",
    "calculate",
    "determine",
    "following",
    "value",
    "answer",
    "assume",
    "has",
    "have",
    "will",
    "can",
    "per",
    "when",
    "than",
    "then",
    "there",
    "their",
    "they",
    "was",
    "were",
    "been",
    "much",
    "many",
    "does",
    "should",
    "would",
    "could",
    "one",
    "two",
    "all",
    "any",
    "each",
    "use",
    "using",
    "only",
];

fn words(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .map(str::to_lowercase)
        .filter(|w| {
            w.len() >= 3 && !w.chars().all(|c| c.is_ascii_digit()) && !STOP.contains(&w.as_str())
        })
        .map(|w| {
            w.strip_suffix('s')
                .filter(|s| s.len() >= 3)
                .map(str::to_string)
                .unwrap_or(w)
        })
        .collect()
}

/// Numbers with units in free text ("100 km/h", "1,5 kN", "20Ah", "60 °C", "10 mm²").
pub fn quantities(text: &str) -> Vec<Quantity> {
    let t = text
        .replace('²', "**2")
        .replace('³', "**3")
        .replace("°C", "degC")
        .replace('°', "deg");
    let chars: Vec<char> = t.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let starts_number =
            chars[i].is_ascii_digit() && (i == 0 || !chars[i - 1].is_alphanumeric());
        if !starts_number {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len()
            && (chars[j].is_ascii_digit()
                || ((chars[j] == '.' || chars[j] == ',')
                    && chars.get(j + 1).is_some_and(|c| c.is_ascii_digit())))
        {
            j += 1;
        }
        let number: String = chars[i..j].iter().collect::<String>().replace(',', ".");
        let mut k = j;
        while k < chars.len() && chars[k] == ' ' {
            k += 1;
        }
        let mut end = k;
        while end < chars.len() && (chars[end].is_alphanumeric() || "/*%µΩ_".contains(chars[end]))
        {
            end += 1;
        }
        // the whole word must be a unit; "in" (inch) and "t" (tonne) are too often plain English ("0 to 100 in 4 s")
        let unit: String = chars[k..end].iter().collect();
        let unit = unit.trim_end_matches(['/', '*']);
        let found = if unit.is_empty() || ["in", "t"].contains(&unit) {
            None
        } else {
            units::parse_quantity(&format!("{number} {unit}"))
                .ok()
                .filter(|q| q.dims != DIMENSIONLESS || unit == "%")
                .map(|q| (format!("{number} {unit}"), q.dims))
        };
        if let Some((text, dims)) = found {
            let before: String = chars[i.saturating_sub(50)..i].iter().collect();
            let after: String = chars[end..(end + 20).min(chars.len())].iter().collect();
            let context = words(&format!("{before} {after}")).into_iter().collect();
            out.push(Quantity {
                text,
                dims,
                context,
            });
        }
        i = end.max(j).max(i + 1);
    }
    out
}

fn var_dims(name: &str) -> Dims {
    units::unit_of(&registry().var(name).unit)
        .map(|q| q.dims)
        .unwrap_or(DIMENSIONLESS)
}

fn text_score(question: &HashSet<String>, strong: &str, weak: &str) -> f64 {
    let (s, w) = (words(strong), words(weak));
    question
        .iter()
        .map(|q| {
            if s.contains(q) {
                2.0
            } else if w.contains(q) {
                1.0
            } else {
                0.0
            }
        })
        .sum()
}

/// Map quantities to variables of the same dimension. Several candidates: the one whose description shares most
/// words with the quantity's context, then the plainest (shortest description); a remaining tie stays unfilled.
fn prefill(names: &[String], qs: &[Quantity]) -> Vec<(String, String)> {
    let r = registry();
    let mut taken: HashSet<&String> = HashSet::new();
    let mut out = Vec::new();
    for q in qs {
        if q.dims == DIMENSIONLESS {
            continue;
        }
        let mut cands: Vec<(usize, usize, &String)> = names
            .iter()
            .filter(|n| !taken.contains(n) && var_dims(n) == q.dims)
            .map(|n| {
                let d = words(&r.var(n).desc);
                let overlap = q.context.iter().filter(|w| d.contains(*w)).count();
                (overlap, r.var(n).desc.len(), n)
            })
            .collect();
        cands.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let pick = match cands.as_slice() {
            [] => None,
            [only] => Some(only.2),
            [a, b, ..] => (a.0 != b.0 || a.1 != b.1).then_some(a.2),
        };
        if let Some(n) = pick {
            taken.insert(n);
            out.push((n.clone(), q.text.clone()));
        }
    }
    out
}

pub fn find(question: &str, limit: usize) -> Vec<Hit> {
    let q = words(question);
    let qs = quantities(question);
    let q_dims: HashSet<Dims> = qs
        .iter()
        .map(|x| x.dims)
        .filter(|d| *d != DIMENSIONLESS)
        .collect();
    let r = registry();
    let mut hits = Vec::new();

    for f in &r.formulas {
        let descs: Vec<&str> = f.names.iter().map(|n| r.var(n).desc.as_str()).collect();
        let mut score = text_score(
            &q,
            &format!(
                "{} {} {}",
                f.key.replace('_', " "),
                f.title,
                f.tags.join(" ")
            ),
            &format!("{} {}", f.notes, descs.join(" ")),
        );
        let f_dims: HashSet<Dims> = f.names.iter().map(|n| var_dims(n)).collect();
        score += 1.5 * q_dims.intersection(&f_dims).count() as f64;
        if score <= 0.0 {
            continue;
        }
        let prefill = prefill(&f.names, &qs);
        hits.push(Hit {
            kind: "formula",
            id: f.key.clone(),
            title: f.title.clone(),
            score,
            prefill,
            warning: None,
        });
    }

    for t in tools() {
        let score = text_score(&q, &t.name.replace('_', " "), t.doc);
        if score > 0.0 {
            hits.push(Hit {
                kind: "tool",
                id: t.name.into(),
                title: t.doc.into(),
                score,
                prefill: vec![],
                warning: None,
            });
        }
    }

    let keys: HashMap<u32, String> = known_keys().into_iter().map(|k| (k.id, k.note)).collect();
    for e in examples() {
        let score = text_score(&q, &e.what, &e.cmd.replace('_', " "));
        if score > 0.0 {
            hits.push(Hit {
                kind: "example",
                id: format!("Q{}", e.id),
                title: e.what.clone(),
                score: score * 0.9,
                prefill: vec![],
                warning: keys.get(&e.id).cloned(),
            });
        }
    }

    hits.sort_by(|a, b| b.score.total_cmp(&a.score));
    hits.truncate(limit);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_quantities_with_units() {
        let qs = quantities(
            "A car of 280 kg accelerates from 0 to 100 km/h in 4,2 s. The motor gives 80 kW.",
        );
        let texts: Vec<&str> = qs.iter().map(|q| q.text.as_str()).collect();
        assert_eq!(texts, ["280 kg", "100 km/h", "4.2 s", "80 kW"]);
        let qs = quantities("tube 25 x 2.5 mm, area 10 mm² at 60 °C");
        assert!(qs.iter().any(|q| q.text == "10 mm**2"));
        assert!(qs.iter().any(|q| q.text == "60 degC"));
    }

    #[test]
    fn ranks_the_right_formula_and_prefills() {
        let hits = find(
            "The accumulator has 103 cells in series at 3.8 V and an internal resistance of 0.08 Ω. \
             What current is drawn at 30 kW?",
            8,
        );
        let top: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
        assert!(top[..3].contains(&"battery_load"), "{top:?}");
        let hit = hits.iter().find(|h| h.id == "battery_load").unwrap();
        assert!(
            hit.prefill.contains(&("P".into(), "30 kW".into())),
            "{:?}",
            hit.prefill
        );
        assert!(
            hit.prefill.contains(&("V_cell".into(), "3.8 V".into())),
            "{:?}",
            hit.prefill
        );
        assert!(
            !hit.prefill.iter().any(|(_, v)| v == "0.08 Ω"),
            "R_cell vs R_pack is a real tie: {:?}",
            hit.prefill
        );
        let hits = find(
            "Skidpad: what is the maximum cornering speed with downforce, mu 1.4, 240 kg?",
            5,
        );
        assert!(
            hits.iter().any(|h| h.id == "cornering_downforce"),
            "{:?}",
            hits.iter().map(|h| &h.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn examples_carry_known_key_warnings() {
        let hits = find("endurance time score without finish points legacy", 10);
        let ex = hits.iter().find(|h| h.id == "Q171").expect("Q171 example");
        assert!(ex.warning.as_deref().unwrap().contains("+25"));
    }
}
