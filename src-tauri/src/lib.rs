mod capture;
mod commands;
mod db;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

/// Installer-invoked helper mode: `--enable-autostart` / `--disable-autostart` on the command
/// line make ClipVault apply the choice made on the installer's "additional tasks" page and
/// exit immediately, without showing any window. Going through this exact plugin call (instead
/// of the installer writing the Run-key registry value itself) guarantees it can never drift
/// from what the in-app Settings toggle does (see commands::save_settings).
fn apply_autostart_cli_flag(app: &tauri::AppHandle, args: &[String]) -> bool {
    let enable = args.iter().any(|a| a == "--enable-autostart");
    let disable = args.iter().any(|a| a == "--disable-autostart");
    if !enable && !disable {
        return false;
    }
    let autolaunch = app.autolaunch();
    let result = if enable { autolaunch.enable() } else { autolaunch.disable() };
    if let Err(err) = result {
        let action = if enable { "enable" } else { "disable" };
        eprintln!("clipvault: failed to {action} autostart via CLI flag: {err}");
    }
    true
}

fn toggle_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("popup-shown", ());
    }
}

pub(crate) fn shortcut_handler(app: &tauri::AppHandle, _shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() == ShortcutState::Pressed {
        toggle_main_window(app);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().collect();
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A running instance receives the flag here if the installer launched a second
            // process while ClipVault was already open (e.g. reinstalling/repairing over an
            // existing install) - the single-instance plugin forwards args instead of letting
            // that second process start up on its own.
            if apply_autostart_cli_flag(app, &args) {
                return;
            }
            toggle_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_history,
            commands::toggle_pin,
            commands::delete_entry,
            commands::clear_history,
            commands::copy_entry_to_clipboard,
            commands::open_entry,
            commands::open_url,
            commands::reveal_entry,
            commands::get_entry_preview,
            commands::get_settings,
            commands::set_settings,
            commands::get_stats,
            commands::set_hotkey,
            commands::show_settings_window,
            commands::show_viewer_window,
            commands::list_snippets,
            commands::add_snippet,
            commands::delete_snippet,
            commands::copy_snippet_to_clipboard,
            commands::export_history,
            commands::import_history,
            commands::should_show_onboarding,
            commands::mark_onboarding_seen,
        ])
        .setup(move |app| {
            if apply_autostart_cli_flag(app.handle(), &cli_args) {
                app.handle().exit(0);
                return Ok(());
            }

            // Registered here (not chained with the other .plugin() calls above) because
            // tauri-plugin-updater's Builder needs an AppHandle, only available inside setup.
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

            let app_data_dir = app.path().app_data_dir()?;
            let images_dir = app_data_dir.join("images");
            std::fs::create_dir_all(&images_dir)?;

            let conn = db::open(&app_data_dir)?;
            let conn = Arc::new(Mutex::new(conn));
            app.manage(conn.clone());

            let hotkey = {
                let conn = conn.lock().unwrap();
                db::hotkey(&conn)
            };
            if let Err(err) = app.global_shortcut().on_shortcut(hotkey.as_str(), shortcut_handler) {
                eprintln!("clipvault: failed to register hotkey '{hotkey}': {err}, falling back to default");
                let _ = app.global_shortcut().on_shortcut(db::DEFAULT_HOTKEY, shortcut_handler);
            }

            capture::spawn(app.handle().clone(), conn, images_dir);

            let show_item = MenuItem::with_id(app, "show", "Mostra ClipVault", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Impostazioni", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Esci", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => toggle_main_window(app),
                    "settings" => {
                        let _ = commands::show_settings_window(app.clone());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // All windows are declared statically and created once at startup, so closing
                // one for real (destroying it) would leave no way to show it again without
                // restarting the app. Hide instead, exactly like the main popup does.
                if window.label() == "main" || window.label() == "settings" || window.label() == "viewer" {
                    window.hide().ok();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
