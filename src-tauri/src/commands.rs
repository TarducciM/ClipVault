use crate::db::{self, HistoryEntry};
use clipboard_win::{formats, set_clipboard, Clipboard, Setter};
use rusqlite::Connection;
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_opener::OpenerExt;

pub type DbState = Arc<Mutex<Connection>>;

#[tauri::command]
pub fn get_history(
    db: State<'_, DbState>,
    query: Option<String>,
) -> Result<Vec<HistoryEntry>, String> {
    let conn = db.lock().unwrap();
    db::list(&conn, query.as_deref()).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn toggle_pin(db: State<'_, DbState>, id: i64, pinned: bool) -> Result<(), String> {
    let conn = db.lock().unwrap();
    db::set_pinned(&conn, id, pinned).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn delete_entry(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = db.lock().unwrap();
    db::delete(&conn, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn clear_history(db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.lock().unwrap();
    db::clear_unpinned(&conn).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn copy_entry_to_clipboard(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let entry = full_entry(&db, id)?;

    match entry.kind.as_str() {
        "text" => {
            let text = entry.content.unwrap_or_default();
            set_clipboard(formats::Unicode, text).map_err(|err| err.to_string())
        }
        "image" => {
            let path = entry
                .image_path
                .ok_or("immagine troppo grande, non salvata per intero")?;
            let png_bytes = std::fs::read(&path).map_err(|err| err.to_string())?;
            let img = image::load_from_memory(&png_bytes).map_err(|err| err.to_string())?;
            let mut bmp_bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut bmp_bytes),
                image::ImageFormat::Bmp,
            )
            .map_err(|err| err.to_string())?;
            set_clipboard(formats::Bitmap, bmp_bytes).map_err(|err| err.to_string())
        }
        "files" => {
            let paths = file_list_paths(&entry)?;
            let _clip = Clipboard::new_attempts(10).map_err(|err| err.to_string())?;
            formats::FileList
                .write_clipboard(&paths[..])
                .map_err(|err| err.to_string())
        }
        other => Err(format!("unknown entry kind: {other}")),
    }
}

#[tauri::command]
pub fn open_entry(app: AppHandle, db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let entry = full_entry(&db, id)?;
    let path = match entry.kind.as_str() {
        "image" => entry
            .image_path
            .ok_or("immagine troppo grande, non salvata per intero")?,
        "files" => file_list_paths(&entry)?
            .into_iter()
            .next()
            .ok_or("empty file list")?,
        other => return Err(format!("cannot open entry kind: {other}")),
    };
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn reveal_entry(app: AppHandle, db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let entry = full_entry(&db, id)?;
    let path = match entry.kind.as_str() {
        "image" => entry
            .image_path
            .ok_or("immagine troppo grande, non salvata per intero")?,
        "files" => file_list_paths(&entry)?
            .into_iter()
            .next()
            .ok_or("empty file list")?,
        other => return Err(format!("cannot reveal entry kind: {other}")),
    };
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|err| err.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipEntry {
    pub name: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub crc32: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePreview {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub too_large: bool,
    pub size_bytes: Option<u64>,
    pub modified: Option<i64>,
    pub sha1: Option<String>,
    pub crc32: Option<String>,
    pub zip_entries: Option<Vec<ZipEntry>>,
    /// Text content preview for plain-text/source files, or text extracted from a PDF
    /// (not a rendering of its visual layout). `None` for other file types, or when the
    /// file is above the "max file size" threshold.
    pub text_preview: Option<String>,
    pub text_preview_truncated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewData {
    pub kind: String,
    pub text: Option<String>,
    pub image_data_url: Option<String>,
    /// Set when the preview is a downscaled thumbnail rather than the full-resolution
    /// image (because it was too large to save in full — see the "max file size" setting).
    pub image_is_thumbnail: bool,
    pub files: Option<Vec<FilePreview>>,
}

#[tauri::command]
pub fn get_entry_preview(db: State<'_, DbState>, id: i64) -> Result<PreviewData, String> {
    let entry = full_entry(&db, id)?;

    match entry.kind.as_str() {
        "text" => Ok(PreviewData {
            kind: "text".into(),
            text: entry.content,
            image_data_url: None,
            image_is_thumbnail: false,
            files: None,
        }),
        "image" => {
            // Prefer the full-resolution image; if it wasn't saved (too large per the
            // "max file size" setting), fall back to the small thumbnail we always keep,
            // rather than failing the preview outright.
            let (bytes, image_is_thumbnail) = match &entry.image_path {
                Some(path) => (std::fs::read(path).map_err(|err| err.to_string())?, false),
                None => (
                    entry
                        .thumbnail
                        .clone()
                        .ok_or("nessuna immagine disponibile per l'anteprima")?,
                    true,
                ),
            };
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(PreviewData {
                kind: "image".into(),
                text: None,
                image_data_url: Some(format!("data:image/png;base64,{encoded}")),
                image_is_thumbnail,
                files: None,
            })
        }
        "files" => {
            let max_file_kb = {
                let conn = db.lock().unwrap();
                db::max_file_kb(&conn)
            };
            let files = file_list_paths(&entry)?
                .iter()
                .map(|p| preview_file_meta(p, max_file_kb))
                .collect();
            Ok(PreviewData {
                kind: "files".into(),
                text: None,
                image_data_url: None,
                image_is_thumbnail: false,
                files: Some(files),
            })
        }
        other => Err(format!("unknown entry kind: {other}")),
    }
}

fn preview_file_meta(path: &str, max_file_kb: i64) -> FilePreview {
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    let base = FilePreview {
        name,
        path: path.to_string(),
        exists: false,
        too_large: false,
        size_bytes: None,
        modified: None,
        sha1: None,
        crc32: None,
        zip_entries: read_zip_entries(path),
        text_preview: None,
        text_preview_truncated: false,
    };

    let Ok(mut file) = std::fs::File::open(path) else {
        return base;
    };

    let metadata = file.metadata().ok();
    let size_bytes = metadata.as_ref().map(|m| m.len());
    let modified = metadata
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);

    let too_large =
        max_file_kb > 0 && size_bytes.is_some_and(|bytes| bytes / 1024 > max_file_kb as u64);
    if too_large {
        return FilePreview {
            exists: true,
            too_large: true,
            size_bytes,
            modified,
            ..base
        };
    }

    let mut sha1_hasher = Sha1::new();
    let mut crc32_hasher = crc32fast::Hasher::new();
    let mut buffer = [0u8; 65536];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                sha1_hasher.update(&buffer[..n]);
                crc32_hasher.update(&buffer[..n]);
            }
            Err(_) => break,
        }
    }

    let (text_preview, text_preview_truncated) = text_preview_kind(path)
        .and_then(|kind| read_text_preview(path, kind))
        .map(|(text, truncated)| (Some(text), truncated))
        .unwrap_or((None, false));

    FilePreview {
        exists: true,
        too_large: false,
        size_bytes,
        modified,
        sha1: Some(
            sha1_hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        ),
        crc32: Some(format!("{:08x}", crc32_hasher.finalize())),
        text_preview,
        text_preview_truncated,
        ..base
    }
}

const TEXT_PREVIEW_EXTENSIONS: &[&str] = &[
    "txt", "md", "log", "json", "xml", "yaml", "yml", "toml", "ini", "cfg", "conf", "csv", "c",
    "h", "cpp", "cc", "cxx", "hpp", "hh", "cs", "java", "kt", "go", "rs", "rb", "py", "php", "sh",
    "bash", "ps1", "sql", "swift", "html", "htm", "css", "scss", "js", "jsx", "ts", "tsx", "vue",
    "svelte",
];

/// Caps how much text is read into a preview, so a huge log file or PDF doesn't stall the
/// UI or bloat memory — the point is a quick look, not a full viewer.
const MAX_TEXT_PREVIEW_BYTES: usize = 200_000;

enum TextPreviewKind {
    PlainText,
    Pdf,
}

fn text_preview_kind(path: &str) -> Option<TextPreviewKind> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())?;
    if ext == "pdf" {
        Some(TextPreviewKind::Pdf)
    } else if TEXT_PREVIEW_EXTENSIONS.contains(&ext.as_str()) {
        Some(TextPreviewKind::PlainText)
    } else {
        None
    }
}

/// Returns the preview text and whether it was truncated to fit `MAX_TEXT_PREVIEW_BYTES`.
/// `None` if the file can't be read as text (or, for PDFs, has no extractable text layer —
/// e.g. a scanned image with no OCR).
fn read_text_preview(path: &str, kind: TextPreviewKind) -> Option<(String, bool)> {
    match kind {
        TextPreviewKind::PlainText => {
            let bytes = std::fs::read(path).ok()?;
            let truncated = bytes.len() > MAX_TEXT_PREVIEW_BYTES;
            let slice = &bytes[..bytes.len().min(MAX_TEXT_PREVIEW_BYTES)];
            Some((String::from_utf8_lossy(slice).into_owned(), truncated))
        }
        TextPreviewKind::Pdf => {
            let text = pdf_extract::extract_text(path).ok()?;
            if text.trim().is_empty() {
                return None;
            }
            let truncated = text.len() > MAX_TEXT_PREVIEW_BYTES;
            if !truncated {
                return Some((text, false));
            }
            let mut end = MAX_TEXT_PREVIEW_BYTES;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            Some((text[..end].to_string(), true))
        }
    }
}

const MAX_ZIP_ENTRIES_LISTED: usize = 200;

/// Lists the contents of a `.zip`, `.7z`, or `.rar` archive (name, size, CRC32) without
/// extracting it. Returns `None` for other file types, or if the archive can't be read
/// (corrupt, unsupported, password-protected header, etc).
fn read_zip_entries(path: &str) -> Option<Vec<ZipEntry>> {
    let lower = path.to_lowercase();
    if lower.ends_with(".zip") {
        read_zip_archive(path)
    } else if lower.ends_with(".7z") {
        read_7z_archive(path)
    } else if lower.ends_with(".rar") {
        read_rar_archive(path)
    } else {
        None
    }
}

fn read_zip_archive(path: &str) -> Option<Vec<ZipEntry>> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let count = archive.len().min(MAX_ZIP_ENTRIES_LISTED);

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let entry = archive.by_index(i).ok()?;
        entries.push(ZipEntry {
            name: entry.name().to_string(),
            size_bytes: entry.size(),
            is_dir: entry.is_dir(),
            crc32: format!("{:08x}", entry.crc32()),
        });
    }
    Some(entries)
}

