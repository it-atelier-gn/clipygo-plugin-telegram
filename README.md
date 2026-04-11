# clipygo-plugin-telegram

Telegram target provider for [clipygo](https://github.com/it-atelier-gn/clipygo).

Sends clipboard content (text and images) to Telegram chats via the Bot API.

## Setup

1. Message [@BotFather](https://t.me/BotFather) on Telegram and create a bot (`/newbot`)
2. Copy the bot token
3. Add the bot to your desired chats/groups/channels
4. In clipygo Settings → Plugins, add the plugin and configure:
   - Paste the bot token
   - Add chats with a display name and chat ID

### Finding a chat ID

After adding the bot to a chat and sending it a message, visit:

```
https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates
```

Look for `"chat": {"id": ...}` in the response.

## Supported formats

- **text** — sent as a regular message via `sendMessage`
- **image** — sent as a photo via `sendPhoto` (base64-encoded PNG)

## Build

```sh
cargo build --release
```

## License

MIT
