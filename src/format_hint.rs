//! Reads how a pasted question wants its answer given: the rounding ("round to one decimal place", "3 significant
//! digits", "to the nearest tenth"), the unit ("Answer in kN", "(answer in m/s)", "[kW]" after "answer") and, when
//! the unit is stated right after the asked quantity, that quantity ("average lap speed in km/h?"). Patterns come
//! from the phrasings in the real FS-Quiz bank; a unit is only taken from an answer instruction (never from the
//! given values: "a 200 kg car") and only when the engine's unit parser accepts it. Conflicting instructions in one
//! question (two different roundings or units) are dropped rather than guessed.

use serde::Serialize;

use crate::answer::Precision;
use crate::units::{dims_str, unit_of, DIMENSIONLESS};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FormatHint {
    pub rounding: Option<Precision>,
    /// Display unit in the engine's syntax (`km/h`, `N*m`, `percent`).
    pub unit: Option<String>,
    /// The unit as the question wrote it (`kilonewtons`, `km/h`), for the note shown to the user.
    pub unit_label: Option<String>,
    /// What the unit measures (`dims_key`): a variable only takes the unit when its own key is equal.
    pub dims: Option<String>,
    /// The asked quantity when the question names it next to the unit ("average lap speed").
    pub quantity: Option<String>,
}

/// Dimension key of a unit string. Angles are their own kind (radian is dimensionless in the engine, so without
/// this "in percent" would fit an angle and "in Hz" would fit rad/s).
pub fn dims_key(unit: &str) -> Option<String> {
    let q = unit_of(unit).ok()?;
    let angular = unit
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|t| ANGLE_UNITS.contains(&t));
    let base = if q.dims == DIMENSIONLESS {
        "dimensionless".to_string()
    } else {
        dims_str(q.dims)
    };
    Some(if angular {
        format!("{base}·angle")
    } else {
        base
    })
}

const ANGLE_UNITS: [&str; 6] = ["rad", "deg", "degree", "degrees", "rpm", "rev"];

/// Unit words (lower case, typos seen in the bank included) and their engine syntax.
const WORDS: &[(&str, &str)] = &[
    ("newton", "N"),
    ("newtons", "N"),
    ("kilonewton", "kN"),
    ("kilonewtons", "kN"),
    ("meter", "m"),
    ("meters", "m"),
    ("metre", "m"),
    ("metres", "m"),
    ("millimeter", "mm"),
    ("millimeters", "mm"),
    ("millimetre", "mm"),
    ("millimetres", "mm"),
    ("centimeter", "cm"),
    ("centimeters", "cm"),
    ("centimetre", "cm"),
    ("centimetres", "cm"),
    ("kilometer", "km"),
    ("kilometers", "km"),
    ("kilometre", "km"),
    ("kilometres", "km"),
    ("second", "s"),
    ("seconds", "s"),
    ("secound", "s"),
    ("secounds", "s"),
    ("millisecond", "ms"),
    ("milliseconds", "ms"),
    ("minute", "min"),
    ("minutes", "min"),
    ("hour", "h"),
    ("hours", "h"),
    ("hertz", "Hz"),
    ("herz", "Hz"),
    ("kilohertz", "kHz"),
    ("volt", "V"),
    ("volts", "V"),
    ("millivolt", "mV"),
    ("millivolts", "mV"),
    ("ampere", "A"),
    ("amperes", "A"),
    ("amp", "A"),
    ("amps", "A"),
    ("watt", "W"),
    ("watts", "W"),
    ("kilowatt", "kW"),
    ("kilowatts", "kW"),
    ("joule", "J"),
    ("joules", "J"),
    ("kilojoule", "kJ"),
    ("kilojoules", "kJ"),
    ("pascal", "Pa"),
    ("pascals", "Pa"),
    ("kelvin", "K"),
    ("gram", "g"),
    ("grams", "g"),
    ("gramm", "g"),
    ("gramms", "g"),
    ("kilogram", "kg"),
    ("kilograms", "kg"),
    ("kilogramm", "kg"),
    ("kilogramms", "kg"),
    ("liter", "L"),
    ("liters", "L"),
    ("litre", "L"),
    ("litres", "L"),
    ("degree", "deg"),
    ("degrees", "deg"),
    ("radian", "rad"),
    ("radians", "rad"),
    ("percent", "percent"),
    ("percentage", "percent"),
    ("ohm", "ohm"),
    ("ohms", "ohm"),
    ("farad", "F"),
    ("coulomb", "C"),
    ("horsepower", "hp"),
    ("kwh", "kWh"),
    ("hz", "Hz"),
    ("j", "J"),
    ("bar", "bar"),
];

