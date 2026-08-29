use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

/// Default for the "max_history" setting: unpinned entries beyond this count are pruned,
/// oldest first. Pinned entries are never pruned regardless of this setting.
const DEFAULT_MAX_HISTORY: i64 = 500;
/// Default for the "max_file_kb" setting: images larger than this are kept as a thumbnail
/// only (not saved full-size to disk), and files larger than this skip SHA1/CRC32
/// computation in the preview. 0 means unlimited.
const DEFAULT_MAX_FILE_KB: i64 = 5000;
/// Default for the "hotkey" setting: the global shortcut that opens/closes the popup.
pub const DEFAULT_HOTKEY: &str = "Ctrl+Shift+V";
/// Default for the "max_age_days" setting: unpinned entries older than this are pruned.
/// 0 means disabled (age-based pruning off, only the count-based cap applies).
const DEFAULT_MAX_AGE_DAYS: i64 = 0;
/// Default for the "language" setting.
const DEFAULT_LANGUAGE: &str = "it";

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
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
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
    prune(conn, max_history(conn))?;
    prune_by_age(conn, max_age_days(conn), created_at)?;
    Ok(id)
}

fn prune(conn: &Connection, max_unpinned: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM entries WHERE pinned = 0 AND id NOT IN (
            SELECT id FROM entries WHERE pinned = 0 ORDER BY id DESC LIMIT ?1
        )",
        params![max_unpinned],
    )?;
    Ok(())
}

