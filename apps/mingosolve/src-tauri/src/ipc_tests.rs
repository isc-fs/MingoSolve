//! The command contract, exercised through Tauri's real IPC layer (mock runtime, real JSON (de)serialisation) with
//! the argument names and shapes the frontend sends. If a field is renamed on either side, these fail.

use serde_json::{json, Value};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::WebviewWindow;

fn webview() -> WebviewWindow<tauri::test::MockRuntime> {
    // the real config and capabilities, so permissions behave as in the shipped app
    let app = super::with_commands(mock_builder())
        .build(tauri::generate_context!(test = true))
        .unwrap();
    tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap()
}

fn invoke(
    w: &WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    args: Value,
) -> Result<Value, Value> {
    get_ipc_response(
        w,
        InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            // the app's own origin: http://tauri.localhost on Windows, tauri://localhost elsewhere
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: tauri::ipc::InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize::<Value>().unwrap())
}

fn num(shown: &Value) -> f64 {
    shown
        .as_str()
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs().max(1e-12)
}

#[test]
fn solve_returns_paired_roots_in_the_requested_display_unit() {
    let w = webview();
    let r = invoke(
        &w,
        "solve_formula",
        json!({
            "key": "battery_load",
            "given": [["N_s", "103"], ["V_cell", "3.8 V"], ["R_pack", "0.08 ohm"], ["P", "30 kW"], ["N_p", ""]],
            "display": {"I": "mA", "V_t": ""}
        }),
    )
    .unwrap();
    let found = r["found"].as_array().unwrap();
    let get = |n: &str| found.iter().find(|f| f["name"] == n).unwrap().clone();
    let (i, vt) = (get("I"), get("V_t"));
    // official answer of FS-Quiz Q34: 77.9 A; shown in mA as asked, values stay in the variable's unit (A)
    assert!(close(num(&i["shown"][0]), 77_887.9, 1e-5), "{i}");
    assert!(i["shown"][0].as_str().unwrap().ends_with("mA"));
    assert!(close(i["values"][0].as_f64().unwrap(), 77.8879, 1e-5));
    // roots are pairs: V_t[k] belongs to I[k] (V_t = V_oc - I R)
    for k in 0..2 {
        let (ik, vk) = (
            i["values"][k].as_f64().unwrap(),
            vt["values"][k].as_f64().unwrap(),
        );
        assert!(close(vk, 391.4 - ik * 0.08, 1e-9));
    }
    assert_eq!(r["conflicts"], json!([]));
}

