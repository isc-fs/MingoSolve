//! Topic library (data/topics.toml): every script, formula or procedural tool, filed under one engineering
//! domain, plus the past questions that use a script as worked examples.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::cli::{examples, Example};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub blurb: String,
    pub hue: String,
    pub formulas: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Deserialize)]
struct File {
    topic: Vec<Topic>,
    tool_titles: HashMap<String, String>,
    aliases: HashMap<String, String>,
}

fn file() -> &'static File {
    static F: OnceLock<File> = OnceLock::new();
    F.get_or_init(|| toml::from_str(crate::data::TOPICS).expect("data/topics.toml"))
}

pub fn topics() -> &'static [Topic] {
    &file().topic
}

/// Short title of a procedural tool.
pub fn tool_title(name: &str) -> Option<&'static str> {
    file().tool_titles.get(name).map(String::as_str)
}

/// Words people use for a script that its title, tags and variables don't contain (searched alongside them).
pub fn aliases(script: &str) -> &'static str {
    file().aliases.get(script).map_or("", String::as_str)
}

/// The topic a formula key or tool name is filed under.
pub fn topic_of(script: &str) -> Option<&'static Topic> {
    topics()
        .iter()
        .find(|t| t.formulas.iter().chain(&t.tools).any(|s| s == script))
}

/// Past questions whose command runs this script (formula key, tool name, or a chain through it).
pub fn worked_examples(script: &str) -> Vec<Example> {
    examples()
        .into_iter()
        .filter(|e| e.cmd.split_whitespace().next() == Some(script))
        .collect()
}

/// Past questions solved with `chain <target> ...` (no single script owns them).
pub fn chain_examples() -> Vec<Example> {
    worked_examples("chain")
}

/// Past questions solved with `calc <expression>`.
pub fn calc_examples() -> Vec<Example> {
    worked_examples("calc")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::registry::registry;
    use crate::tools::tools;

    #[test]
    fn every_script_is_filed_exactly_once() {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for t in topics() {
            for s in t.formulas.iter().chain(&t.tools) {
                assert!(seen.insert(s, &t.id).is_none(), "{s} is in two topics");
            }
        }
        for f in &registry().formulas {
            assert!(
                seen.contains_key(f.key.as_str()),
                "formula {} has no topic",
                f.key
            );
        }
        for t in tools() {
            assert!(seen.contains_key(t.name), "tool {} has no topic", t.name);
        }
        let known: Vec<&str> = registry()
            .formulas
            .iter()
            .map(|f| f.key.as_str())
            .chain(tools().iter().map(|t| t.name))
            .collect();
        for s in seen.keys() {
            assert!(known.contains(s), "topics.toml names unknown script {s}");
        }
    }

    #[test]
    fn every_tool_has_a_title_and_no_metadata_names_a_missing_script() {
        let f = file();
        for t in tools() {
            assert!(
                tool_title(t.name).is_some(),
                "tool {} has no entry in [tool_titles]",
                t.name
            );
        }
        for k in f.tool_titles.keys() {
            assert!(
                tools().iter().any(|t| t.name == k),
                "[tool_titles] names unknown tool {k}"
            );
        }
        for k in f.aliases.keys() {
            assert!(topic_of(k).is_some(), "[aliases] names unknown script {k}");
        }
    }

    #[test]
    fn chain_and_calc_lists_hold_exactly_the_rows_of_examples_toml() {
        #[derive(Deserialize)]
        struct Rows {
            example: Vec<Row>,
        }
        #[derive(Deserialize)]
        struct Row {
            id: u32,
            cmd: String,
        }
        let rows: Rows = toml::from_str(include_str!("../data/examples.toml")).unwrap();
        for head in ["chain", "calc"] {
            let want: Vec<u32> = rows
                .example
                .iter()
                .filter(|r| r.cmd.starts_with(&format!("{head} ")))
                .map(|r| r.id)
                .collect();
            let got: Vec<u32> = match head {
                "chain" => chain_examples(),
                _ => calc_examples(),
            }
            .iter()
            .map(|e| e.id)
            .collect();
            assert!(!want.is_empty(), "no {head} rows in examples.toml");
            assert_eq!(got, want, "{head} examples");
        }
    }

    #[test]
    fn worked_examples_link_to_scripts() {
        assert!(worked_examples("battery_load").iter().any(|e| e.id == 34));
        assert!(worked_examples("event_score").iter().any(|e| e.id == 378));
        assert_eq!(topic_of("nodal").unwrap().id, "circuits");
    }
}
