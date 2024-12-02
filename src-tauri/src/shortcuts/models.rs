use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct ShortcutData {
    pub shortcuts: HashMap<String, String>
}

#[derive(Clone, Serialize)]
pub struct ShortcutPayload {
    pub shortcut: String,
    pub key: String,
}