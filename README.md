# codex-telegram
Telegram binding for codex

## Architecture

`cargo run` launches a single process with two subsystems:

1. **eframe/egui control panel** — runs on the main thread (egui requirement). Provides debugging, monitoring, and configuration. NOT for interacting with the bot as a user.
2. **Telegram bot** — runs on background tokio tasks. Handles all Telegram message/callback dispatching.

Communication between GUI and bot via channels.

## Control Panel (egui)

Purpose: debugging/monitoring/configuration only.

Panels:
- **Bot status** — running/stopped, start/stop button, connection state
- **Active instances** — table of chat IDs, message counts, last activity
- **Event log** — live stream of all EventMsg from all instances (scrollable, filterable)
- **Token usage** — per-instance and total token counts, cost estimate
- **Configuration** — token file path, sandbox exe path, model, editable settings

The GUI does NOT handle approvals — those stay on Telegram inline keyboards only.

## Token storage

Bot token stored in `.local/telegram_bot_token` (gitignored). App takes optional file path as first CLI arg, defaults to `.local/telegram_bot_token`.

## Dependencies on codex

Path deps to `../codex/codex-rs/`:
- `codex-core` — embedded agent runtime (ThreadManager, CodexThread)
- `codex-protocol` — Event/EventMsg types, Op, ReviewDecision
- `codex-login` — AuthManager, token storage
- `codex-exec-server` — EnvironmentManager
- `codex-models-manager` — CollaborationModesConfig

Sandbox binary: `../codex/codex-rs/target/release/codex-linux-sandbox`

## Dev notes

- Don't skip `AgentMessage` display — deltas already cover streaming text, `AgentMessage` is the final duplicate
- Telegram message edit throttle: 500ms to avoid rate limits
- `ConfigOverrides` used to pass sandbox exe path
- `[patch.crates-io]` needed for tokio-tungstenite fork (proxy feature)
