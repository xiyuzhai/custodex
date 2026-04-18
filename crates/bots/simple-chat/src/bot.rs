use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use codex_bridge::{
    CodexConversationId, CodexConversationManager, CodexConversationManagerConfig, EventMsg,
};
use dashboard::{ServiceStatus, SharedDashboard};
use telegram_adapter::{TelegramAction, classify_event, send_or_edit_delta};
use teloxide::prelude::*;
use teloxide::types::{Me, MessageId};

const EDIT_INTERVAL_MS: u128 = 500;

struct BotState {
    instance_mgr: Arc<CodexConversationManager>,
    dashboard: SharedDashboard,
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
            d.push_log(None, format!("simple-chat bot failed to start: {e}"));
            return;
        }
    };

    {
        let mut d = dashboard.lock().unwrap();
        d.service_status = ServiceStatus::Running;
        d.push_log(None, format!("simple-chat bot started: @{}", me.username()));
    }

    let instance_mgr = Arc::new(
        CodexConversationManager::new(CodexConversationManagerConfig {
            work_dir,
            sandbox_exe,
            custodex_dir,
        })
        .await,
    );
    let state = Arc::new(BotState {
        instance_mgr,
        dashboard: dashboard.clone(),
    });

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build()
        .dispatch()
        .await;

    {
        let mut d = dashboard.lock().unwrap();
        d.service_status = ServiceStatus::Stopped;
        d.push_log(None, "simple-chat bot stopped.".to_string());
    }
}

async fn handle_message(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let chat_id = msg.chat.id;
    let cid = chat_id.0;
    let conv_id = CodexConversationId::new(cid);

    state.log(Some(cid), format!("User: {text}"));
    {
        let mut d = state.dashboard.lock().unwrap();
        d.update_instance_activity(cid);
    }

    let thread = state.instance_mgr.submit_text(conv_id, text).await;

    // Simple drain: only collect text deltas, skip approvals/commands
    let mut delta_buf = String::new();
    let mut delta_msg_id: Option<MessageId> = bot
        .send_message(chat_id, "In progress...")
        .await
        .ok()
        .map(|m| m.id);
    let mut last_edit = Instant::now();

    loop {
        let event = match thread.next_event().await {
            Ok(ev) => ev,
            Err(e) => {
                tracing::error!("event stream error for chat {cid}: {e}");
                state.log(Some(cid), format!("Event stream error: {e}"));
                break;
            }
        };

        let is_turn_end = matches!(
            event.msg,
            EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_)
        );

        match classify_event(&event.msg) {
            TelegramAction::Delta(text) => {
                delta_buf.push_str(&text);
                let now = Instant::now();
                if now.duration_since(last_edit).as_millis() >= EDIT_INTERVAL_MS || is_turn_end {
                    delta_msg_id =
                        send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
                    last_edit = now;
                }
            }
            TelegramAction::Send(text) => {
                if !delta_buf.is_empty() {
                    send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
                    delta_buf.clear();
                    delta_msg_id = None;
                } else if let Some(msg_id) = delta_msg_id.take() {
                    bot.edit_message_text(chat_id, msg_id, &text).await.ok();
                    continue;
                }
                bot.send_message(chat_id, text).await.ok();
            }
            // Simple chat ignores approval prompts — no tool execution expected
            _ => {}
        }

        if is_turn_end {
            if !delta_buf.is_empty() {
                send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
            }
            break;
        }
    }

    Ok(())
}
