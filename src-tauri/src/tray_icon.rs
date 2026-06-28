/// System tray icon loaded from PNG file.

pub fn tray_icon_image() -> tauri::image::Image<'static> {
    let bytes = include_bytes!("../../icon/iconsmall.png");
    let img = image::load_from_memory(bytes).expect("failed to decode tray icon PNG");
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    tauri::image::Image::new_owned(rgba.into_raw(), w, h)
}
