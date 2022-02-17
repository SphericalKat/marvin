use crate::entities::Note;
use crate::repo::filters::insert_note;
use crate::utils::{self, perms};
use sqlx::{Pool, Postgres};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude2::*;
use teloxide::utils::html;

pub async fn save_note(
    bot: &crate::Bot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_user_admin(bot, message), // user requires admin permissions
    )?;

    // extract text from message
    let (_, text) = utils::extract_user_and_text(bot, message, pool).await;
    if text.is_none() {
        // no text in message
        bot.send_message(message.chat.id, "You need to give the note a name!")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let content = text.as_ref().unwrap().split_once(' ');
    if content.is_none() {
        // no content in text
        bot.send_message(message.chat.id, "You need to give the note some content!")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let (note_id, note_content) = content.unwrap();

    let chat_id = message.chat.id;
    let note = Note {
        chat_id,
        note_id: note_id.to_owned(),
        note_content: note_content.to_owned(),
    };

    match insert_note(&note, pool).await {
        Ok(_) => {
            bot.send_message(
                message.chat.id,
                format!("Saved note {}.", html::code_inline(note_id)),
            )
            .reply_to_message_id(message.id)
            .await?;
        }
        Err(err) => {
            bot.send_message(message.chat.id, err.to_string())
                .reply_to_message_id(message.id)
                .await?;
        }
    }

    Ok(())
}
