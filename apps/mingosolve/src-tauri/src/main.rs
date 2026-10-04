// ISC MingoSolve: Tauri 2 entry point. On Windows release builds the GUI subsystem hides the console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mingosolve_lib::run();
}
