# custodex — fork customizations that survive upstream syncs

This directory holds everything specific to **this fork** of `openai/codex`.
The goal: keep `git merge upstream/main` (almost) conflict-free by putting all
custom code in files upstream **never creates or edits**.

## Why this layout

Merge conflicts only happen on lines that *both* you and upstream changed. So
the strategy is to touch upstream files as little as possible:

- **New behavior → new crates under `custodex/`.** Upstream has never seen these
  files, so they can never conflict.
- **The extension API does the wiring.** `codex` is built for this: OpenAI ships
  their own features (`goal`, `guardian`, `memories`, `web-search`,
  `image-generation`) the exact same way — as crates registered through
  `ExtensionRegistryBuilder`. We follow that pattern instead of editing core.

### Total upstream footprint: two lines

1. `codex-rs/Cargo.toml` — the `"custodex/*"` glob in `members` (added once;
   new custodex crates need **no** further edits here).
2. `codex-rs/app-server/src/extensions.rs` — one `custodex_extension::install_all`
   call (added when you ship your first real extension; see below).

Both are tiny, append-style lines that are trivial to re-apply if they ever
conflict.

## Adding a customization

Most things fit the extension API — add them to `custodex/extension/`:

1. Create a module file, e.g. `custodex/extension/src/my_tool.rs`, implementing
   the relevant contributor trait (`ToolContributor`, `TurnInputContributor`,
   `ContextContributor`, `ThreadLifecycleContributor`, …). See the trait
   definitions in `codex-rs/ext/extension-api/src/` and a worked example in
   `codex-rs/ext/web-search/`.
2. Register it inside `install_all` in `custodex/extension/src/lib.rs`.
3. **Wire `install_all` into the binary (first time only).** Add to
   `codex-rs/app-server/src/extensions.rs`, inside `thread_extensions(...)`,
   right before `Arc::new(builder.build())`:

   ```rust
   custodex_extension::install_all(&mut builder);
   ```

   and add the dependency. In `codex-rs/Cargo.toml` under
   `[workspace.dependencies]`:

   ```toml
   custodex-extension = { path = "custodex/extension" }
   ```

   and in `codex-rs/app-server/Cargo.toml` under `[dependencies]`:

   ```toml
   custodex-extension = { workspace = true }
   ```

If a change genuinely can't be expressed through the extension API (e.g.
altering existing core/TUI behavior), edit upstream files directly — but keep
those edits small and isolated, and note them in this README so you remember
what to re-check after each sync.

## Syncing with upstream

```sh
git fetch upstream
git merge --ff-only upstream/main   # fast-forward when you have no local commits
# ...or, once you have custodex commits of your own:
git merge upstream/main             # resolve the (tiny) Cargo.toml / extensions.rs lines if prompted
git push origin main
```

`upstream` → `git@github.com:openai/codex.git`, `origin` → your fork.
