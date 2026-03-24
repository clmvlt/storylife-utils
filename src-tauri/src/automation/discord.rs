use chrono::Utc;

/// Send a Discord webhook embed. Pure HTTP — no Tauri dependency.
pub async fn send_embed(
    url: &str,
    title: &str,
    description: &str,
    color: u32,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "embeds": [{
            "title": title,
            "description": description,
            "color": color,
            "footer": { "text": "StoryLifeUtils" },
            "timestamp": Utc::now().to_rfc3339()
        }]
    });
    let resp = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Webhook error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Webhook HTTP {}", resp.status()));
    }
    Ok(())
}
