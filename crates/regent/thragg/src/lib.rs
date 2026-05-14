// Thragg: prototype review-agent ("regent"), shipped as a CLI tool that
// Claude Code invokes as a Stop hook.
//
// Flow: Claude Code spawns `thragg` and writes the Stop hook payload to
// stdin. Thragg parses it, reads the transcript, asks a codex chat to
// judge whether to block the stop or allow it, and writes the Stop hook
// response to stdout.
//
// Planned subsystems (modules to be added as the prototype grows):
//   - memory:  durable + working memory the regent can read/write
//   - mode:    user-switchable behavior modes (e.g. strict, lenient, silent)
//   - skill:   pluggable review skills the user can author and toggle
//   - learn:   local distillation of regent decisions into small models

use serde::{Deserialize, Serialize};

/// Claude Code Stop hook input, read from stdin.
#[derive(Debug, Deserialize)]
pub struct StopHookInput {
    pub session_id: String,
    pub transcript_path: String,
    #[serde(default)]
    pub cwd: Option<String>,
    pub hook_event_name: String,
    #[serde(default)]
    pub stop_hook_active: bool,
}

/// Claude Code Stop hook output, written to stdout.
///
/// `decision = "block"` blocks the stop and feeds `reason` back to Claude.
/// Omitting `decision` (or writing `{}`) allows the stop to proceed.
#[derive(Debug, Serialize, Default)]
pub struct StopHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// What the regent decides after reviewing the transcript.
pub enum Decision {
    /// Block the stop, inject this reply so the agent keeps going.
    Reply(String),
    /// Allow the stop to proceed.
    AllowStop,
}

impl Decision {
    pub fn into_stop_hook_output(self) -> StopHookOutput {
        match self {
            Decision::Reply(reason) => StopHookOutput {
                decision: Some("block".to_string()),
                reason: Some(reason),
            },
            Decision::AllowStop => StopHookOutput::default(),
        }
    }
}
