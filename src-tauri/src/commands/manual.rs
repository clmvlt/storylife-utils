use crate::automation::{
    input_sender, ocr, process_manager, screen_capture, window_finder,
};
use crate::state::WindowInfo;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

fn emit_log(app: &AppHandle, level: &str, message: &str) {
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit(
        "automation:log",
        serde_json::json!({ "level": level, "message": message, "timestamp": timestamp }),
    );
}

// ── Step 1: Kill FiveM processes ──

#[tauri::command]
pub fn manual_kill_fivem(app: AppHandle) -> Result<String, String> {
    emit_log(&app, "info", "[Manuel] Kill des processus FiveM...");
    match process_manager::kill_fivem_processes() {
        Ok(killed) => {
            let msg = format!("{} processus tué(s)", killed);
            emit_log(&app, "info", &format!("[Manuel] {}", msg));
            Ok(msg)
        }
        Err(e) => {
            emit_log(&app, "error", &format!("[Manuel] Erreur kill: {}", e));
            Err(e)
        }
    }
}

// ── Step 2: Launch FiveM ──

#[tauri::command]
pub fn manual_launch_fivem(app: AppHandle) -> Result<String, String> {
    emit_log(&app, "info", "[Manuel] Lancement de FiveM (StoryLife)...");
    match process_manager::launch_fivem() {
        Ok(()) => {
            emit_log(&app, "info", "[Manuel] FiveM lancé via fivem://connect/cfx.re/join/aaex7k");
            Ok("FiveM lancé".to_string())
        }
        Err(e) => {
            emit_log(&app, "error", &format!("[Manuel] Erreur lancement: {}", e));
            Err(e)
        }
    }
}

// ── Step 3: Find FiveM window ──

#[tauri::command]
pub fn manual_find_window(app: AppHandle) -> Result<WindowInfo, String> {
    emit_log(&app, "info", "[Manuel] Recherche de la fenêtre FiveM...");
    match window_finder::find_fivem_window() {
        Some(info) => {
            emit_log(
                &app,
                "info",
                &format!(
                    "[Manuel] Fenêtre trouvée: '{}' ({}x{}) à ({},{})",
                    info.title, info.width, info.height, info.x, info.y
                ),
            );
            let _ = app.emit("automation:window", &info);
            Ok(info)
        }
        None => {
            emit_log(&app, "warn", "[Manuel] Aucune fenêtre FiveM trouvée");
            Err("Aucune fenêtre FiveM trouvée".to_string())
        }
    }
}

// ── Step 4: OCR detect character ──

#[derive(Serialize, Clone)]
pub struct OcrResult {
    pub found: bool,
    pub text: String,
    pub score: f64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub fn manual_ocr_detect(
    app: AppHandle,
    character_name: String,
) -> Result<OcrResult, String> {
    emit_log(&app, "info", &format!("[Manuel] OCR — recherche de '{}'...", character_name));

    let info = window_finder::find_fivem_window()
        .ok_or("Fenêtre FiveM non trouvée — lancez l'étape 3 d'abord")?;

    let img = screen_capture::capture_window(&info)?;
    emit_log(&app, "info", &format!("[Manuel] Capture effectuée ({}x{})", img.width(), img.height()));

    match ocr::find_character_in_image(&img, &character_name) {
        Some(m) => {
            emit_log(
                &app,
                "info",
                &format!(
                    "[Manuel] OCR match: '{}' (score={:.0}%) à ({:.0},{:.0})",
                    m.text,
                    m.confidence * 100.0,
                    m.x,
                    m.y
                ),
            );
            Ok(OcrResult {
                found: true,
                text: m.text,
                score: m.confidence,
                x: m.x,
                y: m.y,
                width: m.width,
                height: m.height,
            })
        }
        None => {
            emit_log(&app, "warn", "[Manuel] OCR: personnage non trouvé dans cette capture");
            Ok(OcrResult {
                found: false,
                text: String::new(),
                score: 0.0,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            })
        }
    }
}

// ── Step 5: Click character ──

#[tauri::command]
pub fn manual_click_character(
    app: AppHandle,
    window_x: i32,
    window_y: i32,
    ocr_x: f64,
    ocr_y: f64,
    ocr_w: f64,
    ocr_h: f64,
) -> Result<String, String> {
    let screen_x = window_x + ocr_x as i32 + (ocr_w / 2.0) as i32;
    let screen_y = window_y + ocr_y as i32 + (ocr_h / 2.0) as i32;
    emit_log(
        &app,
        "info",
        &format!("[Manuel] Clic personnage à ({}, {})", screen_x, screen_y),
    );
    input_sender::click_character(screen_x, screen_y);
    Ok(format!("Clic effectué à ({}, {})", screen_x, screen_y))
}

// ── Step 6: Click Suivant ──

#[tauri::command]
pub fn manual_click_suivant(
    app: AppHandle,
    window_x: i32,
    window_y: i32,
    window_height: i32,
) -> Result<String, String> {
    let btn_x = window_x + 70;
    let btn_y = window_y + window_height - 50;
    emit_log(
        &app,
        "info",
        &format!("[Manuel] Clic 'Suivant' à ({}, {})", btn_x, btn_y),
    );
    input_sender::click_suivant(window_x, window_y, window_height);
    Ok(format!("Clic Suivant à ({}, {})", btn_x, btn_y))
}

// ── Step 7: Focus window ──

#[tauri::command]
pub fn manual_focus_window(app: AppHandle) -> Result<String, String> {
    emit_log(&app, "info", "[Manuel] Focus de la fenêtre FiveM...");
    let info = window_finder::find_fivem_window()
        .ok_or("Fenêtre FiveM non trouvée")?;
    input_sender::focus_window(info.hwnd)?;
    emit_log(&app, "info", "[Manuel] Fenêtre focalisée");
    Ok("Fenêtre focalisée".to_string())
}

// ── Step 8: Send AFK keys ──

#[tauri::command]
pub fn manual_send_afk_keys(app: AppHandle) -> Result<String, String> {
    emit_log(&app, "info", "[Manuel] Envoi de la séquence AFK (F5 → 4×Down → Enter → 3×Down → Enter)...");

    // Focus first
    if let Some(info) = window_finder::find_fivem_window() {
        let _ = input_sender::focus_window(info.hwnd);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    input_sender::send_afk_sequence();
    emit_log(&app, "info", "[Manuel] Séquence AFK envoyée avec succès");
    Ok("Séquence AFK envoyée".to_string())
}
