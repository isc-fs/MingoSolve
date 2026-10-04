//! ISC MingoSolve: Tauri command registry. Each feature lives in its own module and calls the `fsq` engine
//! crate directly (no shell-out), mirrored on the frontend by one thin `src/lib/*.ts` wrapper per module.

pub mod finder;
pub mod solve;
pub mod tools;
pub mod topics;

#[tauri::command]
fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The app's commands on any runtime (the real one, or tauri::test's mock runtime in the IPC tests).
fn with_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
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
        topics::list_topics,
        topics::script_examples,
    ])
}

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());
    with_commands(builder)
        .run(tauri::generate_context!())
        .expect("error while running ISC MingoSolve");
}

#[cfg(test)]
mod ipc_tests;
