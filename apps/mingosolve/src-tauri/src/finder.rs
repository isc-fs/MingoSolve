//! Finder (paste a problem, get the scripts that fit, pre-filled), option matching and answer formatting.

use fsq::answer::{self, Matching, Precision};
use fsq::cli::pretty_unit;
use fsq::finder::{find, quantities, Hit};
use fsq::units::DIMENSIONLESS;
use serde::Serialize;

#[derive(Serialize)]
pub struct Found {
    hits: Vec<Hit>,
    /// Quantities with units detected in the text, as typed ("3.8 V").
    quantities: Vec<String>,
}

/// Scripts (formulas and tools) that fit a pasted problem; past-question hits are left out of the app.
#[tauri::command]
pub async fn find_question(text: String) -> Result<Found, String> {
    crate::blocking(move || Ok(find_question_impl(&text))).await
}

pub fn find_question_impl(text: &str) -> Found {
    let hits = find(text, 24)
        .into_iter()
        .filter(|h| h.kind != "example")
        .take(8)
        .collect();
    // chips show only values with units; unitless numbers still pre-fill when their words name a variable
    let quantities = quantities(text)
        .into_iter()
        .filter(|q| q.dims != DIMENSIONLESS)
        .map(|q| pretty_unit(&q.text))
        .collect();
    Found { hits, quantities }
}

#[tauri::command]
pub async fn match_options(answer: String, options: String) -> Result<Matching, String> {
    crate::blocking(move || Ok(match_options_impl(&answer, &options))).await
}

pub fn match_options_impl(answer: &str, options: &str) -> Matching {
    answer::match_options(answer, options)
}

#[tauri::command]
pub async fn format_answer(
    value: f64,
    precision: Precision,
    decimal_comma: bool,
) -> Result<String, String> {
    crate::blocking(move || answer::format_answer(value, precision, decimal_comma)).await
}
