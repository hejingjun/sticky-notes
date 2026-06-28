mod commands;
mod db;
mod shortcuts;
mod sync;
mod win32;
mod tray_icon;

use std::sync::{atomic::AtomicBool, atomic::Ordering, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

static PEN: AtomicBool = AtomicBool::new(false);
static ONTOP: AtomicBool = AtomicBool::new(false);

fn set_penetrate(w: &tauri::WebviewWindow, v: bool) -> Result<bool, String> {
    PEN.store(v, Ordering::SeqCst);
    w.set_ignore_cursor_events(v).map_err(|e| e.to_string())?;
    Ok(v)
}

#[tauri::command]
fn toggle_penetrate(w: tauri::WebviewWindow) -> Result<bool, String> {
    let v = !PEN.load(Ordering::SeqCst);
    set_penetrate(&w, v)
}

#[tauri::command]
fn get_penetrate() -> Result<bool, String> {
    Ok(PEN.load(Ordering::SeqCst))
}

#[tauri::command]
fn toggle_ontop(w: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<bool, String> {
    let v = !ONTOP.load(Ordering::SeqCst);
    ONTOP.store(v, Ordering::SeqCst);
    // Embed/unembed first — re-parenting resets Z-order in Windows
    #[cfg(target_os = "windows")]
    if let Ok(h) = w.hwnd() {
        let h = h.0;
        unsafe {
            if v {
                win32::unembed_desktop(h);
            } else {
                win32::embed_desktop(h);
            }
        }
    }
    w.set_always_on_top(v).map_err(|e| e.to_string())?;
    let _ = app.emit("ontop-changed", v);
    Ok(v)
}

#[tauri::command]
fn get_ontop() -> Result<bool, String> {
    Ok(ONTOP.load(Ordering::SeqCst))
}

#[tauri::command]
fn get_settings(sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>) -> Result<shortcuts::SettingsConfig, String> {
    let mgr = sm.lock().map_err(|e| e.to_string())?;
    Ok(mgr.get_config().clone())
}

#[tauri::command]
fn set_shortcut(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
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
            let v = !PEN.load(Ordering::SeqCst);
            PEN.store(v, Ordering::SeqCst);
            let _ = wh_clone.set_ignore_cursor_events(v);
            let _ = app.emit("penetrate-changed", v);
        }
    }).map_err(|e| format!("注册快捷键失败: {e}"))?;
    log::info!("shortcut updated to: {accelerator}");
    Ok(())
}

#[tauri::command]
fn toggle_auto_purge(
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
) -> Result<bool, String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    let v = !mgr.get_config().auto_purge;
    mgr.update_auto_purge(v);
    Ok(v)
}

#[tauri::command]
fn set_opacity(
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    value: f64,
) -> Result<(), String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    mgr.update_opacity(value);
    Ok(())
}

#[tauri::command]
fn save_webdav(
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
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
fn set_theme(
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    theme: String,
) -> Result<(), String> {
    let mut mgr = sm.lock().map_err(|e| e.to_string())?;
    mgr.update_theme(&theme);
    log::info!("theme set to: {theme}");
    Ok(())
}

#[tauri::command]
async fn sync_notes(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let (base_url, user, password) = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        let cfg = mgr.get_config();
        if cfg.webdav_url.is_empty() {
            return Err("请先配置 WebDAV 地址".into());
        }
        (cfg.webdav_url.clone(), cfg.webdav_user.clone(), cfg.webdav_password.clone())
    };

    // Build the full file URL: <base_url>/sticky-notes/notes.json
    let base = base_url.trim_end_matches('/');
    let remote_dir = format!("{}/sticky-notes", base);
    let file_url = format!("{}/notes.json", remote_dir);
    let dir_url = remote_dir;

    let etag_path = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("sticky-notes")
        .join(".sync_etag");
    let _local_etag = std::fs::read_to_string(&etag_path).unwrap_or_default();

    // 0. Ensure remote directory exists (MKCOL — 坚果云 etc.)
    sync::ensure_dir(&dir_url, &user, &password).await?;

    // 1. Fetch remote
    let (remote_payload, remote_etag) = sync::fetch(&file_url, &user, &password).await?;

    // 2. Get local notes (drop lock before await)
    let local_notes = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        db::list_notes(&conn).map_err(|e| e.to_string())
    }?;

    // 3. Merge (entity LWW)
    let (merged, has_conflict) = sync::merge(local_notes.clone(), remote_payload.notes);

    // 4. Write merged back to local DB
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        for n in &merged {
            db::upsert_note(&conn, n).map_err(|e| e.to_string())?;
        }
    }

    // 5. Push merged to remote
    let push_etag = sync::push(&file_url, &user, &password, &merged, &remote_etag).await?;

    // 6. Save new etag
    let save_etag = if push_etag.is_empty() { &remote_etag } else { &push_etag };
    if let Err(e) = std::fs::write(&etag_path, save_etag) {
        log::error!("[sticky-notes] 同步 etag 写入失败: {e}");
    }

    // 7. Reload on frontend
    let _ = app.emit("notes-reloaded", ());

    if has_conflict {
        Ok("同步完成 (有冲突，已保留本地版本)".into())
    } else {
        Ok("同步完成".into())
    }
}

