use std::ffi::c_void;
use tauri::{Emitter, Window};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use super::models::DofusWindow;
use super::api::{fetch_dofus_windows, focus_window};

pub struct WindowManager {
    windows: Vec<DofusWindow>,
    current_index: usize,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            current_index: 0,
        }
    }

    pub fn refresh_windows(&mut self) -> Vec<DofusWindow>  {
        let new_windows = fetch_dofus_windows();
        if new_windows.len() != self.windows.len() || self.current_index >= new_windows.len() {
            self.current_index = 0;
        }
        self.windows = new_windows;
        self.windows.clone()
    }

    pub fn next_window(&mut self) -> Option<&DofusWindow> {    
        if self.windows.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.windows.len();
        self.windows.get(self.current_index)
    }

    pub fn previous_window(&mut self) -> Option<&DofusWindow> {
        if self.windows.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + self.windows.len() - 1) % self.windows.len();
        self.windows.get(self.current_index)
    }

    pub fn is_focused_on_app_or_dofus(&self, app_hwnd: HWND) -> bool {
        let active_hwnd = unsafe { GetForegroundWindow() };
        let dofus_handles: Vec<HWND> = self.dofus_windows()
            .iter()
            .map(|w| HWND(w.hwnd as *mut c_void))
            .collect();
    
        active_hwnd == app_hwnd || dofus_handles.contains(&active_hwnd)
    }

    pub fn active_dofus_window(&self) -> Option<DofusWindow> {
        self.windows.get(self.current_index).cloned()
    }

    pub fn focus_dofus_window(&mut self, hwnd: HWND) -> Result<(), String> {
        if !self.dofus_windows().iter().any(|w| HWND(w.hwnd as *mut c_void) == hwnd) {
            return Err("Le hwnd n'est pas une fenêtre Dofus".into());
        }
        focus_window(hwnd)?;
        self.current_index = self.windows.iter().position(|w| w.hwnd == hwnd.0 as usize).unwrap_or(0);
        Ok(())
    }

    pub fn emit_active_dofus_changed(&self, window: &Window) -> Result<(), String> {
        if let Some(active_dofus) = self.active_dofus_window() {
            let payload = serde_json::to_value(&active_dofus).map_err(|e| e.to_string())?;
            window
                .emit("active_dofus_changed", payload)
                .map_err(|e| e.to_string())
        } else {
            window
                .emit("active_dofus_changed", serde_json::json!(null))
                .map_err(|e| e.to_string())
        }
    }

    pub fn dofus_windows(&self) -> Vec<DofusWindow> {
        self.windows.clone()
    }

}