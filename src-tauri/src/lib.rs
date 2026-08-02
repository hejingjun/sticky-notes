mod commands;
mod db;
mod shortcuts;
mod sync;
mod sync_cmd;
mod settings_cmd;
mod window;
mod win32;
mod tray_icon;

use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[tauri::command]
async fn sync_notes(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let engine = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        sync_cmd::SyncEngine::from_config(&mgr)?
    };
    let (_merged, has_conflict) = engine.sync(&state).await?;
    let _ = app.emit("notes-reloaded", ());
    if has_conflict { Ok("同步完成 (有冲突，已保留本地版本)".into()) } else { Ok("同步完成".into()) }
}

#[tauri::command]
async fn sync_push(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let engine = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        sync_cmd::SyncEngine::from_config(&mgr)?
    };
    let msg = engine.push(&state).await?;
    let _ = app.emit("notes-reloaded", ());
    Ok(msg)
}

#[tauri::command]
async fn sync_pull(
    app: tauri::AppHandle,
    sm: tauri::State<'_, Mutex<shortcuts::SettingsManager>>,
    state: tauri::State<'_, commands::DbState>,
) -> Result<String, String> {
    let engine = {
        let mgr = sm.lock().map_err(|e| e.to_string())?;
        sync_cmd::SyncEngine::from_config(&mgr)?
    };
    let msg = engine.pull(&state).await?;
    let _ = app.emit("notes-reloaded", ());
    Ok(msg)
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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION
                    | tauri_plugin_window_state::StateFlags::SIZE)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, show and focus the existing window
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(db_state)
        .manage(settings_mgr)
        .invoke_handler(tauri::generate_handler![
            // CRUD + 导出 + 提醒
            commands::list_notes,
            commands::save_note,
            commands::delete_note,
            commands::purge_old,
            commands::check_reminders,
            commands::export_notes,
            commands::get_conflict,
            commands::resolve_conflict,
            // 窗口状态
            window::toggle_penetrate,
            window::get_penetrate,
            window::toggle_ontop,
            window::get_ontop,
            window::set_opacity,
            window::start_drag,
            window::exit_app,
            window::hide_to_tray,
            // 设置
            settings_cmd::get_settings,
            settings_cmd::set_shortcut,
            settings_cmd::toggle_auto_purge,
            settings_cmd::set_theme,
            settings_cmd::save_webdav,
            settings_cmd::is_autostart,
            settings_cmd::toggle_autostart,
            // 同步
            sync_notes,
            sync_push,
            sync_pull,
        ])
        .setup(move |app| {
            let w = app.get_webview_window("main").unwrap();

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
                    let v = !window::PEN.load(std::sync::atomic::Ordering::SeqCst);
                    window::PEN.store(v, std::sync::atomic::Ordering::SeqCst);
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

                let saved_opacity = {
                    let mgr = app.state::<Mutex<shortcuts::SettingsManager>>();
                    mgr.lock().map(|m| m.get_config().opacity).unwrap_or(1.0)
                };
                log::info!("[sticky-notes] saved opacity: {saved_opacity}");
                unsafe { win32::apply_opacity(h, saved_opacity); }

                let embedded = unsafe { win32::embed_desktop(h) };
                log::info!("[sticky-notes] embed_desktop result: {embedded}");

                unsafe { win32::subclass::install_guard(h, &window::PEN); }

                let _ = w.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("launch failed");
}
