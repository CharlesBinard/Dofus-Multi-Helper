use super::models::DofusWindow;
use std::ffi::c_void;
use std::thread;
use std::time::{Duration, Instant};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT};
use windows::Win32::Graphics::Gdi::{ClientToScreen, ScreenToClient};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameA;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    MOUSEINPUT, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, SendInput, SetActiveWindow, SetFocus, VIRTUAL_KEY,
    VkKeyScanA, MapVirtualKeyA, MAPVK_VK_TO_VSC, KEYBD_EVENT_FLAGS, VK_SHIFT, VK_CONTROL, VK_MENU,
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

fn create_mouse_input(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Maps a screen pixel to SendInput's 0..65535 absolute coordinate space
/// over the virtual desktop (all monitors).
fn normalize_absolute(
    x: i32,
    y: i32,
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
) -> (i32, i32) {
    let w = (width - 1).max(1) as i64;
    let h = (height - 1).max(1) as i64;
    let dx = ((x - origin_x) as i64 * 65535 / w) as i32;
    let dy = ((y - origin_y) as i64 * 65535 / h) as i32;
    (dx, dy)
}

fn screen_to_absolute(x: i32, y: i32) -> (i32, i32) {
    unsafe {
        let origin_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let origin_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        normalize_absolute(x, y, origin_x, origin_y, width, height)
    }
}

/// Move + left click in a single hardware injection (no sleeps between events).
fn send_click_at_screen(x: i32, y: i32) -> Result<(), String> {
    unsafe {
        let (dx, dy) = screen_to_absolute(x, y);
        let move_flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        let inputs = [
            create_mouse_input(dx, dy, move_flags),
            create_mouse_input(0, 0, MOUSEEVENTF_LEFTDOWN),
            create_mouse_input(0, 0, MOUSEEVENTF_LEFTUP),
        ];
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            return Err("Failed to send mouse click".to_string());
        }
        Ok(())
    }
}

fn wait_until_foreground(hwnd: HWND, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if unsafe { GetForegroundWindow() } == hwnd {
            return true;
        }
        thread::sleep(Duration::from_millis(1));
    }
    unsafe { GetForegroundWindow() == hwnd }
}

/// Activates `hwnd` just enough for Unity Raw Input, then waits until it
/// actually is the foreground window instead of sleeping a fixed delay.
fn activate_for_click(hwnd: HWND) -> Result<(), String> {
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
            return Err("Invalid window handle".to_string());
        }
        if GetForegroundWindow() == hwnd {
            return Ok(());
        }

        let target_thread = GetWindowThreadProcessId(hwnd, None);
        let current_thread = GetCurrentThreadId();
        let foreground_window = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground_window, None);

        let attached_fg = foreground_thread != target_thread
            && AttachThreadInput(foreground_thread, target_thread, true).as_bool();
        let attached_cur = current_thread != target_thread
            && AttachThreadInput(current_thread, target_thread, true).as_bool();

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        let _ = SetForegroundWindow(hwnd);
        let _ = BringWindowToTop(hwnd);
        let _ = SetActiveWindow(hwnd);

        if attached_fg {
            let _ = AttachThreadInput(foreground_thread, target_thread, false);
        }
        if attached_cur {
            let _ = AttachThreadInput(current_thread, target_thread, false);
        }

        if !wait_until_foreground(hwnd, Duration::from_millis(80)) {
            return Err("Window did not become foreground".to_string());
        }
        Ok(())
    }
}

/// Clicks the same client-area point on every window.
///
/// Unity 6 only honors hardware mouse input on the foreground window, so each
/// target is activated, then clicked with one `SendInput` (move + down + up).
/// Activation waits until the window is actually foreground instead of a
/// fixed sleep. The original foreground window and cursor are restored.
pub fn click_windows_sequence(
    windows: &[DofusWindow],
    reference: HWND,
    delays: Option<(u64, u64)>,
) -> Result<(), String> {
    use rand::RngExt;

    unsafe {
        let mut original_pos = POINT::default();
        GetCursorPos(&mut original_pos)
            .map_err(|e| format!("Failed to get cursor position: {}", e))?;

        let mut client_pos = original_pos;
        if !ScreenToClient(reference, &mut client_pos).as_bool() {
            return Err("Failed to convert screen to client coordinates".to_string());
        }

        let original_foreground = GetForegroundWindow();
        let mut rng = rand::rng();
        let wait_delay = |rng: &mut rand::rngs::ThreadRng| {
            if let Some((min, max)) = delays {
                thread::sleep(Duration::from_millis(rng.random_range(min..=max)));
            }
        };

        for win in windows {
            let hwnd = HWND(win.hwnd as *mut c_void);
            if hwnd != original_foreground {
                continue;
            }
            if !IsWindow(Some(hwnd)).as_bool() || IsIconic(hwnd).as_bool() {
                continue;
            }
            let mut screen_pos = client_pos;
            if !ClientToScreen(hwnd, &mut screen_pos).as_bool() {
                continue;
            }
            if let Err(e) = send_click_at_screen(screen_pos.x, screen_pos.y) {
                eprintln!("Erreur lors de l'envoi du clic à '{}': {}", win.title, e);
            }
            wait_delay(&mut rng);
        }

        for win in windows {
            let hwnd = HWND(win.hwnd as *mut c_void);
            if hwnd == original_foreground {
                continue;
            }
            if !IsWindow(Some(hwnd)).as_bool() || IsIconic(hwnd).as_bool() {
                continue;
            }

            let mut screen_pos = client_pos;
            if !ClientToScreen(hwnd, &mut screen_pos).as_bool() {
                continue;
            }

            if let Err(e) = activate_for_click(hwnd) {
                eprintln!("Focus impossible pour '{}': {}", win.title, e);
                continue;
            }
            if let Err(e) = send_click_at_screen(screen_pos.x, screen_pos.y) {
                eprintln!("Erreur lors de l'envoi du clic à '{}': {}", win.title, e);
            }
            wait_delay(&mut rng);
        }

        if IsWindow(Some(original_foreground)).as_bool()
            && GetForegroundWindow() != original_foreground
        {
            let _ = activate_for_click(original_foreground);
        }
        let _ = SetCursorPos(original_pos.x, original_pos.y);
        Ok(())
    }
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

pub fn focus_window(hwnd: HWND) -> Result<(), String> {
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
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
        SetFocus(Some(hwnd)).map_err(|e| e.to_string())?;

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
            let _ = CloseHandle(handle);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_absolute_maps_origin_to_zero() {
        assert_eq!(normalize_absolute(0, 0, 0, 0, 1920, 1080), (0, 0));
    }

    #[test]
    fn normalize_absolute_maps_bottom_right_to_65535() {
        assert_eq!(
            normalize_absolute(1919, 1079, 0, 0, 1920, 1080),
            (65535, 65535)
        );
    }

    #[test]
    fn normalize_absolute_handles_negative_virtual_desktop_origin() {
        let (dx, dy) = normalize_absolute(-960, 0, -1920, 0, 3840, 1080);
        assert_eq!(dx, 16388);
        assert_eq!(dy, 0);
    }
}
