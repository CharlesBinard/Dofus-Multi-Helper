#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod shortcuts;
mod windows;
use rdev::listen;
use shortcuts::handler::handle_event;
use shortcuts::store::ShortcutStore;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use windows::manager::WindowManager;

fn main() {
    let window_manager = Arc::new(Mutex::new(WindowManager::new()));

    tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .manage(window_manager.clone())
        .invoke_handler(tauri::generate_handler![
            shortcuts::commands::register_shortcut,
            shortcuts::commands::get_shortcuts,
            shortcuts::commands::remove_shortcut_key,
            windows::commands::get_active_dofus_window,
            windows::commands::focus_window_command,
            windows::commands::set_tauri_always_on_top,
            windows::commands::is_focused_on_app_or_dofus,
            windows::commands::set_window_size,
            windows::commands::next_dofus_window,
            windows::commands::prev_dofus_window,
            windows::commands::click_all_dofus_windows,
            windows::commands::click_all_dofus_windows_with_delay,
            windows::commands::get_dofus_windows,
            windows::commands::refresh_windows,
        ])
        .setup(move |app| {
            let store = ShortcutStore::new(&app.handle());
            let store_clone = store.clone();

            let mut wm = window_manager
                .lock()
                .map_err(|_| "Échec du verrouillage de WindowManager")?;

            wm.refresh_windows();

            let window = app
                .get_webview_window("main")
                .ok_or("Échec de récupération de la fenêtre principale")?;

            window.set_always_on_top(true)?;

            app.manage(store);

            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                listen(move |event| {
                    handle_event(event, &store_clone, &app_handle);
                })
                .unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
