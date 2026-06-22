use crate::db;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);

#[tauri::command]
pub fn list_notes(state: State<DbState>) -> Result<Vec<db::Note>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::list_notes(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_note(state: State<DbState>, note: db::Note) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::upsert_note(&conn, &note).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    db::soft_delete(&conn, &id, now).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn purge_old(state: State<DbState>) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        - 30 * 24 * 60 * 60 * 1000;
    db::purge_old(&conn, cutoff).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_reminders(state: State<DbState>) -> Result<Vec<db::Note>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let notes = db::list_notes(&conn).map_err(|e| e.to_string())?;
    Ok(notes
        .into_iter()
        .filter(|n| {
            // remind_at within the last 60 seconds (first time we catch it)
            // or due_date is past and not completed
            let reminded = n.remind_at.map(|r| r <= now && r > now - 60_000).unwrap_or(false);
            let overdue = n.due_date.map(|d| d <= now).unwrap_or(false) && !n.completed;
            reminded || overdue
        })
        .collect())
}

#[tauri::command]
pub fn write_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, &contents).map_err(|e| format!("写入失败: {e}"))
}

#[tauri::command]
pub fn export_notes(state: State<DbState>, format: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let notes = db::list_notes(&conn).map_err(|e| e.to_string())?;
    match format.as_str() {
        "csv" => {
            let mut csv = String::from("\u{feff}id,标题,内容,完成,颜色,创建时间,更新时间,截止日期\n");
            for n in &notes {
                let title = n.title.replace('"', "\"\"");
                let content = n.content.replace('"', "\"\"");
                let due = n.due_date.map(|d| d.to_string()).unwrap_or_default();
                csv.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",{},\"{}\",{},{},{}\n",
                    n.id, title, content, n.completed as i32, n.color, n.created_at, n.updated_at, due,
                ));
            }
            Ok(csv)
        }
        _ => Err(format!("不支持的导出格式: {format}")),
    }
}