//! Desktop Sticky Notes — GUI application, no console window.
//! This attribute tells Windows to treat this as a GUI (not console) binary,
//! so it won't create a cmd.exe background window on launch.
//! Only applies in release builds; debug builds keep the console for logging.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() { sticky_notes_lib::run(); }