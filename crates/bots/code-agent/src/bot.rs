use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use codex_bridge::{EventMsg, InstanceManager, InstanceManagerConfig, Op, ReviewDecision};
use dashboard::{ServiceStatus, SharedDashboard};
use telegram_adapter::{ApprovalKind, TelegramAction, classify_event, send_or_edit_delta};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me, MessageId, ParseMode};
use tokio::sync::Mutex;

/// Minimum interval between Telegram message edits to avoid rate limits.
const EDIT_INTERVAL_MS: u128 = 500;

struct PendingApproval {
    call_id: String,
    turn_id: String,
    kind: ApprovalKind,
    thread: Arc<codex_bridge::CodexThread>,
}

struct BotState {
    instance_mgr: Arc<InstanceManager>,
    dashboard: SharedDashboard,
    pending_approvals: Mutex<HashMap<i64, PendingApproval>>,
}

impl BotState {
    fn log(&self, chat_id: Option<i64>, message: String) {
        let mut d = self.dashboard.lock().unwrap();
        d.push_log(chat_id, message);
    }
}

pub async fn run_bot(
    token: String,
    dashboard: SharedDashboard,
    work_dir: PathBuf,
    custodex_dir: PathBuf,
    sandbox_exe: Option<PathBuf>,
) {
    let bot = Bot::new(&token);

    let me: Me = match bot.get_me().await {
        Ok(me) => me,
        Err(e) => {
            let mut d = dashboard.lock().unwrap();
            d.service_status = ServiceStatus::Error(format!("Failed to connect: {e}"));
            d.push_log(None, format!("Bot failed to start: {e}"));
            return;
        }
    };

    {
        let mut d = dashboard.lock().unwrap();
        d.service_status = ServiceStatus::Running;
        d.push_log(None, format!("Bot started: @{}", me.username()));
    }

    let instance_mgr = Arc::new(
        InstanceManager::new(InstanceManagerConfig {
            work_dir,
            sandbox_exe,
            custodex_dir,
        })
        .await,
    );
    let state = Arc::new(BotState {
        instance_mgr,
        dashboard: dashboard.clone(),
        pending_approvals: Mutex::new(HashMap::new()),
    });

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build()
        .dispatch()
        .await;

    {
        let mut d = dashboard.lock().unwrap();
        d.service_status = ServiceStatus::Stopped;
        d.push_log(None, "Bot stopped.".to_string());
    }
}

async fn handle_message(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let chat_id = msg.chat.id;
    let cid = chat_id.0;

    state.log(Some(cid), format!("User: {text}"));
    {
        let mut d = state.dashboard.lock().unwrap();
        d.update_instance_activity(cid);
    }

    let thread = state.instance_mgr.submit_text(cid, text).await;
    let progress_msg_id = bot
        .send_message(chat_id, "In progress...")
        .await
        .ok()
        .map(|m| m.id);
    drain_events(&bot, chat_id, &thread, &state, progress_msg_id).await;
    Ok(())
}

async fn handle_callback(bot: Bot, q: CallbackQuery, state: Arc<BotState>) -> ResponseResult<()> {
    let Some(data) = q.data.as_deref() else {
        return Ok(());
    };
    let Some(msg) = q.message else {
        return Ok(());
    };

    let chat_id = msg.chat().id;
    let cid = chat_id.0;

    let approval = {
        let mut pending = state.pending_approvals.lock().await;
        pending.remove(&cid)
    };

    let Some(approval) = approval else {
        state.log(Some(cid), "Callback with no pending approval.".to_string());
        return Ok(());
    };

    let decision = match data {
        "approve" => ReviewDecision::Approved,
        "approve_session" => ReviewDecision::ApprovedForSession,
        _ => ReviewDecision::Denied,
    };

    state.log(
        Some(cid),
        format!("Approval: {data} for {}", approval.call_id),
    );

    let op = match approval.kind {
        ApprovalKind::Exec => Op::ExecApproval {
            id: approval.call_id,
            turn_id: Some(approval.turn_id),
            decision,
        },
        ApprovalKind::Patch => Op::PatchApproval {
            id: approval.call_id,
            decision,
        },
    };

    if let Err(e) = state.instance_mgr.submit_approval(cid, op).await {
        tracing::error!("failed to submit approval: {e}");
        bot.send_message(chat_id, format!("Failed to submit approval: {e}"))
            .await
            .ok();
        return Ok(());
    }

    bot.send_message(
        chat_id,
        match data {
            "approve" | "approve_session" => "Approved.",
            _ => "Denied.",
        },
    )
    .await
    .ok();

    // Resume draining events after approval
    let progress_msg_id = bot
        .send_message(chat_id, "In progress...")
        .await
        .ok()
        .map(|m| m.id);
    drain_events(&bot, chat_id, &approval.thread, &state, progress_msg_id).await;
    Ok(())
}

