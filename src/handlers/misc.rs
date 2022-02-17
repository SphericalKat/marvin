use sqlx::{Pool, Postgres};
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{ChatKind, ForwardedFrom};
use teloxide::{prelude2::*, utils::html};

use crate::utils;

pub async fn handle_id(
    bot: &crate::Bot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let (user_id, _) = utils::extract_user_and_text(bot, message, pool).await;
    if let Some(user_id) = user_id {
        if let Some(prev_msg) = message.reply_to_message() {
            let user1 = prev_msg.from().unwrap();
            if let Some(user2) = prev_msg.forward_from() {
                if let ForwardedFrom::User(u) = user2 {
                    bot.send_message(
											message.chat.id,
                        format!(
                            "The original sender, {} has an ID of {}.\nThe forwarder, {}, has an ID of {}.",
                            html::escape(&u.first_name),
                            html::code_inline(&u.id.to_string()),
                            html::escape(&user1.first_name),
                            html::code_inline(&user1.id.to_string()),
                        ),
                    )
										.reply_to_message_id(message.id)
										.await?;
                } else if let ForwardedFrom::SenderName(_) = user2 {
                    bot.send_message(
                        message.chat.id,
                        format!(
                            "{}'s ID is {}",
                            html::escape(&user1.first_name),
                            html::code_inline(&user1.id.to_string())
                        ),
                    )
                    .reply_to_message_id(message.id)
                    .await?;
                }
            }
        } else if let ChatKind::Private(user) = bot.get_chat(user_id).await?.kind {
            bot.send_message(
                message.chat.id,
                format!(
                    "{}'s ID is {}",
                    html::escape(&user.first_name.unwrap_or_else(|| "".to_owned())),
                    html::code_inline(&user_id.to_string())
                ),
            )
            .reply_to_message_id(message.id)
            .await?;
        }
    }
    Ok(())
}
