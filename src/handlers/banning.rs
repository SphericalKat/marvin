use std::convert::TryInto;

use chrono::Duration;
use sqlx::{Pool, Postgres};
use teloxide::{
    prelude2::*,
    types::{ChatMember, ChatMemberStatus},
};

use anyhow::anyhow;

use crate::{utils::UnitOfTime, BOT_ID};

use crate::utils::{self, perms};

pub async fn ban(
    bot: &AutoSend<Bot>,
    message: &Message,
    is_tban: bool,
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
            .await?;
        return Ok(());
    }

    // user didn't specify a time for temp ban
    if args.is_none() && is_tban {
        bot.send_message(
            message.chat.id,
            "You need to specify a duration in d/h/m/s (days, hours, minutes, seconds)",
        )
        .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(m) => m, // user is valid
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    // don't try to ban admins
    if matches!(
        chat_member.status(),
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner
    ) {
        bot.send_message(message.chat.id, "I'm not banning an administrator!")
            .await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "No u").await?;
        return Ok(());
    }

    if let Some(args) = args {
        if is_tban {
            // get unit of time
            let unit = args.parse::<UnitOfTime>();
            if unit.is_err() {
                bot.send_message(message.chat.id, "failed to get specified time; expected one of d/h/m/s (days, hours, minutes, seconds)").await?;
                return Ok(());
            }

            // convert to seconds
            let time = utils::extract_time(unit.as_ref().unwrap());
            let until_time = message
                .date
                .checked_add_signed(Duration::seconds(time.try_into().unwrap()))
                .ok_or(anyhow!("Something went wrong!"))?;

            // ban chat member for specified time
            bot.kick_chat_member(chat.id, user_id.unwrap())
                .until_date(until_time)
                .await?;
            bot.send_message(message.chat.id, format!("Banned for {}!", unit.unwrap()))
                .await?;
        }
    } else {
        // permanently ban chat member
        bot.kick_chat_member(chat.id, user_id.unwrap()).await?;

        // let user know something happened
        bot.send_message(message.chat.id, "Banned!").await?;
    }

    Ok(())
}

pub async fn kick(
    bot: &AutoSend<Bot>,
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
            .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(m) => m, // user is valid
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    // don't try to ban admins
    if match chat_member.status() {
        ChatMemberStatus::Owner | ChatMemberStatus::Administrator => true,
        ChatMemberStatus::Banned | ChatMemberStatus::Left => {
            // user is trying to be smart, but we're smarter
            bot.send_message(message.chat.id, "This user isn't in the chat!")
                .await?;
            return Ok(());
        }
        _ => false,
    } {
        bot.send_message(message.chat.id, "I'm not kicking an administrator!")
            .await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "No u").await?;
        return Ok(());
    }

    // kick the user
    // calling unban on a user in the chat bans and immediately unbans them
    bot.unban_chat_member(chat.id, user_id.unwrap()).await?;

    // let the user know something happened
    bot.send_message(message.chat.id, "Kicked!").await?;

    Ok(())
}

pub async fn kickme(bot: &AutoSend<Bot>, message: &Message) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_group(bot, message), // command needs to be in a public group
        perms::require_bot_restrict_chat_members(bot, message) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // attempt to get user from message
    let user = message.from().ok_or_else(|| anyhow!("No user found"))?;

    // don't try to ban admins
    if perms::is_user_admin(bot, message, user.id).await.is_ok() {
        bot.send_message(message.chat.id, "Yeah no, not banning an admin.")
            .await?;
        return Ok(());
    }

    // let the user know something is happening
    bot.send_message(message.chat.id, "Sure thing boss.")
        .await?;

    // kick the user
    // calling unban on a user in the chat bans and immediately unbans them
    bot.unban_chat_member(message.chat.id, user.id).await?;

    Ok(())
}

pub async fn unban(
    bot: &AutoSend<Bot>,
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
            .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match bot.get_chat_member(chat.id, user_id.unwrap()).await {
        Ok(m) => m, // user is valid
        Err(_) => {
            bot.send_message(message.chat.id, "This user is ded mate.")
                .await?; // invalid user
            return Ok(());
        }
    };

    // don't try to unban users still in the chat
    if !matches!(chat_member.status(), ChatMemberStatus::Banned) {
        bot.send_message(message.chat.id, "This user wasn't banned!")
            .await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        bot.send_message(message.chat.id, "What exactly are you trying to do?")
            .await?;
        return Ok(());
    }

    // unban the user
    bot.unban_chat_member(chat.id, user_id.unwrap()).await?;

    // let user know something happened
    bot.send_message(message.chat.id, "Unbanned!").await?;

    Ok(())
}
