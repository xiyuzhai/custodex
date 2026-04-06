use std::sync::Arc;
use std::time::Instant;

use std::path::PathBuf;

use codex_bridge::{EventMsg, InstanceManager, InstanceManagerConfig};
use dashboard::{SharedDashboard, ServiceStatus};
use telegram_adapter::{TelegramAction, classify_event, send_or_edit_delta};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me, MessageId, ParseMode};

/// Minimum interval between Telegram message edits to avoid rate limits.
const EDIT_INTERVAL_MS: u128 = 500;

struct BotState {
    instance_mgr: Arc<InstanceManager>,
    dashboard: SharedDashboard,
}

impl BotState {
    fn log(&self, chat_id: Option<i64>, message: String) {
        let mut d = self.dashboard.lock().unwrap();
        d.push_log(chat_id, message);
    }
}

pub async fn run_bot(token: String, dashboard: SharedDashboard, work_dir: PathBuf, custodex_dir: PathBuf, sandbox_exe: Option<PathBuf>) {
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

    let instance_mgr = Arc::new(InstanceManager::new(InstanceManagerConfig {
        work_dir,
        sandbox_exe,
        custodex_dir,
    }).await);
    let state = Arc::new(BotState {
        instance_mgr,
        dashboard: dashboard.clone(),
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

async fn handle_message(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> ResponseResult<()> {
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
    drain_events(&bot, chat_id, &thread, &state).await;
    Ok(())
}

async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    state: Arc<BotState>,
) -> ResponseResult<()> {
    let Some(data) = q.data.as_deref() else {
        return Ok(());
    };
    let Some(msg) = q.message else {
        return Ok(());
    };

    let chat_id = msg.chat().id;
    // For now, callbacks are acknowledged but approval state management
    // needs to be integrated with the instance manager.
    // TODO: wire up pending approval tracking
    bot.send_message(
        chat_id,
        match data {
            "approve" | "approve_session" => "Approved.",
            _ => "Denied.",
        },
    )
    .await
    .ok();

    state.log(Some(chat_id.0), format!("Callback: {data}"));
    Ok(())
}

#[allow(unused_assignments)]
async fn drain_events(
    bot: &Bot,
    chat_id: ChatId,
    thread: &codex_bridge::CodexThread,
    state: &BotState,
) {
    let cid = chat_id.0;
    let mut delta_buf = String::new();
    let mut delta_msg_id: Option<MessageId> = None;
    let mut last_edit = Instant::now();

    loop {
        let event = match thread.next_event().await {
            Ok(ev) => ev,
            Err(e) => {
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

                // TODO: store pending approval and resume after callback
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
