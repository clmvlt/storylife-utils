use crate::state::{
    AutomationState, Config, SharedStats, SharedStatus, SharedWebhookConfig, Stats, WebhookConfig,
};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn app_dir() -> PathBuf {
    let appdata = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = appdata.join("StoryLifeUtils");
    fs::create_dir_all(&dir).ok();
    dir
}

fn config_path() -> PathBuf {
    app_dir().join("config.json")
}

fn webhook_config_path() -> PathBuf {
    app_dir().join("webhook_config.json")
}

fn stats_path() -> PathBuf {
    app_dir().join("stats.json")
}

// ── Character config ──

#[tauri::command]
pub fn get_config() -> Result<Config, String> {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        let config = Config::default();
        save_config_inner(&config)?;
        Ok(config)
    }
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    save_config_inner(&config)
}

fn save_config_inner(config: &Config) -> Result<(), String> {
    let path = config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

// ── Webhook config ──

pub fn load_webhook_config_from_disk() -> WebhookConfig {
    let path = webhook_config_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        WebhookConfig::default()
    }
}

fn save_webhook_config_to_disk(config: &WebhookConfig) -> Result<(), String> {
    let path = webhook_config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_webhook_config(
    config: tauri::State<'_, SharedWebhookConfig>,
) -> Result<WebhookConfig, String> {
    Ok(config.lock().await.clone())
}

#[tauri::command]
pub async fn save_webhook_config(
    config: WebhookConfig,
    state: tauri::State<'_, SharedWebhookConfig>,
) -> Result<(), String> {
    save_webhook_config_to_disk(&config)?;
    *state.lock().await = config;
    Ok(())
}

#[tauri::command]
pub async fn test_webhook(url: String) -> Result<(), String> {
    if url.trim().is_empty() {
        return Err("URL du webhook requise".to_string());
    }
    crate::automation::discord::send_embed(
        &url,
        "\u{2705} Test Webhook",
        "Le webhook Discord est correctement configuré !",
        0x2ECC71,
    )
    .await
}

// ── Stats ──

pub fn load_stats_from_disk() -> Stats {
    let path = stats_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        Stats::default()
    }
}

pub fn save_stats_to_disk(stats: &Stats) -> Result<(), String> {
    let path = stats_path();
    let content = serde_json::to_string_pretty(stats).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats(
    stats: tauri::State<'_, SharedStats>,
    automation_status: tauri::State<'_, SharedStatus>,
) -> Result<Stats, String> {
    let mut s = stats.lock().await.clone();
    let status = automation_status.lock().await;

    // If AFK is currently active, add live elapsed time
    if status.running && status.state == AutomationState::AfkActive {
        if let Some(ref start_str) = status.afk_start_time {
            if let Ok(start) =
                chrono::NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S")
            {
                let now = chrono::Local::now().naive_local();
                let elapsed = (now - start).num_seconds().max(0) as u64;
                s.total_afk_seconds += elapsed;
            }
        }
    }
    Ok(s)
}

#[tauri::command]
pub async fn reset_stats(stats: tauri::State<'_, SharedStats>) -> Result<(), String> {
    let mut s = stats.lock().await;
    *s = Stats::default();
    save_stats_to_disk(&s)
}

// ── Webhook notify helper (called from bot loops) ──

pub async fn webhook_notify(
    app: &tauri::AppHandle,
    event: &str,
    description: &str,
) {
    let cfg_state = app.state::<SharedWebhookConfig>();
    let cfg = cfg_state.lock().await;

    if cfg.url.is_empty() {
        return;
    }

    let (should_send, color, title) = match event {
        "afk_start" => (cfg.notify_afk_start, 0x2ECC71u32, "\u{1F7E2} AFK Activé"),
        "afk_stop" => (cfg.notify_afk_stop, 0xE74C3C, "\u{1F534} AFK Arrêté"),
        "crash" => (cfg.notify_crash, 0xE67E22, "\u{26A0}\u{FE0F} Crash Détecté"),
        "muscu_start" => (cfg.notify_muscu_start, 0x9B59B6, "\u{1F4AA} Muscu Démarré"),
        "muscu_stop" => (cfg.notify_muscu_stop, 0x8E44AD, "\u{1F6D1} Muscu Arrêté"),
        "mining_start" => (cfg.notify_mining_start, 0xF1C40F, "\u{26CF}\u{FE0F} Mining Démarré"),
        "mining_stop" => (cfg.notify_mining_stop, 0xE67E22, "\u{1F6D1} Mining Arrêté"),
        _ => return,
    };

    if !should_send {
        return;
    }

    let url = cfg.url.clone();
    let title = title.to_string();
    let description = description.to_string();
    drop(cfg);

    // Fire-and-forget — don't block the bot loop
    tokio::spawn(async move {
        let _ = crate::automation::discord::send_embed(&url, &title, &description, color).await;
    });
}