/// Deletes unpinned entries older than `max_age_days`, measured relative to `now_millis`.
/// A `max_age_days` of 0 disables age-based pruning entirely.
fn prune_by_age(conn: &Connection, max_age_days: i64, now_millis: i64) -> rusqlite::Result<()> {
    if max_age_days <= 0 {
        return Ok(());
    }
    let cutoff = now_millis - max_age_days * 24 * 60 * 60 * 1000;
    conn.execute(
        "DELETE FROM entries WHERE pinned = 0 AND created_at < ?1",
        params![cutoff],
    )?;
    Ok(())
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn max_history(conn: &Connection) -> i64 {
    setting_i64(conn, "max_history", DEFAULT_MAX_HISTORY)
}

pub fn max_file_kb(conn: &Connection) -> i64 {
    setting_i64(conn, "max_file_kb", DEFAULT_MAX_FILE_KB)
}

pub fn hotkey(conn: &Connection) -> String {
    setting_str(conn, "hotkey", DEFAULT_HOTKEY)
}

pub fn max_age_days(conn: &Connection) -> i64 {
    setting_i64(conn, "max_age_days", DEFAULT_MAX_AGE_DAYS)
}

pub fn language(conn: &Connection) -> String {
    setting_str(conn, "language", DEFAULT_LANGUAGE)
}

/// Whether the first-run onboarding guide has already been shown and dismissed.
pub fn onboarding_seen(conn: &Connection) -> bool {
    setting_str(conn, "onboarding_seen", "0") == "1"
}

pub fn mark_onboarding_seen(conn: &Connection) -> rusqlite::Result<()> {
    set_setting(conn, "onboarding_seen", "1")
}

fn setting_i64(conn: &Connection, key: &str, default: i64) -> i64 {
    setting_str(conn, key, &default.to_string())
        .parse()
        .unwrap_or(default)
}

fn setting_str(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
    .unwrap_or_else(|| default.to_string())
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total: i64,
    pub text_count: i64,
    pub image_count: i64,
    pub files_count: i64,
    pub pinned_count: i64,
    pub oldest_created_at: Option<i64>,
    pub newest_created_at: Option<i64>,
    pub db_size_bytes: i64,
}

pub fn stats(conn: &Connection, db_size_bytes: i64) -> rusqlite::Result<Stats> {
    let total = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))?;
    let count_of = |kind: &str| -> rusqlite::Result<i64> {
        conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE kind = ?1",
            params![kind],
            |row| row.get(0),
        )
    };
    let pinned_count =
        conn.query_row("SELECT COUNT(*) FROM entries WHERE pinned = 1", [], |row| {
            row.get(0)
        })?;
    let oldest_created_at: Option<i64> =
        conn.query_row("SELECT MIN(created_at) FROM entries", [], |row| row.get(0))?;
    let newest_created_at: Option<i64> =
        conn.query_row("SELECT MAX(created_at) FROM entries", [], |row| row.get(0))?;

    Ok(Stats {
        total,
        text_count: count_of("text")?,
        image_count: count_of("image")?,
        files_count: count_of("files")?,
        pinned_count,
        oldest_created_at,
        newest_created_at,
        db_size_bytes,
    })
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

        for i in 0..(DEFAULT_MAX_HISTORY + 5) {
            insert(&conn, &text_entry(&format!("entry-{i}")), i).unwrap();
        }

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, DEFAULT_MAX_HISTORY);

        // The oldest entries should have been pruned, newest kept.
        let entries = list(&conn, None).unwrap();
        assert_eq!(
            entries[0].preview,
            format!("entry-{}", DEFAULT_MAX_HISTORY + 4)
        );
    }

    #[test]
    fn max_history_falls_back_to_default_when_unset() {
        let conn = open_in_memory();
        assert_eq!(max_history(&conn), DEFAULT_MAX_HISTORY);
    }

    #[test]
    fn set_setting_overrides_the_default_and_is_respected_by_prune() {
        let conn = open_in_memory();
        set_setting(&conn, "max_history", "2").unwrap();
        assert_eq!(max_history(&conn), 2);

        for i in 0..5 {
            insert(&conn, &text_entry(&format!("entry-{i}")), i).unwrap();
        }

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn set_setting_upserts_on_repeated_writes() {
        let conn = open_in_memory();
        set_setting(&conn, "max_file_kb", "1000").unwrap();
        set_setting(&conn, "max_file_kb", "2000").unwrap();
        assert_eq!(max_file_kb(&conn), 2000);
    }

    #[test]
    fn hotkey_falls_back_to_default_when_unset() {
        let conn = open_in_memory();
        assert_eq!(hotkey(&conn), DEFAULT_HOTKEY);
    }

    #[test]
    fn hotkey_reflects_override() {
        let conn = open_in_memory();
        set_setting(&conn, "hotkey", "Ctrl+Alt+V").unwrap();
        assert_eq!(hotkey(&conn), "Ctrl+Alt+V");
    }

    #[test]
    fn onboarding_seen_defaults_to_false_then_sticks_after_marking() {
        let conn = open_in_memory();
        assert!(!onboarding_seen(&conn));
        mark_onboarding_seen(&conn).unwrap();
        assert!(onboarding_seen(&conn));
    }

    #[test]
    fn language_falls_back_to_default_when_unset() {
        let conn = open_in_memory();
        assert_eq!(language(&conn), DEFAULT_LANGUAGE);
    }

    #[test]
    fn language_reflects_override() {
        let conn = open_in_memory();
        set_setting(&conn, "language", "en").unwrap();
        assert_eq!(language(&conn), "en");
    }

    #[test]
    fn prune_by_age_disabled_by_default_keeps_old_entries() {
        let conn = open_in_memory();
        let one_year_ms = 365 * 24 * 60 * 60 * 1000;
        insert(&conn, &text_entry("old"), 1_000_000).unwrap();
        insert(&conn, &text_entry("now"), 1_000_000 + one_year_ms).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn prune_by_age_removes_unpinned_entries_past_the_cutoff() {
        let conn = open_in_memory();
        set_setting(&conn, "max_age_days", "7").unwrap();
        let day_ms: i64 = 24 * 60 * 60 * 1000;
        let now = 100 * day_ms;

        insert(&conn, &text_entry("very old"), now - 10 * day_ms).unwrap();
        insert(&conn, &text_entry("recent"), now).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "recent");
    }

    #[test]
    fn prune_by_age_never_removes_pinned_entries() {
        let conn = open_in_memory();
        set_setting(&conn, "max_age_days", "7").unwrap();
        let day_ms: i64 = 24 * 60 * 60 * 1000;
        let now = 100 * day_ms;

        let old_id = insert(&conn, &text_entry("very old but pinned"), now - 10 * day_ms).unwrap();
        set_pinned(&conn, old_id, true).unwrap();

        // Trigger prune_by_age again via another insert.
        insert(&conn, &text_entry("recent"), now).unwrap();

        let entries = list(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn stats_reports_counts_by_kind_and_pinned() {
        let conn = open_in_memory();
        let id1 = insert(&conn, &text_entry("a"), 1).unwrap();
        insert(&conn, &text_entry("b"), 2).unwrap();
        set_pinned(&conn, id1, true).unwrap();

        let stats = stats(&conn, 4096).unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.text_count, 2);
        assert_eq!(stats.image_count, 0);
        assert_eq!(stats.files_count, 0);
        assert_eq!(stats.pinned_count, 1);
        assert_eq!(stats.oldest_created_at, Some(1));
        assert_eq!(stats.newest_created_at, Some(2));
        assert_eq!(stats.db_size_bytes, 4096);
    }

    #[test]
    fn stats_on_empty_db_has_no_oldest_or_newest() {
        let conn = open_in_memory();
        let stats = stats(&conn, 0).unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.oldest_created_at, None);
        assert_eq!(stats.newest_created_at, None);
    }
}
