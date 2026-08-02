use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

pub(crate) static PEN: AtomicBool = AtomicBool::new(false);
pub(crate) static ONTOP: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn toggle_penetrate(w: tauri::WebviewWindow) -> Result<bool, String> {
    let v = !PEN.load(Ordering::SeqCst);
    PEN.store(v, Ordering::SeqCst);
    w.set_ignore_cursor_events(v).map_err(|e| e.to_string())?;
    Ok(v)
}

#[tauri::command]
pub fn get_penetrate() -> Result<bool, String> {
    Ok(PEN.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn toggle_ontop(w: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<bool, String> {
    let v = !ONTOP.load(Ordering::SeqCst);
    ONTOP.store(v, Ordering::SeqCst);
    #[cfg(target_os = "windows")]
    if let Ok(h) = w.hwnd() {
        let h = h.0;
        unsafe {
            if v {
                crate::win32::unembed_desktop(h);
            } else {
                crate::win32::embed_desktop(h);
            }
        }
    }
    w.set_always_on_top(v).map_err(|e| e.to_string())?;
    let _ = app.emit("ontop-changed", v);
    Ok(v)
}

#[tauri::command]
pub fn get_ontop() -> Result<bool, String> {
    Ok(ONTOP.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn set_opacity(
    w: tauri::WebviewWindow,
    sm: tauri::State<'_, std::sync::Mutex<crate::shortcuts::SettingsManager>>,
    value: f64,
) -> Result<(), String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    mgr.update_opacity(value);
    drop(mgr);
    #[cfg(target_os = "windows")]
    if let Ok(h) = w.hwnd() {
        unsafe { crate::win32::apply_opacity(h.0, value); }
    }
    Ok(())
}

#[tauri::command]
pub fn start_drag(w: tauri::WebviewWindow) -> Result<(), String> {
    w.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exit_app(w: tauri::WebviewWindow) -> Result<(), String> {
    w.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_to_tray(w: tauri::WebviewWindow) -> Result<(), String> {
    w.hide().map_err(|e| e.to_string())
}
