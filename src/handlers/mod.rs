use sqlx::{Pool, Postgres};
use teloxide::{prelude::*, types::ChatKind};

use crate::repo;

pub async fn save_user_handler(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if let Some(user) = cx.update.from() {
        repo::insert_user(user.id, user.username.clone(), pool).await?;
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
            ChatKind::Public(pu) => {repo::insert_chat(chat.id, pu.title.clone(), pool).await?}
            ChatKind::Private(_) => {}
        }
    }

    Ok(())
}
