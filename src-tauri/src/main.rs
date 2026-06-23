//! Desktop Sticky Notes — GUI application, no console window.
//! This attribute tells Windows to treat this as a GUI (not console) binary,
//! so it won't create a cmd.exe background window on launch.
//! Only applies in release builds; debug builds keep the console for logging.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize logging — writes to %APPDATA%/sticky-notes/app.log
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("sticky-notes");
    let _ = std::fs::create_dir_all(&data_dir);
    let log_path = data_dir.join("app.log");

    // In release mode, log to file only (no console).
    // In debug mode, also log to stderr.
    let _ = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(
            std::fs::File::create(&log_path)
                .unwrap_or_else(|_| std::fs::File::create("sticky-notes.log").unwrap()),
        )))
        .try_init();

    log::info!("=== Sticky Notes v{} starting ===", env!("CARGO_PKG_VERSION"));
    sticky_notes_lib::run();
}