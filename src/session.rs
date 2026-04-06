use std::sync::Arc;
use std::time::Instant;

use codex_core::config::{Config, ConfigOverrides};
use codex_core::CodexThread;
use codex_core::NewThread;
use codex_core::ThreadManager;
use codex_exec_server::EnvironmentManager;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthManager;
use codex_models_manager::collaboration_mode_presets::CollaborationModesConfig;
use codex_protocol::protocol::{EventMsg, Op, ReviewDecision, SessionSource};
use codex_protocol::user_input::UserInput;
use dashmap::DashMap;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode};
use tokio::sync::Mutex;

use crate::adapter::{self, ApprovalKind, TelegramAction};
use crate::monitor::SharedMonitor;

/// Minimum interval between Telegram message edits to avoid rate limits.
const EDIT_INTERVAL_MS: u128 = 500;

/// Pending approval waiting for user callback.
struct PendingApproval {
    call_id: String,
    turn_id: String,
    kind: ApprovalKind,
}

/// Per-chat state.
struct ChatState {
    thread: Arc<CodexThread>,
    pending_approval: Option<PendingApproval>,
}

pub struct SessionManager {
    thread_mgr: Arc<ThreadManager>,
    chats: DashMap<ChatId, Arc<Mutex<ChatState>>>,
    monitor: SharedMonitor,
}

impl SessionManager {
    pub async fn new(monitor: SharedMonitor) -> Self {
        let sandbox_exe = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../codex/codex-rs/target/release/codex-linux-sandbox");
        let overrides = ConfigOverrides {
            codex_linux_sandbox_exe: Some(sandbox_exe),
            ..Default::default()
        };
        let config = Config::load_with_cli_overrides_and_harness_overrides(vec![], overrides)
            .await
            .expect("failed to load codex config");

        let auth_manager = AuthManager::shared(
            config.codex_home.clone(),
            /*enable_codex_api_key_env*/ true,
            AuthCredentialsStoreMode::Auto,
        );

        let environment_manager = Arc::new(EnvironmentManager::new(/*exec_server_url*/ None));

        let thread_mgr = Arc::new(ThreadManager::new(
            &config,
            auth_manager,
            SessionSource::Custom("telegram".to_string()),
            CollaborationModesConfig {
                default_mode_request_user_input: false,
            },
            environment_manager,
        ));

        Self {
            thread_mgr,
            chats: DashMap::new(),
            monitor,
        }
    }

    fn log(&self, chat_id: Option<i64>, message: String) {
        let mut m = self.monitor.lock().unwrap();
        m.push_log(chat_id, message);
    }

    pub async fn handle_user_message(&self, bot: Bot, chat_id: ChatId, text: &str) {
        let cid = chat_id.0;
        self.log(Some(cid), format!("User: {text}"));
        {
            let mut m = self.monitor.lock().unwrap();
            m.update_session_activity(cid);
        }

        let chat_state = self.get_or_create_chat(chat_id).await;
        let state = chat_state.lock().await;
        let thread = Arc::clone(&state.thread);
        drop(state);

        let _sub_id = thread
            .submit(Op::UserInput {
                items: vec![UserInput::Text {
                    text: text.to_string(),
                    text_elements: vec![],
                }],
                final_output_json_schema: None,
            })
            .await
            .expect("failed to submit user input");

        self.drain_events(bot, chat_id, &thread).await;
    }

