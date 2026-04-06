# Event Handling

## EventMsg → TelegramAction mapping

codex-core emits `EventMsg` variants through `CodexThread::next_event()`. The adapter (`adapter.rs`) classifies each into a `TelegramAction`.

## Handled events

| EventMsg | TelegramAction | Behavior |
|----------|---------------|----------|
| `AgentMessageDelta` | `Delta(text)` | Buffered, sent/edited as single message with 500ms throttle |
| `AgentMessage` | `Skip` | Skipped — deltas already cover the full text |
| `Error` | `Send` | Sent as standalone error message |
| `ExecCommandBegin` | `Send` | Shows command being run |
| `ExecCommandEnd` | `Send` | Shows command completion |
| `PatchApplyBegin` | `Send` | Lists files being patched |
| `PatchApplyEnd` | `Send` | Reports success/failure |
| `ExecApprovalRequest` | `ApprovalPrompt` | Inline keyboard: Approve / Approve (session) / Deny |
| `ApplyPatchApprovalRequest` | `ApprovalPrompt` | Inline keyboard: Approve / Approve (session) / Deny |
| `TokenCount` | `Skip` | Token counts extracted and pushed to MonitorState |
| `TurnStarted` | `Skip` | — |
| `TurnComplete` | `Skip` | Breaks the event drain loop |
| `TurnAborted` | `Skip` | Breaks the event drain loop |
| `SessionConfigured` | `Skip` | — |
| All others | `Skip` | Logged to monitor, not sent to Telegram |

## Approval flow

When an approval event arrives:

1. Delta buffer is flushed
2. Inline keyboard sent to Telegram with three options:
   - **Approve** → `ReviewDecision::Approved`
   - **Approve (session)** → `ReviewDecision::ApprovedForSession`
   - **Deny** → `ReviewDecision::Denied`
3. `PendingApproval` stored in `ChatState`
4. Event drain loop pauses (returns)
5. When user taps a button → `handle_callback` → submits `Op::ExecApproval` or `Op::PatchApproval`
6. Event drain loop resumes

## Delta accumulation

Agent text responses arrive as many small `AgentMessageDelta` events. Without buffering, each would create a separate Telegram message (spam).

Strategy:
- Buffer all deltas into `delta_buf: String`
- On first delta: `bot.send_message()` → get `MessageId`
- On subsequent deltas: `bot.edit_message_text()` with accumulated text
- Throttled to one edit per 500ms to avoid Telegram rate limits
- Final flush on turn end

## Events not yet handled

These events are logged to the monitor but not surfaced to Telegram users:

- `AgentReasoning` / `AgentReasoningDelta` — could show thinking indicator
- `ExecCommandOutputDelta` — could stream command output
- `McpToolCallBegin/End` — could show MCP tool usage
- `WebSearchBegin/End` — could show search activity
- `GuardianAssessment` — safety review events
- `StreamError` — retry/backoff notifications
- `RequestUserInput` — agent asking for clarification (would need Telegram UI)
- `RequestPermissions` — broader permission requests

These can be added incrementally as needed.
