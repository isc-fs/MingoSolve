//! Quiz answer helpers: match a computed value against pasted multiple-choice options (units and thousands
//! separators understood, nearest wins, warning when two options are close), format a value the way the quiz
//! expects (decimal comma or point, significant figures or decimals), and the list of official keys known not to
//! match the physics or rules.

use serde::{Deserialize, Serialize};

use crate::units::{self, Quantity};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OptionMatch {
    pub text: String,
    /// The option's number in the answer's unit (None if it has no number or a different kind of unit).
    pub value: Option<f64>,
    /// Relative difference to the answer (None if the option was not compared).
    pub rel_diff: Option<f64>,
    pub best: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Matching {
    pub options: Vec<OptionMatch>,
    /// Two options are both within 2 % of the value, or the best is more than 2 % off, or an option was
    /// skipped for its unit: check by hand.
    pub warning: Option<String>,
}

/// A number as typed in an option or shown as an answer, with the unit written after it (if it parses as one).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typed {
    pub number: f64,
    pub unit: Option<Quantity>,
}

impl Typed {
    /// Value in SI (the plain number when there is no unit).
    pub fn si(&self) -> f64 {
        self.number * self.unit.map_or(1.0, |u| u.value)
    }
}

fn is_blank(c: char) -> bool {
    c.is_whitespace() || c == '\u{200b}' || c == '\u{feff}'
}

/// Drop a list label at the start: "a)", "(b)", "A.", "1)", "1.", "iv)", "-", "•".
fn strip_label(t: &str) -> &str {
    let t = t.trim_start_matches(is_blank);
    let c: Vec<char> = t.chars().collect();
    let blank_at = |n: usize| c.get(n).is_some_and(|x| x.is_whitespace());
    let cut = |n: usize| t.char_indices().nth(n).map_or("", |(i, _)| &t[i..]);
    if matches!(c.first(), Some('-' | '–' | '—' | '•' | '*')) && blank_at(1) {
        return cut(1);
    }
    let paren = usize::from(c.first() == Some(&'('));
    let mut i = paren;
    while c.get(i).is_some_and(|x| x.is_ascii_alphabetic()) {
        i += 1;
    }
    let letters = i - paren;
    let roman = c[paren..i].iter().all(|x| "ivxIVX".contains(*x));
    if c.get(i) == Some(&')') && (letters == 1 || (letters <= 4 && roman)) {
        return cut(i + 1);
    }
    if paren == 0 && letters == 1 && c.get(1) == Some(&'.') && blank_at(2) {
        return cut(2);
    }
    let mut i = paren;
    while c.get(i).is_some_and(|x| x.is_ascii_digit()) {
        i += 1;
    }
    let numbered = c.get(i) == Some(&')') || (c.get(i) == Some(&'.') && blank_at(i + 1));
    if i > paren && i - paren <= 2 && numbered {
        return cut(i + 1);
    }
    t
}

/// Number text with thousands and decimal separators sorted out ("1,250.5", "59,988", "77,9", "284.097,30").
/// A comma followed by exactly three digits (integer part not 0) is a thousands separator, any other is decimal.
fn normalise_separators(raw: &str) -> Option<String> {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let (dots, commas) = (s.matches('.').count(), s.matches(',').count());
    let groups_of_three = |sep: char| {
        s.split(sep)
            .skip(1)
            .all(|g| g.len() == 3 && g.chars().all(|c| c.is_ascii_digit()))
    };
    let int_part_zero = s
        .split(',')
        .next()
        .is_some_and(|i| i.trim_start_matches('0').is_empty());
    match (dots, commas) {
        (0 | 1, 0) => Some(s),
        (_, 0) => groups_of_three('.').then(|| s.replace('.', "")),
        (0, 1) if groups_of_three(',') && !int_part_zero => Some(s.replace(',', "")),
        (0, 1) => Some(s.replace(',', ".")),
        (0, _) => groups_of_three(',').then(|| s.replace(',', "")),
        _ => {
            let (last_dot, last_comma) = (s.rfind('.')?, s.rfind(',')?);
            if last_dot > last_comma && dots == 1 {
                Some(s.replace(',', ""))
            } else if last_comma > last_dot && commas == 1 {
                Some(s.replace('.', "").replace(',', "."))
            } else {
                None
            }
        }
    }
}

