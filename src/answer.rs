//! Quiz answer helpers: match a computed value against pasted multiple-choice options (nearest, with a warning
//! when two options are close), format a value the way the quiz expects (decimal comma or point, significant
//! figures or decimals), and the list of official keys known not to match the physics or rules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OptionMatch {
    pub text: String,
    pub value: Option<f64>,
    /// Relative difference to the computed value (None if the option has no number).
    pub rel_diff: Option<f64>,
    pub best: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Matching {
    pub options: Vec<OptionMatch>,
    /// Two options are both within 2 % of the value, or the best is more than 2 % off: check by hand.
    pub warning: Option<String>,
}

/// First number in an option ("31,75 km/h" -> 31.75, "1.2e3 N" -> 1200). A lone comma is a decimal comma.
pub fn option_number(text: &str) -> Option<f64> {
    let t = if text.matches(',').count() == 1 && !text.contains('.') {
        text.replace(',', ".")
    } else {
        text.to_string()
    };
    let chars: Vec<char> = t.chars().collect();
    let start = chars.iter().enumerate().position(|(i, c)| {
        c.is_ascii_digit() || (*c == '-' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
    })?;
    let mut end = start + 1;
    while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
        end += 1;
    }
    if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
        let mut j = end + 1;
        if j < chars.len() && (chars[j] == '-' || chars[j] == '+') {
            j += 1;
        }
        if j < chars.len() && chars[j].is_ascii_digit() {
            end = j;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
        }
    }
    let n: f64 = chars[start..end].iter().collect::<String>().parse().ok()?;
    n.is_finite().then_some(n)
}

/// Options separated by new lines (or `;` / `|` when pasted on one line).
pub fn match_options(value: f64, options: &str) -> Matching {
    let lines: Vec<&str> = if options.contains('\n') {
        options.lines().collect()
    } else {
        options.split([';', '|']).collect()
    };
    let mut out: Vec<OptionMatch> = lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            let v = option_number(l);
            OptionMatch {
                text: l.to_string(),
                value: v,
                rel_diff: v.map(|v| (v - value).abs() / value.abs().max(1e-300)),
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
    let warning = match ranked.as_slice() {
        [] => Some("no numeric options found".into()),
        [(b, d), rest @ ..] => {
            out[*b].best = true;
            if *d > 0.02 {
                Some(format!(
                    "nearest option is {:.1} % off: check units, rounding or the convention",
                    d * 100.0
                ))
            } else if rest.first().is_some_and(|(_, d2)| *d2 <= 0.02) {
                Some("two options are within 2 %: check the rounding the question asks for".into())
            } else {
                None
            }
        }
    };
    Matching {
        options: out,
        warning,
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    Sig(u32),
    Decimals(u32),
}

/// Most significant figures or decimals the formatter accepts (an f64 carries 17).
pub const MAX_PRECISION: u32 = 50;

/// Value as the quiz wants it typed.
pub fn format_answer(
    value: f64,
    precision: Precision,
    decimal_comma: bool,
) -> Result<String, String> {
    if !value.is_finite() {
        return Err(crate::units::NOT_FINITE.into());
    }
    let s = match precision {
        Precision::Decimals(d) if d <= MAX_PRECISION => format!("{value:.*}", d as usize),
        Precision::Sig(sig) if (1..=MAX_PRECISION).contains(&sig) => {
            if value == 0.0 {
                "0".into()
            } else {
                let exp = value.abs().log10().floor() as i32;
                let decimals = (sig as i32 - 1 - exp).max(0) as usize;
                let k = 10f64.powi(sig as i32 - 1 - exp);
                let rounded = (value * k).round() / k;
                format!(
                    "{:.decimals$}",
                    if rounded.is_finite() { rounded } else { value }
                )
            }
        }
        _ => return Err(format!("precision must be 1 to {MAX_PRECISION}")),
    };
    Ok(if decimal_comma {
        s.replace('.', ",")
    } else {
        s
    })
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

    #[test]
    fn numbers_in_options() {
        assert_eq!(option_number("31,75 km/h"), Some(31.75));
        assert_eq!(option_number("a) 1.2e3 N"), Some(1200.0));
        assert_eq!(option_number("-33.7°"), Some(-33.7));
        assert_eq!(option_number("There is no lower bound"), None);
        assert_eq!(option_number("560, 30"), Some(560.0));
    }

    #[test]
    fn picks_nearest_and_warns_on_close_calls() {
        // Q517: 31.77 km/h computed, key option 31,75
        let m = match_options(31.77, "28,5 km/h\n31,75 km/h\n35,2 km/h\n40 km/h");
        assert!(m.options[1].best);
        assert!(m.warning.is_none());
        // Q378 legacy 41.12 vs 71.25/3.75 variant 41.24: both in the options
        let m = match_options(41.117, "41.12; 41.24; 38.5");
        assert!(m.options[0].best);
        assert!(m.warning.unwrap().contains("two options"));
        let m = match_options(100.0, "80 | 90 | 120");
        assert!(m.warning.unwrap().contains("off"));
    }

    #[test]
    fn formats_like_the_quiz() {
        assert_eq!(
            format_answer(77.887_89, Precision::Decimals(1), true).unwrap(),
            "77,9"
        );
        assert_eq!(
            format_answer(0.002_041_666_7, Precision::Sig(5), false).unwrap(),
            "0.0020417"
        );
        assert_eq!(
            format_answer(14_567.8, Precision::Sig(3), false).unwrap(),
            "14600"
        );
        assert_eq!(
            format_answer(-33.690_07, Precision::Sig(3), true).unwrap(),
            "-33,7"
        );
    }
}
