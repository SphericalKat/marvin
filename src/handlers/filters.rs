use crate::utils::{self, perms, PinMode};
use crate::Cx;
use crate::BOT_ID;
use teloxide::{
    prelude::*,
    types::{ChatMember, ChatMemberStatus},
};

pub async fn save_note(cx: Cx) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_group(&cx),      // command needs to be in a public group
        perms::require_bot_admin(&cx),  // bot requires admin permissions
        perms::require_user_admin(&cx), // user requires admin permissions
    )?;

    cx.reply_to("test").send().await?;

    Ok(())
}