fn read_7z_archive(path: &str) -> Option<Vec<ZipEntry>> {
    let archive = sevenz_rust2::Archive::open(path).ok()?;
    let entries = archive
        .files
        .iter()
        .take(MAX_ZIP_ENTRIES_LISTED)
        .map(|entry| ZipEntry {
            name: entry.name.clone(),
            size_bytes: entry.size,
            is_dir: entry.is_directory,
            crc32: if entry.has_crc {
                format!("{:08x}", entry.crc as u32)
            } else {
                "-".to_string()
            },
        })
        .collect();
    Some(entries)
}

/// Uses the `unrar` crate, which statically compiles the official UnRAR source (freeware,
/// read-only use permitted without restriction — see THIRD-PARTY-NOTICES.md at the repo
/// root for the full license text this crate requires reproducing).
fn read_rar_archive(path: &str) -> Option<Vec<ZipEntry>> {
    let archive = unrar::Archive::new(path).open_for_listing().ok()?;
    let mut entries = Vec::new();
    for entry in archive {
        if entries.len() >= MAX_ZIP_ENTRIES_LISTED {
            break;
        }
        let entry = entry.ok()?;
        entries.push(ZipEntry {
            name: entry.filename.to_string_lossy().into_owned(),
            size_bytes: entry.unpacked_size,
            is_dir: entry.is_directory(),
            crc32: format!("{:08x}", entry.file_crc),
        });
    }
    Some(entries)
}

