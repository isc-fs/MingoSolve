//! Topic library (data/topics.toml): every script, formula or procedural tool, filed under one engineering
//! domain, plus the past questions that use a script as worked examples.

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
}

pub fn topics() -> &'static [Topic] {
    static T: OnceLock<Vec<Topic>> = OnceLock::new();
    T.get_or_init(|| {
        toml::from_str::<File>(crate::data::TOPICS)
            .expect("data/topics.toml")
            .topic
    })
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
    fn worked_examples_link_to_scripts() {
        assert!(worked_examples("battery_load").iter().any(|e| e.id == 34));
        assert!(worked_examples("event_score").iter().any(|e| e.id == 378));
        assert_eq!(topic_of("nodal").unwrap().id, "circuits");
    }
}
