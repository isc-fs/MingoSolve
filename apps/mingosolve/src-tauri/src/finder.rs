//! Quiz-speed helpers: question finder with pre-fill, multiple-choice option matching, answer formatting and the
//! past-question examples (with known-wrong-key warnings).

use std::collections::HashMap;

use fsq::answer::{self, known_keys, Matching, Precision};
use fsq::finder::{find, Hit};
use serde::Serialize;

#[tauri::command]
pub fn find_question(text: String) -> Vec<Hit> {
    find(&text, 12)
}

#[tauri::command]
pub fn match_options(value: f64, options: String) -> Matching {
    answer::match_options(value, &options)
}

#[tauri::command]
pub fn format_answer(value: f64, precision: Precision, decimal_comma: bool) -> String {
    answer::format_answer(value, precision, decimal_comma)
}

#[derive(Serialize)]
pub struct ExampleInfo {
    id: u32,
    what: String,
    cmd: String,
    answer: f64,
    warning: Option<String>,
}

#[tauri::command]
pub fn list_examples() -> Vec<ExampleInfo> {
    let keys: HashMap<u32, String> = known_keys().into_iter().map(|k| (k.id, k.note)).collect();
    fsq::cli::examples()
        .into_iter()
        .map(|e| ExampleInfo { warning: keys.get(&e.id).cloned(), id: e.id, what: e.what, cmd: e.cmd, answer: e.answer })
        .collect()
}
