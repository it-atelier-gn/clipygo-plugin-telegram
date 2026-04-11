use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Chat {
    pub name: String,
    pub chat_id: String,
}

/// Stored at:
///   Windows : %APPDATA%\clipygo-plugin-telegram\config.json
///   macOS   : ~/Library/Application Support/clipygo-plugin-telegram/config.json
///   Linux   : ~/.config/clipygo-plugin-telegram/config.json
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub chats: Vec<Chat>,
}

pub fn config_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("clipygo-plugin-telegram");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("config.json")
}

pub fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) {
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path(), data);
    }
}