/// English words that are also unit names; never read as a unit.
const NOT_UNITS: [&str; 18] = [
    "a", "an", "at", "as", "be", "is", "it", "or", "of", "on", "to", "in", "no", "so", "the",
    "and", "are", "one",
];

const QUESTION_WORDS: [&str; 7] = [
    "what",
    "how",
    "calculate",
    "determine",
    "find",
    "compute",
    "enter",
];
const TO_BE_VERBS: [&str; 8] = [
    "calculated",
    "given",
    "expressed",
    "stated",
    "reported",
    "computed",
    "displayed",
    "provided",
];

struct Tok {
    raw: String,
    /// Lower case, edge punctuation and brackets removed.
    lc: String,
    /// Same without lower-casing: units are case-sensitive.
    core: String,
}

impl Tok {
    fn ends(&self, c: char) -> bool {
        self.raw.ends_with(c)
    }

    fn starts_with(&self, c: char) -> bool {
        self.raw.starts_with(c)
    }

    /// `[kW]` (with any trailing punctuation): the text inside.
    fn bracket(&self) -> Option<&str> {
        let t = self
            .raw
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
        t.strip_prefix('[')?.strip_suffix(']')
    }
}

fn tokens(text: &str) -> Vec<Tok> {
    text.replace("<br>", " ")
        .split_whitespace()
        .map(|raw| {
            let core = raw
                .trim_start_matches(['(', '[', '{', '"', '\'', '“'])
                .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'', '”'])
                .to_string();
            Tok {
                raw: raw.to_string(),
                lc: core.to_lowercase(),
                core,
            }
        })
        .collect()
}

fn number_word(s: &str) -> Option<u32> {
    const WORDS: [&str; 11] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    if let Some(i) = WORDS.iter().position(|w| *w == s) {
        return Some(i as u32);
    }
    if s == "first" {
        return Some(1);
    }
    (s.len() <= 2).then(|| s.parse().ok()).flatten()
}

fn is_round_cue(t: &Tok) -> bool {
    t.lc.starts_with("round")
}

/// A rounding word or a format verb within the few tokens before `i`, without crossing a question or exclamation.
fn instruction_before(t: &[Tok], i: usize, window: usize) -> bool {
    for k in (i.saturating_sub(window)..i).rev() {
        if k + 1 < i && (t[k].ends('?') || t[k].ends('!')) {
            return false;
        }
        if is_round_cue(&t[k])
            || [
                "provide", "give", "enter", "express", "format", "write", "written",
            ]
            .contains(&t[k].lc.as_str())
        {
            return true;
        }
    }
    false
}

fn decimals_of_example(s: &str) -> Option<u32> {
    let s = s.trim_start_matches("eg:").trim_start_matches("e.g.:");
    let (int, frac) = s.split_once(['.', ','])?;
    let digits = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
    (digits(int) && digits(frac)).then_some(frac.len() as u32)
}

