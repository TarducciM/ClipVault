use crate::db::{self, HistoryEntry};
use clipboard_win::{formats, set_clipboard, Clipboard, Setter};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

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
    let entry = {
        let conn = db.lock().unwrap();
        db::get_full(&conn, id).map_err(|err| err.to_string())?
    };
    let Some(entry) = entry else {
        return Err("entry not found".into());
    };

    match entry.kind.as_str() {
        "text" => {
            let text = entry.content.unwrap_or_default();
            set_clipboard(formats::Unicode, text).map_err(|err| err.to_string())
        }
        "image" => {
            let path = entry.image_path.ok_or("missing image path")?;
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
            let json = entry.file_list.ok_or("missing file list")?;
            let paths: Vec<String> = serde_json::from_str(&json).map_err(|err| err.to_string())?;
            let _clip = Clipboard::new_attempts(10).map_err(|err| err.to_string())?;
            formats::FileList
                .write_clipboard(&paths[..])
                .map_err(|err| err.to_string())
        }
        other => Err(format!("unknown entry kind: {other}")),
    }
}
