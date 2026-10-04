//! Embedded copies of everything under `data/`. Edit the TOML, not this file.

pub const FORMULAS: &[(&str, &str)] = &[
    ("00_common", include_str!("../data/formulas/00_common.toml")),
    ("01_aero", include_str!("../data/formulas/01_aero.toml")),
    (
        "02_dynamics",
        include_str!("../data/formulas/02_dynamics.toml"),
    ),
    (
        "03_electrical",
        include_str!("../data/formulas/03_electrical.toml"),
    ),
    ("04_energy", include_str!("../data/formulas/04_energy.toml")),
    (
        "05_kinematics",
        include_str!("../data/formulas/05_kinematics.toml"),
    ),
    (
        "06_powertrain",
        include_str!("../data/formulas/06_powertrain.toml"),
    ),
    (
        "07_structures",
        include_str!("../data/formulas/07_structures.toml"),
    ),
    ("08_thermo", include_str!("../data/formulas/08_thermo.toml")),
];

pub const RULES: &[(&str, &str)] = &[
    ("legacy", include_str!("../data/rules/legacy.toml")),
    ("2026", include_str!("../data/rules/2026.toml")),
    ("2027", include_str!("../data/rules/2027.toml")),
];

pub const PENALTIES: &str = include_str!("../data/rules/penalties.toml");
pub const EXAMPLES: &str = include_str!("../data/examples.toml");
pub const KNOWN_KEYS: &str = include_str!("../data/known_keys.toml");
pub const TOPICS: &str = include_str!("../data/topics.toml");
pub const UNITS: &str = include_str!("../data/units.toml");
pub const UNITS_GOLDEN: &str = include_str!("../data/units_golden.toml");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_file_is_valid_toml() {
        let all = FORMULAS.iter().chain(RULES).map(|(_, s)| *s);
        for s in all.chain([PENALTIES, EXAMPLES, KNOWN_KEYS, TOPICS, UNITS, UNITS_GOLDEN]) {
            s.parse::<toml::Table>().expect("valid TOML");
        }
    }

    #[test]
    fn every_formula_file_is_embedded() {
        let on_disk = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/data/formulas"))
            .unwrap()
            .count();
        assert_eq!(
            on_disk,
            FORMULAS.len(),
            "add the new data/formulas file to src/data.rs"
        );
    }
}
