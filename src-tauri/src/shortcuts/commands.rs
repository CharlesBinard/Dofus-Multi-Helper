use super::{models::ShortcutPayload, store::ShortcutStore};
use std::collections::HashMap;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn register_shortcut(
    shortcut_type: String,
    store: State<'_, ShortcutStore>,
) -> Result<(), String> {
    *store.registering.lock().unwrap() = Some(shortcut_type);
    Ok(())
}

#[tauri::command]
pub async fn get_shortcuts(
    store: State<'_, ShortcutStore>,
) -> Result<HashMap<String, String>, String> {
    let shortcuts = store.shortcuts.lock().unwrap().clone();
    Ok(shortcuts)
}

#[tauri::command]
pub async fn remove_shortcut_key(
    shortcut_type: String,
    store: State<'_, ShortcutStore>,
    window: Window,
) -> Result<(), String> {
    {
        let mut shortcuts = store.shortcuts.lock().unwrap();
        shortcuts.remove(&shortcut_type);
    }

    if let Err(e) = store.save_shortcuts() {
        return Err(format!("Failed to save shortcuts: {}", e));
    }

    let payload = ShortcutPayload {
        shortcut: shortcut_type,
        key: String::new(),
    };

    window
        .emit("input_register_event", payload)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(())
}
