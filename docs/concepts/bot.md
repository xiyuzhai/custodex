# Bot

Three related concepts: Bot Template, Bot Config, Bot Instance.

## Bot Template

A bot template defines a category of bot behavior. It determines:

- How user messages are processed
- What tools and capabilities are available
- What configuration is required
- How the config wizard steps are rendered

Each template lives as a crate in `crates/bots/<template-name>/`.

Examples:
- `code-agent` — full code agent with tool execution, file editing, approval flow
- `simple-chat` — text conversation only, no tools
- `auto-approve` — code agent that auto-approves all tool calls
- `review-only` — can read and search code but cannot modify

## Bot Config

Configuration specific to a bot template. Each template defines its own config struct.

Properties:
- Owned by the template — the dashboard does not know config internals
- Implements its own rendering — each config struct renders its own egui UI for the wizard and display
- Validated per step — the wizard advances only when the current step passes validation
- Minimal shared config — only the telegram bot token and basic settings are common across templates

Example: a `code-agent` config might include working directory, model, approval policy, and sandbox settings. A `simple-chat` config might only include model selection.

## Bot Instance

A running combination of a bot template and a bot config, bound to a Telegram chat.

```
Bot Instance = Bot Template + Bot Config + Chat binding
```

Lifecycle:
1. User creates a new instance from the dashboard
2. Selects a template
3. Walks through the config wizard (template-specific steps)
4. Instance starts and is bound to a Telegram chat
5. Instance processes messages according to its template's behavior
6. Instance can be stopped, reconfigured, or destroyed from the dashboard

An instance holds a CodexThread (via the bridge) and maintains per-chat state like pending approvals and conversation history.
