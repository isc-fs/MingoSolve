//! Rules reference: every embedded rule set (points, Tmax/Pmin, PCB spacing...) and the 2027 change notes.

use std::collections::HashMap;

#[tauri::command]
pub fn rule_sets() -> HashMap<String, toml::Table> {
    let mut all = fsq::tools::rule_sets().clone();
    all.insert("penalties".into(), fsq::tools::penalties().clone());
    all
}

#[tauri::command]
pub fn rules_changes() -> &'static str {
    include_str!("../../../../docs/rules-2027-changes.md")
}
