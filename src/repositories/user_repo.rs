use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::User;

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    password_hash: &str,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, password_hash) \
         VALUES ($1, $2, $3) \
         RETURNING id, username, password_hash, created_at",
    )
    .bind(id)
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at \
         FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
