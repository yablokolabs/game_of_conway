use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::services::game_service;
use crate::AppState;

#[derive(Deserialize)]
#[serde(untagged)]
enum CellValue {
    Bool(bool),
    Int(u8),
}

impl CellValue {
    fn into_u8(self) -> u8 {
        match self {
            Self::Bool(b) => u8::from(b),
            Self::Int(i) => i,
        }
    }
}

#[derive(Deserialize)]
pub struct NextStateRequest {
    cells: Vec<Vec<CellValue>>,
}

#[derive(Serialize)]
pub struct NextStateResponse {
    pub cells: Vec<Vec<u8>>,
}

pub async fn next_state(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<NextStateRequest>,
) -> Result<Json<NextStateResponse>, AppError> {
    let cells: Vec<Vec<u8>> = body
        .cells
        .into_iter()
        .map(|row| row.into_iter().map(CellValue::into_u8).collect())
        .collect();

    let result =
        game_service::compute_and_store(&state.pool, &state.event_tx, auth.user_id, cells).await?;

    Ok(Json(NextStateResponse {
        cells: result.into_cells(),
    }))
}
