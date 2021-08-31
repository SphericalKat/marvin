use sqlx::{Pool, Postgres};

use crate::entities::Note;

pub async fn insert_note(note: &Note, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
				INSERT into notes (chat_id, note_id, note_content) VALUES ($1, $2, $3)
				ON CONFLICT (chat_id, note_id) DO
				UPDATE SET note_content = excluded.note_content
				WHERE (notes.note_content) IS DISTINCT FROM (excluded.note_content)
				"#,
        note.chat_id,
        note.note_id,
        note.note_content,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_note(
    chat_id: Option<i64>,
    note_id: Option<i64>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let note = sqlx::query_as!(
        Note,
        "SELECT * FROM notes WHERE chat_id = $1 AND note_id = $2",
        chat_id,
        note_id
    )
    .fetch_one(pool)
    .await?;

    Ok(note)
}
