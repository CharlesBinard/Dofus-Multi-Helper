use super::api::{focus_window, send_click};
use super::manager::WindowManager;
use rand::Rng;
use serde::Deserialize;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{command, Emitter, State, Window};
use tokio::time::Duration;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

#[command]
pub fn get_active_dofus_window(
    state: State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Option<super::models::DofusWindow>, String> {
    let manager = state.lock().unwrap();
    Ok(manager.active_dofus_window())
}

#[command]
pub fn focus_window_command(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    hwnd: usize,
    window: Window,
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();
    manager
        .focus_dofus_window(HWND(hwnd as *mut c_void))
        .map_err(|e| e.to_string())?;
    manager.emit_active_dofus_changed(&window)?;
    Ok(())
}

#[command]
pub fn set_tauri_always_on_top(window: Window, always_on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_dofus_windows(
    state: State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Vec<super::models::DofusWindow>, String> {
    let manager = state.lock().unwrap();
    Ok(manager.dofus_windows())
}

#[command]
pub fn is_focused_on_app_or_dofus(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<bool, String> {
    let app_hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let manager = state.lock().unwrap();
    Ok(manager.is_focused_on_app_or_dofus(app_hwnd))
}

#[command]
pub fn set_window_size(window: Window, width: f64, height: f64) -> Result<(), String> {
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())
}

#[command]
pub async fn next_dofus_window(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    let mut manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    if let Some(next_win) = manager.next_window() {
        focus_window(HWND(next_win.hwnd as *mut c_void)).map_err(|e| e.to_string())?;
        manager.emit_active_dofus_changed(&window)?;
    }
    Ok(())
}

#[command]
pub async fn refresh_windows(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    let mut manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let updated_windows = manager.refresh_windows();
    window
        .emit("dofus_windows_changed", serde_json::json!(updated_windows))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn prev_dofus_window(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    let mut manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    if let Some(prev_win) = manager.previous_window() {
        focus_window(HWND(prev_win.hwnd as *mut c_void)).map_err(|e| e.to_string())?;
        manager.emit_active_dofus_changed(&window)?;
    }
    Ok(())
}

#[command]
pub fn click_all_dofus_windows(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    let mut cursor_pos = POINT::default();
    unsafe {
        if GetCursorPos(&mut cursor_pos).is_err() {
            return Err("Échec de la récupération de la position du curseur".into());
        }
    }

    let manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    let dofus_windows = manager.dofus_windows();

    for win in dofus_windows {
        let cursor_pos = cursor_pos;
        let hwnd_raw = win.hwnd as isize;

        thread::spawn(move || {
            let hwnd = HWND(hwnd_raw as *mut c_void);
            let client_pos = cursor_pos;

            if let Err(e) = send_click(hwnd, client_pos) {
                eprintln!("Erreur lors de l'envoi du clic: {}", e);
            }

            thread::sleep(Duration::from_millis(50));
        });
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct ClickAllDelaysParams {
    delay_min_ms: Option<u64>,
    delay_max_ms: Option<u64>,
}

#[command]
pub fn click_all_dofus_windows_with_delay(
    params: ClickAllDelaysParams,
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    let delay_min = params.delay_min_ms.unwrap_or(100);
    let delay_max = params.delay_max_ms.unwrap_or(130);
    let mut rng = rand::thread_rng();

    let mut cursor_pos = POINT::default();
    unsafe {
        if GetCursorPos(&mut cursor_pos).is_err() {
            return Err("Échec de la récupération de la position du curseur".into());
        }
    }

    let manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    let dofus_windows = manager.dofus_windows();

    for win in dofus_windows {
        let hwnd = HWND(win.hwnd as *mut c_void);
        let client_pos = cursor_pos;

        if let Err(e) = send_click(hwnd, client_pos) {
            eprintln!("Erreur lors de l'envoi du clic: {}", e);
        }

        let random_delay = rng.gen_range(delay_min..=delay_max);
        thread::sleep(Duration::from_millis(random_delay));
    }

    Ok(())
}
