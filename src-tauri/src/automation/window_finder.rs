use crate::state::WindowInfo;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT, TRUE};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
};

struct EnumData {
    results: Vec<WindowInfo>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);

    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let length = GetWindowTextLengthW(hwnd);
    if length == 0 {
        return TRUE;
    }

    let mut buffer = vec![0u16; (length + 1) as usize];
    let read = GetWindowTextW(hwnd, &mut buffer);
    if read == 0 {
        return TRUE;
    }

    let title = String::from_utf16_lossy(&buffer[..read as usize]);
    let title_lower = title.to_lowercase();

    if title_lower.contains("storylife") && title_lower.contains("fivem") {
        // Get CLIENT area rect (the game rendering area, excludes title bar & borders)
        let mut client_rect = RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_ok() {
            // Convert client area (0,0) to screen coordinates
            let mut top_left = POINT { x: 0, y: 0 };
            if ClientToScreen(hwnd, &mut top_left).as_bool() {
                let width = client_rect.right - client_rect.left;
                let height = client_rect.bottom - client_rect.top;

                data.results.push(WindowInfo {
                    found: true,
                    title,
                    x: top_left.x,
                    y: top_left.y,
                    width,
                    height,
                    hwnd: hwnd.0 as isize,
                });
            }
        }
    }

    TRUE
}

pub fn find_fivem_window() -> Option<WindowInfo> {
    let mut data = EnumData {
        results: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }

    data.results.into_iter().next()
}
