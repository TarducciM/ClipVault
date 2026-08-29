use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

/// Unpinned entries beyond this count are pruned, oldest first. Pinned entries are never pruned.
const MAX_UNPINNED_ENTRIES: i64 = 500;

pub fn open(app_data_dir: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(app_data_dir.join("clipvault.db"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            preview TEXT NOT NULL,
            search_text TEXT NOT NULL,
            content TEXT,
            image_path TEXT,
            file_list TEXT,
            thumbnail BLOB,
            hash TEXT NOT NULL,
            pinned INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_entries_pinned_id ON entries(pinned, id);",
    )?;
    Ok(conn)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub kind: String,
    pub preview: String,
    pub pinned: bool,
    pub created_at: i64,
    pub thumbnail: Option<String>,
}

/// Fields needed to persist a freshly captured clipboard entry.
pub struct NewEntry {
    pub kind: &'static str,
    pub preview: String,
    pub search_text: String,
    pub content: Option<String>,
    pub image_path: Option<String>,
    pub file_list: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub hash: String,
}

pub fn last_hash(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT hash FROM entries ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

pub fn insert(conn: &Connection, entry: &NewEntry, created_at: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO entries (kind, preview, search_text, content, image_path, file_list, thumbnail, hash, pinned, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)",
        params![
            entry.kind,
            entry.preview,
            entry.search_text,
            entry.content,
            entry.image_path,
            entry.file_list,
            entry.thumbnail,
            entry.hash,
            created_at,
        ],
    )?;
    let id = conn.last_insert_rowid();
    prune(conn)?;
    Ok(id)
}

fn prune(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM entries WHERE pinned = 0 AND id NOT IN (
            SELECT id FROM entries WHERE pinned = 0 ORDER BY id DESC LIMIT ?1
        )",
        params![MAX_UNPINNED_ENTRIES],
    )?;
    Ok(())
}

pub fn list(conn: &Connection, query: Option<&str>) -> rusqlite::Result<Vec<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, preview, pinned, created_at, thumbnail FROM entries
         WHERE (?1 IS NULL OR search_text LIKE ?1 ESCAPE '\\')
         ORDER BY pinned DESC, id DESC
         LIMIT 300",
    )?;

    let like_pattern = query.map(escape_like);

    let rows = stmt.query_map(params![like_pattern], |row| {
        let thumbnail: Option<Vec<u8>> = row.get(5)?;
        Ok(HistoryEntry {
            id: row.get(0)?,
            kind: row.get(1)?,
            preview: row.get(2)?,
            pinned: row.get::<_, i64>(3)? != 0,
            created_at: row.get(4)?,
            thumbnail: thumbnail.map(|bytes| {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(bytes)
            }),
        })
    })?;

    rows.collect()
}

fn escape_like(query: &str) -> String {
    let mut escaped = String::with_capacity(query.len() + 2);
    escaped.push('%');
    for ch in query.chars() {
        if ch == '%' || ch == '_' || ch == '\\' {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped.push('%');
    escaped
}

pub struct FullEntry {
    pub kind: String,
    pub content: Option<String>,
    pub image_path: Option<String>,
    pub file_list: Option<String>,
}

pub fn get_full(conn: &Connection, id: i64) -> rusqlite::Result<Option<FullEntry>> {
    conn.query_row(
        "SELECT kind, content, image_path, file_list FROM entries WHERE id = ?1",
        params![id],
        |row| {
            Ok(FullEntry {
                kind: row.get(0)?,
                content: row.get(1)?,
                image_path: row.get(2)?,
                file_list: row.get(3)?,
            })
        },
    )
    .optional()
}

pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE entries SET pinned = ?1 WHERE id = ?2",
        params![pinned as i64, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_unpinned(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM entries WHERE pinned = 0", [])?;
    Ok(())
}
