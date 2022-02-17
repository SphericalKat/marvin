use std::convert::TryInto;

use chrono::Duration;
use sqlx::{Pool, Postgres};
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{ChatMember, ChatMemberStatus, ChatPermissions},
};

use anyhow::anyhow;

use crate::utils::{self, perms};
use crate::{utils::UnitOfTime, BOT_ID};

pub async fn mute(
    bot: &crate::Bot,
    message: &Message,
    is_tmute: bool,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let chat = &message.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_restrict_chat_members(bot, message), // user requires RESTRICT_CHAT_MEMBERS permissions
        perms::require_bot_restrict_chat_members(bot, message) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // extract user and text from message
    let (user_id, args) = utils::extract_user_and_text(bot, message, pool).await;
    if user_id.is_none() {
        // no user was targeted
        bot.send_message(message.chat.id, "Try targeting a user next time bud.")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    // user didn't specify a time for temp mute
    if args.is_none() && is_tmute {
        bot.send_message(
            message.chat.id,
            "You need to specify a duration in d/h/m/s (days, hours, minutes, seconds)",
        )
        .reply_to_message_id(message.id)
        .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(m) => m, // user is valid
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .reply_to_message_id(message.id)
                .await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    match chat_member.status() {
        // don't try to mute admins
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner => {
            bot.send_message(message.chat.id, "I'm not muting an administrator!")
                .reply_to_message_id(message.id)
                .await?;
            return Ok(());
        }

        // don't try to mute users not in the chat
        ChatMemberStatus::Banned | ChatMemberStatus::Left => {
            bot.send_message(message.chat.id, "This user isn't in the chat!")
                .reply_to_message_id(message.id)
                .await?;
            return Ok(());
        }
        _ => {}
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "No u").await?;
        return Ok(());
    }

    // check if user is already restricted
    let is_restricted = perms::is_user_restricted(bot, message, user_id.unwrap()).await?;

    let permissions = ChatPermissions::empty();

    if let Some(args) = args {
        if is_tmute {
            // get unit of time
            let unit = args.parse::<UnitOfTime>();
            if unit.is_err() {
                bot.send_message(message.chat.id,"failed to get specified time; expected one of d/h/m/s (days, hours, minutes, seconds)")
								.reply_to_message_id(message.id)
								.await?;
                return Ok(());
            }

            // convert to seconds
            let time = utils::extract_time(unit.as_ref().unwrap());
            let until_time = message
                .date
                .checked_add_signed(Duration::seconds(time.try_into().unwrap()))
                .ok_or(anyhow!("Something went wrong!"))?;

            // mute chat member for specified time
            bot.restrict_chat_member(chat.id, user_id.unwrap(), permissions)
                .until_date(until_time)
                .await?;

            if is_restricted {
                bot.send_message(
                    message.chat.id,
                    format!(
                        "Restrictions have been updated. Muted for {}!",
                        unit.unwrap()
                    ),
                )
                .reply_to_message_id(message.id)
                .await?;
            } else {
                bot.send_message(message.chat.id, format!("Muted for {}!", unit.unwrap()))
                    .reply_to_message_id(message.id)
                    .await?;
            }
        }
    } else {
        // permanently mute chat member
        bot.restrict_chat_member(chat.id, user_id.unwrap(), permissions)
            .await?;

        if is_restricted {
            bot.send_message(
                message.chat.id,
                "Restrictions have been updated. Permanently muted!",
            )
            .reply_to_message_id(message.id)
            .await?;
        } else {
            bot.send_message(message.chat.id, "Muted!")
                .reply_to_message_id(message.id)
                .await?;
        }
    }

    Ok(())
}

pub async fn unmute(
    bot: &crate::Bot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let chat = &message.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_restrict_chat_members(bot, message), // user requires RESTRICT_CHAT_MEMBERS permissions
        perms::require_bot_restrict_chat_members(bot, message) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // extract user and text from message
    let (user_id, _) = utils::extract_user_and_text(bot, message, pool).await;
    if user_id.is_none() {
        bot.send_message(message.chat.id, "Try targeting a user next time bud.")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(m) => m, // user is valid
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .reply_to_message_id(message.id)
                .await?; // invalid user
            return Ok(());
        }
    };

    // don't try to unmute users not in the chat
    if matches!(
        chat_member.status(),
        ChatMemberStatus::Banned | ChatMemberStatus::Left
    ) {
        bot.send_message(message.chat.id, "This user isn't in the chat!")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    // don't try to unmute unrestricted users
    if !perms::is_user_restricted(bot, message, user_id.unwrap()).await? {
        bot.send_message(message.chat.id, "This user can already speak freely!")
            .await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "What exactly are you trying to do?")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let permissions = ChatPermissions::empty()
        | ChatPermissions::SEND_MESSAGES
        | ChatPermissions::SEND_MEDIA_MESSAGES
        | ChatPermissions::SEND_OTHER_MESSAGES
        | ChatPermissions::SEND_POLLS
        | ChatPermissions::ADD_WEB_PAGE_PREVIEWS;

    // unmute the user
    bot.restrict_chat_member(chat.id, user_id.unwrap(), permissions)
        .await?;

    // let user know something happened
    bot.send_message(message.chat.id, "Unmuted!")
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}
