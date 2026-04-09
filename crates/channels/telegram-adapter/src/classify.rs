use codex_bridge::EventMsg;

/// Categorized adapter output for the event loop to handle differently.
pub enum TelegramAction {
    /// Append this text to the current streaming message (edit in place).
    Delta(String),
    /// Send a complete standalone message (not part of the streaming buffer).
    Send(String),
    /// Show an approval prompt with inline keyboard.
    ApprovalPrompt(ApprovalInfo),
    /// No visible action needed.
    Skip,
}

pub struct ApprovalInfo {
    pub text: String,
    pub call_id: String,
    pub turn_id: String,
    pub kind: ApprovalKind,
}

pub enum ApprovalKind {
    Exec,
    Patch,
}

pub fn classify_event(event: &EventMsg) -> TelegramAction {
    match event {
        EventMsg::AgentMessageDelta(delta) => TelegramAction::Delta(delta.delta.clone()),

        // AgentMessage is the final complete message — skip it since deltas already cover it.
        EventMsg::AgentMessage(_) => TelegramAction::Skip,

        EventMsg::Error(err) => TelegramAction::Send(format!("Error: {}", err.message)),

        // Command lifecycle noise stays in the local dashboard log rather than spamming Telegram.
        EventMsg::ExecCommandBegin(_) => TelegramAction::Skip,

        EventMsg::ExecCommandEnd(_) => TelegramAction::Skip,

        EventMsg::PatchApplyBegin(_) => TelegramAction::Skip,

        EventMsg::PatchApplyEnd(patch) => {
            if patch.success {
                TelegramAction::Skip
            } else {
                TelegramAction::Send(format!("Patch failed: {}", patch.stderr))
            }
        }

        EventMsg::ExecApprovalRequest(req) => TelegramAction::ApprovalPrompt(ApprovalInfo {
            text: format!(
                "Approve command?\n`{}`\nin {}",
                req.command.join(" "),
                req.cwd.display()
            ),
            call_id: req.call_id.clone(),
            turn_id: req.turn_id.clone(),
            kind: ApprovalKind::Exec,
        }),

        EventMsg::ApplyPatchApprovalRequest(req) => {
            let files: Vec<_> = req
                .changes
                .keys()
                .map(|p| p.display().to_string())
                .collect();
            TelegramAction::ApprovalPrompt(ApprovalInfo {
                text: format!("Approve file changes?\n{}", files.join("\n")),
                call_id: req.call_id.clone(),
                turn_id: req.turn_id.clone(),
                kind: ApprovalKind::Patch,
            })
        }

        EventMsg::RequestUserInput(request) => {
            let summary = request
                .questions
                .iter()
                .map(|q| format!("{}: {}", q.header, q.question))
                .collect::<Vec<_>>()
                .join("\n");
            TelegramAction::Send(format!("Question from agent:\n{summary}"))
        }

        _ => TelegramAction::Skip,
    }
}
