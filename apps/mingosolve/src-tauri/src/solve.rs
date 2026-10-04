//! Formulas: catalogue, solve one formula for whatever is missing, chain formulas to a target, calculator.

use std::collections::{HashMap, HashSet};

use fsq::cli::{fmt_var, pretty_unit};
use fsq::engine::{self, conflicts, conflicts_in};
use fsq::latex::{equation_latex, var_latex};
use fsq::registry::registry;
use serde::Serialize;

#[derive(Serialize)]
pub struct VarInfo {
    name: String,
    unit: String,
    desc: String,
    signed: bool,
    default: Option<f64>,
    /// The name typeset (LaTeX): rho_air → ρ_air.
    tex: String,
    /// The unit for display: m**2 → m², ohm → Ω.
    unit_shown: String,
}

#[derive(Serialize)]
pub struct FormulaInfo {
    key: String,
    title: String,
    eqs: Vec<String>,
    /// The equations typeset (LaTeX), in the same order as eqs.
    tex: Vec<String>,
    tags: Vec<String>,
    notes: String,
    vars: Vec<VarInfo>,
}

#[tauri::command]
pub fn list_formulas() -> Vec<FormulaInfo> {
    let r = registry();
    r.formulas
        .iter()
        .map(|f| FormulaInfo {
            key: f.key.clone(),
            title: f.title.clone(),
            eqs: f.src.clone(),
            tex: f.src.iter().map(|e| equation_latex(e)).collect(),
            tags: f.tags.clone(),
            notes: f.notes.clone(),
            vars: f
                .names
                .iter()
                .map(|n| {
                    let v = r.var(n);
                    VarInfo {
                        name: n.clone(),
                        unit: v.unit.clone(),
                        desc: v.desc.clone(),
                        signed: v.signed,
                        default: v.default,
                        tex: var_latex(n),
                        unit_shown: pretty_unit(&v.unit),
                    }
                })
                .collect(),
        })
        .collect()
}

#[derive(Serialize)]
pub struct FoundVar {
    name: String,
    desc: String,
    /// Values in the variable's own unit; `shown` is the same in the requested display unit.
    values: Vec<f64>,
    shown: Vec<String>,
}

#[derive(Serialize)]
pub struct Shown {
    name: String,
    shown: String,
}

#[derive(Serialize)]
pub struct SolveResult {
    found: Vec<FoundVar>,
    defaults: Vec<Shown>,
    conflicts: Vec<String>,
}

fn show_all(
    name: &str,
    values: &[f64],
    display: &HashMap<String, String>,
) -> Result<Vec<String>, String> {
    let unit = display
        .get(name)
        .map(String::as_str)
        .filter(|u| !u.trim().is_empty());
    values.iter().map(|v| fmt_var(name, *v, unit)).collect()
}

#[tauri::command]
pub fn solve_formula(
    key: String,
    given: Vec<(String, String)>,
    display: HashMap<String, String>,
) -> Result<SolveResult, String> {
    let given_ref: Vec<(&str, &str)> = given
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    let r = engine::solve(&key, &given_ref)?;
    let mut found = Vec::new();
    for (n, vals) in &r.found.0 {
        found.push(FoundVar {
            name: n.clone(),
            desc: registry().var(n).desc.clone(),
            values: vals.clone(),
            shown: show_all(n, vals, &display)?,
        });
    }
    let mut defaults: Vec<Shown> = r
        .defaults
        .iter()
        .map(|(n, v)| {
            Ok(Shown {
                name: n.clone(),
                shown: fmt_var(n, *v, None)?,
            })
        })
        .collect::<Result<_, String>>()?;
    defaults.sort_by(|a, b| a.name.cmp(&b.name));
    // all variables known: report any equation the given values break (over-specified input)
    let f = registry().formula(&key).ok_or("unknown formula")?;
    let mut known = HashMap::new();
    for (n, v) in &given_ref {
        known.insert(n.to_string(), engine::to_si(n, v).map_err(|e| e.0)?);
    }
    known.extend(r.defaults.clone());
    for (n, vals) in &r.found.0 {
        known.insert(n.clone(), vals[0]);
    }
    let conflicts = if f.names.iter().all(|n| known.contains_key(n)) {
        conflicts_in(f, &known, 1e-3)
    } else {
        vec![]
    };
    Ok(SolveResult {
        found,
        defaults,
        conflicts,
    })
}

#[derive(Serialize)]
pub struct ChainStep {
    formula: String,
    var: String,
    shown: String,
}

#[derive(Serialize)]
pub struct ChainResult {
    steps: Vec<ChainStep>,
    defaults: Vec<Shown>,
    reached: bool,
    target: Option<String>,
    known: Vec<String>,
    conflicts: Vec<String>,
}

#[tauri::command]
pub fn chain_formulas(
    target: String,
    given: Vec<(String, String)>,
    only: Vec<String>,
    display: HashMap<String, String>,
) -> Result<ChainResult, String> {
    let given_ref: Vec<(&str, &str)> = given
        .iter()
        .filter(|(n, v)| !n.trim().is_empty() && !v.trim().is_empty())
        .map(|(n, v)| (n.trim(), v.trim()))
        .collect();
    let only: Option<HashSet<String>> = (!only.is_empty()).then(|| only.into_iter().collect());
    let c = engine::chain(&target, &given_ref, only.as_ref())?;
    let unit = |n: &str| {
        display
            .get(n)
            .map(String::as_str)
            .filter(|u| !u.trim().is_empty())
    };
    let steps = c
        .steps
        .iter()
        .map(|s| {
            Ok(ChainStep {
                formula: s.formula.clone(),
                var: s.var.clone(),
                shown: fmt_var(&s.var, s.values[0], unit(&s.var))?,
            })
        })
        .collect::<Result<_, String>>()?;
    let mut defaults: Vec<Shown> = c
        .defaults
        .iter()
        .map(|(n, v)| {
            Ok(Shown {
                name: n.clone(),
                shown: fmt_var(n, *v, None)?,
            })
        })
        .collect::<Result<_, String>>()?;
    defaults.sort_by(|a, b| a.name.cmp(&b.name));
    let target_shown = if c.reached {
        Some(fmt_var(&target, c.known[&target], unit(&target))?)
    } else {
        None
    };
    let mut known: Vec<String> = c.known.keys().cloned().collect();
    known.sort();
    Ok(ChainResult {
        steps,
        defaults,
        reached: c.reached,
        target: target_shown,
        known,
        conflicts: conflicts(&c.known),
    })
}

#[tauri::command]
pub fn calc(expr: String) -> String {
    fsq::cli::run(&format!("calc {expr}")).trim().to_string()
}
