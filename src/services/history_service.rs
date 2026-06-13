use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::GridRequestRow;
use crate::repositories::grid_repo;

pub struct Filters {
    pub user_id: Option<Uuid>,
    pub grid_size: Option<i32>,
    pub input_state: Option<serde_json::Value>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: i64,
    pub per_page: i64,
}

pub async fn query(pool: &PgPool, filters: Filters) -> Result<Vec<GridRequestRow>, AppError> {
    let offset = (filters.page - 1) * filters.per_page;
    grid_repo::query(
        pool,
        grid_repo::QueryFilters {
            user_id: filters.user_id,
            grid_size: filters.grid_size,
            input_state: filters.input_state,
            from: filters.from,
            to: filters.to,
            limit: filters.per_page,
            offset,
        },
    )
    .await
}
