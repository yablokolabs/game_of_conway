# Critical Files

The 20 most important files in the project, ranked by impact.

## 1. `src/domain/grid.rs` (297 lines)
**Purpose:** Core Game of Life engine — Grid type, validation, Conway's rules, unit tests
**Key exports:** `Grid`, `GridError`
**Key functions:** `Grid::new()`, `Grid::next_state()`, `Grid::count_neighbors()`
**Why it matters:** The entire product is this. If the rules are wrong, everything is wrong.
**Related:** `src/services/game_service.rs`, `src/handlers/game.rs`

## 2. `src/auth/mod.rs` (90 lines)
**Purpose:** Authentication — Argon2id hashing, JWT tokens, AuthUser extractor
**Key exports:** `AuthUser`, `Claims`, `hash_password`, `verify_password`, `create_token`, `validate_token`
**Why it matters:** Every protected endpoint depends on the `AuthUser` extractor. Bugs here = security holes.
**Related:** `src/services/auth_service.rs`, `src/lib.rs` (AppState.jwt_secret)

## 3. `src/lib.rs` (19 lines)
**Purpose:** Crate root — module declarations, `AppState` struct
**Key exports:** `AppState { pool, jwt_secret, event_tx }`
**Why it matters:** Central state shared across all handlers. Breaking this breaks everything.
**Related:** `src/main.rs`, all handlers

## 4. `src/main.rs` (54 lines)
**Purpose:** Binary entrypoint — server bootstrap, migrations, graceful shutdown
**Why it matters:** If startup fails, nothing works.
**Related:** `src/config.rs`, `src/lib.rs`, `src/handlers/mod.rs`

## 5. `src/handlers/mod.rs` (20 lines)
**Purpose:** Router assembly — maps paths to handlers, body limit
**Why it matters:** The single source of truth for all routes. Missing a route = missing an endpoint.
**Related:** All handler files

## 6. `src/error.rs` (62 lines)
**Purpose:** AppError enum → HTTP responses with status codes and JSON body
**Key exports:** `AppError` (Validation, Auth, NotFound, Internal, Conflict)
**Why it matters:** Consistent error responses across the API. Wrong mapping = wrong HTTP status codes.
**Related:** All services and handlers

## 7. `src/services/auth_service.rs` (49 lines)
**Purpose:** Register/login orchestration with input validation
**Key exports:** `register()`, `login()`
**Why it matters:** Business rules for auth (password length, username uniqueness).
**Related:** `src/auth/mod.rs`, `src/repositories/user_repo.rs`

## 8. `src/services/game_service.rs` (42 lines)
**Purpose:** Compute → persist → broadcast pipeline
**Key exports:** `compute_and_store()`
**Why it matters:** Orchestrates the core game flow and SSE broadcasting.
**Related:** `src/domain/grid.rs`, `src/repositories/grid_repo.rs`

## 9. `src/handlers/game.rs` (53 lines)
**Purpose:** POST /api/game/next handler with CellValue deserialization
**Key exports:** `next_state()`, `CellValue` enum (bool/int)
**Why it matters:** API boundary where grid input is parsed and validated.
**Related:** `src/services/game_service.rs`

## 10. `src/repositories/grid_repo.rs` (69 lines)
**Purpose:** Grid request storage and dynamic filtered queries
**Key exports:** `save()`, `query()`, `QueryFilters`
**Why it matters:** Dynamic query builder — bugs here = wrong history results or SQL errors.
**Related:** `src/services/game_service.rs`, `src/services/history_service.rs`

## 11. `src/repositories/user_repo.rs` (37 lines)
**Purpose:** User CRUD queries
**Key exports:** `create()`, `find_by_username()`
**Why it matters:** Auth depends on correct user lookup.
**Related:** `src/services/auth_service.rs`

## 12. `src/models.rs` (31 lines)
**Purpose:** Shared domain types — User, GridRequestRow, GameEvent
**Why it matters:** Structural contract between layers. Column mismatch = runtime panic.
**Related:** All repositories and services

## 13. `src/config.rs` (24 lines)
**Purpose:** Environment-based configuration loading
**Key exports:** `Config`
**Why it matters:** Missing env vars = server won't start.
**Related:** `src/main.rs`, `.env.example`

## 14. `src/handlers/events.rs` (26 lines)
**Purpose:** SSE streaming handler with keep-alive
**Why it matters:** Real-time spectator view feature.
**Related:** `src/services/game_service.rs` (broadcast sender)

## 15. `src/handlers/history.rs` (56 lines)
**Purpose:** History query handler with pagination
**Why it matters:** Bonus feature I — filtered grid history.
**Related:** `src/services/history_service.rs`

## 16. `src/handlers/auth.rs` (60 lines)
**Purpose:** Register/login HTTP handlers
**Why it matters:** API boundary for auth endpoints.
**Related:** `src/services/auth_service.rs`

## 17. `migrations/001_create_users.sql`
**Purpose:** Users table schema
**Why it matters:** Database schema is the foundation. Wrong schema = broken queries.

## 18. `migrations/002_create_grid_requests.sql`
**Purpose:** Grid requests table schema with indexes
**Why it matters:** History queries depend on correct indexing.

## 19. `tests/common/mod.rs` (82 lines)
**Purpose:** Shared test infrastructure — TestApp spawner, register/login helper
**Why it matters:** All integration tests depend on this. Broken helper = all integration tests fail.

## 20. `docker-compose.yml`
**Purpose:** PostgreSQL container definition
**Why it matters:** Required for local development and testing.
