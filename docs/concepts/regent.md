# Regent

A regent is a review-agent: it stands in for the user while another agent is working, reviewing recent activity and deciding whether to reply on the user's behalf, message the user, or hand control back.

## Primary use case

Claude Code Stop hook. When Claude is about to stop and yield to the user, the regent intercepts and decides:

- **Block the stop** with an injected reply that keeps Claude going.
- **Allow the stop** so the user takes over.

## Current prototype

`crates/regent/thragg/` — CLI binary invoked by Claude Code as a Stop hook. Reads the hook payload from stdin, asks a codex chat to judge, writes the hook response to stdout.

## Planned subsystems

- **memory** — durable + working memory the regent reads and writes
- **mode** — user-switchable behavior modes (strict / lenient / silent)
- **skill** — pluggable review skills the user authors and toggles
- **learn** — local distillation of regent decisions into small models

## Why a regent

The user wants oversight without being in the loop on every turn. The regent absorbs the user's preferences over time and stands in for them on routine decisions, escalating only when needed.
