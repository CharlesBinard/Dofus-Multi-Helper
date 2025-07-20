use super::api::{focus_window, send_click, send_key, send_text, send_enter};
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
        println!("Focus not on app or dofus");
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

#[command]
pub fn send_key_to_all_dofus_windows(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
    key: String,
    repeat: u32,
) -> Result<(), String> {
    let manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    let dofus_windows = manager.dofus_windows();
    let first_window = dofus_windows.first();

    for win in dofus_windows.iter().skip(1) {
        let hwnd = HWND(win.hwnd as *mut c_void);

        if let Err(e) = focus_window(hwnd) {
            eprintln!("Erreur lors du focus de la fenêtre: {}", e);
        }
        
        thread::sleep(Duration::from_millis(20));

        for _ in 0..repeat {    
            if let Err(e) = send_key(&key) {
                eprintln!("Erreur lors de l'envoi de la touche '{}': {}", key, e);
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    if let Some(first) = first_window {
        let hwnd = HWND(first.hwnd as *mut c_void);
        if let Err(e) = focus_window(hwnd) {
            eprintln!("Erreur lors du focus de la première fenêtre: {}", e);
        }
    }

    Ok(())
}

#[command]
pub fn auto_invite_all_characters(
    state: State<'_, Arc<Mutex<WindowManager>>>,
    window: Window,
) -> Result<(), String> {
    
    let manager = state
        .lock()
        .map_err(|_| "Échec du verrouillage de WindowManager".to_string())?;

    let app_hwnd = window.hwnd().unwrap();
    if !manager.is_focused_on_app_or_dofus(app_hwnd) {
        return Ok(());
    }

    let dofus_windows = manager.dofus_windows();
    
    if dofus_windows.is_empty() {
        return Err("Aucune fenêtre Dofus trouvée".to_string());
    }

    // Récupérer la première fenêtre
    let first_window = &dofus_windows[0];
    let first_hwnd = HWND(first_window.hwnd as *mut c_void);
    
    // Focus sur la première fenêtre
    focus_window(first_hwnd).map_err(|e| format!("Erreur lors du focus de la première fenêtre: {}", e))?;
    thread::sleep(Duration::from_millis(100));

    send_key("=").map_err(|e| format!("Erreur lors de l'envoi de la touche '=': {}", e))?;
    thread::sleep(Duration::from_millis(50));

    // Inviter tous les autres personnages depuis la première fenêtre
    for (index, target_window) in dofus_windows.iter().skip(1).enumerate() {
        let character_name = &target_window.name;
        
        // Taper la commande d'invitation
        let invite_command = format!("/invite {}", character_name);
        send_text(&invite_command).map_err(|e| format!("Erreur lors de l'envoi du texte '{}': {}", invite_command, e))?;
        thread::sleep(Duration::from_millis(50));
        
        // Appuyer sur Entrée
        send_enter().map_err(|e| format!("Erreur lors de l'envoi de la touche Entrée: {}", e))?;
        thread::sleep(Duration::from_millis(200)); // Délai plus long entre les invitations
    }

    Ok(())
}
