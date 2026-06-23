/// System tray icon loaded from a PNG file embedded at compile time.
/// The icon is a 32x32 PNG, loaded via `include_bytes!` and parsed by Tauri's image decoder.

/// Returns a 32x32 tray icon image. The PNG is embedded into the binary at build time,
/// so no external file is needed at runtime.
pub fn tray_icon_image() -> tauri::image::Image<'static> {
    let bytes: &[u8] = include_bytes!("../icons/icon32.png");
    tauri::image::Image::from_bytes(bytes).expect("icon32.png is embedded at build time; cannot fail")
}