/// Integer exponent with optional sign at `j`, and the index after it.
fn signed_int(c: &[char], mut j: usize) -> Option<(i32, usize)> {
    let neg = matches!(c.get(j), Some('-' | '−'));
    if neg || c.get(j) == Some(&'+') {
        j += 1;
    }
    let s = j;
    while c.get(j).is_some_and(|x| x.is_ascii_digit()) {
        j += 1;
    }
    let n: i32 = c[s..j].iter().collect::<String>().parse().ok()?;
    Some((if neg { -n } else { n }, j))
}

/// Exponent after a mantissa: "e-3", "·10^-3", " × 10^-3". Returns the exponent and the index after it.
fn scan_exponent(c: &[char], i: usize) -> Option<(i32, usize)> {
    if matches!(c.get(i), Some('e' | 'E')) {
        return signed_int(c, i + 1);
    }
    let skip_blanks = |mut j: usize| {
        while c.get(j) == Some(&' ') {
            j += 1;
        }
        j
    };
    let j = skip_blanks(i);
    if !matches!(c.get(j), Some('·' | '⋅' | '×' | 'x' | 'X' | '*')) {
        return None;
    }
    let j = skip_blanks(j + 1);
    if c.get(j) != Some(&'1') || c.get(j + 1) != Some(&'0') {
        return None;
    }
    let j = skip_blanks(j + 2);
    if c.get(j) != Some(&'^') {
        return None;
    }
    signed_int(c, j + 1)
}

/// Number at `start` and the index after it, exponent included.
fn scan_number(c: &[char], start: usize) -> Option<(f64, usize)> {
    let digit = |j: usize| c.get(j).is_some_and(|x| x.is_ascii_digit());
    let mut i = start;
    let neg = matches!(c[i], '-' | '−');
    if neg || c[i] == '+' {
        i += 1;
    }
    let num_start = i;
    while i < c.len() {
        let group = c[num_start..i]
            .iter()
            .rev()
            .take_while(|x| x.is_ascii_digit())
            .count();
        let plain = !c[num_start..i].iter().any(|x| matches!(x, '.' | ','));
        let space_thousands = c[i] == ' '
            && plain
            && (1..=3).contains(&group)
            && (1..=3).all(|k| digit(i + k))
            && !digit(i + 4);
        let separator = matches!(c[i], '.' | ',') && i > num_start && digit(i + 1);
        if digit(i) || space_thousands || separator {
            i += 1;
        } else {
            break;
        }
    }
    if i == num_start {
        return None;
    }
    let raw: String = c[num_start..i].iter().collect();
    let mut value: f64 = normalise_separators(&raw)?.parse().ok()?;
    let mut end = i;
    if let Some((e, j)) = scan_exponent(c, i) {
        value *= 10f64.powi(e);
        end = j;
    }
    Some((if neg { -value } else { value }, end))
}

/// Unit text as the crate's unit parser wants it: spelled-out names, Ω, °, superscripts, "m2".
fn unit_text(tok: &str) -> String {
    let t = tok.replace(['µ', 'μ'], "u");
    let word = match t.to_lowercase().as_str() {
        "second" | "seconds" | "sec" | "secs" => Some("s"),
        "minute" | "minutes" => Some("min"),
        "hour" | "hours" => Some("h"),
        "meter" | "meters" | "metre" | "metres" => Some("m"),
        "volt" | "volts" => Some("V"),
        "joule" | "joules" => Some("J"),
        "liter" | "liters" | "litre" | "litres" => Some("l"),
        "milisecond" | "miliseconds" | "millisecond" | "milliseconds" => Some("ms"),
        "hz" => Some("Hz"),
        _ => None,
    };
    if let Some(w) = word {
        return w.into();
    }
    let t = t
        .replace("Ohm", "ohm")
        .replace(['\u{2126}', '\u{3a9}'], "ohm")
        .replace("°C", "degC")
        .replace("°F", "degF")
        .replace('°', "deg")
        .replace(['·', '⋅'], "*")
        .replace('²', "**2")
        .replace('³', "**3")
        .replace("⁻¹", "**-1")
        .replace('^', "**");
    let digits = t.chars().rev().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0
        && t.chars()
            .rev()
            .nth(digits)
            .is_some_and(|c| c.is_alphabetic())
    {
        let (head, tail) = t.split_at(t.len() - digits);
        return format!("{head}**{tail}");
    }
    t
}

