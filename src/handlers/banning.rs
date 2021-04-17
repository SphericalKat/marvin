use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::Requester,
    types::{ChatMember, ChatMemberStatus},
};

use crate::BOT_TOKEN;

use crate::{
    utils::{self, perms},
    Cx,
};

pub async fn ban(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    if chat.is_private() {
        cx.reply_to("This command is meant to be used in a group!")
            .await?;
    }

    perms::require_bot_admin(&cx).await?;
    perms::require_restrict_chat_members(&cx).await?;

    let (user_id, _) = utils::extract_user_and_text(&cx, pool).await;
    if user_id.is_none() {
        cx.reply_to("Try targeting a user next time bud.").await?;
        return Ok(());
    }

    let chat_member: ChatMember = match cx
        .requester
        .get_chat_member(chat.id, user_id.unwrap())
        .await
    {
        Ok(m) => m,
        Err(_) => {
            cx.reply_to("This user is ded mate.").await?;
            return Ok(());
        }
    };

    if match chat_member.status() {
        ChatMemberStatus::Administrator => true,
        ChatMemberStatus::Creator => true,
        _ => false,
    } {
        cx.reply_to("I'm not banning an administrator!").await?;
        return Ok(());
    }

    if user_id.unwrap() == *BOT_TOKEN {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    cx.requester
        .kick_chat_member(cx.update.chat_id(), user_id.unwrap())
        .await?;

    cx.reply_to("Banned!").await?;

    Ok(())
}
