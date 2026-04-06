# Bridge

A bridge is an abstraction layer over an LLM backend. It hides the backend's internals so bot templates and channels don't couple to a specific provider.

## What a bridge does

- Manages LLM threads/sessions (instance lifecycle)
- Submits user input and receives events
- Handles authentication and configuration
- Re-exports the types that downstream crates need

## Current bridges

**codex-bridge** (`crates/codex/bridge/`)
- Wraps codex-core's ThreadManager and CodexThread
- Handles codex Config, ConfigOverrides, AuthManager setup
- Provides InstanceManager for per-chat thread management
- Re-exports EventMsg, Op, ReviewDecision, UserInput from codex-protocol

## Why a bridge

If the LLM backend changes (different API, different provider, codex internals refactored), only the bridge crate needs updating. Bot templates and channels remain untouched.

## Relationship to llm-core

`crates/llm/core/` will define backend-agnostic traits. The bridge implements those traits. Bot templates depend on `llm-core`, not on the bridge directly. This layer will be built out when a second backend is needed.