/// The unit written right after a number: the first word, cut at a range dash, without brackets or sentence
/// punctuation. None when it is not a unit the parser knows ("points", "cones").
fn unit_after(rest: &[char]) -> Option<Quantity> {
    let rest: String = rest.iter().collect();
    let mut words = rest
        .trim_start_matches(|c: char| c.is_whitespace() || c == '[' || c == '(')
        .split_whitespace();
    let first = words.next()?;
    let mut tok = String::new();
    let mut prev = ' ';
    for ch in first.chars() {
        if matches!(ch, '-' | '+' | '–' | '−') && prev != '^' && prev != '*' {
            break;
        }
        tok.push(ch);
        prev = ch;
    }
    let tok = tok.trim_matches(|c: char| ".,;:[]()".contains(c));
    let starts_like_unit = tok
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || matches!(c, '%' | '°'));
    if !starts_like_unit {
        return None;
    }
    let mut text = unit_text(tok);
    // "A^2 Ω": the ohm may be written as a second word
    if let Some(w) = words.next() {
        if w.starts_with(['\u{2126}', '\u{3a9}']) || w.to_lowercase().starts_with("ohm") {
            text = format!("{text}*ohm");
        }
    }
    units::parse_quantity(&text).ok()
}

/// A number with its unit from the start of an option or of a shown answer ("b) 73.4 kN", "-33,7°", "1,250.5").
pub fn parse_typed(text: &str) -> Option<Typed> {
    let c: Vec<char> = strip_label(text)
        .chars()
        .map(|x| if is_blank(x) { ' ' } else { x })
        .collect();
    let start = (0..c.len()).find(|&i| {
        c[i].is_ascii_digit()
            || (matches!(c[i], '-' | '−') && c.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
    })?;
    let (number, end) = scan_number(&c, start)?;
    Some(Typed {
        number,
        unit: unit_after(&c[end..]),
    })
}

fn rel(option: f64, answer: f64) -> f64 {
    (option - answer).abs() / answer.abs().max(1e-300)
}

/// Options separated by new lines (or `;` / `|` when pasted on one line), compared with the answer as the sheet
/// shows it ("77.887 A"). An option with a unit is compared in SI, one without as a plain number in the
/// answer's unit; an option in an incompatible unit is skipped and named in the warning.
pub fn match_options(answer: &str, options: &str) -> Matching {
    let Some(ans) = parse_typed(answer) else {
        return Matching {
            options: vec![],
            warning: Some(format!("the answer {answer:?} is not a number")),
        };
    };
    let lines: Vec<&str> = if options.contains('\n') {
        options.lines().collect()
    } else {
        options.split([';', '|']).collect()
    };
    let mut wrong_unit: Vec<&str> = vec![];
    let mut out: Vec<OptionMatch> = lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (value, rel_diff) = match (parse_typed(l), ans.unit) {
                (None, _) => (None, None),
                (Some(o), Some(au)) if o.unit.is_some_and(|ou| ou.dims != au.dims) => {
                    wrong_unit.push(l);
                    (None, None)
                }
                (Some(o), Some(au)) if o.unit.is_some() => {
                    (Some(o.si() / au.value), Some(rel(o.si(), ans.si())))
                }
                // a bare answer (tool result) may be in SI or in the unit the option is written in
                (Some(o), None) if o.unit.is_some() => {
                    let (raw, si) = (rel(o.number, ans.number), rel(o.si(), ans.number));
                    if si < raw {
                        (Some(o.si()), Some(si))
                    } else {
                        (Some(o.number), Some(raw))
                    }
                }
                (Some(o), _) => (Some(o.number), Some(rel(o.number, ans.number))),
            };
            OptionMatch {
                text: l.to_string(),
                value,
                rel_diff,
                best: false,
            }
        })
        .collect();
    let mut ranked: Vec<(usize, f64)> = out
        .iter()
        .enumerate()
        .filter_map(|(i, o)| o.rel_diff.map(|d| (i, d)))
        .collect();
    ranked.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut warnings: Vec<String> = vec![];
    match ranked.as_slice() {
        [] => warnings.push("no numeric options found".into()),
        [(b, d), rest @ ..] => {
            out[*b].best = true;
            if *d > 0.02 {
                warnings.push(format!(
                    "nearest option is {:.1} % off: check units, rounding or the convention",
                    d * 100.0
                ));
            } else if rest.first().is_some_and(|(_, d2)| *d2 <= 0.02) {
                warnings.push(
                    "two options are within 2 %: check the rounding the question asks for".into(),
                );
            }
        }
    }
    if !wrong_unit.is_empty() {
        warnings.push(format!(
            "skipped, the unit is not the answer's: {}",
            wrong_unit.join(" / ")
        ));
    }
    Matching {
        options: out,
        warning: (!warnings.is_empty()).then(|| warnings.join("; ")),
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    Sig(u32),
    Decimals(u32),
}

