use sqlx::{Pool, Postgres};

use crate::utils::{self, perms};
use crate::Cx;
use crate::BOT_ID;
use teloxide::{prelude::*, types::ChatMember};

pub async fn promote(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let chat = &cx.update.chat;

    // check for required conditions
    tokio::try_join!(
        perms::require_public_group(&cx), // command needs to be in a public group
        perms::require_promote_chat_members(&cx), // user requires CAN_PROMOTE_MEMBERS permissions
        perms::require_bot_promote_chat_members(&cx)  // bot requires CAN_PROMOTE_MEMBERS permissions
    )?;

    // extract user ID from message
    let (user_id, _) = utils::extract_user_and_text(&cx, pool).await;
    if user_id.is_none() {
        // no user was targeted
        cx.reply_to("Try targeting a user next time bud.").await?;
        return Ok(());
    }

    // check if user is valid
    if let Err(_) = cx
        .requester
        .get_chat_member(chat.id, user_id.unwrap())
        .await
    {
        cx.reply_to("This user is ded mate.").await?; // invalid user (outdated info in db?)
        return Ok(());
    }

    // user is a dumbass
    if user_id.unwrap() == *BOT_ID {
        cx.reply_to("No u").await?;
        return Ok(());
    }

    let bot_chat_member: ChatMember = cx.requester.get_chat_member(chat.id, *BOT_ID).await?;

    cx.requester
        .promote_chat_member(chat.id, user_id.unwrap())
        .can_manage_chat(bot_chat_member.kind.can_manage_chat().unwrap_or(false))
        .can_change_info(bot_chat_member.kind.can_change_info().unwrap_or(false))
        .can_delete_messages(bot_chat_member.kind.can_delete_messages().unwrap_or(false))
        .can_manage_voice_chats(
            bot_chat_member
                .kind
                .can_manage_voice_chats()
                .unwrap_or(false),
        )
        .can_invite_users(bot_chat_member.kind.can_invite_users().unwrap_or(false))
        .can_restrict_members(bot_chat_member.kind.can_restrict_members().unwrap_or(false))
        .can_pin_messages(
            bot_chat_member.kind.can_pin_messages().unwrap_or(false) && chat.is_supergroup(),
        )
        .can_promote_members(bot_chat_member.kind.can_promote_members().unwrap_or(false))
        .await?;

    cx.reply_to("Successfully promoted!").await?;

    Ok(())
}
