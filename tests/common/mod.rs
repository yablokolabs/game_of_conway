use game_of_conway::handlers;
use game_of_conway::models::GameEvent;
use game_of_conway::AppState;
use sqlx::PgPool;
use tokio::sync::broadcast;

pub struct TestApp {
    pub base_url: String,
    pub pool: PgPool,
    pub event_tx: broadcast::Sender<GameEvent>,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://conway:conway@localhost:5432/conway".into());

        let pool = PgPool::connect(&database_url)
            .await
            .expect("failed to connect to test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        let (event_tx, _) = broadcast::channel(1024);

        let state = AppState {
            pool: pool.clone(),
            jwt_secret: "test-jwt-secret".into(),
            event_tx: event_tx.clone(),
        };

        let app = handlers::router().with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test server");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            base_url: format!("http://{addr}"),
            pool,
            event_tx,
        }
    }

    pub fn unique_name(&self, prefix: &str) -> String {
        format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
    }

    pub async fn register_and_login(&self, username: &str, password: &str) -> String {
        let client = reqwest::Client::new();

        client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await
            .expect("register request failed");

        let res = client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await
            .expect("login request failed");

        let body: serde_json::Value = res.json().await.unwrap();
        body["token"].as_str().unwrap().to_string()
    }
}
