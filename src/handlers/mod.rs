use sqlx::{Pool, Postgres};
use teloxide::{prelude::*, types::ChatKind};

use crate::repo::{chats, users};

pub async fn save_user_handler(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if let Some(user) = cx.update.from() {
        let username = user.username.as_ref().map(|s| s.to_lowercase());
        users::insert_user(user.id, username, pool).await?;
    }

    Ok(())
}

pub async fn save_chat_handler(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if cx.update.chat.is_chat() {
        let chat = &cx.update.chat;

        match &chat.kind {
            ChatKind::Public(pu) => chats::insert_chat(chat.id, pu.title.clone(), pool).await?,
            ChatKind::Private(_) => {}
        }
    }

    Ok(())
}