    pub async fn handle_approval_callback(&self, bot: Bot, chat_id: ChatId, data: &str) {
        let Some(chat_state) = self.chats.get(&chat_id) else {
            return;
        };
        let mut state = chat_state.lock().await;
        let Some(approval) = state.pending_approval.take() else {
            return;
        };
        let thread = Arc::clone(&state.thread);
        drop(state);

        let decision = match data {
            "approve" => ReviewDecision::Approved,
            "approve_session" => ReviewDecision::ApprovedForSession,
            _ => ReviewDecision::Denied,
        };

        self.log(
            Some(chat_id.0),
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

        if let Err(e) = thread.submit(op).await {
            tracing::error!("failed to submit approval: {e}");
            bot.send_message(chat_id, format!("Failed to submit approval: {e}"))
                .await
                .ok();
            return;
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

        // Continue draining events after approval
        self.drain_events(bot, chat_id, &thread).await;
    }

    /// Consume events from the thread until turn completes or an approval is needed.
    #[allow(unused_assignments)]
    async fn drain_events(&self, bot: Bot, chat_id: ChatId, thread: &CodexThread) {
        let cid = chat_id.0;
        let mut delta_buf = String::new();
        let mut delta_msg_id: Option<MessageId> = None;
        let mut last_edit = Instant::now();

        loop {
            let event = match thread.next_event().await {
                Ok(ev) => ev,
                Err(e) => {
                    self.log(Some(cid), format!("Event stream error: {e}"));
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
                    let mut m = self.monitor.lock().unwrap();
                    m.input_tokens = info.total_token_usage.input_tokens as u64;
                    m.output_tokens = info.total_token_usage.output_tokens as u64;
                }
            }

            // Log event to monitor
            let event_summary = format!("{:?}", std::mem::discriminant(&event.msg));
            self.log(Some(cid), event_summary);

            match adapter::classify_event(&event.msg) {
                TelegramAction::Delta(text) => {
                    delta_buf.push_str(&text);
                    let now = Instant::now();
                    if now.duration_since(last_edit).as_millis() >= EDIT_INTERVAL_MS
                        || is_turn_end
                    {
                        delta_msg_id =
                            send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
                        last_edit = now;
                    }
                }
                TelegramAction::Send(text) => {
                    self.log(Some(cid), format!("-> {text}"));
                    // Flush any pending delta first
                    if !delta_buf.is_empty() {
                        send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
                        delta_buf.clear();
                        delta_msg_id = None;
                    }
                    bot.send_message(chat_id, text).await.ok();
                }
                TelegramAction::ApprovalPrompt(info) => {
                    self.log(
                        Some(cid),
                        format!("Approval needed: {} ({})", info.call_id, info.text),
                    );
                    // Flush delta
                    if !delta_buf.is_empty() {
                        send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
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

                    // Store pending approval and pause event loop
                    if let Some(chat_state) = self.chats.get(&chat_id) {
                        let mut state = chat_state.lock().await;
                        state.pending_approval = Some(PendingApproval {
                            call_id: info.call_id,
                            turn_id: info.turn_id,
                            kind: info.kind,
                        });
                    }
                    return; // Stop draining — will resume after callback
                }
                TelegramAction::Skip => {}
            }

            if is_turn_end {
                self.log(Some(cid), "Turn ended.".to_string());
                // Final flush of any remaining delta
                if !delta_buf.is_empty() {
                    send_or_edit_delta(&bot, chat_id, delta_msg_id, &delta_buf).await;
                }
                break;
            }
        }
    }

    async fn get_or_create_chat(&self, chat_id: ChatId) -> Arc<Mutex<ChatState>> {
        if let Some(state) = self.chats.get(&chat_id) {
            return state.clone();
        }

        self.log(Some(chat_id.0), "Creating new session.".to_string());

        let thread_config = Config::load_with_cli_overrides(vec![])
            .await
            .expect("failed to load config for new thread");

        let NewThread {
            thread_id: _,
            thread,
            session_configured: _,
        } = self
            .thread_mgr
            .start_thread(thread_config)
            .await
            .expect("failed to start codex thread");

        let state = Arc::new(Mutex::new(ChatState {
            thread,
            pending_approval: None,
        }));
        self.chats.insert(chat_id, Arc::clone(&state));
        state
    }
}

/// Send a new message or edit an existing one with accumulated delta text.
/// Returns the message ID (existing or newly created).
async fn send_or_edit_delta(
    bot: &Bot,
    chat_id: ChatId,
    existing_msg: Option<MessageId>,
    text: &str,
) -> Option<MessageId> {
    if text.is_empty() {
        return existing_msg;
    }
    match existing_msg {
        Some(msg_id) => {
            bot.edit_message_text(chat_id, msg_id, text).await.ok();
            Some(msg_id)
        }
        None => match bot.send_message(chat_id, text).await {
            Ok(msg) => Some(msg.id),
            Err(e) => {
                tracing::error!("failed to send message: {e}");
                None
            }
        },
    }
}
