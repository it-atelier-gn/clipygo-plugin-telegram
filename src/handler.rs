use crate::api;
use crate::config::{load_config, save_config, Chat};
use crate::protocol::{InfoResponse, Request, SendResponse, Target, TargetsResponse};

// 1×1 pixel PNG, Telegram blue (#0088cc)
const ICON: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg==";

pub fn handle(request: Request) -> serde_json::Value {
    match request {
        Request::GetInfo => serde_json::to_value(InfoResponse {
            name: "Telegram",
            version: env!("CARGO_PKG_VERSION"),
            description: "Send clipboard content to Telegram chats",
            author: "clipygo",
            link: Some("https://github.com/it-atelier-gn/clipygo-plugin-telegram"),
        })
        .unwrap(),

        Request::GetTargets => {
            let config = load_config();

            if config.bot_token.is_empty() {
                eprintln!("[telegram] Bot token not configured");
                return serde_json::to_value(TargetsResponse { targets: vec![] }).unwrap();
            }

            let targets = config
                .chats
                .iter()
                .map(|chat| Target {
                    id: format!("chat:{}", chat.chat_id),
                    provider: "Telegram".to_string(),
                    formats: vec!["text".to_string(), "image".to_string()],
                    title: chat.name.clone(),
                    description: format!("Chat ID: {}", chat.chat_id),
                    image: ICON.to_string(),
                })
                .collect();

            serde_json::to_value(TargetsResponse { targets }).unwrap()
        }

        Request::GetConfigSchema => {
            let config = load_config();
            serde_json::json!({
                "instructions": "1. Message @BotFather on Telegram and create a bot (/newbot)\n\
                    2. Copy the bot token and paste it below\n\
                    3. Add the bot to your chats/groups/channels\n\
                    4. Add each chat below with a name and its chat ID\n\n\
                    To find a chat ID: message the bot in the chat, then visit\n\
                    https://api.telegram.org/bot<TOKEN>/getUpdates",
                "schema": {
                    "type": "object",
                    "title": "Telegram",
                    "properties": {
                        "bot_token": {
                            "type": "string",
                            "title": "Bot Token",
                            "description": "Token from @BotFather",
                            "format": "password"
                        },
                        "chats": {
                            "type": "array",
                            "title": "Chats",
                            "description": "Telegram chats to send clipboard content to",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "title": "Name"
                                    },
                                    "chat_id": {
                                        "type": "string",
                                        "title": "Chat ID"
                                    }
                                },
                                "required": ["name", "chat_id"]
                            }
                        }
                    },
                    "required": ["bot_token"]
                },
                "values": {
                    "bot_token": config.bot_token,
                    "chats": config.chats
                }
            })
        }

        Request::SetConfig { values } => {
            let mut config = load_config();

            if let Some(v) = values.get("bot_token").and_then(|v| v.as_str()) {
                config.bot_token = v.to_string();
            }

            if let Some(arr) = values.get("chats").and_then(|v| v.as_array()) {
                config.chats = arr
                    .iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        let chat_id = item.get("chat_id")?.as_str()?.to_string();
                        if name.is_empty() || chat_id.is_empty() {
                            return None;
                        }
                        Some(Chat { name, chat_id })
                    })
                    .collect();
            }

            save_config(&config);

            serde_json::to_value(SendResponse {
                success: true,
                error: None,
            })
            .unwrap()
        }

        Request::Send {
            target_id,
            content,
            format,
        } => {
            let config = load_config();

            if config.bot_token.is_empty() {
                return serde_json::to_value(SendResponse {
                    success: false,
                    error: Some("Bot token not configured".to_string()),
                })
                .unwrap();
            }

            let chat_id = target_id.strip_prefix("chat:").unwrap_or(&target_id);

            let result = match format.as_str() {
                "text" => api::send_text(&config, chat_id, &content),
                "image" => api::send_image(&config, chat_id, &content),
                _ => Err(format!("Unsupported format: {format}")),
            };

            match result {
                Ok(()) => serde_json::to_value(SendResponse {
                    success: true,
                    error: None,
                })
                .unwrap(),
                Err(e) => serde_json::to_value(SendResponse {
                    success: false,
                    error: Some(e),
                })
                .unwrap(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_info_fields() {
        let resp = handle(Request::GetInfo);
        assert_eq!(resp["name"], "Telegram");
        assert!(resp["version"].is_string());
        assert!(resp["description"].is_string());
        assert_eq!(resp["author"], "clipygo");
    }

    #[test]
    fn get_info_includes_link() {
        let resp = handle(Request::GetInfo);
        assert!(resp["link"].as_str().unwrap().starts_with("https://"));
    }

    #[test]
    fn get_targets_empty_when_no_token() {
        save_config(&crate::config::Config::default());
        let resp = handle(Request::GetTargets);
        let targets = resp["targets"].as_array().unwrap();
        assert!(targets.is_empty());
    }

    #[test]
    fn get_config_schema_has_required_fields() {
        let resp = handle(Request::GetConfigSchema);
        assert!(resp.get("instructions").is_some());
        assert!(resp.get("schema").is_some());
        assert!(resp.get("values").is_some());
        let props = &resp["schema"]["properties"];
        assert!(props.get("bot_token").is_some());
        assert!(props.get("chats").is_some());
    }

    #[test]
    fn get_config_schema_bot_token_is_password() {
        let resp = handle(Request::GetConfigSchema);
        let format = resp["schema"]["properties"]["bot_token"]["format"]
            .as_str()
            .unwrap();
        assert_eq!(format, "password");
    }

    #[test]
    fn get_config_schema_chats_is_array() {
        let resp = handle(Request::GetConfigSchema);
        let chat_type = resp["schema"]["properties"]["chats"]["type"]
            .as_str()
            .unwrap();
        assert_eq!(chat_type, "array");
    }

    #[test]
    fn set_config_returns_success() {
        let resp = handle(Request::SetConfig {
            values: serde_json::json!({
                "bot_token": "test-token",
                "chats": [{"name": "Test", "chat_id": "123"}]
            }),
        });
        assert_eq!(resp["success"], true);
    }

    #[test]
    fn set_config_filters_empty_entries() {
        handle(Request::SetConfig {
            values: serde_json::json!({
                "chats": [
                    {"name": "Valid", "chat_id": "123"},
                    {"name": "", "chat_id": "456"},
                    {"name": "Also Valid", "chat_id": "789"}
                ]
            }),
        });
        let config = load_config();
        assert_eq!(config.chats.len(), 2);
        assert_eq!(config.chats[0].name, "Valid");
        assert_eq!(config.chats[1].name, "Also Valid");
    }

    #[test]
    fn send_fails_without_token() {
        // Ensure empty config
        save_config(&crate::config::Config::default());
        let resp = handle(Request::Send {
            target_id: "chat:123".to_string(),
            content: "hello".to_string(),
            format: "text".to_string(),
        });
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("token"));
    }

    #[test]
    fn send_rejects_unsupported_format() {
        save_config(&crate::config::Config {
            bot_token: "fake-token".to_string(),
            chats: vec![],
        });
        let resp = handle(Request::Send {
            target_id: "chat:123".to_string(),
            content: "data".to_string(),
            format: "video".to_string(),
        });
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("video"));
    }

    #[test]
    fn invalid_json_rejected() {
        assert!(serde_json::from_str::<Request>("not json").is_err());
    }

    #[test]
    fn unknown_command_rejected() {
        assert!(serde_json::from_str::<Request>(r#"{"command":"unknown"}"#).is_err());
    }

    #[test]
    fn config_roundtrip() {
        let config = crate::config::Config {
            bot_token: "test-token".to_string(),
            chats: vec![Chat {
                name: "Test".to_string(),
                chat_id: "123".to_string(),
            }],
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: crate::config::Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.bot_token, "test-token");
        assert_eq!(back.chats.len(), 1);
        assert_eq!(back.chats[0].chat_id, "123");
    }
}
