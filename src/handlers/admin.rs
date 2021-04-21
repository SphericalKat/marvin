use sqlx::{Pool, Postgres};

use crate::utils::{self, perms};
use crate::Cx;

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

    Ok(())
}
