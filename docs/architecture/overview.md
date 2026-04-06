# Architecture

## Overview

codex-telegram is a single-process application with two subsystems:

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
│  │  │  │         InstanceManager            │   │  │  │
│  │  │  │  DashMap<ChatId, BotInstance>       │   │  │  │
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

### main.rs — Entry point
- Reads bot token from `.local/telegram_bot_token` (or CLI arg)
- Creates tokio runtime
- Creates shared Dashboard
- Launches eframe window (blocks main thread)

### gui.rs — Control Panel (eframe::App)
- Reads Dashboard each frame (200ms repaint interval)
- Start/Stop button spawns or aborts the bot task on the tokio runtime
- Pure monitoring/debugging — no bot interaction

### bot.rs — Telegram dispatcher
- Creates teloxide `Bot` and `Dispatcher`
- Routes messages to `handle_message`, callbacks to `handle_callback`
- Runs until aborted (Stop button) or error

### instance.rs — Session management
- `InstanceManager` maps `ChatId` → `BotInstance` (CodexThread + pending approval)
- On user message: submits `Op::UserInput`, drains events until turn ends
- On approval callback: submits `Op::ExecApproval` or `Op::PatchApproval`, resumes draining
- Pushes all activity to Dashboard for GUI display

### adapter.rs — Event classification
- `classify_event(EventMsg) -> TelegramAction`
- Categories: `Delta` (stream text), `Send` (standalone message), `ApprovalPrompt` (inline keyboard), `Skip`
- AgentMessage skipped (deltas already cover it)

### monitor.rs — Shared state
- `Dashboard`: log entries, instance info, token counts, bot status, config
- Protected by `Arc<std::sync::Mutex<...>>` (std, not tokio — GUI thread isn't async)
- Bounded log (2000 entries, oldest dropped)

## Data flow

### User message
```
Telegram → teloxide → handle_message → InstanceManager::handle_user_message
  → get_or_create_chat (creates CodexThread if new)
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
Telegram inline button → teloxide → handle_callback
  → InstanceManager::handle_approval_callback
  → thread.submit(Op::ExecApproval or Op::PatchApproval)
  → drain_events (resumes from where it paused)
```

## Dependencies on codex

All path dependencies to `../codex/codex-rs/`:

| Crate | What we use |
|-------|-------------|
| codex-core | ThreadManager, CodexThread, Config, ConfigOverrides |
| codex-protocol | EventMsg, Op, ReviewDecision, UserInput, SessionSource |
| codex-login | AuthManager, AuthCredentialsStoreMode |
| codex-exec-server | EnvironmentManager |
| codex-models-manager | CollaborationModesConfig |

The Landlock sandbox binary (`codex-linux-sandbox`) is passed via `ConfigOverrides::codex_linux_sandbox_exe`.

## Threading model

- **Main thread**: eframe/egui render loop
- **Tokio runtime** (multi-thread): bot dispatcher, session event loops, codex-core internals
- **Sync point**: `Arc<std::sync::Mutex<Dashboard>>` — GUI reads, bot writes
