use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::auth_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub username: String,
    pub role: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    let user = auth_service::register(&state.pool, &body.username, &body.password).await?;
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            id: user.id,
            username: user.username,
            role: user.role,
        }),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let token = auth_service::login(
        &state.pool,
        &state.jwt_secret,
        &body.username,
        &body.password,
    )
    .await?;
    Ok(Json(LoginResponse { token }))
}
