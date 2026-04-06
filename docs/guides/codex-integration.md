# Codex Integration

How codex-telegram uses codex-core as its LLM backend.

## Dependency chain

```
Bot template → codex-bridge → codex-core, codex-protocol, codex-login, codex-exec-server
```

Bot templates never import codex crates directly. `codex-bridge` re-exports the types they need.

## Authentication

Uses `AuthManager::shared()` from codex-login with `AuthCredentialsStoreMode::Auto`:
- Tries OS keyring first
- Falls back to `~/.codex/auth.json`
- Also checks `OPENAI_API_KEY` env var

Pre-requisite: run the `codex` CLI once to log in, or set the env var.

## Sandbox

The Landlock sandbox restricts what agent commands can access on the filesystem.

Binary: `codex-linux-sandbox`, built from `../codex/codex-rs/linux-sandbox`.

Passed via `ConfigOverrides::codex_linux_sandbox_exe`. The path is resolved at compile time relative to `CARGO_MANIFEST_DIR`.

Build it with:
```
make sandbox
```

Requires `libcap-dev` (`sudo apt-get install -y libcap-dev`).

## Configuration

codex-bridge loads config from standard codex locations:
- `~/.codex/config.toml` — user-level
- Project-level `codex.toml`

Key settings: `model`, `permissions.approval_policy`, `sandbox_policy`.

## Patched dependencies

codex-core requires forked versions of `tokio-tungstenite` and `tungstenite` (for proxy support). These are declared in `[patch.crates-io]` at the workspace root.
