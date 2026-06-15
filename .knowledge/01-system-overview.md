# System Overview

## What the project does

A REST API server that computes Conway's Game of Life next-state transitions.
Users authenticate, submit grids, receive the next generation, and can query
history or watch a real-time event stream.

## Tech stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (edition 2021, rustc 1.94+) |
| Web framework | Axum 0.8 |
| Async runtime | Tokio (full features) |
| Database | PostgreSQL 16+ |
| DB driver | SQLx 0.8 (compile-time migration embedding) |
| Password hashing | Argon2id (via `argon2` crate 0.5) |
| Auth tokens | JWT (via `jsonwebtoken` crate 9) |
| Streaming | Server-Sent Events via `tokio::sync::broadcast` |
| Serialization | serde / serde_json |
| Config | `dotenvy` (.env file) |
| Observability | `tracing` + `tracing-subscriber` |
| HTTP middleware | `tower-http` (CORS, tracing) |

## High-level request flow

```mermaid
sequenceDiagram
    participant C as Client
    participant R as Axum Router
    participant MW as AuthUser Extractor
    participant H as Handler
    participant S as Service
    participant RP as Repository
    participant DB as PostgreSQL
    participant BC as Broadcast Channel

    C->>R: HTTP request
    R->>MW: Extract JWT from Authorization header
    MW->>MW: Validate token, extract user_id
    MW->>H: AuthUser { user_id }
    H->>S: Business logic call
    S->>RP: SQL query/insert
    RP->>DB: Execute
    DB-->>RP: Result
    RP-->>S: Domain object
    S->>BC: Broadcast event (game requests only)
    S-->>H: Result
    H-->>C: JSON response
```

## Architecture diagram

```mermaid
graph TD
    Client -->|HTTP| Router[Axum Router]
    Router --> Public[Public Routes]
    Router --> Protected[Protected Routes]

    Public --> Register[POST /api/auth/register]
    Public --> Login[POST /api/auth/login]

    Protected --> AuthMW[AuthUser JWT Extractor]
    AuthMW --> GameH[POST /api/game/next]
    AuthMW --> HistoryH[GET /api/history]
    AuthMW --> EventsH[GET /api/events]

    GameH --> GameSvc[game_service]
    GameH --> GameEngine[Grid::next_state — pure]
    GameSvc --> GridRepo[grid_repo]
    GameSvc --> Broadcast[tokio::broadcast]

    Register --> AuthSvc[auth_service]
    Login --> AuthSvc
    AuthSvc --> UserRepo[user_repo]

    HistoryH --> HistSvc[history_service]
    HistSvc --> GridRepo

    EventsH --> Broadcast

    GridRepo --> DB[(PostgreSQL)]
    UserRepo --> DB
```

## Layered architecture

```
handlers → services → repositories → database
                ↘ domain (pure game logic, no I/O)
```

Each layer has a single responsibility:
- **Handlers:** Parse HTTP, validate request shape, return responses
- **Services:** Orchestrate business logic, coordinate multiple repos
- **Repositories:** Own SQL queries, return domain objects
- **Domain:** Pure functions with zero I/O (game engine)
