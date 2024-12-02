use rdev::{Button, EventType};
use tauri::Emitter;
use super::store::ShortcutStore;
use super::models::ShortcutPayload;

pub fn handle_event(event: rdev::Event, store: &ShortcutStore, app_handle: &tauri::AppHandle) {
    let key_str = match event.event_type {
        EventType::KeyPress(key) => format!("{:?}", key),
        EventType::ButtonPress(button) => {
            if button != Button::Left && button != Button::Right {
                format!("{:?}", button)
            } else {
                return
            }
        },
        _ => return
    };

    let mut registering = store.registering.lock().unwrap();
    
    if let Some(shortcut_type) = registering.take() {
        store.shortcuts.lock().unwrap().insert(shortcut_type.clone(), key_str.clone());
        
        if let Err(e) = store.save_shortcuts() {
            eprintln!("Erreur lors de la sauvegarde des raccourcis: {}", e);
        }
        
        let payload = ShortcutPayload {
            shortcut: shortcut_type,
            key: key_str,
        };
        let _ = app_handle.emit("input_register_event", payload);
    } else {
        let shortcuts = store.shortcuts.lock().unwrap();
        for (shortcut_type, stored_key) in shortcuts.iter() {
            if stored_key == &key_str {
                let payload = ShortcutPayload {
                    shortcut: shortcut_type.clone(),
                    key: key_str.clone(),
                };
                let _ = app_handle.emit("shortcut_triggered", payload);
            }
        }
    }
}