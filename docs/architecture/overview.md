# Architecture

## Overview

`custodex` is a single-process application with two main subsystems:

```
┌─────────────────────────────────────────────────────┐
│                    main thread                       │
│  ┌───────────────────────────────────────────────┐  │
│  │            eframe/egui Control Panel           │  │
│  │  ┌─────────┐ ┌──────────┐ ┌────────────────┐ │  │
│  │  │ Status  │ │ Sessions │ │   Event Log    │ │  │
│  │  │  Bar    │ │  Panel   │ │   (scrollable) │ │  │
│  │  └─────────┘ └──────────┘ └────────────────┘ │  │
│  │  ┌──────────────┐ ┌──────────────────────┐   │  │
│  │  │ Token Usage  │ │   Configuration      │   │  │
│  │  └──────────────┘ └──────────────────────┘   │  │
│  └───────────────────────────────────────────────┘  │
│                         ▲                            │
│                         │ Arc<Mutex<Dashboard>>   │
│                         ▼                            │
│  ┌───────────────────────────────────────────────┐  │
│  │          tokio runtime (background)            │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │          Telegram Bot (teloxide)         │  │  │
│  │  │  ┌───────────┐    ┌──────────────────┐  │  │  │
│  │  │  │ Message   │    │   Callback       │  │  │  │
│  │  │  │ Handler   │    │   Handler        │  │  │  │
│  │  │  └─────┬─────┘    └────────┬─────────┘  │  │  │
│  │  │        │                   │             │  │  │
│  │  │        ▼                   ▼             │  │  │
│  │  │  ┌───────────────────────────────────┐   │  │  │
│  │  │  │         CodexConversationManager            │   │  │  │
│  │  │  │  DashMap<ChatId, CodexConversation>    │   │  │  │
│  │  │  └──────────────┬────────────────────┘   │  │  │
│  │  │                 │                        │  │  │
│  │  │                 ▼                        │  │  │
│  │  │  ┌───────────────────────────────────┐   │  │  │
│  │  │  │         ThreadManager             │   │  │  │
│  │  │  │   (from codex-core)               │   │  │  │
│  │  │  │   One CodexThread per chat        │   │  │  │
│  │  │  └───────────────────────────────────┘   │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Components

### `crates/app/src/main.rs` — entry point
- Reads the Telegram token from `.local/telegram_bot_token` or the first CLI arg
- Creates `.local/home/.custodex` for local instance persistence
- Builds the tokio runtime and shared dashboard
- Launches the native `eframe` window
- Currently wires the `code-agent` launcher during startup

### `crates/app/src/state.rs` — desktop app state
- Owns the runtime handle, dashboard handle, bot launcher, and saved instances
- Seeds default instances for first run
- Starts and stops the background bot task
- Filters log display by chat id or search text

### `crates/bots/*/src/bot.rs` — template-specific Telegram runtimes
- `code-agent`: manual approval flow with inline keyboard callbacks
- `simple-chat`: streams text, ignores approval prompts
- `auto-approve`: automatically approves exec and patch requests

### `crates/bots/all-bots/src/lib.rs` — template registry
- Defines `TemplateKind`, `TemplateConfig`, wizard rendering, and saved-instance persistence
- Persists saved instances to `.custodex/instances.json`

### `crates/codex/bridge/src/instance.rs` — Codex thread management
- Owns a `ThreadManager`
- Lazily maps `ChatId` to `CodexConversation`
- Submits user text and approval ops into the correct `CodexThread`

### `crates/channels/telegram-adapter/src/*` — event classification
- Maps `EventMsg` to `TelegramAction`
- Handles streaming delta accumulation and message edit behavior

### `crates/ui/dashboard/src/*` — shared dashboard state
- Stores bounded log entries, active chat activity, token counters, and service status
- Uses `Arc<std::sync::Mutex<Dashboard>>` because the GUI thread is synchronous

## Data flow

### User message
```
Telegram → teloxide bot handler → CodexConversationManager::submit_text
  → get_or_create(chat) if needed
  → thread.submit(Op::UserInput)
  → drain_events loop:
      thread.next_event() → adapter::classify_event → match:
        Delta → buffer text, edit Telegram message (throttled 500ms)
        Send  → flush delta, send new Telegram message
        ApprovalPrompt → send inline keyboard, store PendingApproval, return
        Skip  → continue
      on TurnComplete/TurnAborted → break
```

### Approval callback
```
Telegram inline button → teloxide callback handler
  → CodexConversationManager::submit_approval
  → thread.submit(Op::ExecApproval or Op::PatchApproval)
  → drain_events (resumes from where it paused)
```

## Dependencies on Codex

| Crate | What we use |
|-------|-------------|
| codex-core | ThreadManager, CodexThread, Config, ConfigOverrides |
| codex-protocol | EventMsg, Op, ReviewDecision, UserInput, SessionSource |
| codex-login | AuthManager, AuthCredentialsStoreMode |
| codex-exec-server | EnvironmentManager |
| codex-models-manager | CollaborationModesConfig |

The Landlock sandbox binary (`codex-linux-sandbox`) is passed via `ConfigOverrides::codex_linux_sandbox_exe`.

## Known limitation

- Saved instances are persisted and shown in the UI, but startup still launches a single pre-wired bot runtime instead of instantiating saved templates dynamically.

## Threading model

- **Main thread**: eframe/egui render loop
- **Tokio runtime** (multi-thread): bot dispatcher, session event loops, codex-core internals
- **Sync point**: `Arc<std::sync::Mutex<Dashboard>>` — GUI reads, bot writes
