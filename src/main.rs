//! `fsq` command line: one-shot (`fsq ex skidpad`) or an interactive loop when run without arguments.

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        let line = args
            .iter()
            .map(|a| {
                if a.contains(' ') {
                    format!("\"{a}\"")
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>();
        println!("{}", fsq::cli::run(&line.join(" ")));
        return;
    }
    let r = fsq::registry::registry();
    println!(
        "fsq {}: {} formulas, {} variables. 'help' for commands.",
        env!("CARGO_PKG_VERSION"),
        r.formulas.len(),
        r.vars.len()
    );
    let stdin = io::stdin();
    loop {
        print!("fsq> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            println!();
            return;
        }
        let line = line.trim();
        if matches!(line, "quit" | "exit" | "q") {
            return;
        }
        let out = fsq::cli::run(line);
        if !out.is_empty() {
            println!("{out}");
        }
    }
}
