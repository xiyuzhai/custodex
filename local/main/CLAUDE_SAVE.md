# local/&lt;branch&gt;/CLAUDE_SAVE.md — tracked, per-branch source for CLAUDE.md

Upstream's `.gitignore` ignores `CLAUDE.md` (unanchored, any depth), so the root
`CLAUDE.md` can't be committed. The authoritative guidance is tracked here, **one
copy per branch**, at `local/<branch-name>/CLAUDE_SAVE.md`. The `_SAVE` name
dodges the ignore rule, so **no `.gitignore` changes are needed**.

Convention:
- **Edit `local/<current-branch>/CLAUDE_SAVE.md`** — never the root `CLAUDE.md`.
- At session start, derive the copy Claude Code actually reads:
  ```sh
  cp "local/$(git rev-parse --abbrev-ref HEAD)/CLAUDE_SAVE.md" CLAUDE.md
  ```
- Root `CLAUDE.md` stays gitignored and local-only; it's regenerated, never edited.
- **New branch:** create `local/<new-branch>/CLAUDE_SAVE.md` (copy from the base
  branch's file as a starting point) so the branch joins the convention.
- Per-branch files live at distinct paths, so they never conflict when you merge
  your own branches; and upstream never has a `CLAUDE_SAVE.md`, so they never
  conflict on sync.

---

Guidance for Claude Code working in **this fork** of `openai/codex`.

## What this fork is for

This is a fork of `openai/codex` that we keep **continuously in sync with
upstream** while layering our own customizations on top. The guiding principle
is: **stay easy to sync.** Upstream moves fast (hundreds of commits between
pulls), so our changes must not fight upstream's.

The way we achieve that: put our code in files upstream never edits, and plug it
in through codex's own extension API instead of modifying core. Concretely, all
our work lives in **`codex-rs/custodex/`** — see `codex-rs/custodex/README.md`
for the full strategy and how to add an extension.

## The rule for any change

Before editing a file, ask: *does upstream also edit this file?*

- **New behavior → a new crate/module under `codex-rs/custodex/`**, registered
  through `ExtensionRegistryBuilder` (the same mechanism OpenAI uses for their
  own `goal`, `guardian`, `web-search`, `memories`, `image-generation`
  features). Zero conflict surface.
- **Touching shared upstream files → last resort, and keep it minimal.** Our
  entire permanent footprint on upstream-owned files is two append-style lines:
  the `"custodex/*"` glob in `codex-rs/Cargo.toml`, and (once we ship a real
  extension) one `custodex_extension::install_all(&mut builder)` call in
  `codex-rs/app-server/src/extensions.rs`. If you must add more, document it in
  `codex-rs/custodex/README.md` so it can be re-checked after each sync.

If a change genuinely cannot be expressed through the extension API (e.g.
altering existing core/TUI behavior), edit upstream directly — but isolate it
and record it in the custodex README.

## Remotes & syncing

- `origin` → `git@github.com:xiyuzhai/codex.git` (our fork; we push here)
- `upstream` → `git@github.com:openai/codex.git` (read-only; we pull from here)

```sh
git fetch upstream
git merge --ff-only upstream/main   # fast-forward while we have no local commits
# once we have custodex commits: git merge upstream/main (resolve the tiny shared lines if prompted)
git push origin main
```

## Upstream conventions

For build, test, formatting, and style rules, follow upstream's top-level
`AGENTS.md` (it covers the `codex-rs/` Rust conventions inline). This fork does
not override them.
