# Workspace Layout

## Crate groups

```
crates/
├── codex/                   # Codex-specific code (isolated from the rest)
│   └── bridge/              # codex-bridge: wraps codex-core, re-exports types
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
└── bots/                    # Bot templates (each is a binary crate)
    └── code-agent/          # Full code agent with tool execution and approval flow
```

## Dependency graph

```
bots/code-agent
  ├── codex-bridge
  ├── telegram-adapter
  │   └── codex-bridge
  └── dashboard
```

## Principles

- **Bot templates never depend on codex crates directly** — they go through codex-bridge
- **Channels don't depend on UI** — telegram-adapter and dashboard are independent
- **All dependencies use workspace.dependencies** — versions and paths defined once at root
- **Each crate group has a single responsibility** — codex isolation, channel transport, UI, bot behavior

## Adding a new crate

1. Create directory under the appropriate group: `crates/<group>/<crate-name>/`
2. Add `Cargo.toml` with `version.workspace = true` and `edition.workspace = true`
3. Add the crate to `[workspace.dependencies]` in root `Cargo.toml` if other crates will depend on it
4. Add to `members` list in root `Cargo.toml`
