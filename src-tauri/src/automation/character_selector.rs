use crate::automation::ocr::{find_character_in_image, OcrMatch};
use crate::automation::screen_capture::capture_window;
use crate::state::WindowInfo;
use std::thread::sleep;
use std::time::Duration;

/// Try to find the character in the FiveM window via OCR.
/// Returns the match if found on two consecutive captures (double verification).
pub fn detect_character(
    window_info: &WindowInfo,
    character_name: &str,
) -> Result<OcrMatch, String> {
    // First capture
    let img1 = capture_window(window_info)?;
    let match1 = find_character_in_image(&img1, character_name);

    if match1.is_none() {
        return Err("Character not found in first capture".to_string());
    }

    let m1 = match1.unwrap();
    log::info!("First detection: '{}' at ({:.0}, {:.0})", m1.text, m1.x, m1.y);

    // Wait 500ms for second capture
    sleep(Duration::from_millis(500));

    // Second capture
    let img2 = capture_window(window_info)?;
    let match2 = find_character_in_image(&img2, character_name);

    if match2.is_none() {
        return Err("Character not found in second capture (double verification failed)".to_string());
    }

    let m2 = match2.unwrap();
    log::info!("Second detection confirmed: '{}' at ({:.0}, {:.0})", m2.text, m2.x, m2.y);

    // Use average position from both detections
    Ok(OcrMatch {
        text: m2.text,
        confidence: m2.confidence,
        x: (m1.x + m2.x) / 2.0,
        y: (m1.y + m2.y) / 2.0,
        width: (m1.width + m2.width) / 2.0,
        height: (m1.height + m2.height) / 2.0,
    })
}
