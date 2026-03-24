import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export interface Config {
  character_name: string;
}

export interface AutomationStatus {
  state:
    | "idle"
    | "searching_window"
    | "launching_fivem"
    | "waiting_ocr"
    | "selecting_character"
    | "sending_keys"
    | "afk_active"
    | "reconnecting";
  running: boolean;
  session_treated: boolean;
  reconnect_count: number;
  afk_start_time: string | null;
  last_crash_time: string | null;
}

export interface LogEntry {
  level: "info" | "warn" | "error";
  message: string;
  timestamp: string;
}

export interface WindowStatus {
  found: boolean;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  hwnd: number;
}

export async function getConfig(): Promise<Config> {
  return tauriInvoke<Config>("get_config");
}

export async function saveConfig(config: Config): Promise<void> {
  return tauriInvoke("save_config", { config });
}

export async function startAutomation(characterName: string): Promise<void> {
  return tauriInvoke("start_automation", { characterName });
}

export async function stopAutomation(): Promise<void> {
  return tauriInvoke("stop_automation");
}

export async function getStatus(): Promise<AutomationStatus> {
  return tauriInvoke<AutomationStatus>("get_status");
}

// ── Manual step commands ──

export interface OcrResult {
  found: boolean;
  text: string;
  score: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function manualKillFivem(): Promise<string> {
  return tauriInvoke<string>("manual_kill_fivem");
}

export async function manualLaunchFivem(): Promise<string> {
  return tauriInvoke<string>("manual_launch_fivem");
}

export async function manualFindWindow(): Promise<WindowStatus> {
  return tauriInvoke<WindowStatus>("manual_find_window");
}

export async function manualOcrDetect(characterName: string): Promise<OcrResult> {
  return tauriInvoke<OcrResult>("manual_ocr_detect", { characterName });
}

export async function manualClickCharacter(
  windowX: number,
  windowY: number,
  ocrX: number,
  ocrY: number,
  ocrW: number,
  ocrH: number
): Promise<string> {
  return tauriInvoke<string>("manual_click_character", {
    windowX, windowY, ocrX, ocrY, ocrW, ocrH,
  });
}

export async function manualClickSuivant(
  windowX: number,
  windowY: number,
  windowHeight: number
): Promise<string> {
  return tauriInvoke<string>("manual_click_suivant", { windowX, windowY, windowHeight });
}

export async function manualFocusWindow(): Promise<string> {
  return tauriInvoke<string>("manual_focus_window");
}

export async function manualSendAfkKeys(): Promise<string> {
  return tauriInvoke<string>("manual_send_afk_keys");
}

// ── Muscu bot ──

export interface MuscuStatus {
  running: boolean;
  cycle_count: number;
}

export async function startMuscu(): Promise<void> {
  return tauriInvoke("start_muscu");
}

export async function stopMuscu(): Promise<void> {
  return tauriInvoke("stop_muscu");
}

export async function getMuscuStatus(): Promise<MuscuStatus> {
  return tauriInvoke<MuscuStatus>("get_muscu_status");
}

// ── Mining bot ──

export interface MiningConfig {
  target_color: [number, number, number];
  tolerance: number;
  margin: number;
  min_delay: number;
  max_delay: number;
  max_distance: number;
  toggle_key: string;
}

export interface MiningStatus {
  running: boolean;
  active: boolean;
  click_count: number;
  detection_count: number;
}

export async function startMining(config: MiningConfig): Promise<void> {
  return tauriInvoke("start_mining", { config });
}

export async function stopMining(): Promise<void> {
  return tauriInvoke("stop_mining");
}

export async function getMiningStatus(): Promise<MiningStatus> {
  return tauriInvoke<MiningStatus>("get_mining_status");
}

export async function getMiningConfig(): Promise<MiningConfig> {
  return tauriInvoke<MiningConfig>("get_mining_config");
}

export async function saveMiningConfig(config: MiningConfig): Promise<void> {
  return tauriInvoke("save_mining_config", { config });
}

// ── Webhook config ──

export interface WebhookConfig {
  url: string;
  notify_afk_start: boolean;
  notify_afk_stop: boolean;
  notify_crash: boolean;
  notify_muscu_start: boolean;
  notify_muscu_stop: boolean;
  notify_mining_start: boolean;
  notify_mining_stop: boolean;
}

export async function getWebhookConfig(): Promise<WebhookConfig> {
  return tauriInvoke<WebhookConfig>("get_webhook_config");
}

export async function saveWebhookConfig(config: WebhookConfig): Promise<void> {
  return tauriInvoke("save_webhook_config", { config });
}

export async function testWebhook(url: string): Promise<void> {
  return tauriInvoke("test_webhook", { url });
}

// ── Stats ──

export interface Stats {
  total_muscu_cycles: number;
  total_mining_clicks: number;
  total_afk_seconds: number;
}

export async function getStats(): Promise<Stats> {
  return tauriInvoke<Stats>("get_stats");
}

export async function resetStats(): Promise<void> {
  return tauriInvoke("reset_stats");
}
