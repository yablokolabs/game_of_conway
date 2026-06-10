use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub mod auth;
pub mod events;
pub mod game;
pub mod history;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/game/next", post(game::next_state))
        .route("/api/history", get(history::query))
        .route("/api/events", get(events::stream))
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
}