#[tauri::command]
async fn sync_push(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let (base_url, user, password) = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        let cfg = mgr.get_config();
        if cfg.webdav_url.is_empty() {
            return Err("请先配置 WebDAV 地址".into());
        }
        (cfg.webdav_url.clone(), cfg.webdav_user.clone(), cfg.webdav_password.clone())
    };

    let base = base_url.trim_end_matches('/');
    let remote_dir = format!("{}/sticky-notes", base);
    let file_url = format!("{}/notes.json", remote_dir);

    let etag_path = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("sticky-notes")
        .join(".sync_etag");
    let local_etag = std::fs::read_to_string(&etag_path).unwrap_or_default();

    sync::ensure_dir(&remote_dir, &user, &password).await?;

    let local_notes = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        db::list_notes(&conn).map_err(|e| e.to_string())?
    };

    let push_etag = sync::push(&file_url, &user, &password, &local_notes, &local_etag).await?;

    let save_etag = if push_etag.is_empty() { local_etag } else { push_etag };
    if let Err(e) = std::fs::write(&etag_path, &save_etag) {
        log::warn!("[sticky-notes] sync_push etag write failed: {e}");
    }

    let _ = app.emit("notes-reloaded", ());
    let msg = format!("已上传 {} 条笔记", local_notes.len());
    log::info!("[sticky-notes] sync_push done: {}", msg);
    Ok(msg)
}

#[tauri::command]
async fn sync_pull(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let (base_url, user, password) = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        let cfg = mgr.get_config();
        if cfg.webdav_url.is_empty() {
            return Err("请先配置 WebDAV 地址".into());
        }
        (cfg.webdav_url.clone(), cfg.webdav_user.clone(), cfg.webdav_password.clone())
    };

    let base = base_url.trim_end_matches('/');
    let remote_dir = format!("{}/sticky-notes", base);
    let file_url = format!("{}/notes.json", remote_dir);

    sync::ensure_dir(&remote_dir, &user, &password).await?;

    let (remote_payload, remote_etag) = sync::fetch(&file_url, &user, &password).await?;

    let remote_count = remote_payload.notes.len();
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        for note in &remote_payload.notes {
            db::upsert_note(&conn, note).map_err(|e| e.to_string())?;
        }
    }

    let etag_path = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("sticky-notes")
        .join(".sync_etag");
    if let Some(parent) = etag_path.parent() { let _ = std::fs::create_dir_all(parent); }
    if let Err(e) = std::fs::write(&etag_path, &remote_etag) {
        log::warn!("[sticky-notes] sync_pull etag write failed: {e}");
    }

    let _ = app.emit("notes-reloaded", ());
    let msg = format!("已下载 {} 条笔记", remote_count);
    log::info!("[sticky-notes] sync_pull done: {}", msg);
    Ok(msg)
}

