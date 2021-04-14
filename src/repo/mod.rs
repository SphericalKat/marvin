use sqlx::{Pool, Postgres};

pub async fn insert_user(user_id: i64, user_name: Option<String>, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into users (user_id, user_name) VALUES ($1, $2) 
        ON CONFLICT (user_id) DO 
        UPDATE SET user_name = excluded.user_name
        WHERE (users.user_name) IS DISTINCT FROM (excluded.user_name)
        "#,
        user_id,
        user_name
    )
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn insert_chat(chat_id: i64, chat_name: Option<String>, pool: &Pool<Postgres>) -> anyhow::Result<()> {
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