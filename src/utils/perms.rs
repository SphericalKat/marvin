use crate::BOT_ID;
use anyhow::anyhow;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{ChatMember, ChatMemberKind, ChatMemberStatus},
};

async fn is_bot_admin(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    if message.chat.is_private() {
        return Ok(());
    }

    let bot_member: ChatMember = bot.get_chat_member(message.chat.id, *BOT_ID).await?;

    if let ChatMemberStatus::Administrator = bot_member.status() {
        return Ok(());
    }

    Err(anyhow!("Bot is not admin"))
}

pub async fn require_bot_admin(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    match is_bot_admin(bot, message).await {
        Ok(_) => Ok(()),
        Err(_) => {
            bot.send_message(
                message.chat.id,
                "The bot must be an admin for this to work!",
            )
            .reply_to_message_id(message.id)
            .await?;
            Err(anyhow!("Bot is not admin"))
        }
    }
}

pub async fn is_user_admin(
    bot: &AutoSend<Bot>,
    message: &Message,
    user_id: i64,
) -> anyhow::Result<()> {
    if message.chat.is_private() {
        return Ok(());
    }

    let chat_member: ChatMember = bot.get_chat_member(message.chat.id, user_id).await?;

    match chat_member.status() {
        ChatMemberStatus::Administrator => Ok(()),
        ChatMemberStatus::Owner => Ok(()),
        _ => Err(anyhow!("User is not admin")),
    }
}

pub async fn require_user_admin(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    let user_id = match message.from() {
        Some(user) => user.id,
        None => {
            return Err(anyhow!("User not found"));
        }
    };

    match is_user_admin(bot, message, user_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            bot.send_message(message.chat.id, "You need to be an admin for this to work!")
                .reply_to_message_id(message.id)
                .await?;
            Err(anyhow!("User is not admin"))
        }
    }
}

pub async fn is_user_restricted(
    bot: &AutoSend<Bot>,
    message: &Message,
    user_id: i64,
) -> anyhow::Result<bool> {
    if message.chat.is_private() {
        return Ok(false);
    }

    let chat_member = bot.get_chat_member(message.chat.id, user_id).await?;

    let is_restricted = chat_member.kind.can_send_messages()
        && chat_member.kind.can_send_media_messages()
        && chat_member.kind.can_add_web_page_previews()
        && chat_member.kind.can_send_other_messages();
    Ok(is_restricted)
}

pub async fn require_bot_restrict_chat_members(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let chat_member = bot.get_chat_member(message.chat.id, *BOT_ID).await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_restrict_members {
            return Ok(());
        }
    }

    bot.send_message(
        message.chat.id,
        "I am missing the required permission for this action: CAN_RESTRICT_MEMBERS.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("Bot cannot restrict chat members"))
}

pub async fn require_restrict_chat_members(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let user = message.from();

    if let Some(user) = user {
        let chat_member: ChatMember = bot.get_chat_member(message.chat.id, user.id).await?;

        match &chat_member.kind {
            ChatMemberKind::Owner(_) => {
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

    bot.send_message(
        message.chat.id,
        "You're missing the required permission for this action: CAN_RESTRICT_MEMBERS.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("User cannot restrict chat members"))
}

pub async fn require_group(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    let chat = &message.chat;
    if chat.is_group() || chat.is_supergroup() {
        return Ok(());
    }

    bot.send_message(chat.id, "This command is meant to be used in a group!")
        .reply_to_message_id(message.id)
        .await?;
    Err(anyhow!("This command is meant to be used in a group"))
}

pub async fn require_bot_promote_chat_members(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let chat_member = bot.get_chat_member(message.chat.id, *BOT_ID).await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_promote_members {
            return Ok(());
        }
    }

    bot.send_message(
        message.chat.id,
        "I am missing the required permission for this action: CAN_PROMOTE_MEMBERS.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("Bot cannot promote chat members"))
}

pub async fn require_promote_chat_members(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let user = message.from();

    if let Some(user) = user {
        let chat_member: ChatMember = bot.get_chat_member(message.chat.id, user.id).await?;

        match &chat_member.kind {
            ChatMemberKind::Owner(_) => {
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

    bot.send_message(
        message.chat.id,
        "You're missing the required permission for this action: CAN_PROMOTE_MEMBERS.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("User cannot promote chat members"))
}

pub async fn require_can_pin_messages(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let user = message.from();

    if let Some(user) = user {
        let chat_member: ChatMember = bot.get_chat_member(message.chat.id, user.id).await?;

        match &chat_member.kind {
            ChatMemberKind::Owner(_) => {
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

    bot.send_message(
        message.chat.id,
        "You're missing the required permission for this action: CAN_PIN_MESSAGES.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("User cannot pin messages"))
}

pub async fn require_bot_can_pin_messages(
    bot: &AutoSend<Bot>,
    message: &Message,
) -> anyhow::Result<()> {
    let chat_member = bot.get_chat_member(message.chat.id, *BOT_ID).await?;

    if let ChatMemberKind::Administrator(adm) = &chat_member.kind {
        if adm.can_pin_messages.unwrap_or(false) {
            return Ok(());
        }
    }

    bot.send_message(
        message.chat.id,
        "I am missing the required permission for this action: CAN_PIN_MESSAGES.",
    )
    .reply_to_message_id(message.id)
    .await?;
    Err(anyhow!("Bot cannot pin messages"))
}
