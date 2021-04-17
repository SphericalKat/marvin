use std::any;

use crate::BOT_ID;
use anyhow::anyhow;
use teloxide::{
    prelude::Requester,
    types::{ChatMember, ChatMemberKind, ChatMemberStatus},
};

use crate::Cx;

async fn is_bot_admin(cx: &Cx) -> anyhow::Result<()> {
    if cx.update.chat.is_private() {
        return Ok(());
    }

    let bot_member: ChatMember = cx
        .requester
        .get_chat_member(cx.update.chat_id(), *BOT_ID)
        .await?;

    if let ChatMemberStatus::Administrator = bot_member.status() {
        return Ok(());
    }

    Err(anyhow!("Bot is not admin"))
}

pub async fn require_bot_admin(cx: &Cx) -> anyhow::Result<()> {
    return match is_bot_admin(cx).await {
        Ok(_) => Ok(()),
        Err(_) => {
            cx.reply_to("The bot must be an admin for this to work!")
                .await?;
            return Err(anyhow!("Bot is not admin"));
        }
    };
}

#[allow(dead_code)]
async fn is_user_admin(cx: &Cx, user_id: i64) -> anyhow::Result<()> {
    if cx.update.chat.is_private() {
        return Err(anyhow!("User is not admin"));
    }

    let chat_member: ChatMember = cx
        .requester
        .get_chat_member(cx.update.chat_id(), user_id)
        .await?;

    match chat_member.status() {
        ChatMemberStatus::Administrator => Ok(()),
        ChatMemberStatus::Creator => Ok(()),
        _ => Err(anyhow!("User is not admin")),
    }
}

pub async fn require_bot_restrict_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let chat_member = &cx
        .requester
        .get_chat_member(cx.update.chat_id(), *BOT_ID)
        .await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if !adm.can_restrict_members {
            cx.reply_to(
                "I am missing the required permission for this action: CAN_RESTRICT_MEMBERS.",
            )
            .await?;
            return Err(anyhow!("Bot cannot restrict chat members"));
        }
    }

    Ok(())
}

pub async fn require_restrict_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let user = &cx.update.from();

    if let Some(user) = user {
        let chat_member: ChatMember = cx
            .requester
            .get_chat_member(cx.update.chat_id(), user.id)
            .await?;
        if let ChatMemberKind::Administrator(adm) = chat_member.kind {
            if !adm.can_restrict_members {
                cx.reply_to(
                    "You're missing the required permission for this action: CAN_RESTRICT_MEMBERS.",
                )
                .await?;
                return Err(anyhow!("User cannot restrict chat members"));
            }
        }
    }

    Ok(())
}

pub async fn require_public_group(cx: &Cx) -> anyhow::Result<()> {
    if cx.update.chat.is_private() {
        cx.reply_to("This command is meant to be used in a group!")
            .await?;
    }

    Ok(())
}