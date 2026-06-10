use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::GridRequestRow;
use crate::services::history_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub user_id: Option<Uuid>,
    pub grid_size: Option<i32>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub data: Vec<GridRequestRow>,
    pub page: i64,
    pub per_page: i64,
}

pub async fn query(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let data = history_service::query(
        &state.pool,
        history_service::Filters {
            user_id: params.user_id,
            grid_size: params.grid_size,
            from: params.from,
            to: params.to,
            page,
            per_page,
        },
    )
    .await?;

    Ok(Json(HistoryResponse {
        data,
        page,
        per_page,
    }))
}