fn roundings(t: &[Tok]) -> Vec<Precision> {
    let mut out = Vec::new();
    for i in 0..t.len() {
        let lc = t[i].lc.as_str();
        let prev = |k: usize| i.checked_sub(k).map(|j| &t[j]);
        if lc == "decimal" || lc == "decimals" {
            if let Some(p) = prev(1) {
                if p.lc == "nearest" {
                    // "round to nearest decimal eg:12.34": the example fixes the count
                    let example = t[i + 1..]
                        .iter()
                        .take(4)
                        .find(|x| x.core.chars().any(|c| c.is_ascii_digit()))
                        .and_then(|x| decimals_of_example(&x.core));
                    if let Some(n) = example {
                        out.push(Precision::Decimals(n));
                    }
                    continue;
                }
                if p.lc == "no" || p.lc == "without" {
                    out.push(Precision::Decimals(0));
                    continue;
                }
                if let Some(n) = number_word(&p.lc) {
                    let parenthetical = t[i].ends(')') && p_is_after_comma(t, i);
                    if parenthetical || instruction_before(t, i, 8) {
                        out.push(Precision::Decimals(n));
                    }
                    continue;
                }
            }
        }
        // "1 digit behind decimal point"
        if lc == "decimal" {
            let mut k = 1;
            if prev(k).is_some_and(|p| p.lc == "the") {
                k += 1;
            }
            if prev(k).is_some_and(|p| p.lc == "behind" || p.lc == "after")
                && prev(k + 1).is_some_and(|p| p.lc == "digit" || p.lc == "digits")
            {
                if let Some(n) = prev(k + 2).and_then(|p| number_word(&p.lc)) {
                    if instruction_before(t, i, 10) {
                        out.push(Precision::Decimals(n));
                    }
                }
            }
        }
        if lc.starts_with("significant") {
            if let Some(n) = prev(1).and_then(|p| number_word(&p.lc)) {
                if (1..=15).contains(&n) {
                    out.push(Precision::Sig(n));
                }
            }
        }
        // "rounded to 0.01", "round to .1"
        if let Some(frac) = t[i]
            .core
            .strip_prefix("0.")
            .or_else(|| t[i].core.strip_prefix("0,"))
            .or_else(|| t[i].core.strip_prefix('.'))
        {
            if frac.ends_with('1')
                && frac[..frac.len() - 1].chars().all(|c| c == '0')
                && instruction_before(t, i, 3)
            {
                out.push(Precision::Decimals(frac.len() as u32));
            }
        }
        if lc == "nearest" {
            let next = t.get(i + 1).map(|x| x.lc.as_str()).unwrap_or("");
            match next {
                "integer" | "one" | "whole" => out.push(Precision::Decimals(0)),
                "tenth" => out.push(Precision::Decimals(1)),
                "hundredth" => out.push(Precision::Decimals(2)),
                "thousandth" => out.push(Precision::Decimals(3)),
                _ => {}
            }
        }
        // "round to a tenth", "round to integer", "round to whole numbers", "round to full number"
        let near_round = (1..=5).any(|k| prev(k).is_some_and(is_round_cue));
        if near_round {
            match lc {
                "tenth" => out.push(Precision::Decimals(1)),
                "hundredth" => out.push(Precision::Decimals(2)),
                "thousandth" => out.push(Precision::Decimals(3)),
                "integer" | "whole" | "full" => out.push(Precision::Decimals(0)),
                _ => {}
            }
        }
    }
    out.dedup();
    out
}

/// True when the token before the count ends with a comma: "(in Watt, 1 decimal)".
fn p_is_after_comma(t: &[Tok], i: usize) -> bool {
    i >= 2 && t[i - 2].ends(',')
}

