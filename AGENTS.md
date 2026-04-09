# AGENTS

This repository is a Rust workspace for a Telegram-delivered Codex runtime with a native `eframe` dashboard.

## What lives where

- `crates/app`: desktop entrypoint, top-level app state, instance list UI, new-instance wizard integration
- `crates/bots/code-agent`: Telegram bot with approval prompts
- `crates/bots/simple-chat`: Telegram bot that only streams text responses
- `crates/bots/auto-approve`: Telegram bot that auto-approves tool calls
- `crates/bots/all-bots`: shared template enums, template configs, wizard rendering, saved-instance persistence
- `crates/codex/bridge`: thin wrapper around Codex thread management and protocol types
- `crates/codex/sandbox`: resolves/builds the `codex-linux-sandbox` binary
- `crates/channels/telegram-adapter`: translates Codex events into Telegram actions
- `crates/ui/dashboard`: shared dashboard state and reusable GUI panels
- `docs/`: architecture, concepts, design notes, and setup guides
- `.local/`: local-only runtime files such as the Telegram token

## Current runtime shape

- The app starts in `crates/app/src/main.rs`.
- Saved instance metadata is loaded from `.local/home/.custodex/instances.json`.
- The UI can create and display multiple saved template configs.
- Runtime launch is still single-launcher today: `setup()` currently wires only the `code-agent` launcher.
- Per-chat Codex threads are created lazily by `InstanceManager` in `crates/codex/bridge/src/instance.rs`.

## Commands

- `cargo check --workspace`: fast validation for the whole workspace
- `cargo test --workspace`: full test run
- `make sandbox`: build the pinned sandbox binary
- `make run`: build and run the desktop app

## Editing guidance

- Prefer keeping cross-crate boundaries intact: bots should go through `codex-bridge`, not directly through Codex internals.
- Keep docs in sync with the actual crate layout. Several older docs described a pre-template single-bot version of the app.
- Treat `.local/` as user-local state. Do not commit secrets or generated runtime data from there.
- When changing startup behavior, verify both the desktop dashboard flow and the bot runtime path.
- When changes are coherent and reasonably verified, prefer committing and pushing them instead of stopping at uncommitted local edits.
