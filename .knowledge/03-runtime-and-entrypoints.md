# Runtime & Entrypoints

## Application entrypoint

**File:** `src/main.rs`

The `#[tokio::main]` async function:
1. Initializes `tracing_subscriber` with env filter
2. Loads `Config` from environment via `dotenvy`
3. Connects to PostgreSQL via `PgPool::connect()`
4. Runs embedded SQL migrations via `sqlx::migrate!()`
5. Creates a `broadcast::channel(1024)` for SSE events
6. Assembles `AppState { pool, jwt_secret, event_tx }`
7. Builds router via `handlers::router().with_state(state)`
8. Applies CORS (permissive) and tracing middleware
9. Binds to `{host}:{port}` and serves with graceful shutdown on SIGTERM/SIGINT

## How to run locally

```bash
# 1. Start PostgreSQL
docker-compose up -d

# 2. Set environment
cp .env.example .env
# Edit .env if needed (defaults work with docker-compose)

# 3. Run
cargo run
# Server starts on http://localhost:3000
```

## How to run tests

```bash
# Unit tests only (no database needed)
cargo test --lib

# Full suite (requires PostgreSQL running)
cargo test
```

## Docker / deployment files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | PostgreSQL 16-alpine with healthcheck |
| `.env.example` | Template for required environment variables |

There is no Dockerfile for the application itself. To containerize, build with
`cargo build --release` and copy the binary.

## Build commands

```bash
cargo build            # Debug build
cargo build --release  # Release build (binary at target/release/game_of_conway)
cargo fmt              # Format code
cargo clippy           # Lint
```

## Migrations

Migrations are in `migrations/` and embedded at compile time via `sqlx::migrate!()`.
They run automatically on server startup — no separate migration command needed.

| File | Purpose |
|------|---------|
| `migrations/001_create_users.sql` | `users` table + username index |
| `migrations/002_create_grid_requests.sql` | `grid_requests` table + indexes |
