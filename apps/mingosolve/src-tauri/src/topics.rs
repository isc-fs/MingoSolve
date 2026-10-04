//! Topic library: topics with their scripts (formulas and procedural tools), and a script's worked examples.

use fsq::registry::registry;
use fsq::tools::tool;
use fsq::topics::{topics, worked_examples};
use serde::Serialize;

#[derive(Serialize)]
pub struct ScriptRef {
    /// Formula key or tool name.
    id: String,
    kind: &'static str,
    title: String,
}

#[derive(Serialize)]
pub struct TopicInfo {
    id: String,
    name: String,
    blurb: String,
    hue: String,
    scripts: Vec<ScriptRef>,
}

fn script_ref(id: &str) -> Option<ScriptRef> {
    if let Some(f) = registry().formula(id) {
        return Some(ScriptRef {
            id: id.into(),
            kind: "formula",
            title: f.title.clone(),
        });
    }
    tool(id).map(|t| ScriptRef {
        id: id.into(),
        kind: "tool",
        title: t
            .doc
            .split(['.', ':'])
            .next()
            .unwrap_or(t.doc)
            .trim()
            .to_string(),
    })
}

#[tauri::command]
pub fn list_topics() -> Vec<TopicInfo> {
    topics()
        .iter()
        .map(|t| TopicInfo {
            id: t.id.clone(),
            name: t.name.clone(),
            blurb: t.blurb.clone(),
            hue: t.hue.clone(),
            scripts: t
                .formulas
                .iter()
                .chain(&t.tools)
                .filter_map(|s| script_ref(s))
                .collect(),
        })
        .collect()
}

#[derive(Serialize)]
pub struct WorkedExample {
    id: u32,
    what: String,
    cmd: String,
    answer: f64,
}

#[tauri::command]
pub fn script_examples(script: String) -> Vec<WorkedExample> {
    worked_examples(&script)
        .into_iter()
        .map(|e| WorkedExample {
            id: e.id,
            what: e.what,
            cmd: e.cmd,
            answer: e.answer,
        })
        .collect()
}
