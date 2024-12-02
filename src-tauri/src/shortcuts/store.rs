// src/shortcuts/store.rs
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use tauri::{AppHandle, Manager};
use super::models::ShortcutData;

pub struct ShortcutStore {
    pub shortcuts: Arc<Mutex<HashMap<String, String>>>,
    pub registering: Arc<Mutex<Option<String>>>,
    pub config_path: PathBuf
}

impl ShortcutStore {
    pub fn new(app_handle: &AppHandle) -> Self {
        let config_path = app_handle.path().app_config_dir()
            .unwrap()
            .join("shortcuts.json");

        if let Some(config_dir) = config_path.parent() {
            fs::create_dir_all(config_dir).unwrap();
        }

        let shortcuts = if config_path.exists() {
            let data = fs::read_to_string(&config_path).unwrap();
            let shortcut_data: ShortcutData = serde_json::from_str(&data).unwrap_or(ShortcutData {
                shortcuts: HashMap::new()
            });
            shortcut_data.shortcuts
        } else {
            HashMap::new()
        };

        Self {
            shortcuts: Arc::new(Mutex::new(shortcuts)),
            registering: Arc::new(Mutex::new(None)),
            config_path
        }
    }

    pub fn save_shortcuts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let shortcuts = self.shortcuts.lock().unwrap();
        let data = ShortcutData {
            shortcuts: shortcuts.clone()
        };
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }
}

impl Clone for ShortcutStore {
    fn clone(&self) -> Self {
        Self {
            shortcuts: self.shortcuts.clone(),
            registering: self.registering.clone(),
            config_path: self.config_path.clone()
        }
    }
}