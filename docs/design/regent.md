# Regent Design

## Customization: layered configuration

Three layers, later overrides earlier. Mirrors Claude Code's own settings layering (`.claude/settings.json` + `.claude/settings.local.json` + `~/.claude/`).

### `.regent/` — committed, per-project

Team-shared. Lives at the project root, checked into git. Holds anything the team agrees on for this project:

- Baseline regent prompt
- Skill definitions
- Default mode
- Project-specific review rules

### `.regent/local/` — gitignored, per-checkout

Personal to this clone. Added to the project's `.gitignore`. Holds anything noisy, sensitive, or specific to one machine:

- Working memory (current session state)
- Episodic decision log (what the regent decided, what the user overrode)
- Personal mode toggle for this checkout
- Per-project secrets

### `~/.regent/` — per-user, machine-wide

Follows the human across projects. Holds preferences and identity that don't belong to any single project:

- Default mode that applies when a project doesn't pin one
- Model / API credentials
- "How I think about code" — long-running preferences distilled across projects

### Layer matrix

|                                | `.regent/` | `.regent/local/` | `~/.regent/` |
| ------------------------------ | ---------- | ---------------- | ------------ |
| regent prompt                  | baseline   | override         | override     |
| skill definitions              | ✓          | ✓                | ✓ defaults   |
| mode definitions               | ✓          | —                | —            |
| current mode                   | —          | ✓                | ✓ fallback   |
| episodic log                   | ✗          | ✓                | —            |
| preferences (user disagreed…)  | ✗          | possibly         | ✓ probably   |
| secrets / model credentials    | ✗          | ✗                | ✓            |

### File formats

- **Markdown** for prompts and skills — easy for the user to hand-edit, naturally human-readable.
- **TOML** for config switches (current mode, enabled skills, thresholds).
- **JSONL** for the episodic decision log — append-only, each line one decision record.

### Discovery

Walk up from `cwd` looking for `.regent/`, with `$REGENT_DIR` as an explicit override for ambiguous cases (monorepos, scripts run from elsewhere). Per-user `~/.regent/` is loaded unconditionally and merged underneath.

## CLI runtime

The thragg binary is the prototype runtime. It receives the Stop hook payload on stdin, parses it via `StopHookInput`, and emits `StopHookOutput` on stdout. `decision = "block"` blocks the stop and feeds `reason` back to Claude; an empty object lets the stop proceed.

The `stop_hook_active` flag is honored as a re-entrancy guard — if the regent's own injected reply triggered another stop, thragg allows it through to avoid loops.

## Open

- Where exactly do "preferences" live? They follow the human (`~/.regent/`), but project-specific norms in `.regent/` may need to override them. Resolution rule TBD.
- How does the regent learn? The cleanest signal is "user disagreed with my call" — captured in the episodic log, distilled into preferences over time.
- Is the regent always a codex chat, or do we also want cheap rule-based shortcuts before reaching for an LLM?
