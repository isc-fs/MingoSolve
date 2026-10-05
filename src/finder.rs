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
    /// The word right before the number ("f" in "at f = 200 Hz"), matched against variable names.
    #[serde(skip)]
    pub label: String,
    /// Followed by "each" / "per": a per-item value, not a total.
    #[serde(skip)]
    pub per_item: bool,
    /// The label is followed by "=" ("A = 1 m²"): the quantity is named by that symbol.
    #[serde(skip)]
    pub labelled: bool,
    /// Given with a tolerance ("50 Ω ±10 %"): a nominal value the user must decide on.
    #[serde(skip)]
    pub uncertain: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub kind: &'static str,
    pub id: String,
    pub title: String,
    pub score: f64,
    /// (variable, value as typed in the question) for formula hits.
    pub prefill: Vec<(String, String)>,
    /// The variable the question asks for ("What current is drawn...?" -> I), when the words make it clear.
    pub target: Option<String>,
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

/// Letters-only hyphens dropped so "cut-off" and "cutoff" are the same word.
fn joined_words(text: &str) -> HashSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let kept: String = chars
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            **c != '-'
                || !(*i > 0
                    && chars[i - 1].is_alphabetic()
                    && chars.get(i + 1).is_some_and(|n| n.is_alphabetic()))
        })
        .map(|(_, c)| *c)
        .collect();
    words(&kept)
        .into_iter()
        .map(|w| match w.strip_suffix("ie") {
            Some(stem) => format!("{stem}y"),
            None => w,
        })
        .collect()
}

fn normalise(text: &str) -> String {
    text.replace(['\u{a0}', '\u{202f}', '\u{2009}'], " ")
        .replace('²', "**2")
        .replace('³', "**3")
        .replace('^', "**")
        .replace('\u{2212}', "-")
        .replace("°C", "degC")
        .replace('°', "deg")
}

fn digits_at(chars: &[char], at: usize, n: usize) -> bool {
    chars.len() >= at + n && chars[at..at + n].iter().all(char::is_ascii_digit)
}

/// End of the number starting at `i`, and its text with thousands separators removed and a decimal comma turned
/// into a point. "40 000" and "3,000" are thousands (exactly three digits, then a non-digit, after a 1 to 3 digit
/// group that is not 0); "1,8" and "0,500" are decimals.
fn scan_number(chars: &[char], i: usize) -> (usize, String) {
    let mut j = i;
    let mut num = String::new();
    let mut grouped = false;
    while j < chars.len() && chars[j].is_ascii_digit() {
        num.push(chars[j]);
        j += 1;
    }
    loop {
        let head_ok =
            !grouped && num.len() <= 3 && !num.starts_with('0') || grouped && num.len() > 3;
        let sep = chars.get(j).copied();
        let thousands = matches!(sep, Some(' ' | ',')) && head_ok;
        if thousands && digits_at(chars, j + 1, 3) && !digits_at(chars, j + 1, 4) {
            num.extend(&chars[j + 1..j + 4]);
            grouped = true;
            j += 4;
            continue;
        }
        break;
    }
    if matches!(chars.get(j), Some('.' | ',')) && digits_at(chars, j + 1, 1) {
        num.push('.');
        j += 1;
        while j < chars.len() && chars[j].is_ascii_digit() {
            num.push(chars[j]);
            j += 1;
        }
    }
    (j, num)
}

const ANGLE_UNITS: &[&str] = &["deg", "degree", "degrees", "rad"];

