use crate::entities::Note;
use crate::repo::filters::insert_note;
use crate::utils::{self, perms};
use crate::Cx;
use sqlx::{Pool, Postgres};
use teloxide::prelude::*;
use teloxide::utils::html;

pub async fn save_note(cx: Cx, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    // check for required conditions
    tokio::try_join!(
        perms::require_user_admin(&cx), // user requires admin permissions
    )?;

    // extract text from message
    let (_, text) = utils::extract_user_and_text(&cx, pool).await;
    if text.is_none() {
        // no text in message
        cx.reply_to("You need to give the note a name!").await?;
        return Ok(());
    }

    let content = text.as_ref().unwrap().split_once(" ");
    if content.is_none() {
        // no content in text
        cx.reply_to("You need to give the note some content!")
            .await?;
        return Ok(());
    }

    let (note_id, note_content) = content.unwrap();

    let chat_id = cx.chat_id();
    let note = Note {
        chat_id,
        note_id: note_id.to_owned(),
        note_content: note_content.to_owned(),
    };

    match insert_note(&note, pool).await {
        Ok(_) => {
            cx.reply_to(format!("Saved note {}.", html::code_inline(note_id)))
                .send()
                .await?;
        }
        Err(err) => {
            cx.reply_to(err.to_string()).send().await?;
        }
    }

    Ok(())
}