#[tauri::command]
fn start_drag(w: tauri::WebviewWindow) -> Result<(), String> {
    w.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn exit_app(w: tauri::WebviewWindow) -> Result<(), String> {
    w.close().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_to_tray(w: tauri::WebviewWindow) -> Result<(), String> {
    w.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn is_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    mgr.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_autostart(app: tauri::AppHandle) -> Result<bool, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_dir = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("sticky-notes");
    std::fs::create_dir_all(&app_dir).unwrap_or_else(|e| {
        log::error!("[sticky-notes] 创建数据目录失败: {e}");
    });
    let db_path = app_dir.join("notes.db");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let conn = db::init_db(&db_path_str).expect("DB init failed");
    let db_state = commands::DbState(Mutex::new(conn));
    let settings_mgr = Mutex::new(shortcuts::SettingsManager::new(&app_dir));
    let win_state_path = app_dir.join("window.json");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(db_state)
        .manage(settings_mgr)
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::save_note,
            commands::delete_note,
            commands::purge_old,
            commands::check_reminders,
            commands::export_notes,
            toggle_penetrate,
            get_penetrate,
            toggle_ontop,
            get_ontop,
            get_settings,
            set_shortcut,
            toggle_auto_purge,
            set_opacity,
            set_theme,
            save_webdav,
            sync_notes,
            sync_push,
            sync_pull,
            start_drag,
            exit_app,
            hide_to_tray,
            is_autostart,
            toggle_autostart,
        ])
        .setup(move |app| {
            let w = app.get_webview_window("main").unwrap();

            // Restore window position/size
            if let Ok(json) = std::fs::read_to_string(&win_state_path) {
                if let Ok(state) = serde_json::from_str::<serde_json::Value>(&json) {
                    let x = state["x"].as_i64().map(|v| v as i32);
                    let y = state["y"].as_i64().map(|v| v as i32);
                    let w_ = state["width"].as_u64().map(|v| v as u32);
                    let h = state["height"].as_u64().map(|v| v as u32);
                    if let Err(e) = w.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x.unwrap_or(100), y.unwrap_or(100)))) {
                        log::warn!("[sticky-notes] 恢复窗口位置失败: {e}");
                    }
                    if let (Some(ww), Some(wh)) = (w_, h) {
                        let _ = w.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(ww, wh)));
                    }
                }
            }

            // Save window state on resize/move
            let wsp = win_state_path.clone();
            w.on_window_event(move |ev| {
                if let tauri::WindowEvent::Moved(pos) = ev {
                    if let Ok(json) = std::fs::read_to_string(&wsp) {
                        if let Ok(mut state) = serde_json::from_str::<serde_json::Value>(&json) {
                            state["x"] = serde_json::json!(pos.x);
                            state["y"] = serde_json::json!(pos.y);
                            if let Err(e) = std::fs::write(&wsp, state.to_string()) {
                                log::error!("[sticky-notes] 窗口位置写入失败: {e}");
                            }
                        }
                    }
                }
            });
            let wsp2 = win_state_path.clone();
            let w_resized = w.clone();
            w.on_window_event(move |ev| {
                if let tauri::WindowEvent::Resized(size) = ev {
                    let pos = w_resized.inner_position().ok();
                    if let Err(e) = std::fs::write(&wsp2, serde_json::json!({
                        "x": pos.as_ref().map(|p| p.x).unwrap_or(100),
                        "y": pos.map(|p| p.y).unwrap_or(100),
                        "width": size.width, "height": size.height,
                    }).to_string()) {
                        log::error!("[sticky-notes] 窗口尺寸写入失败: {e}");
                    }
                }
            });

            // Shortcut registration
            let accel = {
                let mgr = app.state::<Mutex<shortcuts::SettingsManager>>();
                let m = mgr.lock().unwrap();
                m.get_config().penetrate.clone()
            };
            let shortcut: Shortcut = accel.parse().unwrap_or_else(|_| "Ctrl+Alt+Shift+P".parse().unwrap());
            let wh = w.clone();
            match app.global_shortcut().on_shortcut(shortcut, move |app, _sc, ev| {
                if ev.state() == ShortcutState::Pressed {
                    let v = !PEN.load(Ordering::SeqCst);
                    PEN.store(v, Ordering::SeqCst);
                    if let Err(e) = wh.set_ignore_cursor_events(v) {
                        log::error!("[sticky-notes] set_ignore_cursor_events 失败: {e}");
                    }
                    let _ = app.emit("penetrate-changed", v);
                    log::info!("[shortcut] penetrate: {v}");
                }
            }) {
                Ok(_) => log::info!("Shortcut registered: {accel}"),
                Err(e) => log::error!("Shortcut FAILED: {accel} — {e}"),
            }

            // System tray
            let icon = tray_icon::tray_icon_image();
            let _tray = tauri::tray::TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("Sticky Notes")
                .on_tray_icon_event(move |tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            if let Err(e) = w.show() {
                                log::error!("[sticky-notes] 托盘显示窗口失败: {e}");
                            }
                            let _ = w.set_focus();
                        }
                    }
                })
                .menu(&tauri::menu::Menu::with_items(app.handle(), &[
                    &tauri::menu::MenuItem::with_id(app.handle(), "show", "显示", true, None::<&str>).unwrap(),
                    &tauri::menu::MenuItem::with_id(app.handle(), "quit", "退出", true, None::<&str>).unwrap(),
                ]).unwrap())
                .on_menu_event(|app, ev| {
                    match ev.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app.handle());

            #[cfg(target_os = "windows")]
            {
                let h = w.hwnd().unwrap().0;
                unsafe { win32::apply_styles(h); win32::embed_desktop(h); win32::subclass::install_guard(h, &PEN); }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("launch failed");
}
