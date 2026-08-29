use crate::db::{self, HistoryEntry};
use clipboard_win::{formats, set_clipboard, Clipboard, Setter};
use rusqlite::Connection;
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
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
pub struct FilePreview {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub too_large: bool,
    pub size_bytes: Option<u64>,
    pub modified: Option<i64>,
    pub sha1: Option<String>,
    pub crc32: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewData {
    pub kind: String,
    pub text: Option<String>,
    pub image_data_url: Option<String>,
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
            files: None,
        }),
        "image" => {
            let path = entry
                .image_path
                .ok_or("immagine troppo grande, non salvata per intero")?;
            let bytes = std::fs::read(&path).map_err(|err| err.to_string())?;
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(PreviewData {
                kind: "image".into(),
                text: None,
                image_data_url: Some(format!("data:image/png;base64,{encoded}")),
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
        ..base
    }
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
    pub autostart: bool,
    pub hotkey: String,
    pub version: String,
}

#[tauri::command]
pub fn get_settings(app: AppHandle, db: State<'_, DbState>) -> Result<Settings, String> {
    let conn = db.lock().unwrap();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    Ok(Settings {
        max_history: db::max_history(&conn),
        max_file_kb: db::max_file_kb(&conn),
        autostart,
        hotkey: db::hotkey(&conn),
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
pub fn set_settings(
    app: AppHandle,
    db: State<'_, DbState>,
    max_history: i64,
    max_file_kb: i64,
    autostart: bool,
) -> Result<(), String> {
    {
        let conn = db.lock().unwrap();
        db::set_setting(&conn, "max_history", &max_history.max(1).to_string())
            .map_err(|err| err.to_string())?;
        db::set_setting(&conn, "max_file_kb", &max_file_kb.max(0).to_string())
            .map_err(|err| err.to_string())?;
    }

    let autolaunch = app.autolaunch();
    let result = if autostart {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|err| err.to_string())
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or("settings window not found")?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())
}
