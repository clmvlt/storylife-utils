mod automation;
mod commands;
mod state;

use state::{
    AutomationStatus, MiningStatus, MuscuStatus, MiningStopSignal, MuscuStopSignal,
    SharedMiningStatus, SharedMuscuStatus, SharedStats, SharedStatus, SharedWebhookConfig,
    StopSignal,
};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Load persisted data from disk
    let webhook_config = commands::config::load_webhook_config_from_disk();
    let stats = commands::config::load_stats_from_disk();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(Mutex::new(AutomationStatus::default())) as SharedStatus)
        .manage(StopSignal(Arc::new(Mutex::new(false))))
        .manage(Arc::new(Mutex::new(MuscuStatus::default())) as SharedMuscuStatus)
        .manage(MuscuStopSignal(Arc::new(Mutex::new(false))))
        .manage(Arc::new(Mutex::new(MiningStatus::default())) as SharedMiningStatus)
        .manage(MiningStopSignal(Arc::new(Mutex::new(false))))
        .manage(Arc::new(Mutex::new(webhook_config)) as SharedWebhookConfig)
        .manage(Arc::new(Mutex::new(stats)) as SharedStats)
        .setup(|app| {
            // Build tray menu
            let show_item = MenuItem::with_id(app, "show", "Afficher", true, None::<&str>)?;
            let status_item =
                MenuItem::with_id(app, "status", "Statut: En attente", false, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &status_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("StoryLifeUtils")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::automation::start_automation,
            commands::automation::stop_automation,
            commands::automation::get_status,
            commands::manual::manual_kill_fivem,
            commands::manual::manual_launch_fivem,
            commands::manual::manual_find_window,
            commands::manual::manual_ocr_detect,
            commands::manual::manual_click_character,
            commands::manual::manual_click_suivant,
            commands::manual::manual_focus_window,
            commands::manual::manual_send_afk_keys,
            commands::muscu::start_muscu,
            commands::muscu::stop_muscu,
            commands::muscu::get_muscu_status,
            commands::mining::start_mining,
            commands::mining::stop_mining,
            commands::mining::get_mining_status,
            commands::mining::get_mining_config,
            commands::mining::save_mining_config,
            commands::config::get_webhook_config,
            commands::config::save_webhook_config,
            commands::config::test_webhook,
            commands::config::get_stats,
            commands::config::reset_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