/// One unit read at token `i`: engine syntax, label as written, tokens consumed.
fn unit_at(t: &[Tok], i: usize, strict: bool) -> Option<(String, String, usize)> {
    let first = t.get(i)?;
    if first.core.is_empty() {
        return None;
    }
    let word = |k: usize| t.get(k).map(|x| x.lc.as_str());
    if matches!(first.lc.as_str(), "kilowatt" | "watt")
        && matches!(word(i + 1), Some("hour" | "hours"))
    {
        let u = if first.lc == "watt" { "Wh" } else { "kWh" };
        return Some((u.into(), format!("{} hours", first.lc), 2));
    }
    if word(i + 1) == Some("per") {
        if let (Some(a), Some(b)) = (
            word_unit(&first.lc),
            t.get(i + 2).and_then(|x| word_unit(&x.lc)),
        ) {
            let u = format!("{a}/{b}");
            if unit_of(&u).is_ok() {
                return Some((u, format!("{} per {}", first.lc, t[i + 2].lc), 3));
            }
        }
    }
    if let Some(u) = word_unit(&first.lc) {
        return Some((u.into(), first.lc.clone(), 1));
    }
    if strict && first.core.len() < 2 {
        return None;
    }
    if let Some(u) = symbol_unit(&first.core) {
        return Some((u, first.core.clone(), 1));
    }
    // "in secounds [s]": the bracket after a misspelled word still names the unit
    let alt = t.get(i + 1)?.bracket()?;
    symbol_unit(alt).map(|u| (u, alt.to_string(), 2))
}

fn word_unit(lc: &str) -> Option<&'static str> {
    WORDS.iter().find(|(w, _)| *w == lc).map(|(_, u)| *u)
}

/// A unit written as symbols (`kN`, `m/s`, `m^3`, `Nm`), accepted when it parses and does not look like a word.
fn symbol_unit(raw: &str) -> Option<String> {
    let first = raw.chars().next()?;
    if !(first.is_alphabetic() || matches!(first, '%' | '°' | 'µ' | 'Ω'))
        || NOT_UNITS.contains(&raw)
    {
        return None;
    }
    if raw == "°" {
        return Some("deg".into());
    }
    if raw.contains("°C") || raw.contains("°F") || raw.contains("degC") || raw.contains("degF") {
        return None;
    }
    let plain = raw.chars().all(|c| c.is_alphabetic());
    if plain && raw.chars().count() >= 5 && raw.chars().all(|c| c.is_lowercase()) {
        return None;
    }
    let mut s = raw
        .replace('^', "**")
        .replace('²', "**2")
        .replace('³', "**3")
        .replace('⁴', "**4")
        .replace('·', "*")
        .replace('Ω', "ohm");
    for (from, to) in [("Nm", "N*m"), ("Ns", "N*s")] {
        if s == from || s.starts_with(&format!("{from}/")) {
            s = s.replacen(from, to, 1);
        }
    }
    if !s
        .chars()
        .all(|c| c.is_alphanumeric() || "%/*µ°-".contains(c))
    {
        return None;
    }
    unit_of(&s).is_ok().then_some(s)
}

struct UnitHit {
    unit: String,
    label: String,
    quantity: Option<String>,
}

