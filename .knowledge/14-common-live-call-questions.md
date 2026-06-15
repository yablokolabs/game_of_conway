# Common Live Call Questions

## How is the app structured?

Layered Rust architecture: **handlers → services → repositories → database**.
The game engine is pure domain logic in `src/domain/grid.rs` with zero I/O.
Axum 0.8 handles HTTP. PostgreSQL stores users and grid history. JWT for auth.

## Where does request handling start?

In `src/main.rs`. The Axum router is built in `src/handlers/mod.rs` with 5
routes. Each request enters the router, passes through middleware (CORS,
tracing), hits the route handler, and flows down through the service and
repository layers.

## How does auth work?

1. User registers with username/password at `POST /api/auth/register`
2. Password is hashed with **Argon2id** and stored in PostgreSQL
3. User logs in at `POST /api/auth/login` and receives a **JWT** (24h expiry)
4. All protected endpoints require `Authorization: Bearer <token>` header
5. The `AuthUser` extractor in `src/auth/mod.rs` validates the JWT and
   extracts `user_id` on every protected request
6. If the token is missing/invalid/expired, the request gets a 401 before
   the handler body runs

## How does admin auth work?

Two-role model: `user` (default) and `admin`. The `role` is stored in the
`users` table and embedded in the JWT claims.

- `AuthUser` extractor: validates JWT, provides `user_id` and `role`
- `AdminUser` extractor: delegates to `AuthUser`, returns 403 if not admin
- History endpoint: regular users see only their own history; admins can
  query any user's history or see all history
- Registration always creates `user` role; admins are created via env var
  bootstrap (`ADMIN_USERNAME`/`ADMIN_PASSWORD`) or direct SQL

See `05-admin-auth-and-rbac.md` for full details.

## How would you add a new role (e.g., moderator)?

Since RBAC is already implemented:
1. Add a `ROLE_MODERATOR` constant in `src/models.rs`
2. Add a helper method or new extractor in `src/auth/mod.rs`
3. Apply the role check in the relevant handlers

No migration needed — `role` is VARCHAR(20).

## What is the blast radius of changing auth middleware?

**High.** The `AuthUser` extractor is used by 3 handlers:
- `handlers::game::next_state`
- `handlers::history::query`
- `handlers::events::stream`

Changing the extractor signature, JWT claims, or validation logic affects all
protected endpoints. Run `cargo test` (full suite) after any auth change.

## How do you add a new protected API endpoint?

1. Create handler function in `src/handlers/<name>.rs` with `AuthUser` param
2. Add service in `src/services/<name>.rs` if business logic needed
3. Add route in `src/handlers/mod.rs`: `.route("/api/<path>", get/post(handler))`
4. Write integration test in `tests/<name>.rs`

## How do you debug a failing auth issue?

1. Check if the JWT is present: `Authorization: Bearer <token>`
2. Decode the JWT manually (jwt.io) — check `sub`, `role`, and `exp` fields
3. Verify the `JWT_SECRET` env var matches between login and validation
4. Check `src/auth/mod.rs::validate_token()` — it returns the specific error
5. Run `cargo test --test register_login` to verify the auth flow works

## What tests would you run after changing RBAC?

- `cargo test --test rbac` (role-based access tests)
- `cargo test --test register_login` (role in registration/login)
- `cargo test` (all protected endpoints still work)

## Where is database access implemented?

`src/repositories/user_repo.rs` and `src/repositories/grid_repo.rs`.
Both use `sqlx::query_as::<_, T>()` with parameterized SQL. `grid_repo` uses
`sqlx::QueryBuilder` for dynamic filtered queries.

## What are the riskiest parts of the codebase?

1. **`src/auth/mod.rs`** — security-critical. Bugs = unauthorized access.
2. **`src/repositories/grid_repo.rs`** — dynamic SQL query builder. Bugs = wrong
   query results (though SQL injection is prevented by parameterized binds).
3. **`src/domain/grid.rs`** — core product logic. Wrong rules = wrong outputs.
4. **`src/services/auth_service.rs`** — password verification happens here.
   Timing attacks are mitigated by Argon2 but the error messages must stay
   generic ("invalid credentials" for both wrong user and wrong password).

## How does the SSE spectator stream work?

1. `game_service::compute_and_store()` sends a `GameEvent` to a
   `tokio::sync::broadcast` channel after every game computation
2. `handlers::events::stream()` subscribes to the broadcast channel
3. Events are serialized to JSON and sent as SSE `data:` frames
4. A 15-second keep-alive heartbeat prevents connection timeouts
5. If a subscriber is too slow, it misses events (broadcast channel backpressure)

## How are grids stored?

As JSONB in the `grid_requests` table. Both `input_grid` and `output_grid`
are stored as 2D arrays of integers. The `grid_size` column allows efficient
filtering without parsing the JSONB. A hash index on `input_grid` supports
exact-match filtering by input state.
