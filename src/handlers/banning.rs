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

    tokio::try_join!(
        perms::require_public_group(&cx),
        perms::require_restrict_chat_members(&cx),
        perms::require_bot_restrict_chat_members(&cx)
    )?;

    let (user_id, args) = utils::extract_user_and_text(&cx, pool).await;
    if user_id.is_none() {
        cx.reply_to("Try targeting a user next time bud.").await?;
        return Ok(());
    }

    if args.is_none() && is_tban {
        cx.reply_to("You need to specify a duration in d/h/m/s (days, hours, minutes, seconds)")
            .await?;
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

    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    if let Some(args) = args {
        if is_tban {
            let unit = args.parse::<UnitOfTime>();
            if unit.is_err() {
                cx.reply_to("failed to get specified time; expected one of d/h/m/s (days, hours, minutes, seconds)").await?;
                return Ok(());
            }

            let time = utils::extract_time(unit.as_ref().unwrap());

            cx.requester
                .kick_chat_member(chat.id, user_id.unwrap())
                .until_date(cx.update.date as u64 + time)
                .await?;
            cx.reply_to(format!("Banned for {}!", unit.unwrap()))
                .await?;
        }
    } else {
        cx.requester
            .kick_chat_member(chat.id, user_id.unwrap())
            .await?;

        cx.reply_to("Banned!").await?;
    }

    Ok(())
}

pub async fn kick(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    tokio::try_join!(
        perms::require_public_group(&cx),
        perms::require_restrict_chat_members(&cx),
        perms::require_bot_restrict_chat_members(&cx)
    )?;

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
        cx.reply_to("I'm not kicking an administrator!").await?;
        return Ok(());
    }

    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    cx.requester
        .unban_chat_member(chat.id, user_id.unwrap())
        .await?;

    cx.reply_to("Kicked!").await?;

    Ok(())
}

pub async fn kickme(cx: Cx) -> anyhow::Result<()> {
    tokio::try_join!(
        perms::require_public_group(&cx),
        perms::require_bot_restrict_chat_members(&cx)
    )?;

    let user = cx.update.from().ok_or(anyhow!("No user found"))?;

    if perms::is_user_admin(&cx, user.id).await.is_ok() {
        cx.reply_to("Yeah no, not banning an admin.").await?;
        return Ok(());
    }

    cx.reply_to("Sure thing boss.").await?;

    cx.requester
        .unban_chat_member(cx.chat_id(), user.id)
        .await?;

    Ok(())
}

pub async fn unban(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    tokio::try_join!(
        perms::require_public_group(&cx),
        perms::require_restrict_chat_members(&cx),
        perms::require_bot_restrict_chat_members(&cx)
    )?;

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
        ChatMemberStatus::Kicked => false,
        _ => true,
    } {
        cx.reply_to("This user wasn't banned!").await?;
        return Ok(());
    }

    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("What exactly are you trying to do?").await?;
        return Ok(());
    }

    cx.requester
        .unban_chat_member(chat.id, user_id.unwrap())
        .await?;

    cx.reply_to("Unbanned!").await?;

    Ok(())
}
