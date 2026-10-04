//! Test-only HTTP bridge: serves the app's real Tauri command functions on 127.0.0.1 so the browser end-to-end
//! tests (apps/mingosolve/e2e) drive the real engine without a native window. POST /invoke/<command> with the same
//! JSON arguments the frontend passes to invoke(); 200 + JSON result, or 400 + JSON error string.
//! Not part of the shipped app (cargo example, built only by the e2e runner).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;

use mingosolve_lib::{finder, guarded, solve, tools, topics};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    serde_json::from_value(args.get(name).cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("argument {name}: {e}"))
}

fn dispatch(cmd: &str, a: &Value) -> Result<Value, String> {
    guarded(|| dispatch_inner(cmd, a))
}

fn dispatch_inner(cmd: &str, a: &Value) -> Result<Value, String> {
    let ok = |v: Value| Ok(v);
    match cmd {
        "engine_version" => ok(json!(env!("CARGO_PKG_VERSION"))),
        "list_formulas" => ok(json!(solve::list_formulas())),
        "solve_formula" => solve::solve_formula_impl(
            arg(a, "key")?,
            arg(a, "given")?,
            arg::<HashMap<String, String>>(a, "display")?,
        )
        .map(|r| json!(r)),
        "chain_formulas" => solve::chain_formulas_impl(
            arg(a, "target")?,
            arg(a, "given")?,
            arg(a, "only")?,
            arg(a, "display")?,
        )
        .map(|r| json!(r)),
        "calc" => ok(json!(solve::calc_impl(&arg::<String>(a, "expr")?))),
        "list_tools" => ok(json!(tools::list_tools())),
        "run_tool" => tools::run_tool_impl(
            &arg::<String>(a, "name")?,
            &arg::<Vec<(String, String)>>(a, "args")?,
        )
        .map(|r| json!(r)),
        "find_question" => ok(json!(finder::find_question_impl(&arg::<String>(
            a, "text"
        )?))),
        "match_options" => ok(json!(finder::match_options_impl(
            &arg::<String>(a, "answer")?,
            &arg::<String>(a, "options")?
        ))),
        "format_answer" => fsq::answer::format_answer(
            arg(a, "value")?,
            arg(a, "precision")?,
            arg(a, "decimalComma")?,
        )
        .map(|r| json!(r)),
        "list_topics" => ok(json!(topics::list_topics())),
        "script_examples" => ok(json!(topics::script_examples(arg(a, "script")?))),
        other => Err(format!("unknown command {other}")),
    }
}

fn main() {
    let port = std::env::var("BRIDGE_PORT").unwrap_or_else(|_| "8799".into());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("bind bridge port");
    eprintln!("ipc bridge on http://127.0.0.1:{port}");
    let t0 = Instant::now();
    let trace = std::env::var("BRIDGE_TRACE").is_ok_and(|v| !v.is_empty());
    // one thread and one request per connection (Connection: close), so nothing waits behind anything else
    for stream in listener.incoming().flatten() {
        std::thread::spawn(move || {
            let accepted = t0.elapsed().as_secs_f64();
            if let Err(e) = serve(stream, t0, accepted, trace) {
                eprintln!("bridge: {e}");
            }
        });
    }
}

fn serve(mut stream: TcpStream, t0: Instant, accepted: f64, trace: bool) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, url) = (
        parts.next().unwrap_or("").to_string(),
        parts.next().unwrap_or("").to_string(),
    );
    let mut length = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = header.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut raw = vec![0; length];
    reader.read_exact(&mut raw)?;
    if trace {
        eprintln!(
            "{accepted:.3} accepted, {:.3} → {method} {url}",
            t0.elapsed().as_secs_f64()
        );
    }
    let started = Instant::now();
    let (status, body) = if method == "OPTIONS" {
        (204, String::new())
    } else if url == "/health" {
        (200, "\"ok\"".to_string())
    } else if let Some(cmd) = url.strip_prefix("/invoke/") {
        let args: Value = serde_json::from_slice(&raw).unwrap_or(json!({}));
        match dispatch(cmd, &args) {
            Ok(v) => (200, v.to_string()),
            Err(e) => (400, json!(e).to_string()),
        }
    } else {
        (404, "\"not found\"".to_string())
    };
    if started.elapsed().as_secs_f64() > 1.0 {
        eprintln!("slow: {url} took {:.1} s", started.elapsed().as_secs_f64());
    }
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        _ => "Not Found",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()?;
    if trace {
        eprintln!("{:.3} ← {url} {status}", t0.elapsed().as_secs_f64());
    }
    Ok(())
}
