use sqlx::{Pool, Postgres};

use crate::entities::User;

pub async fn insert_user(user: &User, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into users (user_id, user_name, full_name) VALUES ($1, $2, $3) 
        ON CONFLICT (user_id) DO 
        UPDATE SET (user_name, full_name) = (excluded.user_name, excluded.full_name)
        WHERE (users.user_name, users.full_name) IS DISTINCT FROM (excluded.user_name, excluded.full_name)
        "#,
        user.user_id,
        user.user_name,
        user.full_name,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user(
    user_id: Option<i64>,
    user_name: Option<String>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE user_id = $1 OR user_name = $2",
        user_id,
        user_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}
