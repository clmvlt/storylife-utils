use crate::automation::{
    character_selector, input_sender, process_manager, window_finder,
};
use crate::state::{AutomationState, AutomationStatus, SharedStats, SharedStatus, StopSignal, WindowInfo};
use chrono::Local;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

fn emit_status(app: &AppHandle, status: &AutomationStatus) {
    let _ = app.emit("automation:status", status);
}

fn emit_log(app: &AppHandle, level: &str, message: &str) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit(
        "automation:log",
        serde_json::json!({
            "level": level,
            "message": message,
            "timestamp": timestamp
        }),
    );
    match level {
        "error" => log::error!("{}", message),
        "warn" => log::warn!("{}", message),
        _ => log::info!("{}", message),
    }
}

fn emit_window(app: &AppHandle, info: &WindowInfo) {
    let _ = app.emit("automation:window", info);
}

async fn update_state(status: &SharedStatus, app: &AppHandle, new_state: AutomationState) {
    let mut s = status.lock().await;
    s.state = new_state;
    emit_status(app, &s);
}

/// Interruptible sleep — checks stop signal every second.
/// Returns true if stop was requested.
async fn interruptible_sleep(stop_signal: &StopSignal, secs: u64) -> bool {
    for _ in 0..secs {
        {
            let should_stop = stop_signal.lock().await;
            if *should_stop {
                return true;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    false
}

/// Check if stop was requested
async fn should_stop(stop_signal: &StopSignal) -> bool {
    let s = stop_signal.lock().await;
    *s
}

/// Save elapsed AFK time to persistent stats
async fn save_afk_elapsed(app: &AppHandle, status: &SharedStatus) {
    let afk_start = {
        let s = status.lock().await;
        if !s.session_treated {
            return;
        }
        s.afk_start_time.clone()
    };

    if let Some(start_str) = afk_start {
        if let Ok(start) =
            chrono::NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%d %H:%M:%S")
        {
            let now = chrono::Local::now().naive_local();
            let elapsed = (now - start).num_seconds().max(0) as u64;
            if elapsed > 0 {
                let stats_state = app.state::<SharedStats>();
                let mut stats = stats_state.lock().await;
                stats.total_afk_seconds += elapsed;
                let _ = super::config::save_stats_to_disk(&stats);
            }
        }
    }
}

/// Clean exit: save AFK time, send webhook, set state to idle
async fn clean_stop(app: &AppHandle, status: &SharedStatus) {
    let was_afk = {
        let s = status.lock().await;
        s.session_treated && s.afk_start_time.is_some()
    };

    save_afk_elapsed(app, status).await;

    if was_afk {
        super::config::webhook_notify(app, "afk_stop", "L'automatisation AFK a été arrêtée.").await;
    }

    let mut s = status.lock().await;
    s.state = AutomationState::Idle;
    s.running = false;
    emit_status(app, &s);
}

pub async fn run_automation_loop(
    app: AppHandle,
    character_name: String,
    status: SharedStatus,
    stop_signal: StopSignal,
) {
    emit_log(&app, "info", "Automation démarrée");

    loop {
        // Check stop
        if should_stop(&stop_signal).await {
            emit_log(&app, "info", "Automation arrêtée par l'utilisateur");
            clean_stop(&app, &status).await;
            return;
        }

        // Search for FiveM window
        update_state(&status, &app, AutomationState::SearchingWindow).await;
        let window_info = window_finder::find_fivem_window();

        match window_info {
            Some(info) => {
                emit_window(&app, &info);
                emit_log(
                    &app,
                    "info",
                    &format!(
                        "Fenêtre FiveM trouvée: {} ({}x{})",
                        info.title, info.width, info.height
                    ),
                );

                let session_treated = {
                    let s = status.lock().await;
                    s.session_treated
                };

                if !session_treated {
                    // --- Character selection via OCR ---
                    update_state(&status, &app, AutomationState::WaitingOcr).await;
                    emit_log(&app, "info", "Recherche du personnage via OCR...");

                    let mut ocr_found = false;
                    let max_attempts = 60; // 60 * 5s = 300s

                    for attempt in 0..max_attempts {
                        if should_stop(&stop_signal).await {
                            emit_log(&app, "info", "Automation arrêtée par l'utilisateur");
                            clean_stop(&app, &status).await;
                            return;
                        }

                        match character_selector::detect_character(&info, &character_name) {
                            Ok(ocr_match) => {
                                emit_log(
                                    &app,
                                    "info",
                                    &format!(
                                        "Personnage '{}' détecté (double vérification OK)",
                                        ocr_match.text
                                    ),
                                );

                                update_state(
                                    &status,
                                    &app,
                                    AutomationState::SelectingCharacter,
                                )
                                .await;

                                let screen_x =
                                    info.x + ocr_match.x as i32 + (ocr_match.width / 2.0) as i32;
                                let screen_y =
                                    info.y + ocr_match.y as i32 + (ocr_match.height / 2.0) as i32;

                                input_sender::click_character(screen_x, screen_y);
                                emit_log(&app, "info", "Clic sur le personnage effectué");

                                sleep(Duration::from_secs(1)).await;

                                input_sender::click_suivant(info.x, info.y, info.height);
                                emit_log(&app, "info", "Clic sur 'Suivant' effectué");

                                sleep(Duration::from_secs(2)).await;

                                ocr_found = true;
                                break;
                            }
                            Err(e) => {
                                if attempt % 6 == 0 {
                                    emit_log(
                                        &app,
                                        "warn",
                                        &format!(
                                            "OCR tentative {}/{}: {}",
                                            attempt + 1,
                                            max_attempts,
                                            e
                                        ),
                                    );
                                }
                                if interruptible_sleep(&stop_signal, 5).await {
                                    emit_log(
                                        &app,
                                        "info",
                                        "Automation arrêtée par l'utilisateur",
                                    );
                                    clean_stop(&app, &status).await;
                                    return;
                                }
                            }
                        }
                    }

                    if !ocr_found {
                        emit_log(&app, "error", "Timeout: personnage non trouvé après 300s");
                        if interruptible_sleep(&stop_signal, 5).await {
                            clean_stop(&app, &status).await;
                            return;
                        }
                        continue;
                    }

                    // --- AFK activation ---
                    update_state(&status, &app, AutomationState::SendingKeys).await;

                    if let Err(e) = input_sender::focus_window(info.hwnd) {
                        emit_log(&app, "error", &format!("Focus échoué: {}", e));
                    }

                    sleep(Duration::from_secs(1)).await;

                    input_sender::send_afk_sequence();
                    emit_log(&app, "info", "Séquence AFK envoyée avec succès");

                    {
                        let mut s = status.lock().await;
                        s.session_treated = true;
                        s.state = AutomationState::AfkActive;
                        s.afk_start_time =
                            Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        emit_status(&app, &s);
                    }
                    emit_log(&app, "info", "Mode AFK actif !");
                    super::config::webhook_notify(
                        &app,
                        "afk_start",
                        &format!("Personnage **{}** connecté, AFK activé.", character_name),
                    )
                    .await;
                } else {
                    update_state(&status, &app, AutomationState::AfkActive).await;
                }
            }
            None => {
                emit_window(&app, &WindowInfo::default());

                let session_treated = {
                    let s = status.lock().await;
                    s.session_treated
                };

                if !session_treated {
                    update_state(&status, &app, AutomationState::LaunchingFivem).await;
                    emit_log(&app, "info", "Aucune fenêtre FiveM trouvée. Lancement...");

                    match process_manager::kill_fivem_processes() {
                        Ok(killed) => {
                            if killed > 0 {
                                emit_log(
                                    &app,
                                    "info",
                                    &format!("{} processus FiveM tués", killed),
                                );
                            }
                        }
                        Err(e) => emit_log(&app, "error", &format!("Erreur kill: {}", e)),
                    }

                    if interruptible_sleep(&stop_signal, 5).await {
                        clean_stop(&app, &status).await;
                        return;
                    }

                    match process_manager::launch_fivem() {
                        Ok(()) => emit_log(&app, "info", "FiveM lancé via protocole URI"),
                        Err(e) => {
                            emit_log(&app, "error", &format!("Erreur lancement: {}", e));
                            if interruptible_sleep(&stop_signal, 5).await {
                                clean_stop(&app, &status).await;
                                return;
                            }
                            continue;
                        }
                    }

                    emit_log(&app, "info", "Attente du démarrage de FiveM (60s)...");
                    if interruptible_sleep(&stop_signal, 60).await {
                        emit_log(&app, "info", "Automation arrêtée pendant l'attente");
                        clean_stop(&app, &status).await;
                        return;
                    }
                } else {
                    // Crash detected — save AFK time before resetting
                    save_afk_elapsed(&app, &status).await;

                    update_state(&status, &app, AutomationState::Reconnecting).await;
                    emit_log(&app, "warn", "Fenêtre FiveM disparue — crash détecté !");

                    let reconnect_n = {
                        let mut s = status.lock().await;
                        s.reconnect_count += 1;
                        s.last_crash_time =
                            Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        s.session_treated = false;
                        emit_status(&app, &s);
                        s.reconnect_count
                    };

                    super::config::webhook_notify(
                        &app,
                        "crash",
                        &format!("FiveM a crashé. Reconnexion #{} en cours...", reconnect_n),
                    )
                    .await;

                    let _ = process_manager::kill_fivem_processes();
                    emit_log(
                        &app,
                        "info",
                        "Processus FiveM tués. Attente 60s avant relancement...",
                    );

                    if interruptible_sleep(&stop_signal, 60).await {
                        emit_log(&app, "info", "Automation arrêtée pendant la reconnexion");
                        clean_stop(&app, &status).await;
                        return;
                    }

                    match process_manager::launch_fivem() {
                        Ok(()) => emit_log(&app, "info", "FiveM relancé"),
                        Err(e) => {
                            emit_log(&app, "error", &format!("Erreur relancement: {}", e))
                        }
                    }

                    emit_log(&app, "info", "Attente du démarrage (60s)...");
                    if interruptible_sleep(&stop_signal, 60).await {
                        emit_log(&app, "info", "Automation arrêtée pendant l'attente");
                        clean_stop(&app, &status).await;
                        return;
                    }
                }
            }
        }

        if interruptible_sleep(&stop_signal, 5).await {
            emit_log(&app, "info", "Automation arrêtée par l'utilisateur");
            clean_stop(&app, &status).await;
            return;
        }
    }
}

#[tauri::command]
pub async fn start_automation(
    character_name: String,
    app: AppHandle,
    status: tauri::State<'_, SharedStatus>,
    stop_signal: tauri::State<'_, StopSignal>,
) -> Result<(), String> {
    if character_name.trim().is_empty() {
        return Err("Le nom du personnage est requis".to_string());
    }

    // Reset stop signal
    {
        let mut stop = stop_signal.lock().await;
        *stop = false;
    }

    // Update status & emit to frontend
    {
        let mut s = status.lock().await;
        s.running = true;
        s.state = AutomationState::SearchingWindow;
        s.session_treated = false;
        s.reconnect_count = 0;
        s.afk_start_time = None;
        s.last_crash_time = None;
        emit_status(&app, &s);
    }

    let status_clone = status.inner().clone();
    let stop_clone = stop_signal.inner().clone();

    tokio::spawn(async move {
        run_automation_loop(app, character_name, status_clone, stop_clone).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_automation(
    app: AppHandle,
    status: tauri::State<'_, SharedStatus>,
    stop_signal: tauri::State<'_, StopSignal>,
) -> Result<(), String> {
    // Set stop signal
    {
        let mut stop = stop_signal.lock().await;
        *stop = true;
    }

    // Update status & emit immediately to frontend
    {
        let mut s = status.lock().await;
        s.running = false;
        s.state = AutomationState::Idle;
        emit_status(&app, &s);
    }

    emit_log(&app, "info", "Arrêt demandé...");

    Ok(())
}

#[tauri::command]
pub async fn get_status(
    status: tauri::State<'_, SharedStatus>,
) -> Result<AutomationStatus, String> {
    let s = status.lock().await;
    Ok(s.clone())
}
