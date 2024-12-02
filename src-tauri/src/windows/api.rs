use super::models::DofusWindow;
use std::thread;
use tokio::time::Duration;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameA;
use windows::Win32::System::Threading::AttachThreadInput;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn get_window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut buffer = [0u8; 512];
        let len = GetWindowTextA(hwnd, &mut buffer);
        if len == 0 {
            return None;
        }
        String::from_utf8(buffer[..len as usize].to_vec()).ok()
    }
}

fn make_lparam(x: i32, y: i32) -> LPARAM {
    LPARAM(((y & 0xFFFF) << 16 | (x & 0xFFFF)) as isize)
}

pub fn send_click(hwnd: HWND, pos: POINT) -> Result<(), String> {
    unsafe {
        PostMessageA(
            hwnd,
            WM_LBUTTONDOWN,
            WPARAM(0x0001),
            make_lparam(pos.x, pos.y),
        )
        .ok()
        .ok_or_else(|| "Failed to post WM_LBUTTONDOWN".to_string())?;

        thread::sleep(Duration::from_millis(50));

        PostMessageA(
            hwnd,
            WM_LBUTTONUP,
            WPARAM(0x0000),
            make_lparam(pos.x, pos.y),
        )
        .ok()
        .ok_or_else(|| "Failed to post WM_LBUTTONUP".to_string())
    }
}

pub fn focus_window(hwnd: HWND) -> Result<(), String> {
    unsafe {
        let target_thread_id = GetWindowThreadProcessId(hwnd, None);
        let foreground_hwnd = GetForegroundWindow();
        let foreground_thread_id = GetWindowThreadProcessId(foreground_hwnd, None);

        if foreground_thread_id != target_thread_id {
            if !AttachThreadInput(foreground_thread_id, target_thread_id, true).as_bool() {
                return Err("Échec de l'attachement du thread input".to_string());
            }
        }

        if !ShowWindow(hwnd, SW_RESTORE).as_bool() {
            return Err("Échec de la restauration de la fenêtre".to_string());
        }

        let _ = BringWindowToTop(hwnd);

        if !SetForegroundWindow(hwnd).as_bool() {
            return Err("Échec de la mise en avant de la fenêtre".to_string());
        }

        if foreground_thread_id != target_thread_id {
            let _ = AttachThreadInput(foreground_thread_id, target_thread_id, false);
        }
    }
    Ok(())
}

pub fn fetch_dofus_windows() -> Vec<DofusWindow> {
    let mut windows = Vec::new();

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        );
        if let Ok(handle) = process_handle {
            let mut name = [0u8; 260];
            if GetModuleBaseNameA(handle, None, &mut name) > 0 {
                let process_name = String::from_utf8_lossy(&name).to_string();
                if process_name.starts_with("Dofus") {
                    if let Some(title) = get_window_title(hwnd) {
                        let parts: Vec<&str> = title.split(" - ").collect();
                        if parts.len() >= 2 {
                            let windows = unsafe { &mut *(lparam.0 as *mut Vec<DofusWindow>) };
                            windows.push(DofusWindow {
                                title: title.clone(),
                                hwnd: hwnd.0 as usize,
                                name: parts[0].trim().to_string(),
                                class: parts[1].trim().to_string(),
                            });
                        }
                    }
                }
            }
        }
        BOOL(1)
    }

    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut windows as *mut _ as isize),
        )
        .ok();
    }

    windows
}
