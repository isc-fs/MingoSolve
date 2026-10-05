//! Variable names as people type them. `v_i`, `μ`, `wheelbase` or `initial velocity` resolve to the registry name
//! (`v0`, `mu`, `L_wb`); a spelling that could mean several variables is reported as ambiguous, and an unknown one
//! comes back with suggestions. The spellings are data (`data/var_aliases.toml`).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::registry::registry;

#[derive(Deserialize)]
struct File {
    greek: HashMap<String, String>,
    alias: HashMap<String, String>,
    ambiguous: HashMap<String, Vec<String>>,
}

fn file() -> &'static File {
    static F: OnceLock<File> = OnceLock::new();
    F.get_or_init(|| toml::from_str(crate::data::VAR_ALIASES).expect("data/var_aliases.toml"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolved {
    /// Already a registry name.
    Exact,
    Alias {
        to: String,
    },
    Ambiguous(Vec<String>),
    Unknown {
        suggestions: Vec<String>,
    },
}

/// A spelling that was understood as another name, for showing to the user.
#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Problem {
    Unknown {
        name: String,
        suggestions: Vec<String>,
    },
    Ambiguous {
        name: String,
        choices: Vec<String>,
    },
}

impl Problem {
    pub fn name(&self) -> &str {
        match self {
            Problem::Unknown { name, .. } | Problem::Ambiguous { name, .. } => name,
        }
    }

    pub fn choices(&self) -> &[String] {
        match self {
            Problem::Unknown { suggestions, .. } => suggestions,
            Problem::Ambiguous { choices, .. } => choices,
        }
    }

    /// The sentence the CLI prints; the app builds its own from the same pieces.
    pub fn message(&self) -> String {
        let head = match self {
            Problem::Unknown { name, .. } => format!("Unknown name “{name}”."),
            Problem::Ambiguous { name, .. } => format!("“{name}” could mean several variables."),
        };
        match self.choices() {
            [] => head,
            c => {
                let said: Vec<String> = c.iter().map(|n| describe(n)).collect();
                format!("{head} Did you mean {}?", join_or(&said))
            }
        }
    }
}

/// The description for a sentence: short bracketed words are kept ("(final) velocity" is "final velocity"), long
/// remarks dropped.
pub fn short_desc(desc: &str) -> String {
    let mut out = String::new();
    let mut rest = desc;
    while let Some(open) = rest.find('(') {
        out.push_str(&rest[..open]);
        let Some(len) = rest[open..].find(')') else {
            break;
        };
        let inner = &rest[open + 1..open + len];
        if inner.chars().count() <= 12 {
            out.push_str(inner);
        }
        rest = &rest[open + len + 1..];
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `v0 (initial velocity)`.
pub fn describe(name: &str) -> String {
    match registry().vars.get(name) {
        Some(v) => format!("{name} ({})", short_desc(&v.desc)),
        None => name.to_string(),
    }
}

fn join_or(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [a] => a.clone(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
    }
}

/// Spellings with a run of three letters or more are words and match in any case; shorter ones are symbols.
fn is_word(s: &str) -> bool {
    let mut run = 0;
    for c in s.chars() {
        run = if c.is_ascii_alphabetic() { run + 1 } else { 0 };
        if run >= 3 {
            return true;
        }
    }
    false
}

/// Greek letters to their names, braces and TeX backslashes dropped, spaces and hyphens to underscores.
fn canon(name: &str) -> String {
    let greek = &file().greek;
    let mut out = String::new();
    for c in name.trim().chars() {
        match c {
            '{' | '}' | '\\' => {}
            ' ' | '-' => out.push('_'),
            c => match greek.get(c.to_string().as_str()) {
                Some(g) => out.push_str(g),
                None => out.push(c),
            },
        }
    }
    out
}

fn words(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// A description without its bracketed remarks: "acceleration (longitudinal)" is "acceleration".
fn plain_desc(desc: &str) -> String {
    let mut depth = 0;
    let bare: String = desc
        .chars()
        .filter(|&c| match c {
            '(' => {
                depth += 1;
                false
            }
            ')' => {
                depth -= 1;
                false
            }
            _ => depth == 0,
        })
        .collect();
    words(&bare).join(" ")
}

fn one_or_ambiguous(mut v: Vec<String>) -> Option<Resolved> {
    v.sort();
    v.dedup();
    match v.len() {
        0 => None,
        1 => Some(Resolved::Alias { to: v.remove(0) }),
        _ => Some(Resolved::Ambiguous(v)),
    }
}

pub fn resolve(name: &str) -> Resolved {
    let r = registry();
    let name = name.trim();
    if r.vars.contains_key(name) {
        return Resolved::Exact;
    }
    let c = canon(name);
    if r.vars.contains_key(&c) {
        return Resolved::Alias { to: c };
    }
    let f = file();
    if let Some(to) = f.alias.get(&c) {
        return Resolved::Alias { to: to.clone() };
    }
    if let Some(many) = f.ambiguous.get(&c) {
        return Resolved::Ambiguous(many.clone());
    }
    if is_word(&c) {
        let lc = c.to_lowercase();
        let hits = f
            .alias
            .iter()
            .filter(|(k, _)| is_word(k) && k.to_lowercase() == lc)
            .map(|(_, v)| v.clone());
        if let Some(x) = one_or_ambiguous(hits.collect()) {
            return x;
        }
        if let Some((_, many)) = f
            .ambiguous
            .iter()
            .find(|(k, _)| is_word(k) && k.to_lowercase() == lc)
        {
            return Resolved::Ambiguous(many.clone());
        }
        let hits = r
            .vars
            .keys()
            .filter(|k| is_word(k) && k.to_lowercase() == lc)
            .cloned();
        if let Some(x) = one_or_ambiguous(hits.collect()) {
            return x;
        }
    }
    let said = words(&c).join(" ");
    if !said.is_empty() {
        let hits = r
            .vars
            .values()
            .filter(|v| {
                plain_desc(&v.desc) == said || words(&short_desc(&v.desc)).join(" ") == said
            })
            .map(|v| v.name.clone());
        if let Some(x) = one_or_ambiguous(hits.collect()) {
            return x;
        }
    }
    Resolved::Unknown {
        suggestions: suggest(&c),
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Registry names close to a typo (edit distance up to 2, less for very short names), then names whose description
/// contains every word typed.
fn suggest(canon_name: &str) -> Vec<String> {
    let r = registry();
    let lc = canon_name.to_lowercase();
    let max = match lc.chars().count() {
        0..=2 => 0,
        3..=4 => 1,
        _ => 2,
    };
    let mut best: HashMap<String, usize> = HashMap::new();
    let spellings = r
        .vars
        .keys()
        .map(|k| (k, k))
        .chain(file().alias.iter().map(|(k, v)| (k, v)));
    for (spelling, target) in spellings {
        let d = edit_distance(&lc, &spelling.to_lowercase());
        if d <= max {
            let e = best.entry(target.clone()).or_insert(d);
            *e = (*e).min(d);
        }
    }
    let mut ranked: Vec<(usize, String)> = best.into_iter().map(|(n, d)| (d, n)).collect();
    ranked.sort();
    let mut out: Vec<String> = ranked.into_iter().map(|(_, n)| n).collect();
    let typed: Vec<String> = words(canon_name)
        .into_iter()
        .filter(|w| w.len() >= 3)
        .collect();
    if !typed.is_empty() {
        let mut by_desc: Vec<String> = r
            .vars
            .values()
            .filter(|v| {
                let d = words(&v.desc);
                typed
                    .iter()
                    .all(|w| d.iter().any(|x| x.contains(w.as_str())))
            })
            .map(|v| v.name.clone())
            .collect();
        by_desc.sort();
        out.extend(by_desc);
    }
    let mut seen = HashSet::new();
    out.retain(|n| seen.insert(n.clone()));
    out.truncate(5);
    out
}

/// A name resolved to the registry name, plus the mapping when it was not typed that way.
pub fn fix(name: &str) -> Result<(String, Option<Mapping>), Problem> {
    let name = name.trim();
    match resolve(name) {
        Resolved::Exact => Ok((name.to_string(), None)),
        Resolved::Alias { to } => {
            let m = Mapping {
                from: name.to_string(),
                to: to.clone(),
            };
            Ok((to, Some(m)))
        }
        Resolved::Ambiguous(choices) => Err(Problem::Ambiguous {
            name: name.to_string(),
            choices,
        }),
        Resolved::Unknown { suggestions } => Err(Problem::Unknown {
            name: name.to_string(),
            suggestions,
        }),
    }
}

#[derive(Debug, Default)]
pub struct Normalised {
    pub target: Option<String>,
    pub given: Vec<(String, String)>,
    pub mapped: Vec<Mapping>,
}

/// Resolve the target (if any) and every typed name; all the problems at once when some do not resolve.
pub fn normalise(target: Option<&str>, given: &[(&str, &str)]) -> Result<Normalised, Vec<Problem>> {
    let mut out = Normalised::default();
    let mut problems = Vec::new();
    let mut take = |name: &str, mapped: &mut Vec<Mapping>| match fix(name) {
        Ok((n, m)) => {
            mapped.extend(m);
            Some(n)
        }
        Err(p) => {
            problems.push(p);
            None
        }
    };
    out.target = target.and_then(|t| take(t, &mut out.mapped));
    for (n, v) in given {
        if let Some(n) = take(n, &mut out.mapped) {
            out.given.push((n, v.to_string()));
        }
    }
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems)
    }
}

/// All problems as one line for the CLI.
pub fn problems_message(problems: &[Problem]) -> String {
    problems
        .iter()
        .map(Problem::message)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine;

    fn alias(name: &str) -> String {
        match resolve(name) {
            Resolved::Alias { to } => to,
            other => panic!("{name}: {other:?}"),
        }
    }

    #[test]
    fn owners_case_through_the_real_chain_path() {
        // s = (v0 + v) / 2 * t = (0 + 20) / 2 * 4 = 40 m
        let c = engine::chain("s", &[("v_i", "0"), ("v_f", "20m/s"), ("t", "4s")], None).unwrap();
        assert!(c.reached);
        assert!((c.known["s"] - 40.0).abs() < 1e-9);
        let said: Vec<_> = c.mapped.iter().map(|m| (&*m.from, &*m.to)).collect();
        assert_eq!(said, [("v_i", "v0"), ("v_f", "v")]);
        // the result is keyed by the registry names
        assert!(c.known.contains_key("v0") && !c.known.contains_key("v_i"));
    }

    #[test]
    fn one_formula_solve_accepts_aliases_too() {
        // a = (v - v0) / t = (20 - 0) / 4 = 5 m/s²
        let r = engine::solve(
            "uniform_accel",
            &[("v_i", "0"), ("v_f", "20m/s"), ("t", "4s")],
        )
        .unwrap();
        assert!((r.found.get("a").unwrap()[0] - 5.0).abs() < 1e-9);
        assert_eq!(r.mapped.len(), 2);
    }

    #[test]
    fn every_alias_points_at_a_registry_variable() {
        let f = file();
        for (k, to) in &f.alias {
            assert!(registry().vars.contains_key(to), "{k} -> {to}");
        }
        for (k, many) in &f.ambiguous {
            assert!(many.len() >= 2, "{k} is not ambiguous");
            for to in many {
                assert!(registry().vars.contains_key(to), "{k} -> {to}");
            }
        }
    }

    #[test]
    fn no_alias_shadows_a_registry_name() {
        let f = file();
        for k in f.alias.keys().chain(f.ambiguous.keys()) {
            assert!(!registry().vars.contains_key(k), "{k} is a registry name");
        }
        for k in f.alias.keys() {
            assert!(
                !f.ambiguous.contains_key(k),
                "{k} is both alias and ambiguous"
            );
        }
        // time and temperature keep their meaning
        assert_eq!(resolve("t"), Resolved::Exact);
        assert_eq!(resolve("T"), Resolved::Exact);
        assert_eq!(resolve("V_f"), Resolved::Exact);
    }

    #[test]
    fn every_alias_resolves_to_its_own_target() {
        for (k, to) in &file().alias {
            assert_eq!(resolve(k), Resolved::Alias { to: to.clone() }, "{k}");
        }
        for (k, many) in &file().ambiguous {
            let mut want = many.clone();
            want.sort();
            match resolve(k) {
                Resolved::Ambiguous(mut got) => {
                    got.sort();
                    assert_eq!(got, want, "{k}");
                }
                other => panic!("{k}: {other:?}"),
            }
        }
    }

    #[test]
    fn word_spellings_differing_only_in_case_agree() {
        let mut seen: HashMap<String, &String> = HashMap::new();
        for (k, to) in file().alias.iter().filter(|(k, _)| is_word(k)) {
            if let Some(prev) = seen.insert(k.to_lowercase(), to) {
                assert_eq!(prev, to, "{k} is spelled two ways to different targets");
            }
        }
    }

    #[test]
    fn ambiguous_spellings_are_reported_not_guessed() {
        let Resolved::Ambiguous(c) = resolve("r") else {
            panic!("r must be ambiguous");
        };
        assert!(c.contains(&"R_c".to_string()) && c.contains(&"r_w".to_string()));
        let e = engine::chain("v", &[("r", "14.5m")], None).unwrap_err();
        assert!(
            e.contains("Did you mean") && e.contains("R_c") && e.contains("r_w"),
            "{e}"
        );
    }

    #[test]
    fn symbols_are_case_sensitive_and_words_are_not() {
        assert_eq!(alias("v_f"), "v");
        assert_eq!(alias("U"), "V");
        assert_eq!(alias("u"), "v0");
        assert_eq!(alias("Wheelbase"), "L_wb");
        assert_eq!(alias("CDA"), "CdA");
        assert!(matches!(resolve("Vf"), Resolved::Unknown { .. }));
    }

    #[test]
    fn greek_letters_and_delta_prefix_map_to_names() {
        assert_eq!(alias("μ"), "mu");
        assert_eq!(alias("µ"), "mu");
        assert_eq!(alias("ρ_air"), "rho_air");
        assert_eq!(alias("η"), "eta");
        assert_eq!(alias("ω_m"), "omega_m");
        assert_eq!(alias("Δh"), "dh");
        assert_eq!(alias("ΔT"), "dT");
        assert_eq!(alias("v_{i}"), "v0");
        let c = engine::chain("ax", &[("μ", "1.4")], None).unwrap();
        assert!(c.known.contains_key("mu"));
    }

    #[test]
    fn a_description_typed_as_a_name_finds_the_variable() {
        assert_eq!(alias("initial velocity"), "v0");
        assert_eq!(alias("initial_velocity"), "v0");
        assert_eq!(alias("final velocity"), "v");
    }

    #[test]
    fn typos_get_suggestions() {
        let Resolved::Unknown { suggestions } = resolve("v_intial") else {
            panic!("a typo is not an alias");
        };
        assert!(suggestions.contains(&"v0".to_string()), "{suggestions:?}");
        let Resolved::Unknown { suggestions } = resolve("wheelbse") else {
            panic!("expected unknown");
        };
        assert_eq!(suggestions.first().map(String::as_str), Some("L_wb"));
        let Resolved::Unknown { suggestions } = resolve("velocty") else {
            panic!("expected unknown");
        };
        assert!(suggestions.contains(&"v".to_string()), "{suggestions:?}");
        let e = engine::chain("s", &[("v_intial", "0")], None).unwrap_err();
        assert!(
            e.contains("Unknown name “v_intial”") && e.contains("v0 (initial velocity)"),
            "{e}"
        );
    }

    #[test]
    fn short_descriptions_read_as_words() {
        assert_eq!(short_desc("(final) velocity"), "final velocity");
        assert_eq!(short_desc("wheel (dynamic) radius"), "wheel dynamic radius");
        assert_eq!(
            short_desc("corner radius (skidpad centreline 9.125)"),
            "corner radius"
        );
    }
}
