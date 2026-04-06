use teloxide::prelude::*;
use teloxide::types::MessageId;

/// Send a new message or edit an existing one with accumulated delta text.
/// Returns the message ID (existing or newly created).
pub async fn send_or_edit_delta(
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
