use super::models::DofusWindow;
use std::thread;
use tokio::time::Duration;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameA;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, SendInput,
    SetActiveWindow, SetFocus, VIRTUAL_KEY, VkKeyScanA, MapVirtualKeyA, MAPVK_VK_TO_VSC, KEYBD_EVENT_FLAGS, VK_SHIFT, VK_CONTROL, VK_MENU
};
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

fn create_scancode_input(scancode: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scancode,
                dwFlags: KEYEVENTF_SCANCODE | flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Ok(());
    }

    unsafe {
        let vk_scan_result = VkKeyScanA(key.as_bytes()[0] as i8);
        if vk_scan_result == -1 {
            return Err(format!("No key found for character: {}", key));
        }

        let vk_code = (vk_scan_result & 0xff) as u32;
        let shift_state = (vk_scan_result >> 8) & 0xff;

        let scancode = MapVirtualKeyA(vk_code, MAPVK_VK_TO_VSC) as u16;
        if scancode == 0 {
            return Err(format!("Could not map virtual key {} to scancode", vk_code));
        }
        
        let needs_shift = (shift_state & 1) != 0;
        let needs_ctrl = (shift_state & 2) != 0;
        let needs_alt = (shift_state & 4) != 0;

        let mut inputs: Vec<INPUT> = Vec::new();

        if needs_shift {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_SHIFT.0 as u32, MAPVK_VK_TO_VSC) as u16, Default::default()));
        }
        if needs_ctrl {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_CONTROL.0 as u32, MAPVK_VK_TO_VSC) as u16, Default::default()));
        }
        if needs_alt {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_MENU.0 as u32, MAPVK_VK_TO_VSC) as u16, Default::default()));
        }

        inputs.push(create_scancode_input(scancode, Default::default()));
        inputs.push(create_scancode_input(scancode, KEYEVENTF_KEYUP));

        if needs_alt {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_MENU.0 as u32, MAPVK_VK_TO_VSC) as u16, KEYEVENTF_KEYUP));
        }
        if needs_ctrl {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_CONTROL.0 as u32, MAPVK_VK_TO_VSC) as u16, KEYEVENTF_KEYUP));
        }
        if needs_shift {
            inputs.push(create_scancode_input(MapVirtualKeyA(VK_SHIFT.0 as u32, MAPVK_VK_TO_VSC) as u16, KEYEVENTF_KEYUP));
        }

        let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);

        if result == 0 {
            return Err("Failed to send key".to_string());
        }

        Ok(())
    }
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
        if !IsWindow(hwnd).as_bool() {
            return Err("Invalid window handle".to_string());
        }

        let target_thread = GetWindowThreadProcessId(hwnd, None);
        let current_thread = GetCurrentThreadId();
        let foreground_window = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground_window, None);

        if foreground_thread != target_thread {
            if !AttachThreadInput(foreground_thread, target_thread, true).as_bool() {
                return Err("Failed to attach foreground thread".to_string());
            }
        }
        if current_thread != target_thread {
            if !AttachThreadInput(current_thread, target_thread, true).as_bool() {
                return Err("Failed to attach current thread".to_string());
            }
        }

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = ShowWindow(hwnd, SW_SHOW);

        let _ = BringWindowToTop(hwnd);

        SetActiveWindow(hwnd).map_err(|e| e.to_string())?;
        SetFocus(hwnd).map_err(|e| e.to_string())?;

        if foreground_thread != target_thread {
            let _ = AttachThreadInput(foreground_thread, target_thread, false);
        }
        if current_thread != target_thread {
            let _ = AttachThreadInput(current_thread, target_thread, false);
        }

        Ok(())
    }
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

pub fn send_text(text: &str) -> Result<(), String> {
    for ch in text.chars() {
        send_key(&ch.to_string())?;
        // Petit délai entre chaque caractère pour éviter les problèmes
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

pub fn send_enter() -> Result<(), String> {
    unsafe {
        let enter_scancode = MapVirtualKeyA(0x0D, MAPVK_VK_TO_VSC) as u16; // VK_RETURN
        if enter_scancode == 0 {
            return Err("Could not map Enter key to scancode".to_string());
        }

        let inputs = vec![
            create_scancode_input(enter_scancode, Default::default()),
            create_scancode_input(enter_scancode, KEYEVENTF_KEYUP),
        ];

        let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);

        if result == 0 {
            return Err("Failed to send Enter key".to_string());
        }

        Ok(())
    }
}
