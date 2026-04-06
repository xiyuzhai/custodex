# Dashboard Layout

## Overall structure

```
┌──────────────────────────────────────────────────────────┐
│  Service: Running  │ Start/Stop │ Instances: 3 │ Tokens  │  ← status bar
├────────────┬───────────────────────────────┬─────────────┤
│            │                               │             │
│ Instances  │       Event Log               │   Config    │
│            │                               │             │
│ [Chat 123] │  0.1s [123] User: hello       │ Token file: │
│  msgs: 5   │  0.3s [123] AgentMessageDelta │ Sandbox:    │
│  last: 2s  │  0.5s [123] TurnComplete      │ Model:      │
│            │  1.2s [456] User: fix bug      │             │
│ [Chat 456] │  1.4s [456] ExecCommandBegin  │             │
│  msgs: 12  │                               │             │
│  last: 0s  │                               │             │
│            │                               │             │
│ [+ New]    │  [Auto-scroll ✓] [Clear]      │             │
│            │                               │             │
└────────────┴───────────────────────────────┴─────────────┘
```

## Panels

- **Top**: Status bar — service status, start/stop button, instance count, token usage
- **Left**: Instances panel — list of active instances, [+ New] button for wizard
- **Center**: Event log — scrollable, monospace, auto-scroll toggle, clear button
- **Right**: Configuration — current system config (read-only display)

## Interaction

- Start/Stop controls the Telegram bot connection
- Clicking an instance could filter the event log (future)
- [+ New] opens the instance creation wizard (see [new_instance.md](new_instance.md))
