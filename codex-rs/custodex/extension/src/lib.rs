//! Custodex — your fork's customizations, kept out of upstream files.
//!
//! Everything specific to this fork lives under `codex-rs/custodex/`, which
//! `openai/codex` never creates or edits. That keeps `git merge upstream/main`
//! conflict-free: your code is in files upstream has never seen.
//!
//! The only upstream-owned lines you maintain are:
//!   1. the `"custodex/*"` glob in `codex-rs/Cargo.toml` (added once), and
//!   2. a single `custodex_extension::install_all(&mut builder);` line in
//!      `codex-rs/app-server/src/extensions.rs` (add when you ship your first
//!      extension — see `custodex/README.md`).
//!
//! Add new behavior as extension *contributors* registered below. The builder
//! exposes: `tool_contributor` (new tools), `turn_input_contributor` (inject
//! context into each turn), `prompt_contributor`, `thread_lifecycle_contributor`,
//! `turn_lifecycle_contributor`, `tool_lifecycle_contributor`, and more — see
//! `codex-rs/ext/extension-api/src/registry.rs`.

use codex_extension_api::ExtensionRegistryBuilder;

/// Register every custodex extension into the running binary.
///
/// Called once from `app-server/src/extensions.rs`. Generic over the config
/// type `C` so the scaffold needs no dependency on `codex-core`; add that dep
/// only if you register a `config_contributor`, which is `C`-specific.
pub fn install_all<C: Sync>(_builder: &mut ExtensionRegistryBuilder<C>) {
    // Register your contributors here, e.g.:
    //
    //   use std::sync::Arc;
    //   _builder.tool_contributor(Arc::new(my_tool::MyTool::default()));
    //   _builder.turn_input_contributor(Arc::new(my_context::MyContext));
    //
    // Put each contributor in its own module file under `src/` so upstream
    // never touches it and merges stay clean.
}
