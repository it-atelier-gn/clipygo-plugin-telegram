use base64::Engine;
use reqwest::blocking::Client;

use crate::config::Config;

fn api_url(config: &Config, method: &str) -> String {
    format!("https://api.telegram.org/bot{}/{method}", config.bot_token)
}

pub fn send_text(config: &Config, chat_id: &str, text: &str) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .post(api_url(config, "sendMessage"))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        }))
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().map_err(|e| format!("Bad response: {e}"))?;

    if body["ok"].as_bool() == Some(true) {
        Ok(())
    } else {
        let desc = body["description"].as_str().unwrap_or("Unknown error");
        Err(format!("Telegram API error ({status}): {desc}"))
    }
}

pub fn send_image(config: &Config, chat_id: &str, base64_data: &str) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;

    let part = reqwest::blocking::multipart::Part::bytes(bytes)
        .file_name("clipboard.png")
        .mime_str("image/png")
        .map_err(|e| format!("MIME error: {e}"))?;

    let form = reqwest::blocking::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
        .part("photo", part);

    let client = Client::new();
    let resp = client
        .post(api_url(config, "sendPhoto"))
        .multipart(form)
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().map_err(|e| format!("Bad response: {e}"))?;

    if body["ok"].as_bool() == Some(true) {
        Ok(())
    } else {
        let desc = body["description"].as_str().unwrap_or("Unknown error");
        Err(format!("Telegram API error ({status}): {desc}"))
    }
}
