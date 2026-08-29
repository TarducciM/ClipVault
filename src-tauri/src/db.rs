use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

/// Unpinned entries beyond this count are pruned, oldest first. Pinned entries are never pruned.
const MAX_UNPINNED_ENTRIES: i64 = 500;

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS entries (
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
    CREATE INDEX IF NOT EXISTS idx_entries_pinned_id ON entries(pinned, id);
";

pub fn open(app_data_dir: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(app_data_dir.join("clipvault.db"))?;
    conn.execute_batch(SCHEMA)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn open_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn
    }

    fn text_entry(text: &str) -> NewEntry {
        NewEntry {
            kind: "text",
            preview: text.to_string(),
            search_text: text.to_string(),
            content: Some(text.to_string()),
            image_path: None,
            file_list: None,
            thumbnail: None,
            hash: format!("hash-{text}"),
        }
    }

    #[test]
    fn insert_and_list_returns_newest_first() {
        let conn = open_in_memory();
        insert(&conn, &text_entry("first"), 1).unwrap();
        insert(&conn, &text_entry("second"), 2).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].preview, "second");
        assert_eq!(entries[1].preview, "first");
    }

    #[test]
    fn list_search_matches_search_text_case_insensitively() {
        let conn = open_in_memory();
        insert(&conn, &text_entry("Hello World"), 1).unwrap();
        insert(&conn, &text_entry("unrelated"), 2).unwrap();

        let entries = list(&conn, Some("world")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "Hello World");
    }

    #[test]
    fn list_search_escapes_like_wildcards() {
        let conn = open_in_memory();
        insert(&conn, &text_entry("100% done"), 1).unwrap();
        insert(&conn, &text_entry("100x done"), 2).unwrap();

        // A literal '%' in the query should not act as a wildcard.
        let entries = list(&conn, Some("100%")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "100% done");
    }

    #[test]
    fn pinned_entries_sort_before_unpinned_regardless_of_age() {
        let conn = open_in_memory();
        let older_id = insert(&conn, &text_entry("older"), 1).unwrap();
        insert(&conn, &text_entry("newer"), 2).unwrap();
        set_pinned(&conn, older_id, true).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries[0].preview, "older");
        assert!(entries[0].pinned);
        assert_eq!(entries[1].preview, "newer");
    }

    #[test]
    fn delete_removes_only_the_targeted_entry() {
        let conn = open_in_memory();
        let keep = insert(&conn, &text_entry("keep"), 1).unwrap();
        let remove = insert(&conn, &text_entry("remove"), 2).unwrap();

        delete(&conn, remove).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, keep);
    }

    #[test]
    fn clear_unpinned_keeps_pinned_entries() {
        let conn = open_in_memory();
        let pinned = insert(&conn, &text_entry("pinned"), 1).unwrap();
        insert(&conn, &text_entry("not pinned"), 2).unwrap();
        set_pinned(&conn, pinned, true).unwrap();

        clear_unpinned(&conn).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, pinned);
    }

    #[test]
    fn get_full_returns_none_for_missing_id() {
        let conn = open_in_memory();
        assert!(get_full(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn get_full_round_trips_stored_fields() {
        let conn = open_in_memory();
        let id = insert(&conn, &text_entry("round trip"), 1).unwrap();

        let full = get_full(&conn, id).unwrap().expect("entry should exist");
        assert_eq!(full.kind, "text");
        assert_eq!(full.content.as_deref(), Some("round trip"));
    }

    #[test]
    fn last_hash_reflects_most_recent_insert() {
        let conn = open_in_memory();
        assert!(last_hash(&conn).is_none());

        insert(&conn, &text_entry("first"), 1).unwrap();
        assert_eq!(last_hash(&conn).as_deref(), Some("hash-first"));

        insert(&conn, &text_entry("second"), 2).unwrap();
        assert_eq!(last_hash(&conn).as_deref(), Some("hash-second"));
    }

    #[test]
    fn prune_keeps_all_pinned_but_caps_unpinned() {
        let conn = open_in_memory();

        for i in 0..(MAX_UNPINNED_ENTRIES + 5) {
            insert(&conn, &text_entry(&format!("entry-{i}")), i).unwrap();
        }

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, MAX_UNPINNED_ENTRIES);

        // The oldest entries should have been pruned, newest kept.
        let entries = list(&conn, None).unwrap();
        assert_eq!(
            entries[0].preview,
            format!("entry-{}", MAX_UNPINNED_ENTRIES + 4)
        );
    }
}
