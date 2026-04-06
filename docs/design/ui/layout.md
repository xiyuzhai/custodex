# Dashboard Layout

## Main view

The left sidebar lists instances. The right area shows the selected instance's detail view. No split panels — one thing at a time.

```
┌──────────────────────────────────────────────────────────────┐
│ ● Running    custodex                       12 in / 8 out   │
├──────────────┬───────────────────────────────────────────────┤
│              │                                               │
│ [+ New]      │                                               │
│              │   (select an instance to view details)        │
│ ┌──────────┐ │                                               │
│ │code-agent│ │                                               │
│ │ ● 5 msgs │ │                                               │
│ └──────────┘ │                                               │
│              │                                               │
│ ┌──────────┐ │                                               │
│ │simple    │ │                                               │
│ │chat      │ │                                               │
│ │ ○ idle   │ │                                               │
│ └──────────┘ │                                               │
│              │                                               │
│ ┌──────────┐ │                                               │
│ │auto      │ │                                               │
│ │approve   │ │                                               │
│ │ ○ idle   │ │                                               │
│ └──────────┘ │                                               │
│              │                                               │
└──────────────┴───────────────────────────────────────────────┘
```

## After clicking an instance

The right area shows that instance's detail window: event log, config, status.

```
┌──────────────────────────────────────────────────────────────┐
│ ● Running    custodex                       12 in / 8 out   │
├──────────────┬───────────────────────────────────────────────┤
│              │                                               │
│ [+ New]      │  code-agent-0                                 │
│              │  Template: code-agent                         │
│ ┌──────────┐ │  Status: ● active                             │
│ │code-agent│◄│  Work dir: .local/home/code-agent             │
│ │ ● 5 msgs │ │  Model: (default)                             │
│ └──────────┘ │                                               │
│              │  ── Event Log ──────────────────────────────   │
│ ┌──────────┐ │  12:01:03  User: fix the login bug            │
│ │simple    │ │  12:01:04  AgentMessageDelta                  │
│ │chat      │ │  12:01:04  ExecCommandBegin                   │
│ │ ○ idle   │ │  12:01:05  → Running: cargo test              │
│ └──────────┘ │  12:01:08  ExecCommandEnd                     │
│              │  12:01:08  TurnComplete                       │
│ ┌──────────┐ │                                               │
│ │auto      │ │                                               │
│ │approve   │ │  ──────────────────────────────────────────   │
│ │ ○ idle   │ │  Filter: [____________]            [Clear]    │
│ └──────────┘ │                                               │
└──────────────┴───────────────────────────────────────────────┘
```

## [+ New] clicked — wizard overlay

```
┌──────────────────────────────────────────────────────────────┐
│ ● Running    custodex                       12 in / 8 out   │
├──────────────┬───────────────────────────────────────────────┤
│              │ ┌──────────────────────────────────────────┐  │
│ [+ New]      │ │                                          │  │
│              │ │   Select Bot Template                    │  │
│ ┌──────────┐ │ │                                          │  │
│ │code-agent│ │ │   [code-agent]                           │  │
│ │ ● 5 msgs │ │ │    Full code agent with tool execution   │  │
│ └──────────┘ │ │                                          │  │
│              │ │   [simple-chat]                          │  │
│ ┌──────────┐ │ │    Text conversation only                │  │
│ │simple    │ │ │                                          │  │
│ │chat      │ │ │   [auto-approve]                        │  │
│ │ ○ idle   │ │ │    Code agent, auto-approves all         │  │
│ └──────────┘ │ │                                          │  │
│              │ │                          [Cancel]        │  │
│ ┌──────────┐ │ │                                          │  │
│ │auto      │ │ └──────────────────────────────────────────┘  │
│ │approve   │ │                                               │
│ │ ○ idle   │ │                                               │
│ └──────────┘ │                                               │
└──────────────┴───────────────────────────────────────────────┘
```

## Interaction summary

1. App launches → three default instances in sidebar
2. Click an instance → right area shows its detail (config + event log filtered to it)
3. Click [+ New] → wizard overlay for creating a new instance
4. Status bar always visible at top — bot status, token usage