/// Value as the quiz wants it typed.
pub fn format_answer(value: f64, precision: Precision, decimal_comma: bool) -> String {
    let s = match precision {
        Precision::Decimals(d) => format!("{value:.*}", d as usize),
        Precision::Sig(sig) => {
            if value == 0.0 {
                "0".into()
            } else {
                let exp = value.abs().log10().floor() as i32;
                let decimals = (sig as i32 - 1 - exp).max(0) as usize;
                let rounded = {
                    let k = 10f64.powi(sig as i32 - 1 - exp);
                    (value * k).round() / k
                };
                format!("{rounded:.decimals$}")
            }
        }
    };
    if decimal_comma {
        s.replace('.', ",")
    } else {
        s
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KnownKey {
    pub id: u32,
    pub note: String,
}

#[derive(Deserialize)]
struct KnownKeys {
    key: Vec<KnownKey>,
}

pub fn known_keys() -> Vec<KnownKey> {
    toml::from_str::<KnownKeys>(crate::data::KNOWN_KEYS)
        .expect("data/known_keys.toml")
        .key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * b.abs().max(1e-300)
    }

    /// Option strings copied from the FS-Quiz bank (question id in the comment), with the value in SI worked out
    /// by hand.
    #[test]
    fn real_bank_options_parse_to_si() {
        let table: &[(&str, f64)] = &[
            ("24.0kW", 24_000.0),               // 20
            ("17.36 ms", 0.017_36),             // 326
            ("167.874kJ", 167_874.0),           // 303
            ("135 GPa", 1.35e11),               // 1
            ("115.8N", 115.8),                  // 146
            ("70.0 km/h", 70.0 / 3.6),          // 115
            ("1.25 mW", 0.001_25),              // 772
            ("6.25 µJ", 6.25e-6),               // 772
            ("2000 µF", 0.002),                 // 107
            ("126 kΩ", 126_000.0),              // 929
            ("1 kOhm", 1000.0),                 // 221
            ("60 mΩ", 0.06),                    // 1049
            ("100 Ohm", 100.0),                 // 221
            ("0.0020416667 kWh", 7_350.000_12), // 687
            ("109646 mm^2", 0.109_646),         // 587
            ("0.497m2", 0.497),                 // 586
            ("185 cm²", 0.0185),                // 43
            ("57.0 [A/mm2]", 57.0e6),           // 575
            ("0,53 A^2 Ω", 0.53),               // 129
            ("216V DC", 216.0),                 // 343
            ("35,3°", 35.3_f64.to_radians()),   // 64
            ("2087,3 [N]", 2087.3),             // 217
            ("187,5 [rad/s]", 187.5),           // 717
            (
                "3,644903817856e+25 [eV]",
                3.644_903_817_856e25 * 1.602_176_634e-19,
            ), // 220
            ("6,4mm", 0.0064),                  // 16
            ("15,25m.", 15.25),                 // 235
            ("100 miliseconds.", 0.1),          // 263
            ("1 kN", 1000.0),                   // 765
            ("1080mm - 1920mm", 1.08),          // 37: first number of a range
            ("A. 195 000 [MPa]", 1.95e11),      // 880: label and space thousands
            ("71 000 000", 71_000_000.0),       // 319
            ("5,897 m/s", 5897.0),              // 202: comma thousands
            ("5,897 km/h", 5897.0 / 3.6),       // 202
            ("4,300 km.", 4.3e6),               // 229
            ("200 GPa (29,000 ksi)", 2.0e11),   // 344: only the first number
            ("40,788.089 ev", 40_788.089),      // 687: thousands comma and decimal point
            ("284.097,30\u{202f}€", 284_097.3), // 72: decimal comma, dot thousands, narrow nbsp
            ("1,365", 1365.0),                  // 629: three digits after a lone comma
            ("0,34cars/h", 0.34),               // 173: not a thousands group, unknown unit
            ("-1450 Euro", -1450.0),            // 40: sign, not a list dash
            ("0.1-0.5", 0.1),                   // 738
        ];
        for (text, want) in table {
            let got = parse_typed(text).unwrap_or_else(|| panic!("{text:?} has no number"));
            assert!(close(got.si(), *want), "{text:?}: {} != {want}", got.si());
        }
    }

    #[test]
    fn list_labels_are_not_numbers() {
        for label in [
            "a) ", "A. ", "(b) ", "1) ", "1. ", "i) ", "iv) ", "- ", "• ", "  c)  ",
        ] {
            let t = parse_typed(&format!("{label}77.9 A")).unwrap();
            assert!(close(t.number, 77.9), "{label:?} -> {}", t.number);
            assert!(t.unit.is_some());
        }
        assert!(close(parse_typed("1)73.4").unwrap().number, 73.4));
        assert!(close(parse_typed("1.5 m").unwrap().number, 1.5));
    }

    #[test]
    fn thousands_versus_decimal_comma() {
        for (text, want) in [
            ("3,000", 3000.0),
            ("59,988", 59_988.0),
            ("1,250.5", 1250.5),
            ("1,234,567", 1_234_567.0),
            ("77,9", 77.9),
            ("0,500", 0.5),
            ("12,5000", 12.5),
            ("40 000", 40_000.0),
            ("1.234,5", 1234.5),
        ] {
            assert!(close(parse_typed(text).unwrap().number, want), "{text}");
        }
    }

    #[test]
    fn scientific_forms() {
        for text in [
            "1.2e-3",
            "1.2E-3",
            "1.2·10^-3",
            "1.2 × 10^-3",
            "1,2 x 10^-3",
        ] {
            assert!(close(parse_typed(text).unwrap().number, 0.0012), "{text}");
        }
    }

    #[test]
    fn plain_numbers_keep_working() {
        assert!(parse_typed("There is no lower bound").is_none());
        assert_eq!(parse_typed("560, 30").unwrap().number, 560.0);
        assert_eq!(parse_typed("-33.7°").unwrap().number, -33.7);
    }

    /// (official key id, answer as the sheet shows it from the example's command, options as the bank lists them,
    /// index of the official option). Each answer was produced by running the example command.
    const END_TO_END: &[(u32, &str, &[&str], usize)] = &[
        (
            785,
            "131.79 N",
            &["121.79 N", "0.1318 kN", "1.317 kN", "1.708 kN"],
            1,
        ),
        (
            753,
            "7500 J",
            &["All wrong", "5 kJ", "10 kJ", "7.5 kJ", "2.5 kJ", "3.75 kJ"],
            3,
        ),
        (
            940,
            "7.56e+07 J",
            &["15.4 kWh", "21.0 kWh", "19.5 kWh", "24.2 kWh"],
            1,
        ),
        (
            811,
            "26.4789 m/s",
            &["61.91 km/h", "112.39 km/h", "95.32 km/h", "101.49 km/h"],
            2,
        ),
        (
            326,
            "0.0173611",
            &["17.36 ms", "15.63 ms", "13.89 ms", "20.83 ms"],
            0,
        ),
        (
            319,
            "3.6e+07",
            &["71 000 000", "12 600 000", "36 000 000", "87 500 000"],
            2,
        ),
        (
            694,
            "172.48 percent",
            &["150%", "129,38%", "172,48%", "216,38%"],
            2,
        ),
        (787, "909.091 N", &["1 kN", "880 N", "909 N", "786 N"], 2),
        (
            1,
            "1.35e+11 Pa",
            &["135 GPa", "180 GPa", "220 GPa", "280 GPa", "315 GPa"],
            0,
        ),
        (16, "9.5", &["2mm", "3mm", "6,4mm", "9,5 mm", "12,7mm"], 3),
    ];

    #[test]
    fn real_questions_pick_the_official_option() {
        for (id, answer, options, official) in END_TO_END {
            let m = match_options(answer, &options.join("\n"));
            let best = m.options.iter().position(|o| o.best);
            assert_eq!(best, Some(*official), "Q{id}: {answer} vs {options:?}");
            assert!(m.warning.is_none(), "Q{id}: {:?}", m.warning);
        }
    }

    #[test]
    fn picks_nearest_and_warns_on_close_calls() {
        // Q517: 31.77 km/h computed, key option 31,75
        let m = match_options("31.77 km/h", "28,5 km/h\n31,75 km/h\n35,2 km/h\n40 km/h");
        assert!(m.options[1].best);
        assert!(m.warning.is_none());
        // Q378 legacy 41.12 vs 71.25/3.75 variant 41.24: both in the options
        let m = match_options("41.117", "41.12; 41.24; 38.5");
        assert!(m.options[0].best);
        assert!(m.warning.unwrap().contains("two options"));
        let m = match_options("100", "80 | 90 | 120");
        assert!(m.warning.unwrap().contains("off"));
        // a label must not be read as the value
        let m = match_options("77.887 A", "a) 73.4 A\nb) 77.9 A\nc) 80.4 A\nd) 4815 A");
        assert!(m.options[1].best && m.warning.is_none());
    }

    #[test]
    fn options_in_another_kind_of_unit_are_skipped_and_named() {
        let m = match_options("77.887 A", "a) 77.9 V\nb) 80.4 A\nc) 77.9 kg");
        assert_eq!(m.options[0].rel_diff, None);
        assert_eq!(m.options[2].rel_diff, None);
        assert!(m.options[1].best);
        let w = m.warning.unwrap();
        assert!(w.contains("77.9 V") && w.contains("77.9 kg"), "{w}");
    }

    #[test]
    fn unit_free_options_use_the_displayed_unit() {
        // the answer is shown in km/h, the options carry no unit
        let m = match_options("31.77 km/h", "28.5\n31.75\n35.2");
        assert!(m.options[1].best);
    }

    #[test]
    fn an_answer_that_is_not_a_number_is_reported() {
        let m = match_options("none", "1\n2");
        assert!(m.options.is_empty() && m.warning.is_some());
    }

    #[test]
    fn formats_like_the_quiz() {
        assert_eq!(
            format_answer(77.887_89, Precision::Decimals(1), true),
            "77,9"
        );
        assert_eq!(
            format_answer(0.002_041_666_7, Precision::Sig(5), false),
            "0.0020417"
        );
        assert_eq!(format_answer(14_567.8, Precision::Sig(3), false), "14600");
        assert_eq!(format_answer(-33.690_07, Precision::Sig(3), true), "-33,7");
    }
}