fn full_entry(db: &State<'_, DbState>, id: i64) -> Result<db::FullEntry, String> {
    let conn = db.lock().unwrap();
    db::get_full(&conn, id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "entry not found".to_string())
}

fn file_list_paths(entry: &db::FullEntry) -> Result<Vec<String>, String> {
    let json = entry.file_list.as_deref().ok_or("missing file list")?;
    serde_json::from_str(json).map_err(|err| err.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub max_history: i64,
    pub max_file_kb: i64,
    pub max_age_days: i64,
    pub autostart: bool,
    pub hotkey: String,
    pub language: String,
    pub version: String,
}

#[tauri::command]
pub fn get_settings(app: AppHandle, db: State<'_, DbState>) -> Result<Settings, String> {
    let conn = db.lock().unwrap();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    Ok(Settings {
        max_history: db::max_history(&conn),
        max_file_kb: db::max_file_kb(&conn),
        max_age_days: db::max_age_days(&conn),
        autostart,
        hotkey: db::hotkey(&conn),
        language: db::language(&conn),
        version: app.package_info().version.to_string(),
    })
}

#[tauri::command]
pub fn set_hotkey(app: AppHandle, db: State<'_, DbState>, hotkey: String) -> Result<(), String> {
    let old_hotkey = {
        let conn = db.lock().unwrap();
        db::hotkey(&conn)
    };

    if hotkey == old_hotkey {
        return Ok(());
    }

    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister(old_hotkey.as_str());

    match shortcuts.on_shortcut(hotkey.as_str(), crate::shortcut_handler) {
        Ok(()) => {
            let conn = db.lock().unwrap();
            db::set_setting(&conn, "hotkey", &hotkey).map_err(|err| err.to_string())
        }
        Err(err) => {
            // Keep the app usable: re-register the previous shortcut rather than leaving none.
            let _ = shortcuts.on_shortcut(old_hotkey.as_str(), crate::shortcut_handler);
            Err(err.to_string())
        }
    }
}

#[tauri::command]
pub fn should_show_onboarding(db: State<'_, DbState>) -> bool {
    let conn = db.lock().unwrap();
    !db::onboarding_seen(&conn)
}

#[tauri::command]
pub fn mark_onboarding_seen(db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.lock().unwrap();
    db::mark_onboarding_seen(&conn).map_err(|err| err.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_settings(
    app: AppHandle,
    db: State<'_, DbState>,
    max_history: i64,
    max_file_kb: i64,
    max_age_days: i64,
    autostart: bool,
    language: String,
) -> Result<(), String> {
    {
        let conn = db.lock().unwrap();
        db::set_setting(&conn, "max_history", &max_history.max(1).to_string())
            .map_err(|err| err.to_string())?;
        db::set_setting(&conn, "max_file_kb", &max_file_kb.max(0).to_string())
            .map_err(|err| err.to_string())?;
        db::set_setting(&conn, "max_age_days", &max_age_days.max(0).to_string())
            .map_err(|err| err.to_string())?;
        db::set_setting(&conn, "language", &language).map_err(|err| err.to_string())?;
    }

    let autolaunch = app.autolaunch();
    // Only touch the registry when the state actually needs to change: calling disable()
    // when autostart was never enabled fails (Windows returns "file not found" trying to
    // delete a registry value that isn't there), which was aborting the whole save.
    let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
    if autostart != currently_enabled {
        let result = if autostart {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        result.map_err(|err| err.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_stats(app: AppHandle, db: State<'_, DbState>) -> Result<db::Stats, String> {
    let conn = db.lock().unwrap();
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|err| err.to_string())?
        .join("clipvault.db");
    let db_size_bytes = std::fs::metadata(&db_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    db::stats(&conn, db_size_bytes).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or("settings window not found")?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    // The settings webview loads once and stays alive in the background (it's declared
    // statically and just shown/hidden), so fields like the stats need to be re-fetched
    // every time the window is shown again, not only on its first load.
    let _ = app.emit("settings-shown", ());
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ViewerEntryPayload {
    id: i64,
    title: String,
}

#[tauri::command]
pub fn show_viewer_window(app: AppHandle, id: i64, title: String) -> Result<(), String> {
    let window = app
        .get_webview_window("viewer")
        .ok_or("viewer window not found")?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    // Same static-window-kept-alive pattern as settings: the window is created once, so
    // tell it (and re-tell it, if it's already open on a different entry) what to show.
    let _ = app.emit("viewer-entry", ViewerEntryPayload { id, title });
    Ok(())
}
