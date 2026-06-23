use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
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
}

pub fn init_db(path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
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
    Ok(conn)
}

pub fn list_notes(conn: &Connection) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, parent_id, [order], completed, pinned, color, \
         created_at, updated_at, deleted_at, conflict_id, due_date, remind_at \
         FROM notes WHERE deleted_at IS NULL ORDER BY [order]"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Note {
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
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn upsert_note(conn: &Connection, note: &Note) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO notes (id,title,content,parent_id,[order],completed,pinned,color,created_at,updated_at,deleted_at,conflict_id,due_date,remind_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(id) DO UPDATE SET
         title=excluded.title, content=excluded.content, parent_id=excluded.parent_id,
         [order]=excluded.[order], completed=excluded.completed, pinned=excluded.pinned,
         color=excluded.color, updated_at=excluded.updated_at,
         deleted_at=excluded.deleted_at, conflict_id=excluded.conflict_id,
         due_date=excluded.due_date, remind_at=excluded.remind_at",
        params![
            note.id, note.title, note.content, note.parent_id, note.order,
            note.completed as i32, note.pinned as i32, note.color,
            note.created_at, note.updated_at, note.deleted_at,
            note.conflict_id, note.due_date, note.remind_at,
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