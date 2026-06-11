use game_of_conway::config::Config;
use game_of_conway::handlers;
use game_of_conway::models::ROLE_ADMIN;
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

    // Bootstrap admin user if ADMIN_USERNAME and ADMIN_PASSWORD are set
    if let (Some(username), Some(password)) = (&config.admin_username, &config.admin_password) {
        bootstrap_admin(&pool, username, password).await?;
    }

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

async fn bootstrap_admin(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use game_of_conway::auth;
    use game_of_conway::repositories::user_repo;

    let existing = user_repo::find_by_username(pool, username)
        .await
        .map_err(|e| format!("failed to check for admin user: {e:?}"))?;
    if existing.is_some() {
        tracing::info!("admin user '{username}' already exists, skipping bootstrap");
        return Ok(());
    }

    let pw = password.to_owned();
    let password_hash = tokio::task::spawn_blocking(move || auth::hash_password(&pw))
        .await
        .map_err(|e| format!("blocking task failed: {e}"))?
        .map_err(|e| format!("password hashing failed: {e:?}"))?;

    user_repo::create(pool, uuid::Uuid::new_v4(), username, &password_hash, ROLE_ADMIN)
        .await
        .map_err(|e| format!("failed to create admin user: {e:?}"))?;
    tracing::info!("admin user '{username}' bootstrapped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
    tracing::info!("shutting down");
}
