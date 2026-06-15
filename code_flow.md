# Conway's Game of Life — Code Flow & Navigation

A detailed guide to how the code is organized and how data flows through each layer.

---

## Architecture

```
main.rs ──► handlers/mod.rs (router) ──► handlers/{auth,game,history,events}.rs
                                              │
                                              ▼
                                       services/{auth,game,history}_service.rs
                                              │
                                         ┌────┴────┐
                                         ▼         ▼
                                   auth/mod.rs   domain/grid.rs
                                         │
                                         ▼
                                   repositories/{user,grid}_repo.rs
                                         │
                                         ▼
                                      PgPool (PostgreSQL)
```

---

## Entry Point — `src/main.rs`

1. Initializes tracing (logging with env filter).
2. Loads `Config::from_env()` (reads `.env` file).
3. Connects `PgPool` and runs SQL migrations.
4. Optionally bootstraps an admin user if `ADMIN_USERNAME` + `ADMIN_PASSWORD` env vars are set.
5. Creates `broadcast::channel(1024)` for SSE events.
6. Builds `AppState` and attaches the router.
7. Layers: `CorsLayer::permissive()` + `TraceLayer`.
8. Binds `TcpListener` and serves with graceful shutdown (`ctrl_c`).

---

## Shared State — `src/lib.rs`

```rust
pub struct AppState {
    pub pool: PgPool,              // Postgres connection pool
    pub jwt_secret: String,        // JWT signing secret
    pub event_tx: broadcast::Sender<GameEvent>,  // SSE broadcast channel
}
```

All handlers receive this via Axum's `State` extractor.

---

## Configuration — `src/config.rs`

| Field | Env Var | Required | Default |
|-------|---------|----------|---------|
| `database_url` | `DATABASE_URL` | ✅ | — |
| `jwt_secret` | `JWT_SECRET` | ✅ | — |
| `host` | `HOST` | ❌ | `0.0.0.0` |
| `port` | `PORT` | ❌ | `3000` |
| `admin_username` | `ADMIN_USERNAME` | ❌ | — |
| `admin_password` | `ADMIN_PASSWORD` | ❌ | — |

---

## Router — `src/handlers/mod.rs`

```
POST /api/auth/register  → auth::register      (public)
POST /api/auth/login     → auth::login          (public)
POST /api/game/next      → game::next_state     (requires AuthUser)
GET  /api/history        → history::query       (requires AuthUser)
GET  /api/events         → events::stream       (requires AuthUser)
```

Body limit: **8 MB** via `DefaultBodyLimit`.

---

## Request Flows

### Flow 1: Register — `POST /api/auth/register`

```
handlers/auth.rs::register
    │  Extracts: State(AppState), Json<RegisterRequest { username, password }>
    ▼
services/auth_service.rs::register
    │  1. Validates username (1–255 chars) & password (≥8 chars)
    │  2. spawn_blocking → auth::hash_password (Argon2id + random salt)
    ▼
repositories/user_repo.rs::create
    │  INSERT INTO users (id, username, password_hash, role) ... RETURNING *
    │  role = "user" (default)
    ▼
Response: 201 { id, username, role }
```

### Flow 2: Login — `POST /api/auth/login`

```
handlers/auth.rs::login
    │  Extracts: State(AppState), Json<LoginRequest { username, password }>
    ▼
services/auth_service.rs::login
    │  1. user_repo::find_by_username → SELECT ... WHERE username = $1
    │  2. spawn_blocking → auth::verify_password (Argon2id)
    │  3. auth::create_token → JWT with { sub: user_id, role, exp: now+24h }
    ▼
Response: 200 { token }
```

### Flow 3: Game Next State — `POST /api/game/next`

```
handlers/game.rs::next_state
    │  Extracts: State(AppState), AuthUser (JWT validated), Json<NextStateRequest>
    │  Note: cells accept both 0/1 and true/false via CellValue enum
    ▼
services/game_service.rs::compute_and_store
    │  1. Grid::new(cells) → validates (3–1000, square, cells ∈ {0,1})
    │  2. input.next_state() → pure Conway computation
    │  3. Serialize grids to JSON
    ▼
repositories/grid_repo.rs::save
    │  INSERT INTO grid_requests (...) RETURNING *
    ▼
broadcast::Sender::send(GameEvent)
    │  Fans out to all SSE subscribers (fire-and-forget)
    ▼
Response: 200 { cells: [[u8]] }
```

### Flow 4: History — `GET /api/history`

```
handlers/history.rs::query
    │  Extracts: State(AppState), AuthUser, Query<HistoryQuery>
    │  RBAC check:
    │    - admin  → can filter by any user_id (or see all)
    │    - user   → forced to own user_id
    │  Parses optional input_state (JSON string → serde_json::Value)
    ▼
services/history_service.rs::query
    │  Computes offset = (page - 1) * per_page
    ▼
repositories/grid_repo.rs::query
    │  Dynamic QueryBuilder:
    │    SELECT * FROM grid_requests
    │    WHERE [user_id = ?] AND [grid_size = ?] AND [input_grid = ?]
    │          AND [created_at >= ?] AND [created_at <= ?]
    │    ORDER BY created_at DESC LIMIT ? OFFSET ?
    ▼
Response: 200 { data: [GridRequestRow], page, per_page }
```

### Flow 5: SSE Events — `GET /api/events`

```
handlers/events.rs::stream
    │  Extracts: State(AppState), AuthUser (validates token, value unused)
    │  1. state.event_tx.subscribe() → new broadcast::Receiver
    │  2. BroadcastStream wraps the receiver
    │  3. filter_map: skip errors, serialize GameEvent → JSON
    ▼
Response: SSE stream
    │  data: {"user_id":"...","grid_size":3,...}
    │  (keep-alive every 15 seconds)
```

