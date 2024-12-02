use tauri::State;
use std::collections::HashMap;
use super::store::ShortcutStore;

#[tauri::command]
pub async fn register_shortcut(shortcut_type: String, store: State<'_, ShortcutStore>) -> Result<(), String> {
    *store.registering.lock().unwrap() = Some(shortcut_type);
    Ok(())
}

#[tauri::command]
pub async fn get_shortcuts(store: State<'_, ShortcutStore>) -> Result<HashMap<String, String>, String> {
    let shortcuts = store.shortcuts.lock().unwrap().clone();
    Ok(shortcuts)
}