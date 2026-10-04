//! Procedural tools (scoring, rule tables, circuits...): catalogue with typed parameters, and running one.

use fsq::tools::{tool, tools, Kind};
use serde::Serialize;

#[derive(Serialize)]
pub struct ParamInfo {
    name: String,
    number: bool,
    default: Option<String>,
}

#[derive(Serialize)]
pub struct ToolInfo {
    name: String,
    doc: String,
    params: Vec<ParamInfo>,
}

#[tauri::command]
pub fn list_tools() -> Vec<ToolInfo> {
    tools()
        .iter()
        .map(|t| ToolInfo {
            name: t.name.into(),
            doc: t.doc.into(),
            params: t
                .params
                .iter()
                .map(|p| ParamInfo {
                    name: p.name.into(),
                    number: p.kind == Kind::Num,
                    default: p.default.map(String::from),
                })
                .collect(),
        })
        .collect()
}

/// Run with `name=value` pairs; blank values fall back to the tool's defaults.
#[tauri::command]
pub fn run_tool(name: String, args: Vec<(String, String)>) -> Result<String, String> {
    let t = tool(&name).ok_or_else(|| format!("no tool {name}"))?;
    let pairs: Vec<String> = args
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| format!("{k}={}", v.trim()))
        .collect();
    let refs: Vec<&str> = pairs.iter().map(String::as_str).collect();
    t.call(&refs)
}
