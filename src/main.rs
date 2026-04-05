mod adapter;
mod session;

use session::SessionManager;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::Me;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".local/telegram_bot_token".to_string());
    let token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("failed to read token from {token_path}: {e}"))
        .trim()
        .to_string();

    let bot = Bot::new(token);
    let me: Me = bot.get_me().await.expect("failed to get bot info");
    tracing::info!("starting bot: @{}", me.username());

    let session_mgr = Arc::new(SessionManager::new().await);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![session_mgr])
        .build()
        .dispatch()
        .await;
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    session_mgr: Arc<SessionManager>,
) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let chat_id = msg.chat.id;
    session_mgr.handle_user_message(bot, chat_id, text).await;
    Ok(())
}

async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    session_mgr: Arc<SessionManager>,
) -> ResponseResult<()> {
    let Some(data) = q.data.as_deref() else {
        return Ok(());
    };
    let Some(msg) = q.message else {
        return Ok(());
    };

    let chat_id = msg.chat().id;
    session_mgr
        .handle_approval_callback(bot, chat_id, data)
        .await;
    Ok(())
}
