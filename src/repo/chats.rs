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
