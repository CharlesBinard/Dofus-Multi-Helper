// src/shortcuts/store.rs
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use super::models::ShortcutData;

#[derive(Clone)]
pub struct ShortcutStore {
    pub shortcuts: Arc<Mutex<HashMap<String, String>>>,
    pub registering: Arc<Mutex<Option<String>>>,
    pub app_handle: AppHandle,
    pub watching_key_to_send: Arc<Mutex<bool>>,
    pub config_path: PathBuf,
}

impl ShortcutStore {
    pub fn new(app_handle: &AppHandle) -> Self {
        let config_path = app_handle
            .path()
            .resolve("dofus-multi-helper.shortcuts.json", tauri::path::BaseDirectory::AppConfig)
            .expect("Failed to resolve config path");

        let shortcuts = if let Ok(file_content) = fs::read_to_string(&config_path) {
            serde_json::from_str::<ShortcutData>(&file_content)
                .map(|data| data.shortcuts)
                .unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        Self {
            shortcuts: Arc::new(Mutex::new(shortcuts)),
            registering: Arc::new(Mutex::new(None)),
            app_handle: app_handle.clone(),
            watching_key_to_send: Arc::new(Mutex::new(false)),
            config_path,
        }
    }

    pub fn save_shortcuts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let shortcuts_guard = self.shortcuts.lock().unwrap();
        let data = ShortcutData {
            shortcuts: shortcuts_guard.clone(),
        };
        let json = serde_json::to_string(&data)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }
}

#[tauri::command]
pub fn watch_key_to_send(state: State<ShortcutStore>) {
    let mut watching = state.watching_key_to_send.lock().unwrap();
    *watching = true;
}