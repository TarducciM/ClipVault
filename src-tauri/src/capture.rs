use crate::db::{self, NewEntry};
use clipboard_win::{formats, get_clipboard, Monitor};
use image::ImageFormat;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const THUMBNAIL_MAX: u32 = 200;

/// Spawns a dedicated thread that owns a Windows clipboard-change listener (a message-only
/// window under the hood) and blocks on it — no polling, near-zero idle cost.
pub fn spawn(app: AppHandle, conn: Arc<Mutex<Connection>>, images_dir: PathBuf) {
    thread::spawn(move || {
        let mut monitor = match Monitor::new() {
            Ok(monitor) => monitor,
            Err(err) => {
                eprintln!("clipvault: failed to start clipboard monitor: {err}");
                return;
            }
        };

        loop {
            match monitor.recv() {
                Ok(true) => {}
                Ok(false) => break,
                Err(err) => {
                    eprintln!("clipvault: clipboard monitor error: {err}");
                    continue;
                }
            }

            let Some(entry) = read_clipboard(&images_dir) else {
                continue;
            };

            let db = conn.lock().unwrap();
            if db::last_hash(&db).as_deref() == Some(entry.hash.as_str()) {
                continue;
            }

            match db::insert(&db, &entry, now_millis()) {
                Ok(id) => {
                    drop(db);
                    eprintln!(
                        "clipvault: captured entry #{id} ({}): {}",
                        entry.kind, entry.preview
                    );
                    let _ = app.emit("history-updated", ());
                }
                Err(err) => eprintln!("clipvault: failed to store clipboard entry: {err}"),
            }
        }
    });
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_clipboard(images_dir: &Path) -> Option<NewEntry> {
    if let Ok(files) = get_clipboard::<Vec<PathBuf>, _>(formats::FileList) {
        if !files.is_empty() {
            return Some(build_files_entry(files));
        }
    }

    if let Ok(bmp_bytes) = get_clipboard::<Vec<u8>, _>(formats::Bitmap) {
        if !bmp_bytes.is_empty() {
            if let Some(entry) = build_image_entry(&bmp_bytes, images_dir) {
                return Some(entry);
            }
        }
    }

    if let Ok(text) = get_clipboard::<String, _>(formats::Unicode) {
        if !text.trim().is_empty() {
            return Some(build_text_entry(text));
        }
    }

    None
}

fn build_text_entry(text: String) -> NewEntry {
    let hash = hash_bytes(text.as_bytes());
    let preview: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(300)
        .collect();
    NewEntry {
        kind: "text",
        preview: preview.clone(),
        search_text: text.clone(),
        content: Some(text),
        image_path: None,
        file_list: None,
        thumbnail: None,
        hash,
    }
}

fn build_files_entry(files: Vec<PathBuf>) -> NewEntry {
    let names: Vec<String> = files
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    let preview = names.join(", ");
    let paths: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let json = serde_json::to_string(&paths).unwrap_or_default();
    let hash = hash_bytes(json.as_bytes());
    NewEntry {
        kind: "files",
        preview: preview.clone(),
        search_text: preview,
        content: None,
        image_path: None,
        file_list: Some(json),
        thumbnail: None,
        hash,
    }
}

fn build_image_entry(bmp_bytes: &[u8], images_dir: &Path) -> Option<NewEntry> {
    let img = image::load_from_memory_with_format(bmp_bytes, ImageFormat::Bmp).ok()?;
    let hash = hash_bytes(bmp_bytes);

    let file_name = format!("{hash}.png");
    let path = images_dir.join(&file_name);
    if !path.exists() {
        img.save_with_format(&path, ImageFormat::Png).ok()?;
    }

    let thumbnail = img.thumbnail(THUMBNAIL_MAX, THUMBNAIL_MAX);
    let mut thumbnail_bytes = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut thumbnail_bytes), ImageFormat::Png)
        .ok()?;

    let (width, height) = (img.width(), img.height());
    let preview = format!("Immagine {width}x{height}");

    Some(NewEntry {
        kind: "image",
        preview: preview.clone(),
        search_text: preview,
        content: None,
        image_path: Some(path.to_string_lossy().into_owned()),
        file_list: None,
        thumbnail: Some(thumbnail_bytes),
        hash,
    })
}