#[test]
fn over_specified_inputs_report_the_broken_equation() {
    let w = webview();
    let r = invoke(&w, "solve_formula", json!({"key": "speed", "given": [["s", "75 m"], ["t", "3 s"], ["v_avg", "30 m/s"]], "display": {}}))
        .unwrap();
    let conflicts = r["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1, "{r}");
    assert!(conflicts[0].as_str().unwrap().contains("v_avg = s/t"));
}

#[test]
fn engine_errors_reach_the_frontend_as_readable_text() {
    let w = webview();
    let err = invoke(
        &w,
        "solve_formula",
        json!({"key": "speed", "given": [["s", "75 furlongs"]], "display": {}}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("unknown unit"), "{err}");
    let err = invoke(
        &w,
        "solve_formula",
        json!({"key": "speed", "given": [["s", "3 kg"]], "display": {}}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("expected m"), "{err}");
    let err = invoke(&w, "run_tool", json!({"name": "corrected_time", "args": [["t_raw", "60"], ["oc", "1"], ["event", "accel"]]}))
        .unwrap_err();
    assert!(
        err.as_str().unwrap().contains("DQ"),
        "an OC in acceleration is a DQ, not a time penalty: {err}"
    );
}

#[test]
fn finder_prefills_from_the_problem_and_hides_past_questions() {
    let w = webview();
    let r = invoke(
        &w,
        "find_question",
        json!({"text": "The accumulator has 103 cells in series at 3.8 V and an internal resistance of 0.08 Ω. \
                        What current is drawn at 30 kW?"}),
    )
    .unwrap();
    let hits = r["hits"].as_array().unwrap();
    assert_eq!(hits[0]["id"], "battery_load", "{r}");
    assert!(hits.iter().all(|h| h["kind"] != "example"));
    let fills: Vec<(String, String)> = serde_json::from_value(hits[0]["prefill"].clone()).unwrap();
    for want in [("N_s", "103"), ("V_cell", "3.8 V"), ("P", "30 kW")] {
        assert!(fills.contains(&(want.0.into(), want.1.into())), "{fills:?}");
    }
    // chips only carry values with units, rendered with pretty units
    assert_eq!(r["quantities"], json!(["3.8 V", "0.08 Ω", "30 kW"]));
    // the pre-fill must actually solve: feed it back and get the official Q34 answer once R_pack is entered
    let mut given = fills.clone();
    given.push(("R_pack".into(), "0.08 ohm".into()));
    let s = invoke(
        &w,
        "solve_formula",
        json!({"key": "battery_load", "given": given, "display": {}}),
    )
    .unwrap();
    let i = s["found"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "I")
        .unwrap()
        .clone();
    assert!(close(i["values"][0].as_f64().unwrap(), 77.9, 0.005));
}

#[test]
fn tools_use_defaults_for_blank_parameters() {
    let w = webview();
    // blank rules -> the current rule set (2027): Tmax = 1.4 Tmin, halfway -> (100-10) * 0.25 + 10
    let r = invoke(&w, "run_tool", json!({"name": "event_score", "args": [["event", "autocross"], ["t_team", "72"], ["t_min", "60"], ["rules", ""]]}))
        .unwrap();
    assert_eq!(r, json!("32.5"));
    // FS-Quiz Q378 (legacy rules): 41.12
    let r = invoke(&w, "run_tool", json!({"name": "event_score", "args": [["event", "skidpad"], ["t_team", "5.6"], ["t_min", "5.1"], ["rules", "legacy"]]}))
        .unwrap();
    assert!(close(r.as_str().unwrap().parse().unwrap(), 41.12, 0.005));
}

#[test]
fn every_script_is_in_exactly_one_topic_with_matching_kind_and_title() {
    let w = webview();
    let topics = invoke(&w, "list_topics", json!({})).unwrap();
    let formulas = invoke(&w, "list_formulas", json!({})).unwrap();
    let tools = invoke(&w, "list_tools", json!({})).unwrap();
    let mut seen = std::collections::HashMap::new();
    for t in topics.as_array().unwrap() {
        for s in t["scripts"].as_array().unwrap() {
            assert!(
                seen.insert(s["id"].as_str().unwrap().to_string(), s.clone())
                    .is_none(),
                "{s} twice"
            );
        }
    }
    for f in formulas.as_array().unwrap() {
        let s = &seen[f["key"].as_str().unwrap()];
        assert_eq!(s["kind"], "formula");
        assert_eq!(s["title"], f["title"], "the catalogue joins these by id");
        assert!(!f["vars"].as_array().unwrap().is_empty());
    }
    for t in tools.as_array().unwrap() {
        assert_eq!(seen[t["name"].as_str().unwrap()]["kind"], "tool");
    }
    assert_eq!(
        seen.len(),
        formulas.as_array().unwrap().len() + tools.as_array().unwrap().len()
    );
}

/// Parse an example command the way the frontend's openCommand does: key=value pairs, @var=unit display units,
/// bare words as positional tool arguments, double quotes grouping.
type Pairs = Vec<(String, String)>;

fn parse(cmd: &str) -> (String, Pairs, Pairs, Vec<String>) {
    let mut parts = Vec::new();
    let (mut cur, mut quoted, mut any) = (String::new(), false, false);
    for c in cmd.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            c if c.is_whitespace() && !quoted => {
                if any || !cur.is_empty() {
                    parts.push(std::mem::take(&mut cur));
                    any = false;
                }
            }
            c => cur.push(c),
        }
    }
    if any || !cur.is_empty() {
        parts.push(cur);
    }
    let head = parts.remove(0);
    let (mut values, mut display, mut positional) = (vec![], vec![], vec![]);
    for p in parts {
        match p.split_once('=') {
            Some((k, v)) if k.starts_with('@') => display.push((k[1..].to_string(), v.to_string())),
            Some((k, v)) => values.push((k.to_string(), v.to_string())),
            None => positional.push(p),
        }
    }
    (head, values, display, positional)
}

#[test]
fn every_worked_example_button_reproduces_its_official_answer() {
    let w = webview();
    let topics = invoke(&w, "list_topics", json!({})).unwrap();
    let tools = invoke(&w, "list_tools", json!({})).unwrap();
    let mut checked = 0;
    let mut failures = Vec::new();
    for t in topics.as_array().unwrap() {
        for s in t["scripts"].as_array().unwrap() {
            let id = s["id"].as_str().unwrap();
            let examples = invoke(&w, "script_examples", json!({"script": id})).unwrap();
            for ex in examples.as_array().unwrap() {
                let (head, values, display, positional) = parse(ex["cmd"].as_str().unwrap());
                assert_eq!(
                    head, id,
                    "script_examples returned a command for another script"
                );
                let answer = ex["answer"].as_f64().unwrap();
                let numbers: Vec<f64> = if s["kind"] == "formula" {
                    let r = invoke(&w, "solve_formula", json!({"key": id, "given": values, "display": display.into_iter().collect::<std::collections::HashMap<_, _>>()}))
                        .unwrap();
                    r["found"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .flat_map(|f| f["shown"].as_array().unwrap().clone())
                        .map(|v| num(&v))
                        .collect()
                } else {
                    // the sheet maps positional arguments onto the tool's parameters in order
                    let params = tools
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|x| x["name"] == id)
                        .unwrap()["params"]
                        .clone();
                    let mut args: Vec<(String, String)> = positional
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (params[i]["name"].as_str().unwrap().to_string(), v.clone()))
                        .collect();
                    args.extend(values);
                    let out = invoke(&w, "run_tool", json!({"name": id, "args": args})).unwrap();
                    out.as_str()
                        .unwrap()
                        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
                        .filter_map(|x| x.parse().ok())
                        .collect()
                };
                checked += 1;
                if !numbers
                    .iter()
                    .any(|n| (n - answer).abs() <= 0.005 * answer.abs() + 1e-9)
                {
                    failures.push(format!(
                        "Q{} {id}: official {answer}, got {numbers:?}",
                        ex["id"]
                    ));
                }
            }
        }
    }
    // every past question whose command opens a script (not chain/calc) must be reachable as a button
    let expected = fsq::cli::examples()
        .iter()
        .filter(|e| !matches!(e.cmd.split_whitespace().next(), Some("chain" | "calc")))
        .count();
    assert_eq!(
        checked, expected,
        "worked examples not reachable through topics"
    );
    assert!(
        failures.is_empty(),
        "worked examples that don't solve in the app:\n{}",
        failures.join("\n")
    );
}

