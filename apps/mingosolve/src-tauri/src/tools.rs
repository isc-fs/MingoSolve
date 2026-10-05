//! Procedural tools (scoring, rule tables, circuits...): catalogue with typed parameters, and running one.

use fsq::tool_params::{param_meta, tool_meta, with_unit};
use fsq::tools::{tool, tools, Kind};
use serde::Serialize;

#[derive(Serialize)]
pub struct ParamInfo {
    name: String,
    number: bool,
    default: Option<String>,
    /// Plain-English name (data/tool_params.toml).
    label: String,
    /// Unit a bare number is read in; the form shows it next to the input.
    unit: Option<String>,
    help: Option<String>,
    /// Allowed values for a select, with a label each.
    choices: Option<Vec<Choice>>,
    /// On/off flag sent as 1 or 0.
    switch: bool,
}

#[derive(Serialize)]
pub struct Choice {
    value: String,
    label: String,
}

#[derive(Serialize)]
pub struct ToolInfo {
    name: String,
    doc: String,
    /// One plain sentence; `doc` keeps the technical description.
    summary: String,
    params: Vec<ParamInfo>,
}

#[tauri::command]
pub fn list_tools() -> Vec<ToolInfo> {
    tools()
        .iter()
        .map(|t| ToolInfo {
            name: t.name.into(),
            doc: t.doc.into(),
            summary: tool_meta(t.name).map_or_else(String::new, |m| m.summary.clone()),
            params: t
                .params
                .iter()
                .map(|p| {
                    let m = param_meta(t.name, p.name);
                    ParamInfo {
                        name: p.name.into(),
                        number: p.kind == Kind::Num,
                        default: p.default.map(String::from),
                        label: m.map_or_else(|| p.name.to_string(), |m| m.label.clone()),
                        unit: m.and_then(|m| m.unit.clone()),
                        help: m.and_then(|m| m.help.clone()),
                        choices: m.and_then(|m| {
                            m.choices.as_ref().map(|c| {
                                c.iter()
                                    .enumerate()
                                    .map(|(i, v)| Choice {
                                        value: v.clone(),
                                        label: m
                                            .choice_labels
                                            .as_ref()
                                            .map_or_else(|| v.clone(), |l| l[i].clone()),
                                    })
                                    .collect()
                            })
                        }),
                        switch: m.is_some_and(|m| m.switch),
                    }
                })
                .collect(),
        })
        .collect()
}

/// Run with `name=value` pairs; blank values fall back to the tool's defaults, and a bare number is read in the
/// unit the form shows next to its input.
#[tauri::command]
pub async fn run_tool(name: String, args: Vec<(String, String)>) -> Result<String, String> {
    crate::blocking(move || run_tool_impl(&name, &args)).await
}

pub fn run_tool_impl(name: &str, args: &[(String, String)]) -> Result<String, String> {
    let t = tool(name).ok_or_else(|| format!("no tool {name}"))?;
    let pairs: Vec<String> = args
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| format!("{k}={}", with_unit(name, k, v)))
        .collect();
    let refs: Vec<&str> = pairs.iter().map(String::as_str).collect();
    t.call(&refs)
}
