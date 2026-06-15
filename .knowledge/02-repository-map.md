# Repository Map

## Top-level structure

```
game_of_conway/
├── Cargo.toml                  # Package manifest, dependencies
├── Cargo.lock                  # Locked dependency versions
├── docker-compose.yml          # PostgreSQL 16 container
├── .env.example                # Environment variable template
├── README.md                   # Project documentation
├── postman_collection.json     # Postman API collection for testing
├── migrations/                 # SQL migration files (embedded at compile time)
│   ├── 001_create_users.sql
│   ├── 002_create_grid_requests.sql
│   └── 003_add_user_role.sql
├── src/                        # Application source
│   ├── main.rs                 # Binary entrypoint — server bootstrap
│   ├── lib.rs                  # Crate root — AppState, module declarations
│   ├── config.rs               # Environment-based configuration
│   ├── error.rs                # AppError enum → HTTP status + JSON body
│   ├── models.rs               # Shared types: User, GridRequestRow, GameEvent
│   ├── domain/                 # Pure business logic (no I/O)
│   │   ├── mod.rs
│   │   └── grid.rs             # Grid type, validation, Conway's rules, unit tests
│   ├── auth/                   # Authentication & authorization
│   │   └── mod.rs              # Argon2id hashing, JWT, AuthUser + AdminUser extractors
│   ├── handlers/               # HTTP request handlers (Axum)
│   │   ├── mod.rs              # Router assembly with all routes
│   │   ├── auth.rs             # Register + Login handlers
│   │   ├── game.rs             # Next-state handler (CellValue deserialization)
│   │   ├── history.rs          # History query handler
│   │   └── events.rs           # SSE streaming handler
│   ├── services/               # Business logic orchestration
│   │   ├── mod.rs
│   │   ├── auth_service.rs     # Register/login orchestration
│   │   ├── game_service.rs     # Compute → persist → broadcast
│   │   └── history_service.rs  # Filtered history queries
│   └── repositories/           # Database access layer
│       ├── mod.rs
│       ├── user_repo.rs        # User CRUD
│       └── grid_repo.rs        # Grid request storage + dynamic query
└── tests/                      # Integration tests
    ├── common/
    │   └── mod.rs              # TestApp helper (spawn server, register/login)
    ├── register_login.rs       # Auth flow tests (6 tests)
    ├── game_next.rs            # Game endpoint tests (7 tests)
    ├── history.rs              # History query tests (4 tests)
    ├── events.rs               # SSE/broadcast tests (3 tests)
    └── rbac.rs                 # RBAC authorization tests (7 tests)
```

## Central files by concern

| Concern | Files |
|---------|-------|
| **App startup** | `src/main.rs`, `src/lib.rs`, `src/config.rs` |
| **Routing** | `src/handlers/mod.rs` |
| **Auth** | `src/auth/mod.rs`, `src/services/auth_service.rs`, `src/handlers/auth.rs` |
| **Game logic** | `src/domain/grid.rs` |
| **Database** | `src/repositories/user_repo.rs`, `src/repositories/grid_repo.rs`, `migrations/` |
| **Error handling** | `src/error.rs` |
| **Config** | `src/config.rs`, `.env.example` |
| **Tests** | `tests/` directory |
