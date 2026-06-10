use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GridRequestRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub input_grid: serde_json::Value,
    pub output_grid: serde_json::Value,
    pub grid_size: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub user_id: Uuid,
    pub grid_size: usize,
    pub input_grid: Vec<Vec<u8>>,
    pub output_grid: Vec<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}
