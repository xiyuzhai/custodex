use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::Me;

use crate::monitor::SharedMonitor;
use crate::session::SessionManager;

pub async fn run_bot(token: String, monitor: SharedMonitor) {
    let bot = Bot::new(&token);

    let me: Me = match bot.get_me().await {
        Ok(me) => me,
        Err(e) => {
            let mut m = monitor.lock().unwrap();
            m.bot_status = crate::monitor::BotStatus::Error(format!("Failed to connect: {e}"));
            m.push_log(None, format!("Bot failed to start: {e}"));
            return;
        }
    };

    {
        let mut m = monitor.lock().unwrap();
        m.bot_status = crate::monitor::BotStatus::Running;
        m.push_log(None, format!("Bot started: @{}", me.username()));
    }

    let session_mgr = Arc::new(SessionManager::new(monitor.clone()).await);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![session_mgr])
        .build()
        .dispatch()
        .await;

    // If dispatch returns, bot has stopped
    {
        let mut m = monitor.lock().unwrap();
        m.bot_status = crate::monitor::BotStatus::Stopped;
        m.push_log(None, "Bot stopped.".to_string());
    }
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
