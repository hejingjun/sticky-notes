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

/// Convert millisecond timestamp to ISO-like datetime string (UTC).
fn ts_to_iso(ms: i64) -> String {
    let secs = (ms / 1000) as u64;
    let s = (secs % 60) as u8;
    let m = ((secs / 60) % 60) as u8;
    let h = ((secs / 3600) % 24) as u8;
    let mut days = (secs / 86400) as i64;
    let mut y = 1970i32;
    loop {
        let dy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if days < dy { break; }
        days -= dy;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md: [i64; 13] = if leap {
        [0,31,60,91,121,152,182,213,244,274,305,335,366]
    } else {
        [0,31,59,90,120,151,181,212,243,273,304,334,365]
    };
    let mut mo = 12usize;
    for i in 1..12 {
        if days < md[i] { mo = i; break; }
    }
    let d = (days - md[mo - 1] + 1) as u8;
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
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
            "SELECT id, title, parent_id, [order], completed, pinned, color, \
             created_at, updated_at, deleted_at, conflict_id, due_date, remind_at, completed_at \
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
                completed_at: row.get("completed_at")?,
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
            let mut csv = String::from(
                "\u{feff}id,标题,状态,类型,父任务ID,颜色,创建时间,完成时间,更新时间,截止日期,耗时(分钟)\n",
            );
            for n in &notes {
                let title = csv_escape(&n.title);
                let status = if n.completed { "已完成" } else { "未完成" };
                let kind = if n.parent_id.is_some() { "子任务" } else { "主任务" };
                let parent = n.parent_id.as_deref().unwrap_or("");
                let created = ts_to_iso(n.created_at);
                let completed = n.completed_at.map(|t| ts_to_iso(t)).unwrap_or_default();
                let updated = ts_to_iso(n.updated_at);
                let due = n.due_date.map(|t| ts_to_iso(t)).unwrap_or_default();
                let duration = if let Some(ca) = n.completed_at {
                    let mins = (ca - n.created_at) / 60000;
                    if mins >= 0 { mins.to_string() } else { String::new() }
                } else {
                    String::new()
                };
                csv.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
                    n.id, title, status, kind, parent, n.color,
                    created, completed, updated, due, duration,
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