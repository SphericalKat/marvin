pub mod admin;
pub mod banning;
pub mod filters;
pub mod misc;
pub mod muting;

use sqlx::{Pool, Postgres};
use teloxide::types::ChatKind;

use crate::{
    entities::User,
    repo::{chats, users},
    Cx,
};

pub async fn save_user_handler(cx: &Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    if let Some(user) = cx.update.from() {
        let username = user.username.as_ref().map(|s| s.to_lowercase());
        users::insert_user(
            &User {
                user_id: user.id,
                user_name: username,
                full_name: user.full_name(),
            },
            pool,
        )
        .await?;
    }

    Ok(())
}

pub async fn save_chat_handler(cx: &Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    if cx.update.chat.is_chat() {
        let chat = &cx.update.chat;

        match &chat.kind {
            ChatKind::Public(pu) => chats::insert_chat(chat.id, pu.title.clone(), pool).await?,
            ChatKind::Private(_) => {}
        }
    }

    Ok(())
}
