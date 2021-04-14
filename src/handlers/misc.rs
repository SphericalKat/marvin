use sqlx::{Pool, Postgres};
use teloxide::utils::html;

use crate::repo::users;
use crate::Cx;

pub async fn handle_id(cx: Cx, username: String, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    if username == "" {
        let id = cx.update.chat_id();
        cx.reply_to(format!(
            "The current chat's ID is: {}",
            html::code_inline(&id.to_string())
        ))
        .await?;
        return Ok(());
    }

    if username.chars().nth(0).unwrap_or_default() == '@' {
        let stripped_username = username.replace("@", "");
        let user = users::get_user(None, Some(stripped_username), pool).await;

        match user {
            Ok(u) => {
                cx.reply_to(format!(
                    "{}'s ID is {}.",
                    u.full_name,
                    html::code_inline(&u.user_id.to_string())
                ))
                .await?;
            }
            Err(_) => {
                cx.reply_to(
                    "Could not find a user by this name; are you sure I've seen them before?",
                )
                .await?;
            }
        }
    }

    Ok(())
}
