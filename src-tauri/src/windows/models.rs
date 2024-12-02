use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct DofusWindow {
    pub title: String,
    pub hwnd: usize,
    pub name: String,
    pub class: String,
}