/// Numbers with units in free text ("100 km/h", "1,5 kN", "20Ah", "60 °C", "10 mm²", "9.81 m/s^2", "40 000 ft").
pub fn quantities(text: &str) -> Vec<Quantity> {
    let t = normalise(text);
    let chars: Vec<char> = t.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut prev_end = 0;
    while i < chars.len() {
        let starts_number = chars[i].is_ascii_digit()
            && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '*'));
        if !starts_number {
            i += 1;
            continue;
        }
        let (j, number) = scan_number(&chars, i);
        let negative = i >= 1 && chars[i - 1] == '-' && (i == 1 || " (=:".contains(chars[i - 2]));
        let number = if negative {
            format!("-{number}")
        } else {
            number
        };
        let mut k = j;
        while k < chars.len() && chars[k] == ' ' {
            k += 1;
        }
        let mut end = k;
        while end < chars.len()
            && !chars[k].is_ascii_digit()
            && (chars[end].is_alphanumeric()
                || "/*%µΩ_".contains(chars[end])
                || (chars[end] == '-'
                    && chars[end - 1] == '*'
                    && chars.get(end + 1).is_some_and(char::is_ascii_digit)))
        {
            end += 1;
        }
        // "J/(kg K)": a parenthesised denominator belongs to the unit
        if end > k && "/*".contains(chars[end - 1]) && chars.get(end) == Some(&'(') {
            if let Some(close) = chars[end..].iter().position(|c| *c == ')') {
                end += close + 1;
            }
        }
        // 16.2" is inches (the bare word "in" is not taken as a unit)
        let inch = matches!(chars.get(j), Some('"' | '”'));
        let (k, end) = if inch { (j, j + 1) } else { (k, end) };
        let ratio = chars.get(j) == Some(&'/')
            && chars.get(j + 1).is_some_and(char::is_ascii_digit)
            || (i >= 2 && chars[i - 1] == '/' && chars[i - 2].is_ascii_digit());
        let before_long = sentence_tail(&chars[i.saturating_sub(200)..i]).to_lowercase();
        let after_text: String = chars[j..(j + 30).min(chars.len())].iter().collect();
        let after_words: Vec<String> = after_text
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect();
        let example = ["example", "e.g", "format", "such as"]
            .iter()
            .any(|w| before_long.contains(w));
        let tolerance_value = before_long.trim_end().ends_with(['±']);
        // "round to 2 decimal places" is an instruction, not data
        let rounding = after_words.first().is_some_and(|w| {
            [
                "decimal",
                "decimals",
                "digit",
                "digits",
                "significant",
                "place",
                "places",
            ]
            .contains(&w.as_str())
        }) || (before_long.contains("round")
            && before_long.trim_end().ends_with(" to"));
        let tail: String = chars[end..(end + 6).min(chars.len())].iter().collect();
        let uncertain = tail.trim_start().starts_with('±') || tail.trim_start().starts_with("+/-");
        // the whole word must be a unit; "in" (inch) and "t" (tonne) are too often plain English ("0 to 100 in 4 s")
        let unit: String = chars[k..end].iter().collect();
        let unit = unit.trim_end_matches(['/', '*']);
        // a standalone capital G after a number is standard gravity (g stays gram)
        let spaced = unit.replace(' ', "*");
        let unit = if unit.contains('(') {
            spaced.as_str()
        } else {
            unit
        };
        let unit = match unit {
            "\"" | "”" => "in",
            "G" => "g0",
            "degree" | "degrees" => "deg",
            u => u,
        };
        let with_unit = if unit.is_empty() || (["in", "t"].contains(&unit) && !inch) || ratio {
            None
        } else {
            units::parse_quantity(&format!("{number} {unit}"))
                .ok()
                .filter(|q| q.dims != DIMENSIONLESS || unit == "%" || ANGLE_UNITS.contains(&unit))
                .map(|q| (format!("{number} {unit}"), q.dims))
        };
        let consumed_unit = with_unit.is_some();
        // a bare number ("103 cells", "friction coefficient of 1.4") is kept as a dimensionless candidate; it only
        // fills a variable whose description matches the words around it
        let found = with_unit.or_else(|| Some((number.clone(), DIMENSIONLESS)));
        let skip = example || tolerance_value || ratio || (rounding && !consumed_unit);
        if let (Some((text, dims)), false) = (found, skip) {
            let start = i - usize::from(negative);
            let before = sentence_tail(&chars[start.saturating_sub(60).max(prev_end)..start]);
            let after = sentence_head(&chars[end..(end + 24).min(chars.len())]);
            let trimmed = before.trim_end_matches([' ', '=', ':']);
            let label = trimmed.rsplit(' ').next().unwrap_or("").to_lowercase();
            let labelled = before.trim_end_matches(' ').ends_with('=');
            let context = joined_words(&format!("{before} {after}"))
                .into_iter()
                .collect();
            let per_item = after
                .split(|c: char| !c.is_alphabetic())
                .any(|w| ["each", "per", "every"].contains(&w.to_lowercase().as_str()));
            out.push(Quantity {
                text,
                dims,
                context,
                label,
                labelled,
                per_item,
                uncertain,
            });
        }
        prev_end = if consumed_unit { end } else { j };
        i = end.max(j).max(i + 1);
    }
    out
}

