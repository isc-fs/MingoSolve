//! Plain-English metadata of the procedural tools (data/tool_params.toml): a sentence per tool, and per parameter a
//! label, an optional unit and help, and whether it is a choice or an on/off switch. The app builds its tool forms
//! from it; the engine in `tools` doesn't use it.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ParamMeta {
    pub label: String,
    pub unit: Option<String>,
    pub help: Option<String>,
    pub choices: Option<Vec<String>>,
    pub choice_labels: Option<Vec<String>>,
    #[serde(default)]
    pub switch: bool,
}

#[derive(Debug, Deserialize)]
pub struct ToolMeta {
    pub summary: String,
    pub params: HashMap<String, ParamMeta>,
}

fn file() -> &'static HashMap<String, ToolMeta> {
    static F: OnceLock<HashMap<String, ToolMeta>> = OnceLock::new();
    F.get_or_init(|| toml::from_str(crate::data::TOOL_PARAMS).expect("data/tool_params.toml"))
}

pub fn tool_meta(tool: &str) -> Option<&'static ToolMeta> {
    file().get(tool)
}

pub fn param_meta(tool: &str, param: &str) -> Option<&'static ParamMeta> {
    tool_meta(tool)?.params.get(param)
}

/// A bare number typed for a parameter that has a unit is read in that unit ("50" -> "50 degC"); anything with
/// its own unit or an expression is left alone.
pub fn with_unit(tool: &str, param: &str, value: &str) -> String {
    let v = value.trim();
    let Some(unit) = param_meta(tool, param).and_then(|m| m.unit.as_deref()) else {
        return v.to_string();
    };
    let n = if v.matches(',').count() == 1 {
        v.replace(',', ".")
    } else {
        v.to_string()
    };
    if n.parse::<f64>().is_ok() {
        format!("{n} {unit}")
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::examples;
    use crate::tools::{tool, tools, Kind, Tool};
    use crate::units;

    #[test]
    fn every_tool_and_parameter_is_described_and_nothing_else() {
        for t in tools() {
            let m = tool_meta(t.name).unwrap_or_else(|| panic!("tool {} has no entry", t.name));
            assert!(
                m.summary.ends_with('.'),
                "{}: summary is a sentence",
                t.name
            );
            for p in &t.params {
                let pm = m.params.get(p.name);
                let pm = pm.unwrap_or_else(|| panic!("{}.{} has no label", t.name, p.name));
                assert!(!pm.label.trim().is_empty(), "{}.{}", t.name, p.name);
            }
            for k in m.params.keys() {
                assert!(
                    t.params.iter().any(|p| p.name == k),
                    "{} describes unknown parameter {k}",
                    t.name
                );
            }
        }
        for k in file().keys() {
            assert!(
                tool(k).is_some(),
                "tool_params.toml describes unknown tool {k}"
            );
        }
    }

    #[test]
    fn kinds_are_consistent() {
        for t in tools() {
            for p in &t.params {
                let m = param_meta(t.name, p.name).unwrap();
                let id = format!("{}.{}", t.name, p.name);
                assert!(
                    !(m.switch && m.choices.is_some()),
                    "{id}: switch and choices"
                );
                if let Some(c) = &m.choices {
                    assert!(c.len() > 1, "{id}: one choice is no choice");
                    if let Some(l) = &m.choice_labels {
                        assert_eq!(l.len(), c.len(), "{id}: one label per choice");
                    }
                    let unique: std::collections::HashSet<_> = c.iter().collect();
                    assert_eq!(unique.len(), c.len(), "{id}: duplicate choice");
                    if let Some(d) = p.default.filter(|d| !d.is_empty()) {
                        assert!(c.iter().any(|x| x == d), "{id}: default {d:?} not offered");
                    }
                }
                assert!(m.choice_labels.is_none() || m.choices.is_some(), "{id}");
                if m.switch {
                    assert!(
                        matches!(p.default, Some("0" | "1")),
                        "{id}: a switch needs a 0/1 default"
                    );
                }
                if let Some(u) = &m.unit {
                    assert_eq!(p.kind, Kind::Num, "{id}: a unit on a text parameter");
                    units::unit_of(u).unwrap_or_else(|e| panic!("{id}: unit {u}: {}", e.0));
                }
            }
        }
    }

    /// Smallest arguments that make each tool run, for the tools with choices or switches.
    fn base(tool: &str) -> &'static [&'static str] {
        match tool {
            "corrected_time" => &["t_raw=70"],
            "event_score" => &["event=skidpad", "t_team=5.6", "t_min=5.1"],
            "cones_from_score" => &["event=skidpad", "score=40", "t_raw=6", "t_min=5.1"],
            "efficiency_score" => &[
                "t=1000",
                "e=10",
                "ef_min=1e6",
                "t_min=900",
                "e_min=9",
                "ef_max=2e6",
            ],
            "static_nonfinalist" => &["p_team=55", "p_best=61"],
            "max_points" => &["events=skidpad,accel"],
            "pcb_spacing" => &["v=100", "mode=coated"],
            "ts_rules" => &["v_max=400"],
            "e_series" => &["value=2411"],
            "uart_time" => &["n_bytes=10", "baud=9600"],
            "can_frame_bits" | "document_penalty" | "rule_value" | "skidpad_layout" => &[],
            "can_transfer" => &["bits=64", "bitrate=500000"],
            "gauge" => &["p=200000"],
            "wheel_slip" => &["vx=10", "a=0.8", "b=0.8", "t=1.2", "alpha=4.6deg"],
            "track_range" => &["track=1.2"],
            "round_to" => &["x=7.3"],
            other => panic!("add base arguments for {other} to this test"),
        }
    }

    fn run(t: &Tool, args: &[&str], over: (&str, &str)) -> Result<String, String> {
        let mut a: Vec<String> = args
            .iter()
            .filter(|x| !x.starts_with(&format!("{}=", over.0)))
            .map(|x| x.to_string())
            .collect();
        a.push(format!("{}={}", over.0, over.1));
        let a: Vec<&str> = a.iter().map(String::as_str).collect();
        t.call(&a)
    }

    /// Ok under the default rules or, for values only some rule years know (legacy keys, driverless events),
    /// under one of the years.
    fn accepted(t: &Tool, param: &str, value: &str) -> bool {
        let years = ["", "legacy", "2026", "2027"];
        let has_rules = t.params.iter().any(|p| p.name == "rules");
        years.iter().any(|y| {
            let mut a: Vec<String> = base(t.name).iter().map(|s| s.to_string()).collect();
            if has_rules && param != "rules" && !y.is_empty() {
                a.push(format!("rules={y}"));
            }
            let a: Vec<&str> = a.iter().map(String::as_str).collect();
            run(t, &a, (param, value)).is_ok()
        })
    }

    #[test]
    fn every_choice_is_accepted_by_its_tool() {
        for t in tools() {
            for p in &t.params {
                let Some(choices) = &param_meta(t.name, p.name).unwrap().choices else {
                    continue;
                };
                for c in choices {
                    assert!(
                        accepted(t, p.name, c),
                        "{}.{}: the tool rejects the choice {c:?}",
                        t.name,
                        p.name
                    );
                }
            }
        }
    }

    #[test]
    fn strict_choices_reject_anything_else_and_cover_what_the_rules_define() {
        // tools that validate the value: a made-up one must fail, so the list is what the tool really takes
        let strict = [
            ("corrected_time", "event"),
            ("corrected_time", "post"),
            ("event_score", "event"),
            ("cones_from_score", "event"),
            ("static_nonfinalist", "event"),
            ("pcb_spacing", "mode"),
            ("wheel_slip", "known"),
            ("rule_value", "name"),
        ];
        let rules_param = |t: &Tool| t.params.iter().any(|p| p.name == "rules");
        for t in tools().iter().filter(|t| rules_param(t)) {
            assert!(
                !accepted(t, "rules", "1999"),
                "{}: rules year not validated",
                t.name
            );
        }
        for (name, param) in strict {
            let t = tool(name).unwrap();
            assert!(
                !accepted(t, param, "zzz"),
                "{name}.{param} accepts anything"
            );
        }
        // the lists against the rule files, which the tools read independently of this data
        let choices = |t: &str, p: &str| -> Vec<String> {
            let mut c = param_meta(t, p).unwrap().choices.clone().unwrap();
            c.sort();
            c
        };
        let sorted = |mut v: Vec<String>| {
            v.sort();
            v
        };
        let sets = crate::tools::rule_sets();
        let keys = |year: &str, table: &str| -> Vec<String> {
            sets[year][table]
                .as_table()
                .unwrap()
                .keys()
                .cloned()
                .collect()
        };
        let pen_keys = |table: &str| -> Vec<String> {
            crate::tools::penalties()[table]
                .as_table()
                .unwrap()
                .keys()
                .cloned()
                .collect()
        };
        let mut years: Vec<String> = sets.keys().cloned().collect();
        years.sort();
        for t in tools().iter().filter(|t| rules_param(t)) {
            assert_eq!(choices(t.name, "rules"), years, "{}", t.name);
        }
        let mut scored = keys("2027", "tmax_pmin");
        scored.extend(keys("legacy", "legacy_scoring"));
        scored.sort();
        scored.dedup();
        assert_eq!(choices("event_score", "event"), scored);
        let mut doo: Vec<String> = pen_keys("doo_oc");
        doo.sort();
        assert_eq!(choices("corrected_time", "event"), doo);
        let mut cones = doo.clone();
        cones.retain(|e| scored.contains(e));
        assert_eq!(choices("cones_from_score", "event"), cones);
        let statics: Vec<String> = keys("2027", "static_nonfinalist")
            .into_iter()
            .filter(|k| !k.contains("minus"))
            .collect();
        assert_eq!(choices("static_nonfinalist", "event"), sorted(statics));
        let mut modes: Vec<String> = ["legacy", "2027"]
            .iter()
            .flat_map(|y| {
                sets[*y]["pcb_spacing"]["modes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m.as_str().unwrap().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        modes.sort();
        modes.dedup();
        assert_eq!(choices("pcb_spacing", "mode"), modes);
        let mut names = keys("2027", "constants");
        names.push(String::new());
        assert_eq!(choices("rule_value", "name"), sorted(names));
        let mut post = pen_keys("post_inspection");
        post.push(String::new());
        assert_eq!(choices("corrected_time", "post"), sorted(post));
    }

    #[test]
    fn every_switch_takes_0_and_1_and_they_mean_something_different() {
        let mut seen = 0;
        for t in tools() {
            for p in &t.params {
                if !param_meta(t.name, p.name).unwrap().switch {
                    continue;
                }
                seen += 1;
                let mut args: Vec<&str> = base(t.name).to_vec();
                // finish only matters for endurance under the older rules (the Q171 key)
                if t.name == "event_score" {
                    args = vec![
                        "event=endurance",
                        "t_team=1672",
                        "t_min=1345",
                        "rules=legacy",
                    ];
                }
                let off = run(t, &args, (p.name, "0"))
                    .unwrap_or_else(|e| panic!("{}.{}=0: {e}", t.name, p.name));
                let on = run(t, &args, (p.name, "1"))
                    .unwrap_or_else(|e| panic!("{}.{}=1: {e}", t.name, p.name));
                assert_ne!(off, on, "{}.{} changes nothing", t.name, p.name);
            }
        }
        assert_eq!(
            seen, 6,
            "finish, parity, and extended and stuffing in two CAN tools"
        );
    }

    /// Words of a worked-example command, double quotes grouping.
    fn words(cmd: &str) -> Vec<String> {
        let (mut out, mut cur, mut quoted) = (Vec::new(), String::new(), false);
        for c in cmd.chars() {
            match c {
                '"' => quoted = !quoted,
                c if c.is_whitespace() && !quoted => {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        out
    }

    /// The (parameter, value) pairs a worked example fills in, as the sheet does: `name=value`, else positional.
    fn example_pairs(t: &Tool, cmd: &str) -> Vec<(&'static str, String)> {
        let mut pos = 0;
        let mut out = Vec::new();
        for w in words(cmd).iter().skip(1) {
            match w.split_once('=') {
                Some((k, v)) if t.params.iter().any(|p| p.name == k) => {
                    out.push((
                        t.params.iter().find(|p| p.name == k).unwrap().name,
                        v.to_string(),
                    ));
                }
                _ => {
                    while t
                        .params
                        .get(pos)
                        .is_some_and(|p| out.iter().any(|(n, _)| *n == p.name))
                    {
                        pos += 1;
                    }
                    out.push((t.params[pos].name, w.clone()));
                    pos += 1;
                }
            }
        }
        out
    }

    #[test]
    fn worked_examples_fit_the_form_inputs_and_keep_their_answer_with_units_applied() {
        let mut checked = 0;
        for e in examples() {
            let head = e.cmd.split_whitespace().next().unwrap();
            let Some(t) = tool(head) else { continue };
            let pairs = example_pairs(t, &e.cmd);
            for (name, v) in &pairs {
                let m = param_meta(t.name, name).unwrap();
                if let Some(c) = &m.choices {
                    assert!(c.contains(v), "Q{}: {name}={v} is not a choice", e.id);
                }
                if m.switch {
                    assert!(v == "0" || v == "1", "Q{}: {name}={v} is not 0/1", e.id);
                }
            }
            let bare: Vec<String> = pairs.iter().map(|(n, v)| format!("{n}={v}")).collect();
            let typed: Vec<String> = pairs
                .iter()
                .map(|(n, v)| format!("{n}={}", with_unit(t.name, n, v)))
                .collect();
            let call = |a: &[String]| t.call(&a.iter().map(String::as_str).collect::<Vec<_>>());
            assert_eq!(
                call(&bare),
                call(&typed),
                "Q{}: a unit changes the answer of {}",
                e.id,
                e.cmd
            );
            checked += 1;
        }
        assert!(checked > 20, "only {checked} tool examples");
    }

    #[test]
    fn a_bare_number_takes_the_unit_and_a_typed_one_keeps_its_own() {
        // 50 read as degrees Celsius is 323.15 K; read as bare SI it would be 50 K
        let k = |v: &str| {
            units::parse_quantity(&with_unit("cable_fuse", "t_amb", v))
                .unwrap()
                .value
        };
        assert!((k("50") - 323.15).abs() < 1e-9);
        assert!((k("50degC") - 323.15).abs() < 1e-9);
        assert!((k("0,5") - 273.65).abs() < 1e-9);
        assert_eq!(with_unit("cable_fuse", "t_ins", "0.36mm"), "0.36mm");
        assert_eq!(with_unit("cable_fuse", "a", "2*1.5"), "2*1.5");
        assert_eq!(with_unit("e_series", "value", "2411"), "2411");
    }
}
