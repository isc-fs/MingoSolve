//! Property-based tests: solving any formula for a hidden variable recovers it, every root satisfies the
//! formula, and units / expressions / number formatting round-trip.

use std::collections::HashMap;

use fsq::engine::{conflicts_in, solve_step, Values};
use fsq::expr::{parse, Expr};
use fsq::format::g6;
use fsq::registry::registry;
use fsq::units::{convert, unit_of};
use proptest::prelude::*;

/// A consistent point for formula `fi`: sample its free inputs log-uniformly, solve the rest.
fn point(fi: usize, seed: &[f64]) -> Option<Values> {
    let f = &registry().formulas[fi];
    let k = f.eqs.len();
    let n = f.names.len();
    for attempt in 0..8 {
        let solve_for: Vec<&String> = (0..k)
            .map(|j| &f.names[(j + attempt + (seed[0] * n as f64) as usize) % n])
            .collect();
        let mut known = Values::new();
        for (i, name) in f.names.iter().enumerate() {
            if solve_for.contains(&name) {
                continue;
            }
            let v = registry().var(name);
            let x = v
                .default
                .unwrap_or_else(|| 10f64.powf(seed[(i + 1) % seed.len()] * 2.0 - 1.0));
            known.insert(name.clone(), x);
        }
        let found = solve_step(f, &known);
        for (name, vals) in &found.0 {
            known.insert(name.clone(), vals[0]);
        }
        let complete = f.names.iter().all(|n| known.contains_key(n));
        let sane = known
            .values()
            .all(|v| v.is_finite() && *v != 0.0 && v.abs() > 1e-12 && v.abs() < 1e15);
        // both sides of every equation must stay representable: V_1**562 underflows to 0 = 0 and any p_2 "solves" it
        let well_conditioned = complete
            && f.eqs.iter().all(|e| match e {
                Expr::Bin(_, a, b) => [a, b].iter().all(|side| {
                    side.eval_map(&known)
                        .is_some_and(|v| v.is_finite() && (v == 0.0 || v.abs() > 1e-200))
                        && side.eval_map(&known) != Some(0.0)
                }),
                _ => true,
            });
        if well_conditioned && sane && conflicts_in(f, &known, 1e-9).is_empty() {
            return Some(known);
        }
    }
    None
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn hiding_one_variable_recovers_it(fi in 0usize..70, seed in prop::collection::vec(0.0f64..1.0, 8), h in 0usize..16) {
        let f = &registry().formulas[fi];
        let Some(pt) = point(fi, &seed) else { return Ok(()) };
        let hidden = &f.names[h % f.names.len()];
        // skip unidentifiable cases (T = T_0 makes any alpha valid): nudging the hidden value must break something
        let mut nudged = pt.clone();
        nudged.insert(hidden.clone(), pt[hidden] * 1.1);
        if conflicts_in(f, &nudged, 1e-9).is_empty() {
            return Ok(());
        }
        let mut given = pt.clone();
        given.remove(hidden);
        let found = solve_step(f, &given);
        let truth = pt[hidden];
        let roots = found.get(hidden).unwrap_or(&[]);
        // periodic formulas (tan, sin, cos) return the principal branch, like sympy; any valid root is fine there
        let periodic = f.src.iter().any(|e| ["tan(", "sin(", "cos("].iter().any(|p| e.contains(p)));
        prop_assert!(
            roots.iter().any(|r| (r - truth).abs() <= 1e-6 * truth.abs().max(1e-12)) || (periodic && !roots.is_empty()),
            "{}: hide {hidden}, truth {truth}, roots {roots:?}, given {given:?}", f.key
        );
        // every root, carried forward, leaves the formula satisfied (or at worst the first one does)
        for r in roots {
            let mut g = given.clone();
            g.insert(hidden.clone(), *r);
            for (n, vals) in solve_step(f, &g).0 {
                g.insert(n, vals[0]);
            }
            if f.names.iter().all(|n| g.contains_key(n)) {
                let c = conflicts_in(f, &g, 1e-6);
                prop_assert!(c.is_empty(), "{}: root {r} of {hidden} violates {c:?}", f.key);
            }
        }
    }

    #[test]
    fn units_round_trip(x in 1e-6f64..1e6, idx in 0usize..12) {
        let pairs = [("km/h", "m/s"), ("rpm", "rad/s"), ("bar", "Pa"), ("Ah", "C"), ("kWh", "J"), ("mm**2", "m**2"),
                     ("uF", "F"), ("N/mm", "N/m"), ("g0", "m/s**2"), ("ft", "m"), ("psi", "Pa"), ("kN*m", "N*m")];
        let (a, b) = pairs[idx];
        let si = convert(&format!("{x} {a}"), b).unwrap();
        let back = si * unit_of(b).unwrap().value / unit_of(a).unwrap().value;
        prop_assert!((back - x).abs() <= 1e-12 * x);
    }

    #[test]
    fn display_and_parse_agree(a in -1e3f64..1e3, b in 0.1f64..1e3, c in -5.0f64..5.0) {
        let src = format!("({a} + x*{b})**2 / (1 + exp({c}*y)) - sqrt({b})*atan(x)");
        let e = parse(&src).unwrap();
        let again = parse(&e.to_string()).unwrap();
        let vals: HashMap<String, f64> = [("x".to_string(), 0.37), ("y".to_string(), -1.3)].into();
        let (v1, v2) = (e.eval_map(&vals).unwrap(), again.eval_map(&vals).unwrap());
        prop_assert!((v1 - v2).abs() <= 1e-12 * v1.abs().max(1.0));
        prop_assert!(!matches!(e, Expr::Num(_)));
    }

    #[test]
    fn g6_round_trips_to_six_figures(x in prop::num::f64::NORMAL) {
        let s = g6(x);
        let back: f64 = s.parse().unwrap();
        prop_assert!((back - x).abs() <= 5e-6 * x.abs(), "{x} -> {s}");
    }
}
