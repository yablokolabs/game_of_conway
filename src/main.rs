use game_of_conway::config::Config;
use game_of_conway::handlers;
use game_of_conway::AppState;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "game_of_conway=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env().map_err(|e| format!("config error: {e}"))?;

    let pool = PgPool::connect(&config.database_url).await?;

    // Run migrations embedded at compile time
    sqlx::migrate!("./migrations").run(&pool).await?;

    let (event_tx, _) = broadcast::channel(1024);

    let state = AppState {
        pool,
        jwt_secret: config.jwt_secret,
        event_tx,
    };

    let app = handlers::router()
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
    tracing::info!("shutting down");
}
