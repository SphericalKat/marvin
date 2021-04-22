use crate::entities::Chat;
use sqlx::{Pool, Postgres};

pub async fn insert_chat(
    chat_id: i64,
    chat_name: Option<String>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into chats (chat_id, chat_name) VALUES ($1, $2)
        ON CONFLICT (chat_id) DO
        UPDATE SET chat_name = excluded.chat_name
        WHERE (chats.chat_name) IS DISTINCT FROM (excluded.chat_name)
        "#,
        chat_id,
        chat_name
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_chat(chat_id: i64, pool: &Pool<Postgres>) -> anyhow::Result<Chat> {
    let chat = sqlx::query_as!(Chat, "SELECT * FROM chats WHERE chat_id = $1", chat_id)
        .fetch_one(pool)
        .await?;

    Ok(chat)
}
