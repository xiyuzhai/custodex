# Creating a Bot Template

How to add a new bot template to the system.

## 1. Create the crate

```
mkdir -p crates/bots/<template-name>/src
```

Create `crates/bots/<template-name>/Cargo.toml`:

```toml
[package]
name = "<template-name>"
version.workspace = true
edition.workspace = true

[dependencies]
codex-bridge.workspace = true
telegram-adapter.workspace = true
dashboard.workspace = true
teloxide.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
eframe.workspace = true
```

Add to workspace root `Cargo.toml`:
- Add `"crates/bots/<template-name>"` to `members`

## 2. Define your config

Create a config struct specific to your template. It should implement egui rendering for the wizard steps.

```rust
pub struct MyBotConfig {
    pub model: String,
    // ... template-specific fields
}
```

## 3. Define wizard steps

Each step corresponds to a screen in the instance creation wizard. The config struct renders each step and validates before advancing.

## 4. Implement message handling

Define how your bot processes incoming messages and events. Use `codex-bridge::InstanceManager` for thread management and `telegram-adapter` for event classification.

## 5. Create main.rs

Wire up the dashboard with your bot launcher. See `crates/bots/code-agent/src/main.rs` for the pattern.

## Reference

See `crates/bots/code-agent/` as a working example.
