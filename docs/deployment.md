# Deployment

## Prerequisites

- Rust toolchain (edition 2024, requires rustc 1.85+)
- `libcap-dev` (for building the sandbox): `sudo apt-get install -y libcap-dev`
- Linux with Landlock support (kernel 5.13+, better with 6.7+)
- A Telegram bot token from [@BotFather](https://t.me/BotFather)
- codex repo cloned at `../codex/` relative to this project

## First-time setup

1. Build the sandbox binary:
   ```
   make sandbox
   ```

2. Store your bot token:
   ```
   mkdir -p .local
   echo "YOUR_TOKEN_HERE" > .local/telegram_bot_token
   ```

3. Ensure codex auth is configured (run `codex` CLI once to log in, or set `OPENAI_API_KEY`).

## Running

```
make run
```

This builds and launches the egui control panel. Click **Start** to connect the bot.

Alternatively, with a custom token path:
```
cargo run -- /path/to/token/file
```

## Configuration

The bot reads codex configuration from the standard locations:
- `~/.codex/config.toml` — user-level config
- Project-level `codex.toml` — if running from a project directory

Key settings that affect the bot:
- `model` — which model to use
- `permissions.approval_policy` — when to ask for approval
- `sandbox_policy` — sandbox restrictions

The sandbox executable is resolved at compile time to `../codex/codex-rs/target/release/codex-linux-sandbox`.

## Environment variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | API key (alternative to codex login) |
| `RUST_LOG` | Logging level (e.g., `RUST_LOG=info`) |

## File layout

```
.local/
  telegram_bot_token    # bot token (gitignored)
docs/
  architecture.md       # system design
  events.md             # event handling reference
  deployment.md         # this file
src/
  main.rs               # entry point
  gui.rs                # egui control panel
  bot.rs                # telegram dispatcher
  instance.rs            # per-chat codex thread management
  adapter.rs            # event classification
  monitor.rs            # shared monitoring state
```
