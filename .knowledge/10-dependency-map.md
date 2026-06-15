# Dependency Map

## Internal module dependency graph

```mermaid
graph TD
    main[main.rs] --> lib[lib.rs / AppState]
    main --> config[config.rs]
    main --> handlers_mod[handlers::mod]
    main --> tower_http[tower-http CORS/Trace]

    lib --> auth_mod[auth::mod]
    lib --> domain_mod[domain::mod]
    lib --> error[error.rs]
    lib --> handlers_mod
    lib --> models[models.rs]
    lib --> repositories_mod[repositories::mod]
    lib --> services_mod[services::mod]

    handlers_mod --> handlers_auth[handlers::auth]
    handlers_mod --> handlers_game[handlers::game]
    handlers_mod --> handlers_history[handlers::history]
    handlers_mod --> handlers_events[handlers::events]

    handlers_auth --> auth_svc[services::auth_service]
    handlers_game --> game_svc[services::game_service]
    handlers_history --> hist_svc[services::history_service]
    handlers_events --> broadcast[tokio::broadcast]

    handlers_game --> auth_mod
    handlers_history --> auth_mod
    handlers_events --> auth_mod

    auth_svc --> auth_mod
    auth_svc --> user_repo[repositories::user_repo]

    game_svc --> grid[domain::grid]
    game_svc --> grid_repo[repositories::grid_repo]
    game_svc --> broadcast

    hist_svc --> grid_repo

    user_repo --> models
    grid_repo --> models
    auth_mod --> error
    grid --> error

    style grid fill:#90EE90
    style auth_mod fill:#FFB6C1
    style error fill:#FFD700
```

**Legend:** 🟢 Pure domain logic | 🔴 Auth/security | 🟡 Error handling

## External crate dependencies

### Runtime

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.8 | HTTP framework, routing, extractors, SSE |
| `tokio` | 1 (full) | Async runtime |
| `sqlx` | 0.8 | PostgreSQL async driver, migrations |
| `argon2` | 0.5 | Argon2id password hashing |
| `jsonwebtoken` | 9 | JWT encode/decode |
| `serde` | 1 | Serialization/deserialization |
| `serde_json` | 1 | JSON parsing |
| `uuid` | 1 | UUID v4 generation |
| `chrono` | 0.4 | Timestamps |
| `dotenvy` | 0.15 | .env file loading |
| `tower-http` | 0.6 | CORS, tracing middleware |
| `tower` | 0.5 | Service trait, middleware |
| `tokio-stream` | 0.1 | BroadcastStream wrapper for SSE |
| `futures` | 0.3 | Stream trait |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output formatting |

### Dev only

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 | HTTP client for integration tests |

## External services

| Service | Connection | Purpose |
|---------|-----------|---------|
| PostgreSQL | `DATABASE_URL` env var | User storage, grid request history |

No other external services (no Redis, no message queue, no third-party APIs).
