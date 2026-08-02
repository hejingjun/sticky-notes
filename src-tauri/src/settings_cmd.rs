use crate::shortcuts::{SettingsConfig, SettingsManager};
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[tauri::command]
pub fn get_settings(sm: tauri::State<'_, Mutex<SettingsManager>>) -> Result<SettingsConfig, String> {
    let mgr = sm.lock().map_err(|e| e.to_string())?;
    Ok(mgr.get_config().clone())
}

#[tauri::command]
pub fn set_shortcut(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<SettingsManager>>,
    accelerator: String,
) -> Result<(), String> {
    let new_sc: Shortcut = accelerator.parse().map_err(|e| format!("无效快捷键: {e}"))?;
    let old_accel = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        mgr.get_config().penetrate.clone()
    };
    let old_sc: Option<Shortcut> = old_accel.parse().ok();
    if let Some(ref sc) = old_sc {
        let _ = app.global_shortcut().unregister(sc.clone());
    }
    {
        let mut mgr = sm.lock().map_err(|e| e.to_string())?;
        mgr.update_penetrate(&accelerator);
    }
    let wh = app.get_webview_window("main").ok_or("no main window")?;
    let wh_clone = wh.clone();
    app.global_shortcut().on_shortcut(new_sc, move |app, _sc, ev| {
        if ev.state() == ShortcutState::Pressed {
            let v = !super::window::PEN.load(std::sync::atomic::Ordering::SeqCst);
            super::window::PEN.store(v, std::sync::atomic::Ordering::SeqCst);
            let _ = wh_clone.set_ignore_cursor_events(v);
            let _ = app.emit("penetrate-changed", v);
        }
    }).map_err(|e| format!("注册快捷键失败: {e}"))?;
    log::info!("shortcut updated to: {accelerator}");
    Ok(())
}

#[tauri::command]
pub fn toggle_auto_purge(sm: tauri::State<'_, Mutex<SettingsManager>>) -> Result<bool, String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    let v = !mgr.get_config().auto_purge;
    mgr.update_auto_purge(v);
    Ok(v)
}

#[tauri::command]
pub fn save_webdav(
    sm: tauri::State<'_, Mutex<SettingsManager>>,
    url: String,
    user: String,
    password: String,
) -> Result<(), String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    mgr.update_webdav(&url, &user, &password);
    log::info!("webdav config saved");
    Ok(())
}

#[tauri::command]
pub fn set_theme(sm: tauri::State<'_, Mutex<SettingsManager>>, theme: String) -> Result<(), String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    mgr.update_theme(&theme);
    log::info!("theme set to: {theme}");
    Ok(())
}

#[tauri::command]
pub fn is_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    mgr.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    if mgr.is_enabled().unwrap_or(false) {
        mgr.disable().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        mgr.enable().map_err(|e| e.to_string())?;
        Ok(true)
    }
}
