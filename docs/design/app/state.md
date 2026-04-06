# App State

## Principle

The app state is a pure data structure with operations. It has no dependency on egui or any UI framework. The UI is a thin view layer that reads state and calls methods.

```
AppState (no UI deps)
  ├── saved_instances: Vec<SavedInstance>
  ├── wizard_state: WizardData
  ├── dashboard: SharedDashboard
  ├── bot_running: bool
  ├── log_filter_chat: Option<i64>
  ├── log_filter_text: String
  └── methods:
      ├── start_bot()
      ├── stop_bot()
      ├── open_wizard()
      ├── select_template(kind)
      ├── advance_wizard()
      ├── back_wizard()
      ├── cancel_wizard()
      ├── finish_wizard() -> Option<SavedInstance>
      ├── set_log_filter(chat_id)
      ├── clear_log_filter()
      └── get_filtered_log() -> Vec<LogEntry>
```

## Two runners, same state

**Real (eframe):** `CustodexApp` implements `eframe::App`, holds `AppState`, renders with egui, calls `AppState` methods on user interaction.

**Fake (headless):** A test binary creates `AppState` directly, calls the same methods in sequence, prints/asserts on the results. No window, no event loop.

The fake version catches most bugs because it exercises the exact same state transitions. The only things it can't catch are rendering glitches and egui-specific interaction bugs.

## What lives where

| Concern | Where |
|---------|-------|
| Instance list, wizard data, bot lifecycle | `AppState` (no UI) |
| Log filtering, search | `AppState` |
| Persistence (save/load) | `AppState` |
| egui layout, widgets, colors | View layer only |
| Bot spawning (tokio tasks) | `AppState` (owns runtime handle) |