fn units(t: &[Tok]) -> Vec<UnitHit> {
    let mut out = Vec::new();
    let mut push = |read: Option<(String, String, usize)>, quantity: Option<String>| {
        if let Some((unit, label, _)) = read {
            out.push(UnitHit {
                unit,
                label,
                quantity,
            });
        }
    };
    let question_before = |i: usize, window: usize| {
        (i.saturating_sub(window)..i).any(|k| QUESTION_WORDS.contains(&t[k].lc.as_str()))
    };
    for i in 0..t.len() {
        let lc = t[i].lc.as_str();
        let prev_lc = |k: usize| i.checked_sub(k).map(|j| t[j].lc.as_str());
        let next_is_in = t.get(i + 1).is_some_and(|x| x.lc == "in");
        // "Answer in kN", "(answer in m/s)", "Enter the solution in Newton", "State the results in seconds"
        let answer_word = matches!(lc, "answer" | "answers" | "solution" | "solutions")
            || (matches!(lc, "result" | "results")
                && matches!(prev_lc(1), Some("the" | "your" | "final" | "these")));
        if answer_word && next_is_in {
            let mut at = i + 2;
            if t.get(at).is_some_and(|x| x.lc == "the")
                && t.get(at + 1).is_some_and(|x| x.lc.starts_with("unit"))
            {
                at += 2;
                if t.get(at).is_some_and(|x| x.lc == "of") {
                    at += 1;
                }
            }
            push(unit_at(t, at, false), None);
            continue;
        }
        // "Answer [kW]"
        if lc == "answer" {
            if let Some(b) = t.get(i + 1).and_then(Tok::bracket) {
                push(symbol_unit(b).map(|u| (u, b.to_string(), 2)), None);
                continue;
            }
        }
        if lc != "in" {
            // "calculate the time in secounds [s]"
            if let Some(b) = t[i].bracket() {
                let word_before = prev_lc(1).is_some_and(|w| w.chars().all(|c| c.is_alphabetic()));
                let single_upper = b.len() == 1 && b.chars().all(|c| c.is_ascii_uppercase());
                let cue = (i.saturating_sub(10)..i)
                    .any(|k| QUESTION_WORDS.contains(&t[k].lc.as_str()) || t[k].lc == "give");
                if word_before && !single_upper && cue {
                    push(symbol_unit(b).map(|u| (u, b.to_string(), 1)), None);
                }
            }
            continue;
        }
        let Some(next) = t.get(i + 1) else { continue };
        // "... is to be calculated in bar!", "should be given in milliseconds"
        if prev_lc(2) == Some("be") && prev_lc(1).is_some_and(|v| TO_BE_VERBS.contains(&v)) {
            push(unit_at(t, i + 1, false), None);
            continue;
        }
        // "...?  (in m/s, rounded to full number)"
        if t[i].starts_with('(')
            && i > 0
            && t[i - 1].ends('?')
            && (next.ends(',') || next.ends(')'))
        {
            push(unit_at(t, i + 1, false), None);
            continue;
        }
        // "Calculate the force in bearing point FB in Newton!"
        if next.ends('!')
            && (i.saturating_sub(12)..i).any(|k| QUESTION_WORDS.contains(&t[k].lc.as_str()))
        {
            push(unit_at(t, i + 1, false), None);
            continue;
        }
        // "What is the average lap speed in km/h?"
        if next.ends('?') && question_before(i, 14) {
            if let Some(read) = unit_at(t, i + 1, true) {
                let quantity = asked_quantity(t, i);
                push(Some(read), quantity);
            }
        }
    }
    out
}

/// "What is the average lap speed in": the words between the question's lead-in and `in`.
fn asked_quantity(t: &[Tok], in_at: usize) -> Option<String> {
    let start = (0..in_at)
        .rev()
        .find(|&k| QUESTION_WORDS.contains(&t[k].lc.as_str()))?;
    let mut words: Vec<&str> = t[start + 1..in_at].iter().map(|x| x.lc.as_str()).collect();
    while words.first().is_some_and(|w| {
        [
            "is", "are", "was", "will", "would", "be", "can", "the", "a", "an",
        ]
        .contains(w)
    }) {
        words.remove(0);
    }
    (!words.is_empty()
        && words.len() <= 6
        && words
            .iter()
            .all(|w| w.chars().all(|c| c.is_alphabetic() || c == '-')))
    .then(|| words.join(" "))
}

/// Percent asked without the word "in": "give the answer as a percentage", "What percentage of ...", "round to the
/// nearest whole percent".
fn percent_cue(t: &[Tok]) -> Option<UnitHit> {
    let hit = |label: &str| {
        Some(UnitHit {
            unit: "percent".into(),
            label: label.into(),
            quantity: None,
        })
    };
    let is_percent = |w: &str| w == "percent" || w == "percentage";
    for i in 0..t.len() {
        let lc = t[i].lc.as_str();
        let prev = |k: usize| i.checked_sub(k).map(|j| t[j].lc.as_str());
        if lc == "as" {
            let at = if t.get(i + 1).is_some_and(|x| x.lc == "a") {
                i + 2
            } else {
                i + 1
            };
            let said = (i.saturating_sub(8)..i).any(|k| {
                [
                    "give", "provide", "express", "enter", "state", "answer", "solution",
                ]
                .contains(&t[k].lc.as_str())
            });
            if said && t.get(at).is_some_and(|x| is_percent(&x.lc)) {
                return hit("percentage");
            }
        }
        if is_percent(lc) {
            let asked = matches!(prev(1), Some("what"))
                || (matches!(prev(1), Some("much" | "many")) && matches!(prev(2), Some("how")));
            let rounded = matches!(prev(1), Some("whole" | "nearest" | "integer"))
                && (1..=5).any(|k| i.checked_sub(k).is_some_and(|j| is_round_cue(&t[j])));
            if asked || rounded {
                return hit(lc);
            }
        }
    }
    None
}