/// The text after the last sentence break.
fn sentence_tail(chars: &[char]) -> String {
    let s: String = chars.iter().collect();
    let cut = [". ", "? ", "! ", "; ", "\n"]
        .iter()
        .filter_map(|b| s.rfind(b).map(|p| p + b.len()))
        .max()
        .unwrap_or(0);
    s[cut..].to_string()
}

/// The text up to the first sentence break, comma or number.
fn sentence_head(chars: &[char]) -> String {
    let s: String = chars.iter().collect();
    let cut = s
        .char_indices()
        .find(|(_, c)| {
            matches!(c, '.' | '?' | '!' | ';' | ',' | '(' | ')' | '\n') || c.is_ascii_digit()
        })
        .map(|(p, _)| p)
        .unwrap_or(s.len());
    s[..cut].to_string()
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

const QUALIFIERS: &[&str] = &[
    "highest", "lowest", "maximum", "minimum", "max", "min", "initial", "final", "before", "after",
    "inner", "outer", "front", "rear", "left", "right", "upper", "lower", "peak",
];

const PER_ITEM_WORDS: &[&str] = &["per", "each", "single", "cell", "individual", "unit", "one"];

fn is_delta(name: &str) -> bool {
    let d = registry().var(name).desc.to_lowercase();
    ["change", "difference", "rise", "delta"]
        .iter()
        .any(|w| d.contains(w))
}

/// The score of a quantity for a variable (None = it may not fill it): words shared with the description, plus a
/// bonus when the quantity is labelled with the variable's own name ("f = 200 Hz"). `rivals` are the free variables
/// of the same dimension in the formula.
fn pair_score(q: &Quantity, name: &str, rivals: &[&String]) -> Option<usize> {
    let v = registry().var(name);
    if var_dims(name) != q.dims || q.uncertain || q.text == "0" {
        return None;
    }
    // rpm and Hz share dimensions but differ by 2 pi: a rotational speed is not a frequency
    if q.text.ends_with("rpm") == (v.unit == "Hz")
        && (q.text.ends_with("rpm") || q.text.ends_with("Hz"))
    {
        return None;
    }
    // a temperature written in degC is absolute; a variable holding a difference must not take it
    if q.text.ends_with("degC") && is_delta(name) {
        return None;
    }
    let desc = joined_words(&v.desc);
    if q.per_item && !PER_ITEM_WORDS.iter().any(|w| desc.contains(*w)) {
        return None;
    }
    // "highest temperature", "initial voltage": the qualifier has to be in the words too
    let head = v.desc.split(['(', ',']).next().unwrap_or("");
    let quals: Vec<String> = joined_words(head)
        .into_iter()
        .filter(|w| QUALIFIERS.contains(&w.as_str()))
        .collect();
    let named = q.label == name.to_lowercase();
    if !named && !quals.is_empty() && !quals.iter().any(|w| q.context.contains(w)) {
        return None;
    }
    let shared: Vec<&String> = q.context.iter().filter(|w| desc.contains(*w)).collect();
    // a variable with a default (gauge pressures, g, rho_air) is only overridden when the words clearly name it
    if v.default.is_some() && !named && shared.len() < 2 {
        return None;
    }
    // a single shared word is only evidence when no rival variable of that dimension has it too
    if shared.len() == 1
        && !named
        && rivals.iter().filter(|r| r.as_str() != name).any(|r| {
            let rd = joined_words(&registry().var(r).desc);
            shared.iter().any(|w| rd.contains(*w))
        })
    {
        return None;
    }
    Some(shared.len() + if named { 3 } else { 0 })
}

/// Number of description words, parentheses included.
fn desc_len(name: &str) -> usize {
    words(&registry().var(name).desc).len()
}

/// Whether a quantity with no matching words may still fill a variable: the description must be short (a qualified
/// one like "mass on the axle considered" needs the words to name it), no other variable of that dimension may be
/// what the words around the quantity talk about, and a symbol label ("A = ...") must be this variable's own.
fn plain_enough(q: &Quantity, name: &str) -> bool {
    if desc_len(name) > 2 || (q.labelled && q.label != name.to_lowercase()) {
        return false;
    }
    !registry().vars.values().any(|o| {
        o.name != name && var_dims(&o.name) == q.dims && {
            let d = joined_words(&o.desc);
            q.context.iter().any(|w| d.contains(w))
        }
    })
}

/// Map quantities to variables of the same dimension, leaving a variable blank rather than guessing. A pair is
/// filled when the words around the quantity match the variable's description (or the quantity is labelled with its
/// name) and neither side has an equally good alternative. Unitless numbers, percentages and angles always need
/// matching words; a plain quantity that is the only one of its dimension may also fill the plainest free variable
/// of that dimension when that is clearly plainer than the others and the question does not ask for it.
fn prefill(names: &[String], qs: &[Quantity], ask: &HashSet<String>) -> Vec<(String, String)> {
    let r = registry();
    let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
    for (qi, q) in qs.iter().enumerate() {
        let rivals: Vec<&String> = names
            .iter()
            .filter(|n| r.var(n).default.is_none() && var_dims(n) == q.dims)
            .collect();
        let scored: Vec<(usize, usize)> = names
            .iter()
            .enumerate()
            .filter_map(|(ni, n)| pair_score(q, n, &rivals).map(|s| (ni, s)))
            .collect();
        pairs.extend(
            scored
                .iter()
                .filter(|(_, s)| *s > 0)
                .map(|(ni, s)| (qi, *ni, *s)),
        );
        let only_of_dims =
            q.dims != DIMENSIONLESS && qs.iter().filter(|o| o.dims == q.dims).count() == 1;
        if only_of_dims && scored.iter().all(|(_, s)| *s == 0) {
            let mut plain: Vec<(usize, usize)> = scored
                .iter()
                .filter(|(ni, _)| {
                    r.var(&names[*ni]).default.is_none()
                        && plain_enough(q, &names[*ni])
                        && !joined_words(&r.var(&names[*ni]).desc)
                            .iter()
                            .any(|w| ask.contains(w))
                })
                .map(|(ni, _)| (desc_len(&names[*ni]), *ni))
                .collect();
            plain.sort();
            let unique = match plain.as_slice() {
                [only] => Some(only.1),
                [a, b, ..] if a.0 < b.0 => Some(a.1),
                _ => None,
            };
            // a clear winner: every other free variable of the dimension has a longer description
            let clear = |ni: &usize| {
                rivals
                    .iter()
                    .all(|n| **n == names[*ni] || desc_len(n) > desc_len(&names[*ni]))
            };
            if let Some(ni) = unique.filter(clear) {
                pairs.push((qi, ni, 0));
            }
        }
    }
    let best = |of: &dyn Fn(&(usize, usize, usize)) -> usize, id: usize| {
        let mut mine: Vec<&(usize, usize, usize)> = pairs.iter().filter(|p| of(p) == id).collect();
        mine.sort_by_key(|p| std::cmp::Reverse(p.2));
        match mine.as_slice() {
            [only] => Some(**only),
            [a, b, ..] if a.2 > b.2 => Some(**a),
            _ => None,
        }
    };
    let mut out = Vec::new();
    for (qi, q) in qs.iter().enumerate() {
        let Some((_, ni, _)) = best(&|p| p.0, qi) else {
            continue;
        };
        if best(&|p| p.1, ni).is_some_and(|p| p.0 == qi) {
            out.push((names[ni].clone(), q.text.clone()));
        }
    }
    out
}

/// The sentence that asks something: the one ending in "?" or opening with what/how/calculate/determine/find
/// (the last such sentence; quiz questions put the ask at the end).
fn ask_sentence(text: &str) -> Option<String> {
    let sentences: Vec<&str> = text
        .split_inclusive(['.', '?', '!'])
        .map(str::trim)
        .collect();
    sentences
        .iter()
        .rev()
        .find(|s| {
            let l = s.to_lowercase();
            s.ends_with('?')
                || [
                    "what",
                    "how",
                    "calculate",
                    "determine",
                    "find",
                    "compute",
                    "estimate",
                ]
                .iter()
                .any(|w| l.starts_with(w))
        })
        .map(|s| s.to_string())
}

/// Quantity words a question asks for, and the unit that quantity is measured in.
const ASKED_QUANTITIES: &[(&str, &str)] = &[
    ("speed", "m/s"),
    ("velocity", "m/s"),
    ("acceleration", "m/s**2"),
    ("deceleration", "m/s**2"),
    ("force", "N"),
    ("load", "N"),
    ("power", "W"),
    ("energy", "J"),
    ("work", "J"),
    ("heat", "J"),
    ("current", "A"),
    ("voltage", "V"),
    ("resistance", "ohm"),
    ("capacitance", "F"),
    ("inductance", "H"),
    ("time", "s"),
    ("duration", "s"),
    ("distance", "m"),
    ("length", "m"),
    ("height", "m"),
    ("radius", "m"),
    ("diameter", "m"),
    ("mass", "kg"),
    ("torque", "N*m"),
    ("pressure", "Pa"),
    ("stress", "Pa"),
    ("temperature", "K"),
    ("frequency", "Hz"),
    ("charge", "C"),
];

/// The not-yet-known variable the question asks for: it must carry the dimension of a quantity named in the asking
/// sentence ("What is the maximum speed?" -> a velocity) or, failing that, share description words with it.
/// Description words break ties between candidates of the same dimension; a remaining tie means no guess.
fn target(names: &[String], prefill: &[(String, String)], question: &str) -> Option<String> {
    let sentence = ask_sentence(question)?;
    let ask = words(&sentence);
    let raw: HashSet<String> = sentence
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .collect();
    let asked_dims: Vec<Dims> = ASKED_QUANTITIES
        .iter()
        .filter(|(w, _)| raw.contains(*w))
        .filter_map(|(_, u)| units::unit_of(u).ok().map(|q| q.dims))
        .collect();
    let r = registry();
    let mut scored: Vec<(usize, &String)> = names
        .iter()
        .filter(|n| !prefill.iter().any(|(p, _)| p == *n) && r.var(n).default.is_none())
        .map(|n| {
            let by_dims = asked_dims.contains(&var_dims(n));
            let overlap = words(&r.var(n).desc)
                .iter()
                .filter(|w| ask.contains(*w))
                .count();
            (if by_dims { 10 + overlap } else { overlap }, n)
        })
        .filter(|(c, _)| *c > 0)
        .collect();
    scored.sort_by_key(|s| std::cmp::Reverse(s.0));
    match scored.as_slice() {
        [] => None,
        [only] => Some(only.1.clone()),
        [a, b, ..] => (a.0 > b.0).then(|| a.1.clone()),
    }
}

pub fn find(question: &str, limit: usize) -> Vec<Hit> {
    let q = words(question);
    let qs = quantities(question);
    let ask = ask_sentence(question)
        .map(|s| joined_words(&s))
        .unwrap_or_default();
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
        let prefill = prefill(&f.names, &qs, &ask);
        let target = target(&f.names, &prefill, question);
        hits.push(Hit {
            kind: "formula",
            id: f.key.clone(),
            title: f.title.clone(),
            score,
            prefill,
            target,
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
                target: None,
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
                target: None,
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
        let texts: Vec<&str> = qs
            .iter()
            .filter(|q| q.dims != DIMENSIONLESS)
            .map(|q| q.text.as_str())
            .collect();
        assert_eq!(texts, ["280 kg", "100 km/h", "4.2 s", "80 kW"]);
        assert!(qs.iter().any(|q| q.text == "0" && q.dims == DIMENSIONLESS));
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
    fn unitless_numbers_fill_only_when_named() {
        let hits = find(
            "A car of mass 240 kg with ClA 3.2 m² drives the skidpad (radius 9.125 m). With a tyre friction \
             coefficient of 1.4, what is the maximum speed?",
            5,
        );
        let hit = hits
            .iter()
            .find(|h| h.id == "cornering_downforce")
            .expect("cornering_downforce");
        assert!(
            hit.prefill.contains(&("mu".into(), "1.4".into())),
            "{:?}",
            hit.prefill
        );
        let hits = find(
            "The accumulator has 103 cells in series at 3.8 V. What is the current at 30 kW?",
            5,
        );
        let hit = hits.iter().find(|h| h.id == "battery_load").unwrap();
        assert!(
            hit.prefill.contains(&("N_s".into(), "103".into())),
            "{:?}",
            hit.prefill
        );
        assert!(
            !hit.prefill.iter().any(|(n, _)| n == "N_p"),
            "{:?}",
            hit.prefill
        );
    }

    #[test]
    fn the_asked_variable_becomes_the_target() {
        let hits = find("The accumulator has 103 cells in series at 3.8 V and 0.08 Ω. What current is drawn at 30 kW?", 5);
        assert_eq!(
            hits.iter()
                .find(|h| h.id == "battery_load")
                .unwrap()
                .target
                .as_deref(),
            Some("I")
        );
        let hits = find(
            "A car of mass 240 kg with ClA 3.2 m² drives the skidpad (radius 9.125 m), friction coefficient 1.4. \
             What is the maximum speed?",
            5,
        );
        assert_eq!(
            hits.iter()
                .find(|h| h.id == "cornering_downforce")
                .unwrap()
                .target
                .as_deref(),
            Some("v")
        );
        // no question sentence, no guess
        let hits = find("battery 103 cells 3.8 V 30 kW", 5);
        assert!(hits.iter().all(|h| h.target.is_none()));
    }

    #[test]
    fn examples_carry_known_key_warnings() {
        let hits = find("endurance time score without finish points legacy", 10);
        let ex = hits.iter().find(|h| h.id == "Q171").expect("Q171 example");
        assert!(ex.warning.as_deref().unwrap().contains("+25"));
    }

    fn texts(src: &str) -> Vec<String> {
        quantities(src).into_iter().map(|q| q.text).collect()
    }

    #[test]
    fn caret_exponents_read_like_double_star() {
        let qs = quantities("g = 9.81 m/s^2, A = 1.1 m^2, rate 5 s^-1, 3 mm^2");
        let found: Vec<(&str, Dims)> = qs.iter().map(|q| (q.text.as_str(), q.dims)).collect();
        let acc = units::unit_of("m/s**2").unwrap().dims;
        let area = units::unit_of("m**2").unwrap().dims;
        let rate = units::unit_of("1/s").unwrap().dims;
        assert_eq!(
            found,
            [
                ("9.81 m/s**2", acc),
                ("1.1 m**2", area),
                ("5 s**-1", rate),
                ("3 mm**2", area)
            ]
        );
    }

    #[test]
    fn thousands_separators_and_decimal_commas() {
        assert_eq!(
            texts("flying at 40 000 ft and 1 500 kg"),
            ["40000 ft", "1500 kg"]
        );
        assert_eq!(texts("3,000 rpm"), ["3000 rpm"]);
        assert_eq!(texts("1,000,000 N"), ["1000000 N"]);
        assert_eq!(
            texts("1,8 kN and 0,500 kg and 12,25 m"),
            ["1.8 kN", "0.500 kg", "12.25 m"]
        );
        // groups that are not exactly three digits, or a leading 0, stay separate numbers
        assert_eq!(texts("from 0 100 km/h"), ["0", "100 km/h"]);
        assert_eq!(texts("10 20 kg"), ["10", "20 kg"]);
    }

    #[test]
    fn capital_g_is_gravity_and_lowercase_g_is_gram() {
        let g = quantities("a lateral load of 1,8 G");
        assert_eq!(g[0].text, "1.8 g0");
        assert_eq!(g[0].dims, units::unit_of("m/s**2").unwrap().dims);
        let gram = quantities("a 5 g sample");
        assert_eq!(gram[0].dims, units::unit_of("kg").unwrap().dims);
        assert!((units::convert(&gram[0].text, "kg").unwrap() - 0.005).abs() < 1e-12);
    }

    #[test]
    fn negatives_angles_inches_and_compound_units() {
        assert_eq!(texts("from −55 °C to 150 °C"), ["-55 degC", "150 degC"]);
        assert_eq!(
            texts("steering angle of 20 degrees and 0.3 rad"),
            ["20 deg", "0.3 rad"]
        );
        assert_eq!(texts("a 16.2\" tyre"), ["16.2 in"]);
        let qs = quantities("steel with 460 J/(kg K) at 20 °C");
        assert_eq!(qs[0].dims, units::unit_of("J/(kg*K)").unwrap().dims);
    }

    #[test]
    fn instructions_ratios_and_tolerances_are_not_data() {
        assert!(texts("round to 2 decimal places").is_empty());
        assert!(texts("weight distribution 50/50 front to rear").is_empty());
        assert!(texts("Answer format: if the result is 12.3 mm enter 12.3").is_empty());
        assert!(texts("a resistor of 50 Ω ±10 %")
            .iter()
            .all(|t| t != "10 %"));
        let r = quantities("a resistor of 50 Ω ±10 %");
        assert!(r[0].uncertain);
    }

    fn prefill_of(formula: &str, question: &str) -> Vec<(String, String)> {
        find(question, 40)
            .into_iter()
            .find(|h| h.id == formula)
            .map(|h| h.prefill)
            .unwrap_or_default()
    }

    #[test]
    fn two_quantities_of_one_dimension_need_disambiguating_words() {
        let p = prefill_of(
            "rc_lowpass",
            "A first-order low-pass filter has a cut-off frequency at f0 = 300 Hz. What is the phase shift of \
             the signal at f = 200 Hz?",
        );
        assert!(p.contains(&("f_c".into(), "300 Hz".into())), "{p:?}");
        assert!(p.contains(&("f".into(), "200 Hz".into())), "{p:?}");
        let p = prefill_of(
            "rc_lowpass",
            "A low-pass filter is driven by a tone; the two numbers given are 300 Hz and 200 Hz. What is the phase?",
        );
        assert!(p.is_empty(), "no words to tell them apart: {p:?}");
    }

    #[test]
    fn variables_with_defaults_are_not_overridden_by_stray_numbers() {
        let p = prefill_of(
            "pneumatic_bottle",
            "On a hot day at 40 °C and 750 mmHg a gas bottle of 0.6 L holds 115 bar; one shot exhausts 500 mL \
             into the atmosphere. How many shots?",
        );
        assert!(!p.iter().any(|(n, _)| n == "p_b2"), "{p:?}");
        assert!(!p.contains(&("V_bot".into(), "500 mL".into())), "{p:?}");
    }

    #[test]
    fn per_item_and_other_symbols_do_not_fill_totals() {
        let p = prefill_of(
            "ts_discharge",
            "The tractive system has 900 uF capacitance each and cells of 3.65 V. How long to discharge to 60 V?",
        );
        assert!(!p.iter().any(|(n, _)| n == "C" || n == "V_0"), "{p:?}");
        let p = prefill_of(
            "cornering_downforce",
            "Skidpad with m = 240 kg, Cl = 3.2 and A = 1 m², mu = 1.4. What is the speed?",
        );
        assert!(
            !p.iter().any(|(n, _)| n == "ClA"),
            "A is the area, ClA is Cl times A: {p:?}"
        );
    }
}
