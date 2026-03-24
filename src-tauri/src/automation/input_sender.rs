use std::thread::sleep;
use std::time::Duration;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, GetWindowRect, IsIconic, SetForegroundWindow, ShowWindow, SM_CXSCREEN,
    SM_CYSCREEN, SW_RESTORE,
};

// Scan codes — AFK menu navigation
const SC_F5: u16 = 0x3F;
const SC_DOWN: u16 = 0x50; // Extended key — needs KEYEVENTF_EXTENDEDKEY
const SC_ENTER: u16 = 0x1C;

// Scan codes — AZERTY movement + interaction (physical key positions, layout-independent)
pub const SC_Z: u16 = 0x11; // W position on QWERTY — forward
pub const SC_Q: u16 = 0x10; // A position on QWERTY — left
pub const SC_S: u16 = 0x1F; // S — backward
pub const SC_D: u16 = 0x20; // D — right
pub const SC_E: u16 = 0x12; // E — interaction

/// Send a regular (non-extended) key press then release, with hold time.
/// Press and release are SEPARATE SendInput calls so the game sees the key held.
fn send_key(scan_code: u16) {
    send_key_inner(scan_code, false);
}

/// Send an extended key (arrows, Home, End, Insert, Delete, PgUp, PgDn, etc.)
fn send_key_extended(scan_code: u16) {
    send_key_inner(scan_code, true);
}

fn send_key_inner(scan_code: u16, extended: bool) {
    let ext_flag = if extended {
        KEYEVENTF_EXTENDEDKEY
    } else {
        Default::default()
    };

    // Key DOWN — separate SendInput call
    let down = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(), // must be 0 for scan code mode
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE | ext_flag,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];

    unsafe {
        SendInput(&down, std::mem::size_of::<INPUT>() as i32);
    }

    // Hold the key for 40ms so the game registers it
    sleep(Duration::from_millis(40));

    // Key UP — separate SendInput call
    let up = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP | ext_flag,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];

    unsafe {
        SendInput(&up, std::mem::size_of::<INPUT>() as i32);
    }
}

fn click_at(x: i32, y: i32) {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let abs_x = (x as f64 / screen_width as f64 * 65535.0) as i32;
    let abs_y = (y as f64 / screen_height as f64 * 65535.0) as i32;

    // Move
    let move_input = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x,
                dy: abs_y,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                ..Default::default()
            },
        },
    }];
    unsafe {
        SendInput(&move_input, std::mem::size_of::<INPUT>() as i32);
    }

    sleep(Duration::from_millis(50));

    // Click down
    let down = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x,
                dy: abs_y,
                dwFlags: MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE,
                ..Default::default()
            },
        },
    }];
    unsafe {
        SendInput(&down, std::mem::size_of::<INPUT>() as i32);
    }

    sleep(Duration::from_millis(40));

    // Click up
    let up = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x,
                dy: abs_y,
                dwFlags: MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE,
                ..Default::default()
            },
        },
    }];
    unsafe {
        SendInput(&up, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Focus the FiveM window (with click at center to grab input focus)
pub fn focus_window(hwnd_val: isize) -> Result<(), String> {
    focus_window_inner(hwnd_val, true)
}

/// Focus the FiveM window without clicking (restore + foreground only)
pub fn focus_window_no_click(hwnd_val: isize) -> Result<(), String> {
    focus_window_inner(hwnd_val, false)
}

fn focus_window_inner(hwnd_val: isize, click: bool) -> Result<(), String> {
    unsafe {
        let hwnd = HWND(hwnd_val as *mut _);

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            sleep(Duration::from_millis(500));
        }

        let _ = SetForegroundWindow(hwnd);
        sleep(Duration::from_millis(300));

        if click {
            let mut rect = windows::Win32::Foundation::RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                let cx = (rect.left + rect.right) / 2;
                let cy = (rect.top + rect.bottom) / 2;
                click_at(cx, cy);
                sleep(Duration::from_millis(500));
            }
        }
    }

    Ok(())
}

/// Click on the character's position (screen coordinates)
pub fn click_character(screen_x: i32, screen_y: i32) {
    click_at(screen_x, screen_y);
}

/// Click the "Suivant" button at fixed position relative to window
pub fn click_suivant(window_x: i32, window_y: i32, window_height: i32) {
    let btn_x = window_x + 70;
    let btn_y = window_y + window_height - 50;
    sleep(Duration::from_secs(1));
    click_at(btn_x, btn_y);
}

/// Hold a key for a custom duration (ms), then release.
/// Used by the muscu bot for 100ms movement holds.
pub fn send_key_hold(scan_code: u16, hold_ms: u64) {
    // Key DOWN
    let down = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];

    // Safety: SendInput is a standard Win32 API call with a well-formed INPUT array
    unsafe {
        SendInput(&down, std::mem::size_of::<INPUT>() as i32);
    }

    sleep(Duration::from_millis(hold_ms));

    // Key UP
    let up = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];

    // Safety: same as above
    unsafe {
        SendInput(&up, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Quick key press (down + up with minimal hold).
/// Used by the muscu bot for the E interaction key.
pub fn send_key_press(scan_code: u16) {
    send_key_inner(scan_code, false);
}

/// Send the AFK activation key sequence.
/// Uses scan codes with proper extended key flags for DirectInput compatibility.
pub fn send_afk_sequence() {
    log::info!("Sending AFK key sequence (DirectInput scan codes)...");

    // F5 — open menu
    send_key(SC_F5);
    sleep(Duration::from_millis(800));

    // Down ×4 — extended key!
    for i in 0..4 {
        send_key_extended(SC_DOWN);
        sleep(Duration::from_millis(300));
        log::debug!("Down arrow {} sent", i + 1);
    }

    // Enter — validate
    send_key(SC_ENTER);
    sleep(Duration::from_millis(800));

    // Down ×3 — extended key!
    for i in 0..3 {
        send_key_extended(SC_DOWN);
        sleep(Duration::from_millis(300));
        log::debug!("Down arrow {} sent", i + 1);
    }

    // Enter — activate AFK
    send_key(SC_ENTER);

    log::info!("AFK key sequence completed");
}
