# Glossary

## Domain terms

| Term | Definition |
|------|-----------|
| **Cell** | A single tile in the grid. Either alive (1/true) or dead (0/false). |
| **Grid** | A square N×N matrix of cells. The `Grid` type in `src/domain/grid.rs`. |
| **Generation** | One step of the Game of Life simulation. `Grid::next_state()` computes one generation. |
| **Next state** | The grid resulting from applying Conway's rules to every cell simultaneously. |
| **Birth** | A dead cell with exactly 3 live neighbors becomes alive. |
| **Survival** | A live cell with 2 or 3 live neighbors stays alive. |
| **Death** | A cell dies from overpopulation (>3 neighbors) or loneliness (<2 neighbors). |
| **Blinker** | A period-2 oscillator: 3 cells in a line alternate between horizontal and vertical. Used as a basic test case. |
| **Block** | A 2×2 still life pattern. Does not change between generations. |
| **Glider** | A pattern that moves diagonally across the grid, completing one cycle every 4 generations. |
| **Still life** | A pattern that remains unchanged between generations. |
| **Finite boundary** | Cells outside the grid are treated as dead. The grid does not wrap. |

## Architecture terms

| Term | Definition |
|------|-----------|
| **Handler** | An Axum async function that receives HTTP requests and returns responses. Located in `src/handlers/`. |
| **Service** | A function that orchestrates business logic between handlers and repositories. Located in `src/services/`. |
| **Repository** | A function that executes SQL queries against PostgreSQL. Located in `src/repositories/`. |
| **Extractor** | An Axum mechanism that pulls data from HTTP requests (JSON body, query params, auth tokens). `AuthUser` and `AdminUser` are custom extractors. |
| **AppState** | Shared application state (`PgPool`, `jwt_secret`, `event_tx`). Passed to handlers via Axum's `State` extractor. |
| **Middleware** | Code that runs before/after handlers. In this project: CORS, tracing, and the `AuthUser` extractor. |

## Technical terms

| Term | Definition |
|------|-----------|
| **Argon2id** | Memory-hard password hashing algorithm. Variant of Argon2 that combines data-dependent and data-independent addressing. |
| **JWT** | JSON Web Token. Stateless auth token with encoded claims, signed with HMAC-SHA256. |
| **Claims** | The payload inside a JWT. In this project: `{ sub: UUID, role: String, exp: timestamp }`. |
| **JSONB** | PostgreSQL binary JSON type. Used to store grid data. Supports indexing and querying. |
| **SSE** | Server-Sent Events. A unidirectional HTTP streaming protocol. The server pushes events to the client over a long-lived connection. |
| **Broadcast channel** | A `tokio::sync::broadcast` channel. Multi-producer, multi-consumer. Each subscriber gets a copy of every message. |
| **spawn_blocking** | `tokio::task::spawn_blocking`. Offloads CPU-heavy work to a dedicated thread pool, preventing async runtime stalls. Used for Argon2. |
| **QueryBuilder** | `sqlx::QueryBuilder`. Constructs dynamic SQL queries with parameterized binds at runtime. |
| **FromRow** | An sqlx derive macro. Maps database rows to Rust structs by column name. |
| **FromRequestParts** | An Axum trait. Allows custom types to be extracted from HTTP request headers/metadata. `AuthUser` implements this. |

## Abbreviations

| Abbreviation | Meaning |
|-------------|---------|
| **RBAC** | Role-Based Access Control. Implemented with `user` and `admin` roles. |
| **CORS** | Cross-Origin Resource Sharing |
| **CRUD** | Create, Read, Update, Delete |
| **PgPool** | PostgreSQL connection pool (`sqlx::PgPool`) |
| **TDD** | Test-Driven Development |
