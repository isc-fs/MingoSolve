//! Dimensional homogeneity of every formula in data/formulas.
//!
//! A physically valid equation holds in any coherent unit system. For each formula we build a consistent point in
//! SI (random inputs, solve the rest), re-express every value in a different unit system (length unit = 2.3 m,
//! mass = 1.7 kg, time = 0.6 s, current = 1.9 A, temperature = 1.3 K) and require every equation to still hold.
//! This catches equations that are wrong (a missing square, s*t instead of s/t) and dimensional constants hidden
//! in an equation (4e-7*pi for mu_0, 298.15 K, 9.81 m/s^2), which silently break as soon as the inputs come in a
//! form the author did not expect.

use std::collections::HashMap;

use fsq::engine::{conflicts_in, solve_step, Values};
use fsq::expr::Expr;
use fsq::registry::{registry, Formula};
use fsq::units::unit_of;

const SCALE: [f64; 7] = [2.3, 1.7, 0.6, 1.9, 1.3, 1.0, 1.0];

fn rescale(name: &str, value: f64) -> f64 {
    let dims = unit_of(&registry().var(name).unit).unwrap().dims;
    let unit_size: f64 = dims.iter().zip(SCALE).map(|(d, k)| k.powi(*d)).product();
    value / unit_size
}

/// Deterministic pseudo-random inputs in [0.3, 3] so no exp() underflows and no root sits at a pole.
fn inputs(f: &Formula, seed: u64, skip: &[String]) -> Values {
    let mut x = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    f.names
        .iter()
        .filter(|n| !skip.contains(n))
        .map(|n| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (x >> 11) as f64 / (1u64 << 53) as f64;
            (
                n.clone(),
                registry().var(n).default.unwrap_or(0.3 * 10f64.powf(u)),
            )
        })
        .collect()
}

fn consistent_point(f: &Formula) -> Option<Values> {
    let k = f.eqs.len();
    for seed in 0..200u64 {
        // a different random set of k unknowns per seed (Fisher-Yates on the names)
        let mut names = f.names.clone();
        let mut x = seed.wrapping_add(17).wrapping_mul(2862933555777941757);
        for i in (1..names.len()).rev() {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            names.swap(i, (x >> 33) as usize % (i + 1));
        }
        let unknowns: Vec<String> = names.into_iter().take(k).collect();
        let mut point = inputs(f, seed, &unknowns);
        for (n, vals) in solve_step(f, &point).0 {
            point.insert(n, vals[0]);
        }
        let complete = f.names.iter().all(|n| point.contains_key(n));
        let tame = point
            .values()
            .all(|v| v.is_finite() && v.abs() > 1e-9 && v.abs() < 1e12);
        if complete && tame && conflicts_in(f, &point, 1e-9).is_empty() {
            return Some(point);
        }
    }
    None
}

fn residuals(f: &Formula, point: &Values) -> Vec<(String, f64)> {
    f.eqs
        .iter()
        .zip(&f.src)
        .map(|(e, src)| {
            let Expr::Bin(_, a, b) = e else {
                unreachable!("equations are residuals")
            };
            let (l, r) = (a.eval_map(point).unwrap(), b.eval_map(point).unwrap());
            (
                src.clone(),
                (l - r).abs() / l.abs().max(r.abs()).max(1e-300),
            )
        })
        .collect()
}

#[test]
fn every_formula_is_dimensionally_homogeneous() {
    let mut broken = Vec::new();
    let mut untestable = Vec::new();
    for f in &registry().formulas {
        let Some(point) = consistent_point(f) else {
            untestable.push(f.key.clone());
            continue;
        };
        let scaled: Values = point
            .iter()
            .map(|(n, v)| (n.clone(), rescale(n, *v)))
            .collect();
        for (src, rel) in residuals(f, &scaled) {
            if rel > 1e-6 {
                broken.push(format!(
                    "{}: {src}  (off by {:.3} % in another unit system)",
                    f.key,
                    rel * 100.0
                ));
            }
        }
    }
    assert!(
        untestable.is_empty(),
        "no consistent point found for: {untestable:?}"
    );
    assert!(
        broken.is_empty(),
        "dimensionally inconsistent equations:\n{}",
        broken.join("\n")
    );
}

#[test]
fn the_check_catches_hidden_dimensional_constants() {
    // Guard against a vacuous test: an equation with mu_0 written as a bare number must fail the rescaling check.
    let e = fsq::expr::parse_equation("L = 4e-7*pi*N_t**2*A_core/l_core").unwrap();
    let f = Formula {
        key: "bad_inductance".into(),
        title: String::new(),
        src: vec!["L = 4e-7*pi*N_t**2*A_core/l_core".into()],
        names: {
            let mut v = Vec::new();
            e.vars(&mut v);
            v
        },
        eqs: vec![e],
        tags: vec![],
        notes: String::new(),
    };
    let point: Values = HashMap::from([
        ("N_t".to_string(), 100.0),
        ("A_core".to_string(), 1e-4),
        ("l_core".to_string(), 0.1),
        (
            "L".to_string(),
            4e-7 * std::f64::consts::PI * 1e4 * 1e-4 / 0.1,
        ),
    ]);
    assert!(
        residuals(&f, &point).iter().all(|(_, r)| *r < 1e-12),
        "the point is consistent in SI"
    );
    let scaled: Values = point
        .iter()
        .map(|(n, v)| (n.clone(), rescale(n, *v)))
        .collect();
    assert!(
        residuals(&f, &scaled).iter().any(|(_, r)| *r > 1e-3),
        "rescaling must expose the hidden H/m constant"
    );
}
