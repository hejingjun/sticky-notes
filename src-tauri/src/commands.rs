use crate::db;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

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
    db::soft_delete(&conn, &id, now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn purge_old(state: State<DbState>) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let cutoff = now_ms() - 30 * 24 * 60 * 60 * 1000;
    db::purge_old(&conn, cutoff).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_reminders(state: State<DbState>) -> Result<Vec<db::Note>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let window_start = now - 60_000;
    // Filter in SQL: remind_at within the last 60s, or due_date is past and not completed
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, parent_id, [order], completed, pinned, color, \
             created_at, updated_at, deleted_at, conflict_id, due_date, remind_at \
             FROM notes WHERE deleted_at IS NULL AND ( \
               (remind_at <= ?1 AND remind_at > ?2) OR \
               (due_date <= ?1 AND completed = 0) \
             ) ORDER BY [order]",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![now, window_start], |row| {
            Ok(db::Note {
                id: row.get("id")?,
                title: row.get("title")?,
                content: row.get("content")?,
                parent_id: row.get("parent_id")?,
                order: row.get("order")?,
                completed: row.get("completed")?,
                pinned: row.get("pinned")?,
                color: row.get("color")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                deleted_at: row.get("deleted_at")?,
                conflict_id: row.get("conflict_id")?,
                due_date: row.get("due_date")?,
                remind_at: row.get("remind_at")?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn export_notes(state: State<DbState>, format: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let notes = db::list_notes(&conn).map_err(|e| e.to_string())?;
    match format.as_str() {
        "csv" => {
            let mut csv = String::from("\u{feff}id,标题,内容,完成,颜色,创建时间,更新时间,截止日期\n");
            for n in &notes {
                let title = csv_escape(&n.title);
                let content = csv_escape(&n.content);
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

/// Escape a CSV field value: double-quote internal quotes, and prefix
/// with a single quote if the value starts with a formula trigger character
/// (=, +, -, @) to prevent CSV injection (CWE-1236).
fn csv_escape(s: &str) -> String {
    let escaped = s.replace('"', "\"\"");
    if escaped.starts_with(['=', '+', '-', '@']) {
        format!("'{}", escaped)
    } else {
        escaped
    }
}