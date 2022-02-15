use std::convert::TryInto;

use chrono::Duration;
use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::*,
    types::{ChatMember, ChatMemberStatus},
};

use anyhow::anyhow;

use crate::{utils::UnitOfTime, BOT_ID};

use crate::{
    utils::{self, perms},
    Cx,
};

pub async fn ban(cx: Cx, is_tban: bool, pool: &Pool<Postgres>) -> anyhow::Result<()> {
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

    // user didn't specify a time for temp ban
    if args.is_none() && is_tban {
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

    // don't try to ban admins
    if matches!(
        chat_member.status(),
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner
    ) {
        cx.reply_to("I'm not banning an administrator!").await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    if let Some(args) = args {
        if is_tban {
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

            // ban chat member for specified time
            cx.requester
                .kick_chat_member(chat.id, user_id.unwrap())
                .until_date(until_time)
                .await?;
            cx.reply_to(format!("Banned for {}!", unit.unwrap()))
                .await?;
        }
    } else {
        // permanently ban chat member
        cx.requester
            .kick_chat_member(chat.id, user_id.unwrap())
            .await?;

        // let user know something happened
        cx.reply_to("Banned!").await?;
    }

    Ok(())
}

pub async fn kick(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
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
            cx.reply_to("This user is ded mate.").await?; // invalid user (outdated info in db?)
            return Ok(());
        }
    };

    // don't try to ban admins
    if match chat_member.status() {
        ChatMemberStatus::Owner | ChatMemberStatus::Administrator => true,
        ChatMemberStatus::Banned | ChatMemberStatus::Left => {
            // user is trying to be smart, but we're smarter
            cx.reply_to("This user isn't in the chat!").await?;
            return Ok(());
        }
        _ => false,
    } {
        cx.reply_to("I'm not kicking an administrator!").await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    // kick the user
    // calling unban on a user in the chat bans and immediately unbans them
    cx.requester
        .unban_chat_member(chat.id, user_id.unwrap())
        .await?;

    // let the user know something happened
    cx.reply_to("Kicked!").await?;

    Ok(())
}

pub async fn kickme(cx: Cx) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_group(&cx), // command needs to be in a public group
        perms::require_bot_restrict_chat_members(&cx) // bot requires RESTRICT_CHAT_MEMBERS permissions
    )?;

    // attempt to get user from message
    let user = cx.update.from().ok_or_else(|| anyhow!("No user found"))?;

    // don't try to ban admins
    if perms::is_user_admin(&cx, user.id).await.is_ok() {
        cx.reply_to("Yeah no, not banning an admin.").await?;
        return Ok(());
    }

    // let the user know something is happening
    cx.reply_to("Sure thing boss.").await?;

    // kick the user
    // calling unban on a user in the chat bans and immediately unbans them
    cx.requester
        .unban_chat_member(cx.chat_id(), user.id)
        .await?;

    Ok(())
}

pub async fn unban(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
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

    // don't try to unban users still in the chat
    if !matches!(chat_member.status(), ChatMemberStatus::Banned) {
        cx.reply_to("This user wasn't banned!").await?;
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("What exactly are you trying to do?").await?;
        return Ok(());
    }

    // unban the user
    cx.requester
        .unban_chat_member(chat.id, user_id.unwrap())
        .await?;

    // let user know something happened
    cx.reply_to("Unbanned!").await?;

    Ok(())
}