#[test]
fn chain_and_calc_past_questions_reproduce_their_official_answers_as_the_app_runs_them() {
    let w = webview();
    let chain = invoke(&w, "chain_examples", json!({})).unwrap();
    let calc = invoke(&w, "calc_examples", json!({})).unwrap();
    let rows = |head: &str| {
        fsq::cli::examples()
            .into_iter()
            .filter(|e| e.cmd.split_whitespace().next() == Some(head))
            .map(|e| (e.id, e.cmd, e.answer))
            .collect::<Vec<_>>()
    };
    for (name, got, want) in [
        ("chain", &chain, rows("chain")),
        ("calc", &calc, rows("calc")),
    ] {
        let got: Vec<(u64, String, f64)> = got
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                (
                    e["id"].as_u64().unwrap(),
                    e["cmd"].as_str().unwrap().to_string(),
                    e["answer"].as_f64().unwrap(),
                )
            })
            .collect();
        assert!(!want.is_empty());
        assert_eq!(
            got,
            want.into_iter()
                .map(|(i, c, a)| (u64::from(i), c, a))
                .collect::<Vec<_>>(),
            "{name} examples"
        );
    }
    let mut failures = Vec::new();
    for ex in chain.as_array().unwrap() {
        // what the Chain view does: first bare word is the target, k=v are known values, @target=unit is "show in"
        let (head, values, display, positional) = parse(ex["cmd"].as_str().unwrap());
        assert_eq!(head, "chain");
        let target = &positional[0];
        let display: std::collections::HashMap<String, String> =
            display.into_iter().filter(|(k, _)| k == target).collect();
        let only: Vec<String> = values
            .iter()
            .filter(|(k, _)| k == "only")
            .flat_map(|(_, v)| v.split(',').map(String::from))
            .collect();
        let given: Vec<_> = values.into_iter().filter(|(k, _)| k != "only").collect();
        let r = invoke(
            &w,
            "chain_formulas",
            json!({"target": target, "given": given, "only": only, "display": display}),
        )
        .unwrap();
        let answer = ex["answer"].as_f64().unwrap();
        if r["reached"] != true || !close(num(&r["target"]), answer, 0.005) {
            failures.push(format!(
                "Q{} chain: official {answer}, got {}",
                ex["id"], r["target"]
            ));
        }
    }
    for ex in calc.as_array().unwrap() {
        let expr = ex["cmd"].as_str().unwrap().strip_prefix("calc ").unwrap();
        let out = invoke(&w, "calc", json!({"expr": expr})).unwrap();
        let answer = ex["answer"].as_f64().unwrap();
        let first = out.as_str().unwrap().split(' ').next().unwrap();
        match first.parse::<f64>() {
            Ok(n) if close(n, answer, 0.005) => {}
            _ => failures.push(format!("Q{} calc: official {answer}, got {out}", ex["id"])),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn chain_restricted_to_tags_and_unreachable_targets() {
    let w = webview();
    let r = invoke(&w, "chain_formulas", json!({"target": "v", "given": [["h_cg", "0.205 m"], ["R_c", "14.5 m"], ["t_tr", "1.24 m"]], "only": [], "display": {"v": "km/h"}}))
        .unwrap();
    assert_eq!(r["reached"], true);
    assert!(close(num(&r["target"]), 74.7, 0.005), "FS-Quiz Q515: {r}");
    // restricted to aero formulas the rollover path is not available
    let r = invoke(&w, "chain_formulas", json!({"target": "v", "given": [["h_cg", "0.205 m"], ["R_c", "14.5 m"], ["t_tr", "1.24 m"]], "only": ["aero"], "display": {}}))
        .unwrap();
    assert_eq!(r["reached"], false);
    assert!(r["known"].as_array().unwrap().iter().any(|k| k == "h_cg"));
}

#[test]
fn answer_helpers_follow_the_frontend_argument_names() {
    let w = webview();
    let f = invoke(
        &w,
        "format_answer",
        json!({"value": 77.88789, "precision": {"decimals": 1}, "decimalComma": true}),
    )
    .unwrap();
    assert_eq!(f, json!("77,9"));
    let f = invoke(
        &w,
        "format_answer",
        json!({"value": 0.0020416667, "precision": {"sig": 5}, "decimalComma": false}),
    )
    .unwrap();
    assert_eq!(f, json!("0.0020417"));
    let m = invoke(
        &w,
        "match_options",
        json!({"answer": "77.887 A", "options": "a) 73.4 A\nb) 77.9 A\nc) 80.4 A"}),
    )
    .unwrap();
    assert_eq!(m["options"][1]["best"], true);
    assert_eq!(m["warning"], Value::Null);
    let m = invoke(
        &w,
        "match_options",
        json!({"answer": "131.79 N", "options": "121.79 N\n0.1318 kN\n1.317 kN\n1.708 kN"}),
    )
    .unwrap();
    assert_eq!(m["options"][1]["best"], true);
    assert_eq!(m["warning"], Value::Null);
}

#[test]
fn a_panic_in_a_command_is_an_error_and_the_app_keeps_answering() {
    let w = webview();
    let err = invoke(&w, "panic_probe", json!({"message": "boom"})).unwrap_err();
    assert_eq!(err, json!("internal error: boom"));
    assert_eq!(
        invoke(&w, "calc", json!({"expr": "2+2"})).unwrap(),
        json!("4")
    );
}

#[test]
fn hostile_input_through_the_async_commands_is_an_error_not_a_crash() {
    let w = webview();
    let deep = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
    let out = invoke(&w, "calc", json!({"expr": deep})).unwrap();
    assert_eq!(out, json!("error: expression nested too deeply"));
    let out = invoke(&w, "calc", json!({"expr": "10**400/10**400"})).unwrap();
    assert!(out.as_str().unwrap().contains("no finite answer"), "{out}");
    let err = invoke(
        &w,
        "solve_formula",
        json!({"key": "speed", "given": [["s", format!("{deep} m")]], "display": {}}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("nested too deeply"), "{err}");
    let err = invoke(
        &w,
        "format_answer",
        json!({"value": 1.5, "precision": {"decimals": 4_000_000_000u64}, "decimalComma": false}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("precision"), "{err}");
    let err = invoke(
        &w,
        "run_tool",
        json!({"name": "twos", "args": [["x", "5"], ["bits", "4e9"]]}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("bits"), "{err}");
    let found = invoke(&w, "find_question", json!({"text": deep})).unwrap();
    assert!(found["hits"].is_array());
    // still alive, and still right
    let r = invoke(
        &w,
        "solve_formula",
        json!({"key": "speed", "given": [["s", "75 m"], ["t", "3 s"]], "display": {}}),
    )
    .unwrap();
    assert!(r["found"].as_array().is_some_and(|f| !f.is_empty()), "{r}");
}

/// Written for this test (not bank text) and fingerprinted at test time into a small bank with real ids: 665 has a
/// worked example in data/examples.toml, 125 is a known-wrong key in data/known_keys.toml.
const SYNTHETIC: &str =
    "A first-order RC filter with a corner frequency of 300 Hz is driven by a 200 Hz sine \
                         wave. By how many degrees does the output lag the input?";
const SYNTHETIC_KEYED: &str =
    "How many degrees of freedom does a quadratic tetrahedral element model have in a \
                               mesh of one cell?";

fn synthetic_bank() -> Vec<fsq::past::PastQuestion> {
    use fsq::past::PastQuestion;
    vec![
        PastQuestion::from_text(
            665,
            SYNTHETIC,
            &["FSG 2023 EV", "FSA 2024 EV"],
            Some("-33.7 °"),
        ),
        PastQuestion::from_text(125, SYNTHETIC_KEYED, &["FSS 2022 EV"], None),
    ]
}

#[test]
fn found_carries_the_past_question_as_the_frontend_reads_it() {
    let changed = SYNTHETIC.replace("300 Hz", "250 Hz");
    let found = crate::finder::find_question_in(&changed, &synthetic_bank());
    let r = serde_json::to_value(&found).unwrap();
    let p = &r["past"];
    assert_eq!(p["id"], 665, "{r}");
    assert_eq!(p["quizzes"], json!(["FSG 2023 EV", "FSA 2024 EV"]));
    assert_eq!(p["answer"], "-33.7 °");
    assert_eq!(
        p["same_numbers"], false,
        "250 Hz is not the 300 Hz question"
    );
    assert_eq!(p["probable"], false);
    assert_eq!(p["example"]["cmd"], "rc_lowpass f=200Hz f_c=300Hz @phi=deg");
    assert_eq!(p["example"]["answer"], -33.69);
    assert!(p["known_key"].is_null());

    let keyed = crate::finder::find_question_in(SYNTHETIC_KEYED, &synthetic_bank());
    let k = serde_json::to_value(&keyed).unwrap();
    assert!(
        k["past"]["known_key"]
            .as_str()
            .unwrap()
            .contains("tetrahedron"),
        "{k}"
    );
    assert!(k["past"]["example"].is_null());
}

#[test]
fn find_question_over_ipc_has_a_null_past_for_text_the_bank_does_not_know() {
    let w = webview();
    let r = invoke(&w, "find_question", json!({"text": SYNTHETIC})).unwrap();
    assert!(r.as_object().unwrap().contains_key("past"), "{r}");
    assert!(
        r["past"].is_null(),
        "invented text must not match the shipped bank: {r}"
    );
}

/// The answer instruction reaches the frontend in the shape types.ts declares, and its dimension key equals the key
/// the formula catalogue gives the variable it would apply to (rc_lowpass.phi is in rad, the question asks degrees).
#[test]
fn find_question_over_ipc_carries_the_format_hint_and_variables_carry_matching_dims() {
    let w = webview();
    let text = format!("{SYNTHETIC} Give your answer in degrees, rounded to one decimal place.");
    let r = invoke(&w, "find_question", json!({ "text": text })).unwrap();
    assert_eq!(
        r["format"],
        json!({
            "rounding": {"decimals": 1},
            "unit": "deg",
            "unit_label": "degrees",
            "dims": "dimensionless·angle",
            "quantity": null,
        }),
        "{r}"
    );
    let none = invoke(&w, "find_question", json!({ "text": SYNTHETIC })).unwrap();
    assert!(none["format"].is_null(), "{none}");

    let formulas = invoke(&w, "list_formulas", json!({})).unwrap();
    let rc = formulas
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["key"] == "rc_lowpass")
        .unwrap();
    let dims = |name: &str| {
        rc["vars"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()["dims"]
            .clone()
    };
    assert_eq!(dims("phi"), r["format"]["dims"]);
    assert_ne!(dims("f"), r["format"]["dims"], "a frequency is not an angle");
}

/// With the real bank (not public, so skipped without it) the shipped fingerprints answer through the real IPC.
#[test]
fn find_question_over_ipc_recognises_a_real_bank_question() {
    let path = std::env::var("FSQ_BANK").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../IFS-Tests/data/fsquiz/bank.json"
        )
        .to_string()
    });
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("ipc past test: skipped, no bank at {path}");
        return;
    };
    let bank: Value = serde_json::from_str(&raw).unwrap();
    let text_of = |id: u64| {
        bank["questions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|q| q["question_id"] == id)
            .unwrap()["text"]
            .clone()
    };
    let w = webview();
    let r = invoke(&w, "find_question", json!({"text": text_of(665)})).unwrap();
    assert_eq!(r["past"]["id"], 665, "{r}");
    assert_eq!(
        r["past"]["example"]["cmd"],
        "rc_lowpass f=200Hz f_c=300Hz @phi=deg"
    );
    let r = invoke(&w, "find_question", json!({"text": text_of(125)})).unwrap();
    assert_eq!(r["past"]["id"], 125, "{r}");
    assert!(r["past"]["known_key"].is_string(), "{r}");
}
