# UI Style Guide

## Principles

- Dark theme, minimal chrome
- Information density over decoration
- Monospace for data, proportional for labels
- Color only for status/severity, not decoration
- No borders unless they separate interactive regions

## Color palette

| Color | Usage |
|-------|-------|
| `#1e1e2e` | Background |
| `#313244` | Panel/card background |
| `#45475a` | Borders, separators |
| `#cdd6f4` | Primary text |
| `#a6adc8` | Secondary/muted text |
| `#a6e3a1` | Running/success |
| `#f38ba8` | Error/danger |
| `#fab387` | Warning/auto-approve |
| `#89b4fa` | Selected/active |
| `#74c7ec` | Links, interactive |

(Catppuccin Mocha inspired)

## Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ ● Running    custodex                    12 in / 8 out    ⚙    │  ← thin status bar
├──────────────┬──────────────────────────────────────────────────┤
│              │                                                  │
│  INSTANCES   │  EVENT LOG                                       │
│              │                                                  │
│  ┌────────┐  │  12:01:03  [4821]  User: fix the login bug      │
│  │ code   │  │  12:01:04  [4821]  AgentMessageDelta             │
│  │ agent  │  │  12:01:04  [4821]  ExecCommandBegin              │
│  │ ● 5msg │  │  12:01:05  [4821]  → Running: cargo test        │
│  └────────┘  │  12:01:08  [4821]  ExecCommandEnd                │
│              │  12:01:08  [4821]  TurnComplete                  │
│  ┌────────┐  │                                                  │
│  │ simple │  │                                                  │
│  │ chat   │  │                                                  │
│  │ ○ idle │  │                                                  │
│  └────────┘  │                                                  │
│              │                                                  │
│  ┌────────┐  │                                                  │
│  │ auto   │  │                                                  │
│  │ approve│  │                                                  │
│  │ ○ idle │  │                                                  │
│  └────────┘  │                                                  │
│              │──────────────────────────────────────────────────│
│  [+ New]     │  Filter: [____________]              [Clear]     │
│              │                                                  │
└──────────────┴──────────────────────────────────────────────────┘
```

## Status bar

- Thin, single line
- Left: colored dot + status text
- Center: app name
- Right: token usage summary, settings gear

## Instance cards

- Compact cards, not a table
- Template name bold, instance ID muted below
- Status dot: green (active), gray (idle), red (error)
- Message count + last activity as subtle secondary text
- Selected card has left accent border in blue
- Click to filter event log

## Event log

- Monospace, fixed-width columns
- Timestamp (HH:MM:SS), chat ID in brackets, message
- Alternating row shading (subtle, not striped)
- Auto-scroll with sticky bottom, toggle in filter bar
- Filter bar at bottom, not top — keeps log visible
- Text filter + chat filter (from clicking instance card)

## Wizard (New Instance)

- Modal overlay with slight background dim
- Clean step indicator: "Step 1 of 3 — Working Directory"
- Form fields with labels above, not beside
- Back / Cancel / Next buttons right-aligned
- Launch button highlighted in green on final step

## Typography

- Headings: 16px, semibold
- Body: 14px, regular
- Monospace (log): 13px
- Labels: 12px, muted color

## Spacing

- Panel padding: 12px
- Card padding: 8px
- Between cards: 6px
- Between form fields: 8px
- Section gap: 16px

## No

- No gradients
- No shadows (except wizard modal)
- No rounded corners > 4px
- No icons except status dots
- No animations except scroll
