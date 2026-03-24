use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AutomationState {
    Idle,
    SearchingWindow,
    LaunchingFivem,
    WaitingOcr,
    SelectingCharacter,
    SendingKeys,
    AfkActive,
    Reconnecting,
}

impl std::fmt::Display for AutomationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "En attente"),
            Self::SearchingWindow => write!(f, "Recherche de la fenêtre FiveM..."),
            Self::LaunchingFivem => write!(f, "Lancement de FiveM..."),
            Self::WaitingOcr => write!(f, "Détection du personnage (OCR)..."),
            Self::SelectingCharacter => write!(f, "Sélection du personnage..."),
            Self::SendingKeys => write!(f, "Envoi des touches AFK..."),
            Self::AfkActive => write!(f, "Mode AFK actif"),
            Self::Reconnecting => write!(f, "Reconnexion en cours..."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStatus {
    pub state: AutomationState,
    pub running: bool,
    pub session_treated: bool,
    pub reconnect_count: u32,
    pub afk_start_time: Option<String>,
    pub last_crash_time: Option<String>,
}

impl Default for AutomationStatus {
    fn default() -> Self {
        Self {
            state: AutomationState::Idle,
            running: false,
            session_treated: false,
            reconnect_count: 0,
            afk_start_time: None,
            last_crash_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub character_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            character_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub found: bool,
    pub title: String,
    /// Client area top-left X (screen coords) — where the game actually renders
    pub x: i32,
    /// Client area top-left Y (screen coords)
    pub y: i32,
    /// Client area width
    pub width: i32,
    /// Client area height
    pub height: i32,
    pub hwnd: isize,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            found: false,
            title: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            hwnd: 0,
        }
    }
}

pub type SharedStatus = Arc<Mutex<AutomationStatus>>;

#[derive(Clone)]
pub struct StopSignal(pub Arc<Mutex<bool>>);
impl std::ops::Deref for StopSignal {
    type Target = Arc<Mutex<bool>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// ── Muscu bot state ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscuStatus {
    pub running: bool,
    pub cycle_count: u32,
}

impl Default for MuscuStatus {
    fn default() -> Self {
        Self {
            running: false,
            cycle_count: 0,
        }
    }
}

pub type SharedMuscuStatus = Arc<Mutex<MuscuStatus>>;

#[derive(Clone)]
pub struct MuscuStopSignal(pub Arc<Mutex<bool>>);
impl std::ops::Deref for MuscuStopSignal {
    type Target = Arc<Mutex<bool>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// ── Mining bot state ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub target_color: [u8; 3],
    pub tolerance: u8,
    pub margin: f64,
    pub min_delay: f64,
    pub max_delay: f64,
    pub max_distance: f64,
    pub toggle_key: String,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            target_color: [255, 3, 14],
            tolerance: 30,
            margin: 0.20,
            min_delay: 0.1,
            max_delay: 0.3,
            max_distance: 600.0,
            toggle_key: "M".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStatus {
    pub running: bool,
    pub active: bool,
    pub click_count: u32,
    pub detection_count: u32,
}

impl Default for MiningStatus {
    fn default() -> Self {
        Self {
            running: false,
            active: false,
            click_count: 0,
            detection_count: 0,
        }
    }
}

pub type SharedMiningStatus = Arc<Mutex<MiningStatus>>;

#[derive(Clone)]
pub struct MiningStopSignal(pub Arc<Mutex<bool>>);
impl std::ops::Deref for MiningStopSignal {
    type Target = Arc<Mutex<bool>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// ── Webhook config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub notify_afk_start: bool,
    pub notify_afk_stop: bool,
    pub notify_crash: bool,
    pub notify_muscu_start: bool,
    pub notify_muscu_stop: bool,
    pub notify_mining_start: bool,
    pub notify_mining_stop: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            notify_afk_start: true,
            notify_afk_stop: true,
            notify_crash: true,
            notify_muscu_start: true,
            notify_muscu_stop: true,
            notify_mining_start: true,
            notify_mining_stop: true,
        }
    }
}

pub type SharedWebhookConfig = Arc<Mutex<WebhookConfig>>;

// ── Persistent stats ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_muscu_cycles: u64,
    pub total_mining_clicks: u64,
    pub total_afk_seconds: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            total_muscu_cycles: 0,
            total_mining_clicks: 0,
            total_afk_seconds: 0,
        }
    }
}

pub type SharedStats = Arc<Mutex<Stats>>;
