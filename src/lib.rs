pub mod auth;
pub mod config;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

use models::GameEvent;
use sqlx::PgPool;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub event_tx: broadcast::Sender<GameEvent>,
}
