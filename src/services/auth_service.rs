use sqlx::PgPool;
use uuid::Uuid;

use crate::auth;
use crate::error::AppError;
use crate::models::User;
use crate::repositories::user_repo;

pub async fn register(pool: &PgPool, username: &str, password: &str) -> Result<User, AppError> {
    if username.is_empty() || username.len() > 255 {
        return Err(AppError::Validation(
            "username must be between 1 and 255 characters".into(),
        ));
    }
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }

    let password = password.to_owned();
    let password_hash = tokio::task::spawn_blocking(move || auth::hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("blocking task failed: {e}")))??;
    user_repo::create(pool, Uuid::new_v4(), username, &password_hash).await
}

pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    username: &str,
    password: &str,
) -> Result<String, AppError> {
    let user = user_repo::find_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::Auth("invalid credentials".into()))?;

    let password = password.to_owned();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || auth::verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(format!("blocking task failed: {e}")))??;

    if !valid {
        return Err(AppError::Auth("invalid credentials".into()));
    }

    auth::create_token(user.id, jwt_secret)
}
