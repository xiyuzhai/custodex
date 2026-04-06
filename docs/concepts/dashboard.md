# Dashboard

The dashboard is the egui-based GUI for monitoring, debugging, and configuring the bot system.

## Purpose

Debugging, monitoring, and configuration only. The dashboard is not a user-facing chat interface — users interact with the bot through channels (Telegram).

## What the dashboard shows

- **Service status** — bot running/stopped, start/stop control
- **Instances** — list of active bot instances with chat ID, message count, last activity
- **Event log** — live stream of all events from all instances
- **Token usage** — input/output token counts
- **Configuration** — current system config (token path, sandbox, model)

## Config rendering

The dashboard does not know about template-specific config internals. Each bot config implements its own rendering:

- The dashboard provides a region of the UI
- The config struct renders itself into that region using egui
- This keeps the dashboard generic — adding a new template doesn't require dashboard changes

## Instance creation

The dashboard hosts the instance creation wizard. See [design/ui/new_instance.md](../design/ui/new_instance.md) for the wizard flow.
