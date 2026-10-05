//! ISC MingoSolve: Tauri command registry. Each feature lives in its own module and calls the `fsq` engine
//! crate directly (no shell-out), mirrored on the frontend by one thin `src/lib/*.ts` wrapper per module.

use std::panic::{catch_unwind, AssertUnwindSafe};

pub mod finder;
pub mod rulebook;
pub mod rules_text;
pub mod solve;
pub mod tools;
pub mod topics;

/// Run `f`, turning a panic into the `Err` the UI already shows.
pub fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|p| {
        let why = p
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        Err(format!("internal error: {why}"))
    })
}

/// Run engine work off the main thread (a blocking-pool thread), so a slow or panicking input never freezes the
/// window or takes the app down.
pub async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || guarded(f))
        .await
        .map_err(|e| format!("internal error: {e}"))?
}

#[tauri::command]
fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Only the IPC tests register this: it panics on purpose, through the same `blocking` path as every command.
#[cfg(test)]
#[tauri::command]
async fn panic_probe(message: String) -> Result<String, String> {
    blocking(move || panic!("{message}")).await
}

macro_rules! handler {
    ($($extra:path),*) => {
        tauri::generate_handler![
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
        rulebook::load_rulebook,
        rulebook::rulebook_status,
        rulebook::search_rules,
        rulebook::remove_rulebook,
        $($extra),*
        ]
    };
}

/// The app's commands on any runtime (the real one, or tauri::test's mock runtime in the IPC tests).
fn with_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    #[cfg(test)]
    return builder.invoke_handler(handler!(panic_probe));
    #[cfg(not(test))]
    builder.invoke_handler(handler!())
}

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init());
    with_commands(builder.setup(|app| {
        use tauri::Manager;
        app.manage(rulebook::RulebookDir(
            app.path().app_data_dir()?.join("rulebooks"),
        ));
        Ok(())
    }))
    .run(tauri::generate_context!())
    .expect("error while running ISC MingoSolve");
}

#[cfg(test)]
mod ipc_tests;
