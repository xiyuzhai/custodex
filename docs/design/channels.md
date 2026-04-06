# Channel Design

How channels adapt between the bot's internal event model and external platforms.

## Telegram adapter

Current implementation in `crates/channels/telegram-adapter/`.

### Event classification

`classify_event(EventMsg) → TelegramAction`

Maps codex events to one of:
- `Delta(text)` — buffer and edit message in place
- `Send(text)` — standalone message
- `ApprovalPrompt(info)` — inline keyboard for user decision
- `Skip` — no user-visible output

### Delta accumulation

Agent responses arrive as many small deltas. The adapter:
1. Buffers deltas into a string
2. Sends one message on first delta, edits it on subsequent ones
3. Throttles edits to 500ms to stay within Telegram rate limits
4. Final flush on turn end

### Approval keyboards

Tool execution and patch application require user approval. The adapter renders inline keyboards with Approve / Approve (session) / Deny buttons.

## Future channels

Each channel crate in `crates/channels/` would implement:
- Message receiving (platform webhook or polling)
- Response delivery (platform-specific formatting)
- Approval UI (platform-specific interactive elements)
- Rate limit handling (platform-specific throttling)

The internal event model stays the same — only the rendering changes per channel.
