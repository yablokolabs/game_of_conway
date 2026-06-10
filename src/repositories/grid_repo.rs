use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::GridRequestRow;

pub async fn save(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    input_grid: &serde_json::Value,
    output_grid: &serde_json::Value,
    grid_size: i32,
) -> Result<GridRequestRow, AppError> {
    let row = sqlx::query_as::<_, GridRequestRow>(
        "INSERT INTO grid_requests (id, user_id, input_grid, output_grid, grid_size) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, user_id, input_grid, output_grid, grid_size, created_at",
    )
    .bind(id)
    .bind(user_id)
    .bind(input_grid)
    .bind(output_grid)
    .bind(grid_size)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub struct QueryFilters {
    pub user_id: Option<Uuid>,
    pub grid_size: Option<i32>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn query(pool: &PgPool, filters: QueryFilters) -> Result<Vec<GridRequestRow>, AppError> {
    let mut qb: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new("SELECT id, user_id, input_grid, output_grid, grid_size, created_at FROM grid_requests WHERE 1=1");

    if let Some(uid) = filters.user_id {
        qb.push(" AND user_id = ").push_bind(uid);
    }
    if let Some(size) = filters.grid_size {
        qb.push(" AND grid_size = ").push_bind(size);
    }
    if let Some(from) = filters.from {
        qb.push(" AND created_at >= ").push_bind(from);
    }
    if let Some(to) = filters.to {
        qb.push(" AND created_at <= ").push_bind(to);
    }

    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(filters.limit)
        .push(" OFFSET ")
        .push_bind(filters.offset);

    let rows = qb
        .build_query_as::<GridRequestRow>()
        .fetch_all(pool)
        .await?;

    Ok(rows)
}
