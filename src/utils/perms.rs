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
    match is_bot_admin(cx).await {
        Ok(_) => Ok(()),
        Err(_) => {
            cx.reply_to("The bot must be an admin for this to work!")
                .await?;
            Err(anyhow!("Bot is not admin"))
        }
    }
}

pub async fn is_user_admin(cx: &Cx, user_id: i64) -> anyhow::Result<()> {
    if cx.update.chat.is_private() {
        return Ok(());
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

pub async fn require_user_admin(cx: &Cx) -> anyhow::Result<()> {
    let user_id = match cx.update.from() {
        Some(user) => user.id,
        None => {
            return Err(anyhow!("User not found"));
        }
    };

    match is_user_admin(cx, user_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            cx.reply_to("You need to be an admin for this to work!")
                .await?;
            Err(anyhow!("User is not admin"))
        }
    }
}

pub async fn is_user_restricted(cx: &Cx, user_id: i64) -> anyhow::Result<bool> {
    if cx.update.chat.is_private() {
        return Ok(false);
    }

    let chat_member = cx
        .requester
        .get_chat_member(cx.update.chat_id(), user_id)
        .await?;

    let is_restricted = chat_member.kind.can_send_messages().unwrap_or(false)
        && chat_member.kind.can_send_media_messages().unwrap_or(false)
        && chat_member
            .kind
            .can_add_web_page_previews()
            .unwrap_or(false)
        && chat_member.kind.can_send_other_messages().unwrap_or(false);
    Ok(is_restricted)
}

pub async fn require_bot_restrict_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let chat_member = &cx
        .requester
        .get_chat_member(cx.update.chat_id(), *BOT_ID)
        .await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_restrict_members {
            return Ok(());
        }
    }

    cx.reply_to("I am missing the required permission for this action: CAN_RESTRICT_MEMBERS.")
        .await?;
    Err(anyhow!("Bot cannot restrict chat members"))
}

pub async fn require_restrict_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let user = &cx.update.from();

    if let Some(user) = user {
        let chat_member: ChatMember = cx
            .requester
            .get_chat_member(cx.update.chat_id(), user.id)
            .await?;

        match &chat_member.kind {
            ChatMemberKind::Creator(_) => {
                return Ok(());
            }
            ChatMemberKind::Administrator(adm) => {
                if adm.can_restrict_members {
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    cx.reply_to("You're missing the required permission for this action: CAN_RESTRICT_MEMBERS.")
        .await?;
    Err(anyhow!("User cannot restrict chat members"))
}

pub async fn require_group(cx: &Cx) -> anyhow::Result<()> {
    let chat = &cx.update.chat;
    if chat.is_group() || chat.is_supergroup() {
        return Ok(());
    }

    cx.reply_to("This command is meant to be used in a group!")
        .await?;
    Err(anyhow!("This command is meant to be used in a group"))
}

pub async fn require_bot_promote_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let chat_member = &cx
        .requester
        .get_chat_member(cx.update.chat_id(), *BOT_ID)
        .await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_promote_members {
            return Ok(());
        }
    }

    cx.reply_to("I am missing the required permission for this action: CAN_PROMOTE_MEMBERS.")
        .await?;
    Err(anyhow!("Bot cannot promote chat members"))
}

pub async fn require_promote_chat_members(cx: &Cx) -> anyhow::Result<()> {
    let user = &cx.update.from();

    if let Some(user) = user {
        let chat_member: ChatMember = cx
            .requester
            .get_chat_member(cx.update.chat_id(), user.id)
            .await?;

        match &chat_member.kind {
            ChatMemberKind::Creator(_) => {
                return Ok(());
            }
            ChatMemberKind::Administrator(adm) => {
                if adm.can_promote_members {
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    cx.reply_to("You're missing the required permission for this action: CAN_PROMOTE_MEMBERS.")
        .await?;
    Err(anyhow!("User cannot promote chat members"))
}

pub async fn require_can_pin_messages(cx: &Cx) -> anyhow::Result<()> {
    let user = &cx.update.from();

    if let Some(user) = user {
        let chat_member: ChatMember = cx
            .requester
            .get_chat_member(cx.update.chat_id(), user.id)
            .await?;

        match &chat_member.kind {
            ChatMemberKind::Creator(_) => {
                return Ok(());
            }
            ChatMemberKind::Administrator(adm) => {
                if adm.can_pin_messages.unwrap_or(false) {
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    cx.reply_to("You're missing the required permission for this action: CAN_PIN_MESSAGES.")
        .await?;
    Err(anyhow!("User cannot pin messages"))
}

pub async fn require_bot_can_pin_messages(cx: &Cx) -> anyhow::Result<()> {
    let chat_member = &cx
        .requester
        .get_chat_member(cx.update.chat_id(), *BOT_ID)
        .await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_pin_messages.unwrap_or(false) {
            return Ok(());
        }
    }

    cx.reply_to("I am missing the required permission for this action: CAN_PIN_MESSAGES.")
        .await?;
    Err(anyhow!("Bot cannot pin messages"))
}