/// What the question asks for: `None` when it says nothing usable about rounding or unit.
pub fn parse(text: &str) -> Option<FormatHint> {
    let t = tokens(text);
    let rounds = roundings(&t);
    let rounding = match rounds.as_slice() {
        [first, rest @ ..] if rest.iter().all(|r| r == first) => Some(*first),
        _ => None,
    };
    let mut found = units(&t);
    found.extend(percent_cue(&t));
    let unit = match found.as_slice() {
        [first, rest @ ..] if rest.iter().all(|u| u.unit == first.unit) => Some(first),
        _ => None,
    };
    let quantity = unit.and_then(|u| {
        found
            .iter()
            .find(|x| x.unit == u.unit && x.quantity.is_some())?
            .quantity
            .clone()
    });
    let hint = FormatHint {
        rounding,
        dims: unit.and_then(|u| dims_key(&u.unit)),
        unit_label: unit.map(|u| u.label.clone()),
        unit: unit.map(|u| u.unit.clone()),
        quantity,
    };
    (hint.rounding.is_some() || hint.unit.is_some()).then_some(hint)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rounding(text: &str) -> Option<Precision> {
        parse(text).and_then(|h| h.rounding)
    }

    fn unit(text: &str) -> Option<String> {
        parse(text).and_then(|h| h.unit)
    }

    #[test]
    fn decimals_as_the_bank_words_them() {
        let d = |n| Some(Precision::Decimals(n));
        assert_eq!(rounding("Round the solution to one decimal place."), d(1));
        assert_eq!(
            rounding("Enter the solution in Hertz and round solution to two decimal places."),
            d(2)
        );
        assert_eq!(
            rounding("Round the solution to three decimal places."),
            d(3)
        );
        assert_eq!(
            rounding("Answer in Newton and round to 0 decimal places."),
            d(0)
        );
        assert_eq!(
            rounding("please provide the value with 1 digit behind decimal point, e.g. 12.3"),
            d(1)
        );
        assert_eq!(
            rounding("provide the value with 0 digit behind decimal point, e.g. 12"),
            d(0)
        );
        assert_eq!(rounding("Use the following format: 12.3 (no ',' comma, no letters), round final result to 2 decimal places."), d(2));
        assert_eq!(
            rounding("give the time in seconds rounded to the nearest two decimals."),
            d(2)
        );
        assert_eq!(
            rounding("Give the answer in Horsepower [hp], and round two decimals (i.e. 1.12)."),
            d(2)
        );
        assert_eq!(
            rounding(
                "How much power does it need? (in Watt, 1 decimal) Please use the following format"
            ),
            d(1)
        );
        assert_eq!(rounding("Round to first decimal."), d(1));
        assert_eq!(
            rounding("Answer rounded to .1 Please use the following format"),
            d(1)
        );
        assert_eq!(rounding("Answer in kilonewtons rounded to 0.01"), d(2));
        assert_eq!(
            rounding("(answer format round to nearest decimal eg:12.34)"),
            d(2)
        );
    }

    #[test]
    fn whole_numbers_and_nearest() {
        let d = |n| Some(Precision::Decimals(n));
        assert_eq!(
            rounding("Round to nearest integer. Please use the following format"),
            d(0)
        );
        assert_eq!(
            rounding("Give the answer in grams and round to the nearest one."),
            d(0)
        );
        assert_eq!(
            rounding("Enter the solution in Newton and round to whole numbers."),
            d(0)
        );
        assert_eq!(
            rounding("Enter the solution in Hertz and round to the whole numbers."),
            d(0)
        );
        assert_eq!(rounding("(in m/s, rounded to full number)"), d(0));
        assert_eq!(
            rounding("Give the answer in millimetres and round to the nearest tenth."),
            d(1)
        );
        assert_eq!(
            rounding("Give the answer in Ohm and round the answer to a tenth."),
            d(1)
        );
        assert_eq!(rounding("Round to the nearest whole gram."), d(0));
    }

    #[test]
    fn significant_digits() {
        assert_eq!(
            rounding("Use the following format: 12.3 or 1.23 (no ',' comma, no latters), round to 3 significant digits."),
            Some(Precision::Sig(3))
        );
        assert_eq!(
            rounding("Give two significant figures."),
            Some(Precision::Sig(2))
        );
    }

    #[test]
    fn rounding_needs_a_number_and_an_instruction() {
        // the bank's boilerplate gives no count
        assert_eq!(
            rounding("(no ',' comma, no letters), round to requested significant digits."),
            None
        );
        // a round object, a given value's precision, a count that is not an instruction
        assert_eq!(
            rounding("A round straight tube with a wall of 1.5 mm is loaded."),
            None
        );
        assert_eq!(
            rounding("The sensor reports the angle with 2 decimal places of resolution."),
            None
        );
        assert_eq!(rounding("Round the voltage in the figure and the answer to the nearest decimal (e.g. 3.15V becomes 3.2)"), None);
        assert_eq!(
            rounding("Numerator and denominator must be integers, e.g. 1/2"),
            None
        );
    }

    #[test]
    fn conflicting_roundings_are_dropped() {
        assert_eq!(rounding("Round the first result to one decimal place and round the second to two decimal places."), None);
        assert_eq!(
            rounding("Round both solutions to one decimal place. Give the solution in the following format: vc, vF"),
            Some(Precision::Decimals(1))
        );
    }

    #[test]
    fn units_from_answer_instructions() {
        let u = |t| unit(t).unwrap_or_default();
        assert_eq!(u("Answer in kilonewtons rounded to 0.01"), "kN");
        assert_eq!(u("What is the force? Answer in [Nm] rounded to .1"), "N*m");
        assert_eq!(u("Answer in [m], rounded to 0.01"), "m");
        assert_eq!(u("(answer in m/s)"), "m/s");
        assert_eq!(u("(answer in Ns/mm) m = 3 kg"), "N*s/mm");
        assert_eq!(
            u("Enter the solution in km/h and round to one decimal place."),
            "km/h"
        );
        assert_eq!(
            u("State the results in milliseconds. Round to one decimal place."),
            "ms"
        );
        assert_eq!(
            u("Enter the result in bar! Round the solution to one decimal place."),
            "bar"
        );
        assert_eq!(
            u("The value T2 is to be calculated in Kelvin! Round solution to one decimal place."),
            "K"
        );
        assert_eq!(
            u("Please calculate the force in bearing point FB in Newton! Round the solution"),
            "N"
        );
        assert_eq!(
            u("Provide your answer in kWh. Use the following format: 12.3"),
            "kWh"
        );
        assert_eq!(
            u("Provide your answer in deg and follow the sign convention"),
            "deg"
        );
        assert_eq!(u("Answer: in degrees, round to 0.01"), "deg");
        assert_eq!(u("Give your answer in m^3!"), "m**3");
        assert_eq!(u("Answer in cm⁴. Use the following format: 12.3"), "cm**4");
        assert_eq!(
            u("Provide your answer in percentage. Use the following format"),
            "percent"
        );
        assert_eq!(u("Give the answer as a percentage."), "percent");
        assert_eq!(
            u("Give the answer in Horsepower [hp], and round two decimals"),
            "hp"
        );
        assert_eq!(
            u("Calculate the time in secounds [s] necessary to stop."),
            "s"
        );
        assert_eq!(
            u("The result should be given in milliseconds [ms]. Note: x"),
            "ms"
        );
        assert_eq!(u("Answer in kilometers per hour."), "km/h");
        assert_eq!(u("Answer in kilowatt hours."), "kWh");
        assert_eq!(u("Answer in kilogramms and round to 1 decimal place"), "kg");
        assert_eq!(u("(absolute value of your answer in °)"), "deg");
    }

    #[test]
    fn percent_without_the_word_in() {
        let h = parse("What percentage of the power is transmitted? Round your answer to the nearest whole percent.").unwrap();
        assert_eq!(h.unit.as_deref(), Some("percent"));
        assert_eq!(h.rounding, Some(Precision::Decimals(0)));
        assert_eq!(
            unit("How much percent increase is that?").as_deref(),
            Some("percent")
        );
        assert_eq!(unit("The efficiency is 93,6%. What is the mass?"), None);
    }

    #[test]
    fn asked_quantity_with_its_unit() {
        let h = parse("What is the average lap speed in km/h?").unwrap();
        assert_eq!(h.unit.as_deref(), Some("km/h"));
        assert_eq!(h.quantity.as_deref(), Some("average lap speed"));
        assert_eq!(h.dims.as_deref(), Some("m·s^-1"));
        let h = parse("What is the total energy consumed during the event in kWh?").unwrap();
        assert_eq!(h.unit.as_deref(), Some("kWh"));
        assert_eq!(
            h.quantity.as_deref(),
            Some("total energy consumed during the event")
        );
    }

    #[test]
    fn units_of_the_given_values_are_not_the_answer_unit() {
        assert_eq!(
            unit("A 200 kg car with a power of 80 kW drives at 50 km/h. How long does it take?"),
            None
        );
        assert_eq!(
            unit("The mass is given in kg and the speed in km/h. What is the force?"),
            None
        );
        assert_eq!(
            unit("Mass m = 5 [kg], voltage 3,3 [V], supplied from the rail. Find the current."),
            None
        );
        assert_eq!(unit("Which team won the EV Class in FS2024?"), None);
        assert_eq!(
            unit("The cat falls onto the ground in balance with the tire."),
            None
        );
        assert_eq!(
            unit("Answer in points and round to 2 decimal places."),
            None
        );
        assert_eq!(unit("Answer in dB and round to 0 decimal places."), None);
        assert_eq!(unit("Give the answer in the following format: 12.3"), None);
        assert_eq!(unit("Answer in a table"), None);
        assert_eq!(unit("That may result in a penalty of 5 s."), None);
    }

    #[test]
    fn temperature_in_celsius_is_not_offered() {
        assert_eq!(unit("Answer in [°C] and round to 1 decimal place"), None);
        assert_eq!(
            rounding("Give the answer in [°C]. Round to 1 decimal place"),
            Some(Precision::Decimals(1))
        );
    }

    #[test]
    fn conflicting_units_are_dropped() {
        assert_eq!(
            unit("Answer in Newton for the first part. Answer in kN for the second."),
            None
        );
    }

    #[test]
    fn dimension_keys_keep_angles_apart() {
        assert_eq!(dims_key("deg"), dims_key("rad"));
        assert_ne!(dims_key("deg"), dims_key("percent"));
        assert_ne!(dims_key("Hz"), dims_key("rad/s"));
        assert_eq!(dims_key("rpm"), dims_key("rad/s"));
        assert_eq!(dims_key("kN"), dims_key("N"));
        assert_eq!(dims_key("percent"), dims_key("dimensionless"));
    }

    #[test]
    fn no_instruction_no_hint() {
        assert_eq!(
            parse("A car of 250 kg accelerates from rest. What is the speed after 3 s?"),
            None
        );
    }
}
