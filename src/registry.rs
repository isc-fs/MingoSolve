//! Variables and formulas loaded from the embedded `data/formulas/*.toml`, in file order (which `chain` relies on).
//! Each equation is stored as the residual `lhs - rhs`; every unit and variable reference is checked at load.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::expr::{self, Expr};
use crate::units;

#[derive(Debug, Clone, Deserialize)]
pub struct Var {
    pub name: String,
    pub unit: String,
    pub desc: String,
    #[serde(default)]
    pub signed: bool,
    pub default: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Formula {
    pub key: String,
    pub title: String,
    pub src: Vec<String>,
    pub eqs: Vec<Expr>,
    pub tags: Vec<String>,
    pub notes: String,
    /// Variables in order of first appearance across the equations.
    pub names: Vec<String>,
}

#[derive(Deserialize)]
struct FormulaRow {
    key: String,
    title: String,
    eqs: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    notes: String,
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    var: Vec<Var>,
    #[serde(default)]
    formula: Vec<FormulaRow>,
}

pub struct Registry {
    pub vars: HashMap<String, Var>,
    pub formulas: Vec<Formula>,
    index: HashMap<String, usize>,
}

impl Registry {
    pub fn formula(&self, key: &str) -> Option<&Formula> {
        self.index.get(key).map(|&i| &self.formulas[i])
    }

    pub fn var(&self, name: &str) -> &Var {
        &self.vars[name]
    }

    /// Formulas whose key, title, tags, notes or variable descriptions contain every word of `query`.
    pub fn search(&self, query: &str) -> Vec<&Formula> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        self.formulas
            .iter()
            .filter(|f| {
                let mut hay = format!("{} {} {} {}", f.key, f.title, f.tags.join(" "), f.notes);
                for n in &f.names {
                    hay.push(' ');
                    hay.push_str(&self.vars[n].desc);
                }
                let hay = hay.to_lowercase();
                words.iter().all(|w| hay.contains(w.as_str()))
            })
            .collect()
    }
}

fn build() -> Result<Registry, String> {
    let mut vars: HashMap<String, Var> = HashMap::new();
    let mut formulas = Vec::new();
    let mut index = HashMap::new();
    for (file, text) in crate::data::FORMULAS {
        let f: File = toml::from_str(text).map_err(|e| format!("{file}: {e}"))?;
        for v in f.var {
            units::unit_of(&v.unit)
                .map_err(|e| format!("{file}: var {}: unit {:?}: {e}", v.name, v.unit))?;
            if let Some(old) = vars.get(&v.name) {
                if (&old.unit, &old.desc) != (&v.unit, &v.desc) {
                    return Err(format!("{file}: variable {} redefined", v.name));
                }
            }
            vars.insert(v.name.clone(), v);
        }
        for row in f.formula {
            let mut eqs = Vec::new();
            let mut names = Vec::new();
            for src in &row.eqs {
                let e =
                    expr::parse_equation(src).map_err(|e| format!("{}: {src}: {e}", row.key))?;
                e.vars(&mut names);
                eqs.push(e);
            }
            if let Some(n) = names.iter().find(|n| !vars.contains_key(*n)) {
                return Err(format!("{}: unregistered variable {n}", row.key));
            }
            if index.insert(row.key.clone(), formulas.len()).is_some() {
                return Err(format!("duplicate formula {}", row.key));
            }
            formulas.push(Formula {
                key: row.key,
                title: row.title,
                src: row.eqs,
                eqs,
                tags: row.tags,
                notes: row.notes,
                names,
            });
        }
    }
    Ok(Registry {
        vars,
        formulas,
        index,
    })
}

pub fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| build().expect("data/formulas is consistent"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_everything_in_order() {
        let r = build().unwrap();
        assert_eq!(r.formulas.len(), 70);
        assert_eq!(r.vars.len(), 231);
        assert_eq!(r.formulas[0].key, "aero_forces");
        let s = r.formula("uniform_accel").unwrap();
        assert_eq!(s.names, ["v", "v0", "a", "t", "s"]);
        assert_eq!(r.var("g").default, Some(9.81));
        assert!(r.var("phi").signed);
    }

    #[test]
    fn search_matches_all_words() {
        let r = registry();
        let keys: Vec<&str> = r
            .search("discharge rule")
            .iter()
            .map(|f| f.key.as_str())
            .collect();
        assert!(keys.contains(&"ts_discharge"));
        assert!(r.search("zzz-nothing").is_empty());
    }
}
