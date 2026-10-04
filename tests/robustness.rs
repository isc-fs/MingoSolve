//! Crash-proofing: the parser's depth cap holds on a 2 MiB thread stack (what a Tokio blocking thread gets), hostile
//! input is rejected instead of aborting, NaN and infinity never reach the user, and fuzzed text never panics or
//! hangs anywhere user text goes (calculator, finder, tools, solver values).

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use fsq::cli::run;
use fsq::engine;
use fsq::expr::{parse, parse_equation, Expr, MAX_DEPTH};
use fsq::finder::{find, quantities};
use fsq::latex::expr_latex;
use fsq::registry::registry;
use fsq::tools::tools;
use fsq::units;
use proptest::prelude::*;

const STACK: usize = 2 * 1024 * 1024;

/// Run `f` on a thread with a Tokio-sized stack; a panic comes back as Err, a hang as a timeout error.
fn bounded<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let _ = tx.send(r.map_err(|_| "panicked".to_string()));
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(20)) {
        Ok(r) => r,
        Err(_) => Err("did not finish within 20 s".into()),
    }
}

/// Inputs of height `n` in every shape that makes the parser, evaluator, printer or drop recurse.
fn shapes(n: usize) -> Vec<(&'static str, String)> {
    vec![
        ("brackets", format!("{}1{}", "(".repeat(n), ")".repeat(n))),
        ("negations", format!("{}1", "-".repeat(n))),
        ("plus signs", format!("{}1", "+".repeat(n))),
        ("calls", format!("{}1{}", "abs(".repeat(n), ")".repeat(n))),
        ("sum chain", vec!["1"; n + 1].join("+")),
        ("product chain", vec!["2"; n + 1].join("*")),
        ("power tower", vec!["1"; n + 1].join("**")),
        ("power of negations", format!("2**{}1", "-".repeat(n - 1))),
        (
            "bracketed sums",
            format!("{}1{}", "(1+".repeat(n), ")".repeat(n)),
        ),
    ]
}

/// Everything user text can trigger on a parsed tree: evaluate, print, typeset, substitute, drop.
fn exercise(src: &str) -> Result<(), String> {
    let e = parse(src).map_err(|e| e.0)?;
    let _ = e.eval(&|_| None);
    let _ = e.to_string();
    let _ = expr_latex(&e);
    let _ = e.substitute(&Default::default());
    let _ = units::parse_quantity(src);
    drop(e);
    Ok(())
}

#[test]
fn parsing_at_the_depth_limit_fits_a_2_mib_stack() {
    for (shape, src) in shapes(MAX_DEPTH - 1) {
        let r = bounded(move || exercise(&src)).unwrap_or_else(|e| panic!("{shape}: {e}"));
        assert_eq!(r, Ok(()), "{shape} at the limit must still parse");
    }
}

#[test]
fn just_past_the_limit_is_a_parse_error() {
    for (shape, src) in shapes(MAX_DEPTH + 2) {
        let err = parse(&src).unwrap_err();
        assert!(err.0.contains("nested too deeply"), "{shape}: {err}");
    }
}

#[test]
fn very_deep_input_is_an_error_not_an_abort() {
    for (shape, src) in shapes(100_000) {
        let line = format!("calc {src}");
        let (err, out) = bounded(move || (parse(&src).unwrap_err().0, run(&line)))
            .unwrap_or_else(|e| panic!("{shape}: {e}"));
        assert_eq!(err, "expression nested too deeply", "{shape}");
        assert_eq!(out, "  error: expression nested too deeply", "{shape}");
    }
}

fn height(e: &Expr) -> usize {
    match e {
        Expr::Num(_) | Expr::Var(_) => 0,
        Expr::Neg(x) | Expr::Call(_, x) => 1 + height(x),
        Expr::Bin(_, a, b) => 1 + height(a).max(height(b)),
    }
}

#[test]
fn no_shipped_formula_comes_near_the_depth_limit() {
    let worst = registry()
        .formulas
        .iter()
        .flat_map(|f| f.eqs.iter())
        .map(height)
        .max()
        .unwrap();
    assert!(
        worst * 4 <= MAX_DEPTH,
        "deepest equation has height {worst}"
    );
    for f in &registry().formulas {
        for src in &f.src {
            assert!(
                src.matches('(').count() * 4 <= MAX_DEPTH,
                "{}: {src}",
                f.key
            );
        }
    }
}

