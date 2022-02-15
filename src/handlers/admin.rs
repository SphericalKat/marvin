use sqlx::{Pool, Postgres};
use teloxide::payloads::SendMessageSetters;

use crate::utils::{self, perms, PinMode};
use crate::BOT_ID;
use teloxide::{
    prelude2::*,
    types::{ChatMember, ChatMemberStatus},
};

pub async fn promote(
    bot: &AutoSend<Bot>,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let chat = &message.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_promote_chat_members(bot, message), // user requires CAN_PROMOTE_MEMBERS permissions
        perms::require_bot_promote_chat_members(bot, message) // bot requires CAN_PROMOTE_MEMBERS permissions
    )?;

    // extract user ID from message
    let (user_id, _) = utils::extract_user_and_text(bot, message, pool).await;
    if user_id.is_none() {
        // no user was targeted
        bot.send_message(message.chat.id, "Try targeting a user next time bud.")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    // check if user is valid
    let user_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(user) => user,
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .reply_to_message_id(message.id)
                .await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "No u")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let bot_chat_member: ChatMember = bot.get_chat_member(chat.id, *BOT_ID).await?;

    if user_member.kind.can_be_edited() {
        bot.promote_chat_member(chat.id, user_id.unwrap())
            .can_manage_chat(bot_chat_member.kind.can_manage_chat())
            .can_change_info(bot_chat_member.kind.can_change_info())
            .can_delete_messages(bot_chat_member.kind.can_delete_messages())
            .can_manage_voice_chats(bot_chat_member.kind.can_manage_voice_chats())
            .can_invite_users(bot_chat_member.kind.can_invite_users())
            .can_restrict_members(bot_chat_member.kind.can_restrict_members())
            .can_pin_messages(bot_chat_member.kind.can_pin_messages() && chat.is_supergroup())
            .can_promote_members(bot_chat_member.kind.can_promote_members())
            .await?;
    }

    bot.send_message(message.chat.id, "Successfully promoted!")
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}

pub async fn demote(
    bot: &AutoSend<Bot>,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let chat = &message.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_promote_chat_members(bot, message), // user requires CAN_PROMOTE_MEMBERS permissions
        perms::require_bot_promote_chat_members(bot, message) // bot requires CAN_PROMOTE_MEMBERS permissions
    )?;

    // extract user ID from message
    let (user_id, _) = utils::extract_user_and_text(bot, message, pool).await;
    if user_id.is_none() {
        // no user was targeted
        bot.send_message(message.chat.id, "Try targeting a user next time bud.")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    // check if user is valid
    let user_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(user) => user,
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .reply_to_message_id(message.id)
                .await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    match user_member.status() {
        ChatMemberStatus::Administrator => {}
        ChatMemberStatus::Owner => {
            bot.send_message(
                message.chat.id,
                "This person CREATED the chat, how would I demote them?",
            )
            .reply_to_message_id(message.id)
            .await?;
            return Ok(());
        }
        _ => {
            bot.send_message(message.chat.id, "Can't demote what wasn't promoted!")
                .reply_to_message_id(message.id)
                .await?;
            return Ok(());
        }
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "No u")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    if user_member.kind.can_be_edited() {
        bot.promote_chat_member(chat.id, user_id.unwrap())
            .can_manage_chat(false)
            .can_change_info(false)
            .can_delete_messages(false)
            .can_manage_voice_chats(false)
            .can_invite_users(false)
            .can_restrict_members(false)
            .can_pin_messages(false)
            .can_promote_members(false)
            .await?;
    } else {
        bot.send_message(message.chat.id, "Could not demote. I might not be admin, or the admin status was appointed by another user, so I can't act upon them!")
				.reply_to_message_id(message.id)
				.await?;
        return Ok(());
    }

    bot.send_message(message.chat.id, "Successfully demoted!")
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}

pub async fn pin(bot: &AutoSend<Bot>, message: &Message, mode: PinMode) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_can_pin_messages(bot, message), // user requires CAN_PROMOTE_MEMBERS permissions
        perms::require_bot_can_pin_messages(bot, message), // bot requires CAN_PROMOTE_MEMBERS permissions
    )?;

    if let Some(prev_msg) = message.reply_to_message() {
        bot.pin_chat_message(message.chat.id, prev_msg.id)
            .disable_notification(mode.is_silent())
            .await?;
    } else {
        bot.send_message(message.chat.id, "Can't pin that message!")
            .reply_to_message_id(message.id)
            .await?;
    }

    Ok(())
}

pub async fn invite(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    match &message.chat.kind {
        teloxide::types::ChatKind::Public(c) => {
            if c.invite_link.is_some() {
                bot.send_message(message.chat.id, c.invite_link.as_ref().unwrap())
                    .reply_to_message_id(message.id)
                    .await?;
            } else {
                match bot.export_chat_invite_link(message.chat.id).await {
                    Ok(u) => {
                        bot.send_message(message.chat.id, u)
                            .reply_to_message_id(message.id)
                            .await?;
                    }
                    Err(_) => {
                        bot.send_message(
                            message.chat.id,
                            "I don't have access to the invite link, try changing my permissions!",
                        )
                        .reply_to_message_id(message.id)
                        .await?;
                    }
                }
            }
        }
        teloxide::types::ChatKind::Private(_) => {
            bot.send_message(
                message.chat.id,
                "I can only give you invite links for supergroups and channels, sorry!",
            )
            .reply_to_message_id(message.id)
            .await?;
        }
    }

    Ok(())
}
