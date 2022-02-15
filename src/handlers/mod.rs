pub mod admin;
pub mod banning;
pub mod filters;
pub mod misc;
pub mod muting;

use sqlx::{Pool, Postgres};
use teloxide::{
    adaptors::AutoSend,
    types::{ChatKind, Message},
    Bot,
};

use crate::{
    entities::User,
    repo::{chats, users},
};

pub async fn save_user_handler(
    _bot: &AutoSend<Bot>,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if let Some(user) = message.from() {
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

pub async fn save_chat_handler(
    _bot: &AutoSend<Bot>,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if message.chat.is_chat() {
        let chat = &message.chat;

        match &chat.kind {
            ChatKind::Public(pu) => chats::insert_chat(chat.id, pu.title.clone(), pool).await?,
            ChatKind::Private(pri) => {
                chats::insert_chat(chat.id, pri.username.clone(), pool).await?
            }
        }
    }

    Ok(())
}
