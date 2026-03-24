use crate::state::WindowInfo;
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetDC,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

// ── Screen capture ──

/// Capture a rectangular zone from the screen using BitBlt.
/// Returns raw BGRA pixel data.
pub fn capture_screen_zone(left: i32, top: i32, width: i32, height: i32) -> Result<Vec<u8>, String> {
    if width <= 0 || height <= 0 {
        return Err("Invalid zone dimensions".to_string());
    }

    // Safety: all Win32 GDI calls use properly formed parameters and resources
    // are released in reverse allocation order.
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err("Failed to get screen DC".to_string());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            ReleaseDC(None, hdc_screen);
            return Err("Failed to create compatible DC".to_string());
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("Failed to create bitmap".to_string());
        }

        let old = SelectObject(hdc_mem, hbm.into());

        let result = BitBlt(hdc_mem, 0, 0, width, height, Some(hdc_screen), left, top, SRCCOPY);
        if result.is_err() {
            SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbm.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("BitBlt failed".to_string());
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let lines = GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup in reverse allocation order
        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        if lines == 0 {
            return Err("GetDIBits failed".to_string());
        }

        Ok(pixels)
    }
}

// ── Watch zone ──

/// Calculate the central watch zone with margins removed.
/// Returns (left, top, width, height) in screen coordinates.
pub fn get_watch_zone(window: &WindowInfo, margin: f64) -> (i32, i32, i32, i32) {
    let margin_x = (window.width as f64 * margin) as i32;
    let margin_y = (window.height as f64 * margin) as i32;
    (
        window.x + margin_x,
        window.y + margin_y,
        window.width - 2 * margin_x,
        window.height - 2 * margin_y,
    )
}

// ── Color detection ──

/// Scan BGRA pixel buffer for a target RGB color within tolerance.
/// Returns Some((x, y, match_count)) for the first match (top→bottom, left→right), or None.
pub fn find_color_pixel(
    pixels: &[u8],
    width: i32,
    height: i32,
    target: [u8; 3],
    tolerance: u8,
) -> Option<(i32, i32, u32)> {
    let tol = tolerance as i16;
    let tr = target[0] as i16;
    let tg = target[1] as i16;
    let tb = target[2] as i16;

    let mut first: Option<(i32, i32)> = None;
    let mut count: u32 = 0;

    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            // BGRA byte order
            let b = pixels[i] as i16;
            let g = pixels[i + 1] as i16;
            let r = pixels[i + 2] as i16;

            if (r - tr).abs() <= tol && (g - tg).abs() <= tol && (b - tb).abs() <= tol {
                count += 1;
                if first.is_none() {
                    first = Some((x, y));
                }
            }
        }
    }

    first.map(|(x, y)| (x, y, count))
}

// ── Mouse helpers ──

/// Get current cursor position in screen coordinates.
pub fn get_cursor_position() -> (i32, i32) {
    // Safety: GetCursorPos is a standard Win32 call that writes to a valid POINT struct
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        (pt.x, pt.y)
    }
}

/// Move mouse to absolute screen coordinates via SendInput.
fn move_mouse_to(x: i32, y: i32) {
    // Safety: GetSystemMetrics reads display dimensions, no side effects
    let (sw, sh) = unsafe {
        (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
    };

    let abs_x = (x as f64 / sw as f64 * 65535.0) as i32;
    let abs_y = (y as f64 / sh as f64 * 65535.0) as i32;

    let input = [INPUT {
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

    // Safety: SendInput with a well-formed MOUSEINPUT for absolute positioning
    unsafe {
        SendInput(&input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Left-click at the current cursor position.
fn click_mouse() {
    let down = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: MOUSEEVENTF_LEFTDOWN,
                ..Default::default()
            },
        },
    }];

    // Safety: SendInput with a well-formed MOUSEINPUT for left button down
    unsafe {
        SendInput(&down, std::mem::size_of::<INPUT>() as i32);
    }

    sleep(Duration::from_millis(40));

    let up = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: MOUSEEVENTF_LEFTUP,
                ..Default::default()
            },
        },
    }];

    // Safety: SendInput with a well-formed MOUSEINPUT for left button up
    unsafe {
        SendInput(&up, std::mem::size_of::<INPUT>() as i32);
    }
}

