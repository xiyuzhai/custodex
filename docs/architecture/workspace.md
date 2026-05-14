# Workspace Layout

## Crate groups

```
crates/
├── app/                     # native desktop entrypoint and app state
├── codex/                   # Codex-specific code (isolated from the rest)
│   ├── bridge/              # codex-bridge: wraps codex-core, re-exports types
│   └── sandbox/             # codex-sandbox: resolves/builds codex-linux-sandbox
│
├── llm/                     # LLM abstractions (backend-agnostic)
│   └── core/                # llm-core: traits and types (grow when needed)
│
├── channels/                # Communication transports
│   └── telegram-adapter/    # Telegram-specific: event classification, delta accumulation
│
├── ui/                      # User interfaces
│   └── dashboard/           # egui control panel, Dashboard state, ServiceStatus
│
├── bots/                    # Bot templates and shared template definitions
│   ├── all-bots/            # template enum/configs, wizard flow, instance persistence
│   ├── code-agent/          # full code agent with tool execution and approval flow
│   ├── simple-chat/         # text conversation only
│   └── auto-approve/        # code agent with auto-approved tool calls
│
└── regent/                  # Review-agents that stand in for the user
    └── thragg/              # CLI prototype: Claude Code Stop hook driver
```

## Dependency graph

```
app
  ├── all-bots
  ├── code-agent
  ├── codex-sandbox
  └── dashboard

bots/*
  ├── codex-bridge
  ├── telegram-adapter
  └── dashboard

telegram-adapter
  └── codex-bridge
```

## Principles

- **Bot templates should depend on Codex through `codex-bridge`** when they need thread/protocol access
- **Channels don't depend on UI** — telegram-adapter and dashboard are independent
- **All dependencies use workspace.dependencies** — versions and paths defined once at root
- **Each crate group has a single responsibility** — codex isolation, channel transport, UI, bot behavior

## Adding a new crate

1. Create directory under the appropriate group: `crates/<group>/<crate-name>/`
2. Add `Cargo.toml` with `version.workspace = true` and `edition.workspace = true`
3. Add the crate to `[workspace.dependencies]` in root `Cargo.toml` if other crates will depend on it
4. Add to `members` list in root `Cargo.toml`
