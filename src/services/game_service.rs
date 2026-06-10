use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::domain::grid::Grid;
use crate::error::AppError;
use crate::models::GameEvent;
use crate::repositories::grid_repo;

pub async fn compute_and_store(
    pool: &PgPool,
    event_tx: &broadcast::Sender<GameEvent>,
    user_id: Uuid,
    cells: Vec<Vec<u8>>,
) -> Result<Grid, AppError> {
    let input = Grid::new(cells)?;
    let output = input.next_state();

    let grid_size = input.size() as i32;
    let input_json = serde_json::to_value(input.cells())?;
    let output_json = serde_json::to_value(output.cells())?;

    grid_repo::save(
        pool,
        Uuid::new_v4(),
        user_id,
        &input_json,
        &output_json,
        grid_size,
    )
    .await?;

    let _ = event_tx.send(GameEvent {
        user_id,
        grid_size: input.size(),
        input_grid: input.into_cells(),
        output_grid: output.cells().to_vec(),
        created_at: chrono::Utc::now(),
    });

    Ok(output)
}
