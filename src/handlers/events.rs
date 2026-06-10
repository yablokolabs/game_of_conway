use axum::extract::State;
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::auth::AuthUser;
use crate::models::GameEvent;
use crate::AppState;

pub async fn stream(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result: Result<GameEvent, _>| {
        let event = result.ok()?;
        let data = serde_json::to_string(&event).ok()?;
        Some(Ok::<_, Infallible>(Event::default().data(data)))
    });
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    )
}
