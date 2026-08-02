use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub order: String,
    pub completed: bool,
    pub pinned: bool,
    pub color: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub conflict_id: Option<String>,
    pub due_date: Option<i64>,
    pub remind_at: Option<i64>,
    pub completed_at: Option<i64>,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            parent_id: None,
            order: String::new(),
            completed: false,
            pinned: false,
            color: "#333333".into(),
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
            conflict_id: None,
            due_date: None,
            remind_at: None,
            completed_at: None,
        }
    }
}

pub fn init_db(path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    // WAL mode: better crash safety and read concurrency (best practice for SQLite)
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    // Busy timeout: wait up to 5s if the DB is locked (e.g. by a sync operation)
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            parent_id TEXT,
            [order] TEXT NOT NULL DEFAULT 'a0',
            completed INTEGER NOT NULL DEFAULT 0,
            pinned INTEGER NOT NULL DEFAULT 0,
            color TEXT NOT NULL DEFAULT '#333333',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER,
            conflict_id TEXT,
            due_date INTEGER,
            remind_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_order ON notes([order]);
        CREATE INDEX IF NOT EXISTS idx_parent ON notes(parent_id);
        CREATE INDEX IF NOT EXISTS idx_updated ON notes(updated_at);
    ")?;
    // Migration: add completed_at column if missing (safe to run on existing DBs)
    if let Err(e) = conn.execute("ALTER TABLE notes ADD COLUMN completed_at INTEGER", []) {
        if !e.to_string().contains("duplicate column") {
            log::warn!("ALTER TABLE completed_at failed: {e}");
        }
    }
    // Conflict table: stores remote version when sync detects a conflict
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS conflicts (
            note_id TEXT PRIMARY KEY,
            remote_json TEXT NOT NULL
        );
    ")?;
    Ok(conn)
}

pub fn list_notes(conn: &Connection) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, parent_id, [order], completed, pinned, color, \
         created_at, updated_at, deleted_at, conflict_id, due_date, remind_at, completed_at \
         FROM notes WHERE deleted_at IS NULL ORDER BY [order]"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Note {
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
    })?;
    Ok(rows.filter_map(|r| match r {
        Ok(note) => Some(note),
        Err(e) => { log::warn!("list_notes: 行解析失败: {e}"); None }
    }).collect())
}

pub fn upsert_note(conn: &Connection, note: &Note) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO notes (id,title,parent_id,[order],completed,pinned,color,created_at,updated_at,deleted_at,conflict_id,due_date,remind_at,completed_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(id) DO UPDATE SET
         title=excluded.title, parent_id=excluded.parent_id,
         [order]=excluded.[order], completed=excluded.completed, pinned=excluded.pinned,
         color=excluded.color, updated_at=excluded.updated_at,
         deleted_at=excluded.deleted_at, conflict_id=excluded.conflict_id,
         due_date=excluded.due_date, remind_at=excluded.remind_at,
         completed_at=excluded.completed_at",
        params![
            note.id, note.title, note.parent_id, note.order,
            note.completed as i32, note.pinned as i32, note.color,
            note.created_at, note.updated_at, note.deleted_at,
            note.conflict_id, note.due_date, note.remind_at, note.completed_at,
        ],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: &str, now: i64) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute("UPDATE notes SET deleted_at=?1, updated_at=?1 WHERE id=?2", params![now, id])?;
    Ok(())
}

pub fn purge_old(conn: &Connection, cutoff: i64) -> Result<usize, Box<dyn std::error::Error>> {
    let count = conn.execute(
        "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        params![cutoff],
    )?;
    Ok(count)
}

/// Store a conflicting remote version for a note.
pub fn save_conflict(conn: &Connection, note_id: &str, remote_note: &Note) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(remote_note)?;
    conn.execute(
        "INSERT INTO conflicts (note_id, remote_json) VALUES (?1, ?2) ON CONFLICT(note_id) DO UPDATE SET remote_json=excluded.remote_json",
        params![note_id, json],
    )?;
    // Mark the note as having a conflict
    conn.execute("UPDATE notes SET conflict_id = ?1 WHERE id = ?2", params![note_id, note_id])?;
    Ok(())
}

/// Get the conflicting remote version for a note.
pub fn get_conflict(conn: &Connection, note_id: &str) -> Result<Option<Note>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT remote_json FROM conflicts WHERE note_id = ?1")?;
    let mut rows = stmt.query_map(params![note_id], |row| {
        let json: String = row.get("remote_json")?;
        Ok(json)
    })?;
    match rows.next() {
        Some(Ok(json)) => Ok(Some(serde_json::from_str(&json)?)),
        _ => Ok(None),
    }
}

/// Resolve a conflict: keep local (just clear conflict) or use remote (overwrite note).
pub fn resolve_conflict(conn: &Connection, note_id: &str, use_remote: bool) -> Result<(), Box<dyn std::error::Error>> {
    if use_remote {
        if let Some(remote) = get_conflict(conn, note_id)? {
            let mut note = remote;
            note.conflict_id = None;
            upsert_note(conn, &note)?;
        }
    } else {
        conn.execute("UPDATE notes SET conflict_id = NULL WHERE id = ?1", params![note_id])?;
    }
    conn.execute("DELETE FROM conflicts WHERE note_id = ?1", params![note_id])?;
    Ok(())
}

pub fn check_reminders(conn: &Connection, now: i64, window_start: i64) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, parent_id, [order], completed, pinned, color, \
         created_at, updated_at, deleted_at, conflict_id, due_date, remind_at, completed_at \
         FROM notes WHERE deleted_at IS NULL AND ( \
           (remind_at <= ?1 AND remind_at > ?2) OR \
           (due_date <= ?1 AND completed = 0) \
         ) ORDER BY [order]",
    )?;
    let rows = stmt.query_map(params![now, window_start], |row| {
        Ok(Note {
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
    })?;
    Ok(rows.filter_map(|r| match r {
        Ok(note) => Some(note),
        Err(e) => { log::warn!("check_reminders: 行解析失败: {e}"); None }
    }).collect())
}