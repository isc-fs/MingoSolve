//! ISC MingoSolve engine. Formulas, rules and units are data (`data/`), compiled into the binary so the app
//! ships without a data folder. This module only exposes the raw embedded files; parsing lives in `registry`.

pub mod answer;
pub mod cli;
pub mod data;
pub mod engine;
pub mod expr;
pub mod finder;
pub mod format;
pub mod latex;
pub mod registry;
pub mod solve;
pub mod tool_params;
pub mod tools;
pub mod topics;
pub mod units;
