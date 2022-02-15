use std::convert::TryInto;

use chrono::Duration;
use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::*,
    types::{ChatMember, ChatMemberStatus, ChatPermissions},
};

use anyhow::anyhow;

use crate::{utils::UnitOfTime, BOT_ID};
use crate::{
    utils::{self, perms},
    Cx,
};

pub async fn mute(cx: Cx, is_tmute: bool, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(&cx), // command needs to be in a public group
        perms::require_restrict_chat_members(&cx), // user requires RESTRICT_CHAT_MEMBERS permissions
        perms::require_bot_restrict_chat_members(&cx) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // extract user and text from message
    let (user_id, args) = utils::extract_user_and_text(&cx, pool).await;
    if user_id.is_none() {
        // no user was targeted
        cx.reply_to("Try targeting a user next time bud.").await?;
        return Ok(());
    }

    // user didn't specify a time for temp mute
    if args.is_none() && is_tmute {
        cx.reply_to("You need to specify a duration in d/h/m/s (days, hours, minutes, seconds)")
            .await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match cx
        .requester
        .get_chat_member(chat.id, user_id.unwrap())
        .await
    {
        Ok(m) => m, // user is valid
        Err(_) => {
            cx.reply_to("This user is ded mate.").await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    match chat_member.status() {
        // don't try to mute admins
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner => {
            cx.reply_to("I'm not muting an administrator!").await?;
            return Ok(());
        }

        // don't try to mute users not in the chat
        ChatMemberStatus::Banned | ChatMemberStatus::Left => {
            cx.reply_to("This user isn't in the chat!").await?;
            return Ok(());
        }
        _ => {}
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    // check if user is already restricted
    let is_restricted = perms::is_user_restricted(&cx, user_id.unwrap()).await?;

    let permissions = ChatPermissions::empty();

    if let Some(args) = args {
        if is_tmute {
            // get unit of time
            let unit = args.parse::<UnitOfTime>();
            if unit.is_err() {
                cx.reply_to("failed to get specified time; expected one of d/h/m/s (days, hours, minutes, seconds)").await?;
                return Ok(());
            }

            // convert to seconds
            let time = utils::extract_time(unit.as_ref().unwrap());
            let until_time = cx
                .update
                .date
                .checked_add_signed(Duration::seconds(time.try_into().unwrap()))
                .ok_or(anyhow!("Something went wrong!"))?;

            // mute chat member for specified time
            cx.requester
                .restrict_chat_member(chat.id, user_id.unwrap(), permissions)
                .until_date(until_time)
                .await?;

            if is_restricted {
                cx.reply_to(format!(
                    "Restrictions have been updated. Muted for {}!",
                    unit.unwrap()
                ))
                .await?;
            } else {
                cx.reply_to(format!("Muted for {}!", unit.unwrap())).await?;
            }
        }
    } else {
        // permanently mute chat member
        cx.requester
            .restrict_chat_member(chat.id, user_id.unwrap(), permissions)
            .await?;

        if is_restricted {
            cx.reply_to("Restrictions have been updated. Permanently muted!")
                .await?;
        } else {
            cx.reply_to("Muted!").await?;
        }
    }

    Ok(())
}

pub async fn unmute(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_group(&cx), // command needs to be in a public group
        perms::require_restrict_chat_members(&cx), // user requires RESTRICT_CHAT_MEMBERS permissions
        perms::require_bot_restrict_chat_members(&cx) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // extract user and text from message
    let (user_id, _) = utils::extract_user_and_text(&cx, pool).await;
    if user_id.is_none() {
        cx.reply_to("Try targeting a user next time bud.").await?;
        return Ok(());
    }

    // check if user is valid
    let chat_member: ChatMember = match cx
        .requester
        .get_chat_member(chat.id, user_id.unwrap())
        .await
    {
        Ok(m) => m, // user is valid
        Err(_) => {
            cx.reply_to("This user is ded mate.").await?; // invalid user
            return Ok(());
        }
    };

    // don't try to unmute users not in the chat
    if matches!(
        chat_member.status(),
        ChatMemberStatus::Banned | ChatMemberStatus::Left
    ) {
        cx.reply_to("This user isn't in the chat!").await?;
        return Ok(());
    }

    // don't try to unmute unrestricted users
    if !perms::is_user_restricted(&cx, user_id.unwrap()).await? {
        cx.reply_to("This user can already speak freely!").await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("What exactly are you trying to do?").await?;
        return Ok(());
    }

    let permissions = ChatPermissions::empty()
        | ChatPermissions::SEND_MESSAGES
        | ChatPermissions::SEND_MEDIA_MESSAGES
        | ChatPermissions::SEND_OTHER_MESSAGES
        | ChatPermissions::SEND_POLLS
        | ChatPermissions::ADD_WEB_PAGE_PREVIEWS;
    // .can_send_messages(true)
    // .can_send_media_messages(true)
    // .can_send_other_messages(true)
    // .can_send_polls(true)
    // .can_add_web_page_previews(true);

    // unmute the user
    cx.requester
        .restrict_chat_member(chat.id, user_id.unwrap(), permissions)
        .await?;

    // let user know something happened
    cx.reply_to("Unmuted!").await?;

    Ok(())
}
