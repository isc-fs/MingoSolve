//! Test-only HTTP bridge: serves the app's real Tauri command functions on 127.0.0.1 so the browser end-to-end
//! tests (apps/mingosolve/e2e) drive the real engine without a native window. POST /invoke/<command> with the same
//! JSON arguments the frontend passes to invoke(); 200 + JSON result, or 400 + JSON error string.
//! Not part of the shipped app (cargo example, built only by the e2e runner).

use std::collections::HashMap;

use mingosolve_lib::{finder, solve, tools, topics};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tiny_http::{Header, Response, Server};

fn arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    serde_json::from_value(args.get(name).cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("argument {name}: {e}"))
}

fn dispatch(cmd: &str, a: &Value) -> Result<Value, String> {
    let ok = |v: Value| Ok(v);
    match cmd {
        "engine_version" => ok(json!(env!("CARGO_PKG_VERSION"))),
        "list_formulas" => ok(json!(solve::list_formulas())),
        "solve_formula" => solve::solve_formula(
            arg(a, "key")?,
            arg(a, "given")?,
            arg::<HashMap<String, String>>(a, "display")?,
        )
        .map(|r| json!(r)),
        "chain_formulas" => solve::chain_formulas(
            arg(a, "target")?,
            arg(a, "given")?,
            arg(a, "only")?,
            arg(a, "display")?,
        )
        .map(|r| json!(r)),
        "calc" => ok(json!(solve::calc(arg(a, "expr")?))),
        "list_tools" => ok(json!(tools::list_tools())),
        "run_tool" => tools::run_tool(arg(a, "name")?, arg(a, "args")?).map(|r| json!(r)),
        "find_question" => ok(json!(finder::find_question(arg(a, "text")?))),
        "match_options" => ok(json!(finder::match_options(
            arg(a, "value")?,
            arg(a, "options")?
        ))),
        "format_answer" => ok(json!(finder::format_answer(
            arg(a, "value")?,
            arg(a, "precision")?,
            arg(a, "decimalComma")?
        ))),
        "list_topics" => ok(json!(topics::list_topics())),
        "script_examples" => ok(json!(topics::script_examples(arg(a, "script")?))),
        other => Err(format!("unknown command {other}")),
    }
}

fn main() {
    let port = std::env::var("BRIDGE_PORT").unwrap_or_else(|_| "8799".into());
    let server = Server::http(format!("127.0.0.1:{port}")).expect("bind bridge port");
    eprintln!("ipc bridge on http://127.0.0.1:{port}");
    let cors = [
        Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
        Header::from_bytes("Access-Control-Allow-Headers", "content-type").unwrap(),
        Header::from_bytes("Content-Type", "application/json").unwrap(),
    ];
    for mut req in server.incoming_requests() {
        let url = req.url().to_string();
        let (status, body) = if req.method().as_str() == "OPTIONS" {
            (204, String::new())
        } else if url == "/health" {
            (200, "\"ok\"".to_string())
        } else if let Some(cmd) = url.strip_prefix("/invoke/") {
            let mut raw = String::new();
            let _ = req.as_reader().read_to_string(&mut raw);
            let args: Value = serde_json::from_str(&raw).unwrap_or(json!({}));
            match dispatch(cmd, &args) {
                Ok(v) => (200, v.to_string()),
                Err(e) => (400, json!(e).to_string()),
            }
        } else {
            (404, "\"not found\"".to_string())
        };
        let mut resp = Response::from_string(body).with_status_code(status);
        for h in &cors {
            resp.add_header(h.clone());
        }
        let _ = req.respond(resp);
    }
}