#[test]
fn nan_and_infinity_become_errors() {
    let nf = "error: no finite answer (division by zero or overflow)";
    for line in [
        "calc 10**400/10**400",
        "calc 0**-1",
        "calc 1/0",
        "calc 10**400",
        "calc 5 m -> 0*m",
    ] {
        assert_eq!(run(line).trim(), nf, "{line}");
    }
    assert_eq!(
        run("calc 1e999 m").trim(),
        "error: number 1e999 is too large"
    );
    // values typed into a solver field
    let e = engine::solve("speed", &[("s", "1e999 m"), ("t", "3 s")]).unwrap_err();
    assert!(e.contains("too large"), "{e}");
    let e = engine::solve("speed", &[("s", "0**-1"), ("t", "3")]).unwrap_err();
    assert!(e.contains("no finite answer"), "{e}");
    // a display unit of zero size would divide by zero
    let out = run("speed s=75m t=3.8s @v_avg=0*m/s");
    assert!(out.contains("no finite answer"), "{out}");
}

#[test]
fn unit_exponents_cannot_overflow() {
    // dims are i32: combining saturated exponents used to panic in debug builds
    let src = "((m**100000000000)**100000000000)*((m**100000000000)**100000000000)";
    let out = bounded(move || run(&format!("calc {src}"))).unwrap();
    assert!(!out.is_empty());
    let out = bounded(|| run("calc 1/(m**-2147483648)")).unwrap();
    assert!(!out.is_empty());
}

/// Minimal inputs the fuzzer found: each used to panic (or abort on a huge allocation) inside a tool.
#[test]
fn fuzz_regressions_in_tools_are_plain_errors() {
    let call = |tool: &str, args: &[&str]| {
        let (tool, args): (String, Vec<String>) =
            (tool.into(), args.iter().map(|a| a.to_string()).collect());
        bounded(move || {
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            fsq::tools::tool(&tool).unwrap().call(&refs)
        })
        .unwrap()
    };
    for (tool, args) in [
        ("e_series", vec!["4.7", "0"]), // empty series: unwrap on no candidates
        ("e_series", vec!["4.7", "1e12"]), // allocation abort
        ("e_series", vec!["-3", "12"]), // log10 of a negative
        ("twos", vec!["5", "200"]),     // 1 << 200 overflows
        ("twos", vec!["5", "4e9"]),     // 4 GB format width
        ("ts_rules", vec!["1e10"]),     // above the last insulation tier
        ("nodal", vec!["R1 1 0 0; V1 1 0 5"]), // 1/0 ohm
        ("nodal", vec!["R1 1 0 1e2000000000; V1 1 0 5"]), // 10^2e9 as a BigInt
    ] {
        let r = call(tool, &args);
        assert!(r.is_err(), "{tool} {args:?} gave {r:?}");
    }
}

// ---------------------------------------------------------------- fuzzing

fn cases() -> ProptestConfig {
    ProptestConfig {
        cases: std::env::var("FSQ_FUZZ_CASES")
            .ok()
            .and_then(|c| c.parse().ok())
            .unwrap_or(40),
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// Text built from the tokens the grammar cares about, so the fuzzer reaches past the tokenizer.
fn tokens() -> impl Strategy<Value = String> {
    let atoms = prop::sample::select(vec![
        "(", ")", "(", ")", "+", "-", "*", "**", "/", "^", ".", ",", "=", "->", " ", "1", "0",
        "2.5", "1e308", "1e999", "1e-999", "10**400", "0**-1", "pi", "m", "kg", "s", "km/h", "rpm",
        "degC", "%", "ohm", "Ω", "µ", "mm", "abs(", "sqrt(", "log(", "exp(", "asin(", "foo(", "e",
        "E", "1e", "1e+", ".5", "5.", "3 m/s", "@", "?", ";", "\n", "\"", "²", "°C", "NaN", "inf",
        "-0",
    ]);
    prop::collection::vec(atoms, 0..40).prop_map(|v| v.concat())
}

fn deep() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec!["(", "-", "+", "abs(", "2**", "1+(", "-(", "2*("]),
        0usize..4000,
        prop::sample::select(vec![")", ""]),
    )
        .prop_map(|(open, n, close)| format!("{}1{}", open.repeat(n), close.repeat(n)))
}

fn text() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => tokens(),
        2 => any::<String>(),
        1 => deep(),
        1 => "[0-9a-zA-Z_ ()+*/.,=°²µΩ@;e%\\-]{0,200}",
        1 => prop::collection::vec(any::<char>(), 0..2000).prop_map(|c| c.into_iter().collect()),
    ]
}

/// Numbers at the edges of what the tools do arithmetic, shifts, allocations and divisions with.
fn edge_number() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(vec![
            "0",
            "1",
            "-1",
            "2",
            "6",
            "12",
            "24",
            "128",
            "129",
            "255",
            "256",
            "0.5",
            "1e9",
            "1e12",
            "-1e12",
            "4294967296",
            "2147483648",
            "1e300",
            "-1e300",
            "1e-300",
            "1e18",
            "9223372036854775808",
            "1e2000000000",
            "1e-2000000000",
            "5e-324",
        ])
        .prop_map(String::from),
        any::<i64>().prop_map(|n| n.to_string()),
        any::<f64>().prop_map(|x| format!("{x:e}")),
    ]
}