---

## Domain Layer — `src/domain/grid.rs`

The **pure game engine** with zero I/O.

```rust
pub struct Grid {
    cells: Vec<Vec<u8>>,  // private
}
```

| Method | Purpose |
|--------|---------|
| `Grid::new(cells)` | Validates: non-empty, 3 ≤ size ≤ 1000, square, cells ∈ {0,1} |
| `next_state(&self) → Grid` | Applies Conway rules, returns new grid |
| `count_neighbors(row, col) → u8` | Counts 8 adjacent live cells, no wrapping |
| `size()`, `cells()`, `into_cells()` | Accessors |

**Conway rules in `next_state()`:**
```
alive + 2|3 neighbors → alive
dead  + 3 neighbors   → alive
otherwise              → dead
```

---

## Auth Layer — `src/auth/mod.rs`

### Functions

| Function | What it does |
|----------|-------------|
| `hash_password(password)` | Argon2id hash with random salt |
| `verify_password(password, hash)` | Argon2id verification |
| `create_token(user_id, role, secret)` | JWT (HS256, 24h expiry) |
| `validate_token(token, secret)` | Decode and validate JWT |

### Extractors (Axum `FromRequestParts`)

| Extractor | Behavior |
|-----------|---------|
| `AuthUser { user_id, role }` | Reads `Authorization: Bearer <token>` → validates → populates fields. Has `is_admin()` helper. |
| `AdminUser { user_id }` | Delegates to `AuthUser`, rejects with 403 if not admin. |

No centralized middleware — auth is enforced by **placing extractors in handler signatures**.

---

## Repository Layer

### `src/repositories/user_repo.rs`

| Function | SQL |
|----------|-----|
| `create(pool, id, username, password_hash, role)` | `INSERT INTO users (...) RETURNING *` |
| `find_by_username(pool, username)` | `SELECT ... WHERE username = $1` |

### `src/repositories/grid_repo.rs`

| Function | SQL |
|----------|-----|
| `save(pool, id, user_id, input_grid, output_grid, grid_size)` | `INSERT INTO grid_requests (...) RETURNING *` |
| `query(pool, filters)` | Dynamic `QueryBuilder` with optional WHERE clauses |

---

## Error Handling — `src/error.rs`

All layers return `Result<_, AppError>`. Axum auto-converts via `IntoResponse`.

| AppError Variant | HTTP Status | Notes |
|-----------------|-------------|-------|
| `Validation(String)` | 400 | Grid errors, input validation |
| `Auth(String)` | 401 | Bad credentials, invalid/expired JWT |
| `Forbidden(String)` | 403 | Non-admin accessing admin routes |
| `NotFound(String)` | 404 | — |
| `Conflict(String)` | 409 | Duplicate username |
| `Internal(String)` | 500 | Message hidden from client, logged via `tracing::error!` |

**Auto-conversions via `From`:**
- `GridError` → `Validation`
- `sqlx::Error` (unique violation) → `Conflict`, otherwise → `Internal`
- `jsonwebtoken::errors::Error` → `Auth`
- `serde_json::Error` → `Internal`

---

## Data Models — `src/models.rs`

| Struct | Used By | Fields |
|--------|---------|--------|
| `User` | DB row (sqlx `FromRow`) | `id, username, password_hash, role, created_at` |
| `GridRequestRow` | DB row + API response | `id, user_id, input_grid (JSONB), output_grid (JSONB), grid_size, created_at` |
| `GameEvent` | SSE broadcast payload | `user_id, grid_size, input_grid, output_grid, created_at` |

---

## Cross-Cutting Patterns

| Pattern | Implementation |
|---------|---------------|
| **Auth** | Axum extractors (`AuthUser`, `AdminUser`) in handler signatures — no global middleware |
| **Blocking work** | `tokio::task::spawn_blocking` for Argon2 hashing (CPU-intensive) |
| **SSE broadcast** | `tokio::broadcast::channel(1024)` — lock-free, multi-consumer fanout |
| **Error propagation** | `?` operator everywhere, `From` impls auto-convert to `AppError` |
| **Body limit** | 8 MB via `DefaultBodyLimit` on the router |
| **CORS** | `CorsLayer::permissive()` (all origins) |
| **Tracing** | `TraceLayer::new_for_http()` with env-configurable filter |
| **Graceful shutdown** | `tokio::signal::ctrl_c()` |

---

## File Navigation Quick Reference

```
src/
├── main.rs                    # Start here — bootstrap, admin, server
├── lib.rs                     # AppState definition
├── config.rs                  # Env vars → Config struct
├── error.rs                   # AppError enum → HTTP responses
├── models.rs                  # User, GridRequestRow, GameEvent
├── domain/
│   └── grid.rs                # Pure game logic + unit tests
├── auth/
│   └── mod.rs                 # JWT, Argon2, AuthUser/AdminUser extractors
├── repositories/
│   ├── mod.rs
│   ├── user_repo.rs           # User CRUD
│   └── grid_repo.rs           # Grid save + dynamic query
├── services/
│   ├── mod.rs
│   ├── auth_service.rs        # Register/login orchestration
│   ├── game_service.rs        # Compute → persist → broadcast
│   └── history_service.rs     # Filtered history queries
└── handlers/
    ├── mod.rs                 # Router definition (all routes)
    ├── auth.rs                # POST register, login
    ├── game.rs                # POST next state
    ├── history.rs             # GET history (RBAC)
    └── events.rs              # GET SSE stream
```
