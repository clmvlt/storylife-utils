use crate::automation::mining;
use crate::automation::window_finder;
use crate::state::{MiningConfig, MiningStatus, MiningStopSignal, SharedMiningStatus, SharedStats};
use chrono::Local;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

const VK_ESCAPE: i32 = 0x1B;
const ESC_COUNT_TO_QUIT: usize = 3;
const ESC_TIME_WINDOW: f64 = 1.0;

fn emit_status(app: &AppHandle, status: &MiningStatus) {
    let _ = app.emit("mining-status", status);
}

fn emit_log(app: &AppHandle, level: &str, message: &str) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit(
        "mining-log",
        serde_json::json!({
            "level": level,
            "message": message,
            "timestamp": timestamp
        }),
    );
    match level {
        "error" => log::error!("[mining] {}", message),
        "warn" => log::warn!("[mining] {}", message),
        _ => log::info!("[mining] {}", message),
    }
}

fn config_path() -> Result<std::path::PathBuf, String> {
    let dir = dirs::config_dir().ok_or("Could not determine config directory")?;
    Ok(dir.join("StoryLifeUtils").join("mining_config.json"))
}

pub async fn run_mining_loop(
    app: AppHandle,
    status: SharedMiningStatus,
    stop_signal: MiningStopSignal,
    config: MiningConfig,
) {
    emit_log(&app, "info", "Mining bot démarré");
    super::config::webhook_notify(&app, "mining_start", "Le bot Mining a démarré.").await;

    let toggle_vk = mining::key_name_to_vk(&config.toggle_key).unwrap_or(0x4D); // M default

    emit_log(
        &app,
        "info",
        &format!(
            "Couleur=({},{},{}), tolérance={}, marge={:.0}%, délai={:.0}-{:.0}ms, touche={}",
            config.target_color[0],
            config.target_color[1],
            config.target_color[2],
            config.tolerance,
            config.margin * 100.0,
            config.min_delay * 1000.0,
            config.max_delay * 1000.0,
            config.toggle_key,
        ),
    );

    let mut toggle_was_pressed = false;
    let mut esc_was_pressed = false;
    let mut esc_times: Vec<Instant> = Vec::new();
    let mut last_window_check = Instant::now() - Duration::from_secs(10); // force immediate check
    let mut window = None;
    let mut active = true;

    // Set initial active state
    {
        let mut s = status.lock().await;
        s.active = true;
        emit_status(&app, &s);
    }

    loop {
        // Check stop signal
        {
            let stop = stop_signal.lock().await;
            if *stop {
                break;
            }
        }

        // Check toggle key
        if mining::is_key_just_pressed(toggle_vk, &mut toggle_was_pressed) {
            active = !active;
            {
                let mut s = status.lock().await;
                s.active = active;
                emit_status(&app, &s);
            }
            emit_log(
                &app,
                "info",
                if active {
                    "Détection activée"
                } else {
                    "Détection en pause"
                },
            );
        }

        // Check triple-Escape to quit
        if mining::is_key_just_pressed(VK_ESCAPE, &mut esc_was_pressed) {
            let now = Instant::now();
            esc_times.push(now);
            esc_times.retain(|t| now.duration_since(*t).as_secs_f64() <= ESC_TIME_WINDOW);
            if esc_times.len() >= ESC_COUNT_TO_QUIT {
                emit_log(&app, "warn", "3×Échap détecté — arrêt du mining");
                break;
            }
        }

        // Re-find window every 2 seconds
        if last_window_check.elapsed() >= Duration::from_secs(2) {
            last_window_check = Instant::now();
            match window_finder::find_fivem_window() {
                Some(w) => {
                    if window.is_none() {
                        emit_log(
                            &app,
                            "info",
                            &format!("Fenêtre FiveM trouvée: {}x{}", w.width, w.height),
                        );
                    }
                    window = Some(w);
                }
                None => {
                    if window.is_some() {
                        emit_log(&app, "warn", "Fenêtre FiveM perdue");
                    }
                    window = None;
                }
            }
        }

        // No window → wait
        if window.is_none() {
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        // Paused → wait
        if !active {
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        let w = window.as_ref().unwrap();
        let (zl, zt, zw, zh) = mining::get_watch_zone(w, config.margin);

        if zw <= 0 || zh <= 0 {
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        // Capture → detect → click
        match mining::capture_screen_zone(zl, zt, zw, zh) {
            Ok(pixels) => {
                if let Some((x, y, count)) = mining::find_color_pixel(
                    &pixels,
                    zw,
                    zh,
                    config.target_color,
                    config.tolerance,
                ) {
                    let screen_x = zl + x;
                    let screen_y = zt + y;

                    {
                        let mut s = status.lock().await;
                        s.detection_count += 1;
                        emit_status(&app, &s);
                    }

                    emit_log(
                        &app,
                        "info",
                        &format!(
                            "Pixel détecté ({}, {}) — {} matchs",
                            screen_x, screen_y, count
                        ),
                    );

                    // Bézier move + click (blocking but short: 100-300ms)
                    mining::bezier_move_and_click(
                        screen_x,
                        screen_y,
                        config.min_delay,
                        config.max_delay,
                        config.max_distance,
                    );

                    {
                        let mut s = status.lock().await;
                        s.click_count += 1;
                        emit_status(&app, &s);
                    }
                    {
                        let stats_state = app.state::<SharedStats>();
                        let mut stats = stats_state.lock().await;
                        stats.total_mining_clicks += 1;
                        let _ = super::config::save_stats_to_disk(&stats);
                    }

                    emit_log(&app, "info", "Clic effectué");
                } else {
                    // No pixel found — short sleep to avoid burning CPU
                    sleep(Duration::from_millis(10)).await;
                }
            }
            Err(e) => {
                emit_log(&app, "error", &format!("Capture échouée: {}", e));
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    let click_count = {
        let s = status.lock().await;
        s.click_count
    };
    super::config::webhook_notify(
        &app,
        "mining_stop",
        &format!("Bot Mining arrêté après **{}** clics.", click_count),
    )
    .await;

    emit_log(&app, "info", "Mining bot arrêté");
    let mut s = status.lock().await;
    s.running = false;
    s.active = false;
    emit_status(&app, &s);
}

// ── Tauri commands ──

#[tauri::command]
pub async fn start_mining(
    app: AppHandle,
    config: MiningConfig,
    status: tauri::State<'_, SharedMiningStatus>,
    stop_signal: tauri::State<'_, MiningStopSignal>,
) -> Result<(), String> {
    {
        let s = status.lock().await;
        if s.running {
            return Err("Le bot mining est déjà en cours".to_string());
        }
    }

    {
        let mut stop = stop_signal.lock().await;
        *stop = false;
    }

    {
        let mut s = status.lock().await;
        s.running = true;
        s.active = true;
        s.click_count = 0;
        s.detection_count = 0;
        emit_status(&app, &s);
    }

    let status_clone = status.inner().clone();
    let stop_clone = stop_signal.inner().clone();

    tokio::spawn(async move {
        run_mining_loop(app, status_clone, stop_clone, config).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_mining(
    app: AppHandle,
    status: tauri::State<'_, SharedMiningStatus>,
    stop_signal: tauri::State<'_, MiningStopSignal>,
) -> Result<(), String> {
    {
        let mut stop = stop_signal.lock().await;
        *stop = true;
    }

    {
        let mut s = status.lock().await;
        s.running = false;
        s.active = false;
        emit_status(&app, &s);
    }

    emit_log(&app, "info", "Arrêt du bot mining demandé...");
    Ok(())
}

#[tauri::command]
pub async fn get_mining_status(
    status: tauri::State<'_, SharedMiningStatus>,
) -> Result<MiningStatus, String> {
    let s = status.lock().await;
    Ok(s.clone())
}

#[tauri::command]
pub async fn get_mining_config() -> Result<MiningConfig, String> {
    let path = config_path()?;
    if path.exists() {
        let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    } else {
        Ok(MiningConfig::default())
    }
}

#[tauri::command]
pub async fn save_mining_config(config: MiningConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}