// ── Bézier movement ──

/// Cubic Bézier interpolation at parameter t.
fn bezier(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Hermite smoothstep easing — accelerates then decelerates.
fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Move mouse along a cubic Bézier curve then click at the target.
/// Simulates human-like movement with adaptive speed and randomized control points.
pub fn bezier_move_and_click(
    target_x: i32,
    target_y: i32,
    min_delay: f64,
    max_delay: f64,
    max_distance: f64,
) {
    let (cx, cy) = get_cursor_position();
    let (fx, fy) = (cx as f64, cy as f64);
    let (tx, ty) = (target_x as f64, target_y as f64);

    let dx = tx - fx;
    let dy = ty - fy;
    let distance = (dx * dx + dy * dy).sqrt();

    let mut rng = rand::rng();

    // Adaptive delay: linear interpolation based on distance
    let ratio = (distance / max_distance).min(1.0);
    let base_delay = min_delay + (max_delay - min_delay) * ratio;
    // ±15% random variation
    let variation = (rng.random_range(0..31) as f64 - 15.0) / 100.0;
    let total_delay = base_delay * (1.0 + variation);

    // Bézier control points at 30% and 70% with ±15% deviation
    let dev1 = (rng.random_range(0..31) as f64 - 15.0) / 100.0 * distance;
    let dev2 = (rng.random_range(0..31) as f64 - 15.0) / 100.0 * distance;

    let p1x = fx + dx * 0.3 + dev1;
    let p1y = fy + dy * 0.3 + dev1;
    let p2x = fx + dx * 0.7 + dev2;
    let p2y = fy + dy * 0.7 + dev2;

    // Steps: max(distance / 5, 10)
    let steps = ((distance / 5.0).ceil() as u32).max(10);

    // 75% of total delay for movement
    let move_delay = total_delay * 0.75;
    let step_delay = move_delay / steps as f64;

    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let te = smoothstep(t);
        let px = bezier(te, fx, p1x, p2x, tx) as i32;
        let py = bezier(te, fy, p1y, p2y, ty) as i32;
        move_mouse_to(px, py);
        sleep(Duration::from_secs_f64(step_delay));
    }

    // Random pre-click pause: 10-30ms
    let pre_click: u64 = rng.random_range(10..31);
    sleep(Duration::from_millis(pre_click));

    click_mouse();

    // 20% of delay after click
    sleep(Duration::from_secs_f64(total_delay * 0.20));
}

// ── Keyboard helpers ──

/// Check if a key was just pressed (transition from up to down).
/// Pass the same `was_pressed` ref each call to track state across iterations.
pub fn is_key_just_pressed(vk_code: i32, was_pressed: &mut bool) -> bool {
    // Safety: GetAsyncKeyState reads the current key state, no side effects
    let state = unsafe { GetAsyncKeyState(vk_code) };
    let is_down = (state & (1i16 << 15)) != 0;
    let just_pressed = is_down && !*was_pressed;
    *was_pressed = is_down;
    just_pressed
}

/// Convert a key name string to its Windows virtual key code.
pub fn key_name_to_vk(name: &str) -> Option<i32> {
    let upper = name.trim().to_uppercase();
    match upper.as_str() {
        s if s.len() == 1 => {
            let c = s.chars().next()?;
            if c.is_ascii_alphabetic() {
                Some(0x41 + (c.to_ascii_uppercase() as i32 - 'A' as i32))
            } else if c.is_ascii_digit() {
                Some(0x30 + (c as i32 - '0' as i32))
            } else {
                None
            }
        }
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        "SPACE" | "ESPACE" => Some(0x20),
        "TAB" => Some(0x09),
        _ => None,
    }
}