#[allow(unused_assignments)]
async fn drain_events(
    bot: &Bot,
    chat_id: ChatId,
    thread: &codex_bridge::CodexThread,
    state: &BotState,
    initial_msg_id: Option<MessageId>,
) {
    let cid = chat_id.0;
    let mut delta_buf = String::new();
    let mut delta_msg_id: Option<MessageId> = initial_msg_id;
    let mut last_edit = Instant::now();

    loop {
        let event = match thread.next_event().await {
            Ok(ev) => ev,
            Err(e) => {
                tracing::error!("event stream error for chat {cid}: {e}");
                state.log(Some(cid), format!("Event stream error: {e}"));
                bot.send_message(chat_id, format!("Internal error: {e}"))
                    .await
                    .ok();
                break;
            }
        };

        let is_turn_end = matches!(
            event.msg,
            EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_)
        );

        // Log token counts
        if let EventMsg::TokenCount(tc) = &event.msg {
            if let Some(info) = &tc.info {
                let mut d = state.dashboard.lock().unwrap();
                d.input_tokens = info.total_token_usage.input_tokens as u64;
                d.output_tokens = info.total_token_usage.output_tokens as u64;
            }
        }

        let event_summary = format!("{:?}", std::mem::discriminant(&event.msg));
        state.log(Some(cid), event_summary);

        match classify_event(&event.msg) {
            TelegramAction::Delta(text) => {
                delta_buf.push_str(&text);
                let now = Instant::now();
                if now.duration_since(last_edit).as_millis() >= EDIT_INTERVAL_MS || is_turn_end {
                    delta_msg_id =
                        send_or_edit_delta(bot, chat_id, delta_msg_id, &delta_buf).await;
                    last_edit = now;
                }
            }
            TelegramAction::Send(text) => {
                state.log(Some(cid), format!("-> {text}"));
                if !delta_buf.is_empty() {
                    send_or_edit_delta(bot, chat_id, delta_msg_id, &delta_buf).await;
                    delta_buf.clear();
                    delta_msg_id = None;
                } else if let Some(msg_id) = delta_msg_id.take() {
                    bot.edit_message_text(chat_id, msg_id, &text).await.ok();
                    continue;
                }
                bot.send_message(chat_id, text).await.ok();
            }
            TelegramAction::ApprovalPrompt(info) => {
                state.log(
                    Some(cid),
                    format!("Approval needed: {} ({})", info.call_id, info.text),
                );
                if !delta_buf.is_empty() {
                    send_or_edit_delta(bot, chat_id, delta_msg_id, &delta_buf).await;
                    delta_buf.clear();
                    delta_msg_id = None;
                } else if let Some(msg_id) = delta_msg_id.take() {
                    bot.edit_message_text(chat_id, msg_id, "In progress. Awaiting approval...")
                        .await
                        .ok();
                }

                let keyboard = InlineKeyboardMarkup::new(vec![vec![
                    InlineKeyboardButton::callback("Approve", "approve"),
                    InlineKeyboardButton::callback("Approve (session)", "approve_session"),
                    InlineKeyboardButton::callback("Deny", "deny"),
                ]]);

                bot.send_message(chat_id, &info.text)
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_markup(keyboard)
                    .await
                    .ok();

                // Store pending approval — handle_callback will resume
                let inst = state.instance_mgr.get_or_create(cid).await;
                let thread_arc = {
                    let i = inst.lock().await;
                    Arc::clone(&i.thread)
                };
                {
                    let mut pending = state.pending_approvals.lock().await;
                    pending.insert(
                        cid,
                        PendingApproval {
                            call_id: info.call_id,
                            turn_id: info.turn_id,
                            kind: info.kind,
                            thread: thread_arc,
                        },
                    );
                }
                return;
            }
            TelegramAction::Skip => {}
        }

        if is_turn_end {
            state.log(Some(cid), "Turn ended.".to_string());
            if !delta_buf.is_empty() {
                send_or_edit_delta(bot, chat_id, delta_msg_id, &delta_buf).await;
            }
            break;
        }
    }
}
