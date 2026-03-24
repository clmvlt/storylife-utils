use crate::automation::input_sender::{self, SC_D, SC_E, SC_Q, SC_S, SC_Z};
use crate::automation::window_finder;
use crate::state::{MuscuStatus, MuscuStopSignal, SharedMuscuStatus, SharedStats};
use chrono::Local;
use rand::Rng;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

fn emit_status(app: &AppHandle, status: &MuscuStatus) {
    let _ = app.emit("muscu-status", status);
}

fn emit_log(app: &AppHandle, level: &str, message: &str) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit(
        "muscu-log",
        serde_json::json!({
            "level": level,
            "message": message,
            "timestamp": timestamp
        }),
    );
    match level {
        "error" => log::error!("[muscu] {}", message),
        "warn" => log::warn!("[muscu] {}", message),
        _ => log::info!("[muscu] {}", message),
    }
}

/// Interruptible sleep — checks stop signal every 200ms for reactive stopping.
/// Returns true if stop was requested.
async fn interruptible_sleep_ms(stop_signal: &MuscuStopSignal, total_ms: u64) -> bool {
    let chunks = total_ms / 200;
    let remainder = total_ms % 200;

    for _ in 0..chunks {
        {
            let should_stop = stop_signal.lock().await;
            if *should_stop {
                return true;
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    if remainder > 0 {
        {
            let should_stop = stop_signal.lock().await;
            if *should_stop {
                return true;
            }
        }
        sleep(Duration::from_millis(remainder)).await;
    }

    false
}

async fn should_stop(stop_signal: &MuscuStopSignal) -> bool {
    let s = stop_signal.lock().await;
    *s
}

/// Map scancode to key name for logging
fn key_name(scan_code: u16) -> &'static str {
    match scan_code {
        SC_Z => "Z (avant)",
        SC_Q => "Q (gauche)",
        SC_S => "S (arrière)",
        SC_D => "D (droite)",
        _ => "?",
    }
}

pub async fn run_muscu_loop(
    app: AppHandle,
    status: SharedMuscuStatus,
    stop_signal: MuscuStopSignal,
) {
    emit_log(&app, "info", "Muscu bot démarré !");
    super::config::webhook_notify(&app, "muscu_start", "Le bot Muscu AFK a démarré.").await;

    let movement_keys = [SC_Z, SC_Q, SC_S, SC_D];

    loop {
        if should_stop(&stop_signal).await {
            break;
        }

        // 0. Focus FiveM window before each cycle
        match window_finder::find_fivem_window() {
            Some(info) => {
                if let Err(e) = input_sender::focus_window_no_click(info.hwnd) {
                    emit_log(&app, "warn", &format!("Focus échoué: {}", e));
                } else {
                    emit_log(&app, "info", "Fenêtre FiveM focus OK");
                }
            }
            None => {
                emit_log(&app, "warn", "Fenêtre FiveM non trouvée — cycle ignoré");
                if interruptible_sleep_ms(&stop_signal, 5000).await {
                    break;
                }
                continue;
            }
        }

        // 1. Random movement key — hold 100ms
        let key = {
            let mut rng = rand::rng();
            movement_keys[rng.random_range(0..movement_keys.len())]
        };

        input_sender::send_key_hold(key, 100);
        emit_log(
            &app,
            "info",
            &format!("Mouvement : {}", key_name(key)),
        );

        // 2. Stabilization pause — 500ms
        if interruptible_sleep_ms(&stop_signal, 500).await {
            break;
        }

        // 3. Press E — interaction (start gym exercise)
        input_sender::send_key_press(SC_E);
        emit_log(&app, "info", "Appui E (interaction muscu)");

        // 4. Wait for exercise animation — 77s cycle between each E press
        // ~600ms already elapsed (100ms key hold + 500ms stabilization)
        // So we wait ~76.4s + random 0-1s to reach ~77s total
        let random_extra: u64 = {
            let mut rng = rand::rng();
            rng.random_range(0..1000)
        };
        let wait_ms = 76_400 + random_extra;

        emit_log(
            &app,
            "info",
            &format!("Attente animation ({:.1}s)...", wait_ms as f64 / 1000.0),
        );

        if interruptible_sleep_ms(&stop_signal, wait_ms).await {
            break;
        }

        // 5. Increment cycle count, update persistent stats, and emit status
        {
            let mut s = status.lock().await;
            s.cycle_count += 1;
            emit_status(&app, &s);
            emit_log(
                &app,
                "info",
                &format!("Cycle {} terminé", s.cycle_count),
            );
        }
        {
            let stats_state = app.state::<SharedStats>();
            let mut stats = stats_state.lock().await;
            stats.total_muscu_cycles += 1;
            let _ = super::config::save_stats_to_disk(&stats);
        }
    }

    let cycle_count = {
        let s = status.lock().await;
        s.cycle_count
    };
    super::config::webhook_notify(
        &app,
        "muscu_stop",
        &format!("Bot Muscu arrêté après **{}** cycles.", cycle_count),
    )
    .await;

    emit_log(&app, "info", "Muscu bot arrêté");
    let mut s = status.lock().await;
    s.running = false;
    emit_status(&app, &s);
}

#[tauri::command]
pub async fn start_muscu(
    app: AppHandle,
    status: tauri::State<'_, SharedMuscuStatus>,
    stop_signal: tauri::State<'_, MuscuStopSignal>,
) -> Result<(), String> {
    // Check if already running
    {
        let s = status.lock().await;
        if s.running {
            return Err("Le bot muscu est déjà en cours d'exécution".to_string());
        }
    }

    // Reset stop signal
    {
        let mut stop = stop_signal.lock().await;
        *stop = false;
    }

    // Update status
    {
        let mut s = status.lock().await;
        s.running = true;
        s.cycle_count = 0;
        emit_status(&app, &s);
    }

    let status_clone = status.inner().clone();
    let stop_clone = stop_signal.inner().clone();

    tokio::spawn(async move {
        run_muscu_loop(app, status_clone, stop_clone).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_muscu(
    app: AppHandle,
    status: tauri::State<'_, SharedMuscuStatus>,
    stop_signal: tauri::State<'_, MuscuStopSignal>,
) -> Result<(), String> {
    {
        let mut stop = stop_signal.lock().await;
        *stop = true;
    }

    // Immediate UI update
    {
        let mut s = status.lock().await;
        s.running = false;
        emit_status(&app, &s);
    }

    emit_log(&app, "info", "Arrêt du bot muscu demandé...");

    Ok(())
}

#[tauri::command]
pub async fn get_muscu_status(
    status: tauri::State<'_, SharedMuscuStatus>,
) -> Result<MuscuStatus, String> {
    let s = status.lock().await;
    Ok(s.clone())
}
