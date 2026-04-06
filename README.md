# custodex

Customizable codex agent runtime delivered over Telegram.

## Quick start

```
make sandbox   # build the Landlock sandbox binary (first time only)
make run       # launch the dashboard + bot
```

See [docs/guides/deployment.md](docs/guides/deployment.md) for full setup instructions.

## Documentation

- [Concepts](docs/concepts.md) — core abstractions (bot template, channel, bridge, dashboard)
- Architecture — [overview](docs/architecture/overview.md), [workspace](docs/architecture/workspace.md), [events](docs/architecture/events.md)
- Guides — [deployment](docs/guides/deployment.md), [creating a bot template](docs/guides/creating-a-bot-template.md), [codex integration](docs/guides/codex-integration.md)
- Design — [dashboard layout](docs/design/ui/layout.md), [new instance wizard](docs/design/ui/new_instance.md), [channels](docs/design/channels.md)

## Workspace layout

```
crates/
├── codex/bridge/              # wraps codex-core, re-exports types
├── llm/core/                  # LLM abstractions (grow when needed)
├── channels/telegram-adapter/ # EventMsg → Telegram actions
├── ui/dashboard/              # egui control panel
└── bots/code-agent/           # full code agent bot template
```

## Dev notes

- `AgentMessage` skipped in adapter — deltas already cover streaming text
- Telegram message edit throttle: 500ms to avoid rate limits
- `ConfigOverrides` used to pass sandbox exe path
- `[patch.crates-io]` needed for tokio-tungstenite fork (proxy feature)
