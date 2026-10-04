// Tauri build script: generates the context `tauri::generate_context!()` embeds (config, assets, icons).
//
// On Windows the Common Controls v6 manifest is linked into every target instead of only the app binary, so the
// test binaries (IPC tests) can start; without it they exit with STATUS_ENTRYPOINT_NOT_FOUND.

fn main() {
    let mut attributes = tauri_build::Attributes::new();
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows-msvc") {
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let manifest =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
    tauri_build::try_build(attributes).expect("tauri build script");
}
