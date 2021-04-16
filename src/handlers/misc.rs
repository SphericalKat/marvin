use sqlx::{Pool, Postgres};
use teloxide::types::{ChatKind, ForwardedFrom};
use teloxide::{prelude::Requester, utils::html};

use crate::utils;
use crate::Cx;

pub async fn handle_id(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let (user_id, _) = utils::extract_user_and_text(&cx, pool).await;
    if let Some(user_id) = user_id {
        if let Some(prev_msg) = cx.update.reply_to_message() {
            let user1 = prev_msg.from().unwrap();
            if let Some(user2) = prev_msg.forward_from() {
                if let ForwardedFrom::User(u) = user2 {
                    cx.reply_to(
                        format!(
                            "The original sender, {} has an ID of {}.\nThe forwarder, {}, has an ID of {}.",
                            html::escape(&u.first_name),
                            html::code_inline(&u.id.to_string()),
                            html::escape(&user1.first_name),
                            html::code_inline(&user1.id.to_string()),
                        ),
                    ).await?;
                } else if let ForwardedFrom::SenderName(_) = user2 {
                    cx.reply_to(format!(
                        "{}'s ID is {}",
                        html::escape(&user1.first_name),
                        html::code_inline(&user1.id.to_string())
                    ))
                    .await?;
                }
            }
        } else {
            if let ChatKind::Private(user) = cx.requester.get_chat(user_id).await?.kind {
                cx.reply_to(format!(
                    "{}'s ID is {}",
                    html::escape(&user.first_name.unwrap_or("".to_owned())),
                    html::code_inline(&user_id.to_string())
                ))
                .await?;
            }
        }
    } else {
        let chat = &cx.update.chat;
        cx.reply_to(format!(
            "This chat's ID is {}",
            html::code_inline(&chat.id.to_string())
        ))
        .await?;
    }
    Ok(())
}
