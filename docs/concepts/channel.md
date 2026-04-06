# Channel

A channel is a communication transport between the bot and users.

## What a channel does

- Receives user messages from an external platform
- Delivers bot responses back to the platform
- Handles platform-specific UI (inline keyboards, message editing, file attachments)
- Adapts between the bot's internal event model and the platform's message format

## Current channels

**Telegram** (`crates/channels/telegram-adapter/`)
- Uses teloxide for the Telegram Bot API
- Supports streaming text via message editing (delta accumulation)
- Supports approval flows via inline keyboards
- Throttles edits to avoid Telegram rate limits

## Future channels

The channel abstraction is designed so new transports can be added:
- Discord
- Slack
- CLI (local terminal)
- Web UI

Each channel crate would live under `crates/channels/<channel-name>/`.

## Channel vs Bridge

A channel handles the *user-facing* transport. A bridge handles the *LLM-facing* backend. They sit on opposite sides of the bot instance:

```
User ←→ Channel ←→ Bot Instance ←→ Bridge ←→ LLM
```
