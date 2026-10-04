//! ISC MingoSolve: Tauri command registry. Each feature lives in its own module and calls the `fsq` engine
//! crate directly (no shell-out), mirrored on the frontend by one thin `src/lib/*.ts` wrapper per module.

mod finder;
mod rules;
mod solve;
mod tools;

#[tauri::command]
fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            engine_version,
            solve::list_formulas,
            solve::solve_formula,
            solve::chain_formulas,
            solve::calc,
            tools::list_tools,
            tools::run_tool,
            finder::find_question,
            finder::match_options,
            finder::format_answer,
            finder::list_examples,
            rules::rule_sets,
            rules::rules_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISC MingoSolve");
}