fn tool_arg() -> impl Strategy<Value = String> {
    prop_oneof![3 => edge_number(), 2 => text(), 1 => (edge_number(), edge_number(), edge_number()).prop_map(|(a, b, c)| format!("R1 {a} 0 {b}; R2 {c} 0 1"))]
}

fn has_non_finite_word(s: &str) -> bool {
    s.split(|c: char| !c.is_alphanumeric() && c != '-')
        .any(|w| matches!(w, "NaN" | "inf" | "-inf"))
}

/// Run `f` on the big-stack thread, fail on panic or hang, and require it to return within 2 s.
fn within_2s<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, TestCaseError> {
    let t0 = Instant::now();
    let r = bounded(f).map_err(TestCaseError::fail)?;
    prop_assert!(
        t0.elapsed() < Duration::from_secs(2),
        "took {:?}",
        t0.elapsed()
    );
    Ok(r)
}

proptest! {
    #![proptest_config(cases())]

    #[test]
    fn parser_never_panics(s in text()) {
        within_2s(move || {
            let _ = parse(&s);
            let _ = parse_equation(&s);
            let _ = exercise(&s);
        })?;
    }

    #[test]
    fn calculator_never_panics_and_never_prints_nan(s in text()) {
        let out = within_2s(move || run(&format!("calc {s}")))?;
        // an error may quote what the user typed ("unknown unit NaN"); a result never shows a non-finite value
        prop_assert!(out.starts_with("  error") || !has_non_finite_word(&out), "{out}");
    }

    #[test]
    fn finder_never_panics(s in text()) {
        within_2s(move || {
            let _ = find(&s, 24);
            let _ = quantities(&s);
        })?;
    }

    #[test]
    fn every_tool_survives_random_arguments(
        tool in 0..tools().len(),
        args in prop::collection::vec(tool_arg(), 0..8),
    ) {
        let out = within_2s(move || {
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            tools()[tool].call(&refs)
        })?;
        if let Ok(o) = out {
            prop_assert!(!has_non_finite_word(&o), "{o}");
        }
    }

    #[test]
    fn nodal_and_truss_survive_well_formed_nonsense(
        nums in prop::collection::vec(edge_number(), 12),
        kinds in prop::collection::vec(prop::sample::select(vec!["R", "V", "I", "X"]), 4),
        ask in prop::sample::select(vec!["", "Rab:1,0", "Rab:1,1", "Rab:9,0"]),
        members in prop::sample::select(vec!["A-B B-C A-C", "A-A", "A-B", "A-B B-C A-C C-A", "A-Z"]),
        supports in prop::sample::select(vec!["A pin; B roller", "A pin", "A pin; B rollerx; C pin", "A pin; B zz"]),
    ) {
        let out = within_2s(move || {
            let net = (0..4)
                .map(|i| format!("{}{} {} {} {}", kinds[i], i, i, (i + 1) % 3, nums[i]))
                .collect::<Vec<_>>()
                .join("; ");
            let nodal = fsq::tools::tool("nodal").unwrap().call(&[&net, &format!("ask={ask}")]);
            let nodes = format!("A {} {}; B {} {}; C {} {}", nums[4], nums[5], nums[6], nums[7], nums[8], nums[9]);
            let loads = format!("C {} {}; B {}@{}", nums[10], nums[11], nums[0], nums[1]);
            let truss = fsq::tools::tool("truss").unwrap().call(&[
                &format!("nodes={nodes}"),
                &format!("members={members}"),
                &format!("supports={supports}"),
                &format!("loads={loads}"),
            ]);
            (nodal, truss)
        })?;
        for o in [out.0, out.1].into_iter().flatten() {
            prop_assert!(!has_non_finite_word(&o), "{o}");
        }
    }

    #[test]
    fn solving_with_hostile_values_never_panics(
        which in 0..registry().formulas.len(),
        values in prop::collection::vec(prop_oneof![2 => edge_number(), 1 => text()], 0..8),
    ) {
        let r = within_2s(move || {
            let f = &registry().formulas[which];
            let given: Vec<(&str, &str)> = f
                .names
                .iter()
                .map(String::as_str)
                .zip(values.iter().map(String::as_str))
                .collect();
            let solved = engine::solve(&f.key, &given);
            let chained = engine::chain(&f.names[0], &given, None);
            (solved, chained)
        })?;
        if let (Ok(s), _) = &r {
            for (_, vals) in &s.found.0 {
                prop_assert!(vals.iter().all(|v| v.is_finite()));
            }
        }
    }
}
