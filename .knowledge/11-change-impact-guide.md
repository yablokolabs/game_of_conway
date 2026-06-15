# Change Impact Guide

## How to: Change authentication

**Files to inspect:** `src/auth/mod.rs`, `src/services/auth_service.rs`
**Files to change:** `src/auth/mod.rs` (token/password logic), `src/services/auth_service.rs` (orchestration)
**Tests to run:** `cargo test --test register_login`, then `cargo test` (all protected endpoints depend on auth)
**Blast radius:** High — every protected endpoint uses `AuthUser` extractor

## How to: Add a new role (e.g., moderator)

RBAC is already implemented with `user` and `admin` roles.

**To add a new role (e.g., `moderator`):**
1. `src/models.rs` — add `ROLE_MODERATOR` constant
2. `src/auth/mod.rs` — add helper method (e.g., `is_moderator()`) or create a `ModeratorUser` extractor
3. `src/handlers/` — apply the new extractor or role check to relevant handlers

**No migration needed** — the `role` column is VARCHAR(20) and accepts any string value.

**Tests to run:** All — `cargo test`
**Blast radius:** Low — additive change, existing roles unaffected

## How to: Add a new protected route

**Files to change:**
1. `src/handlers/mod.rs` — add `.route()` line
2. `src/handlers/<new>.rs` — create handler function with `AuthUser` parameter
3. `src/services/<new>.rs` — create service function if needed
4. `src/services/mod.rs` — add `pub mod` declaration

**Tests to run:** New test file in `tests/`
**Blast radius:** Low — additive change, nothing breaks

## How to: Change a database model

**Files to change:**
1. `migrations/` — new SQL migration file
2. `src/models.rs` — update struct fields
3. `src/repositories/<repo>.rs` — update SQL queries and column lists

**Tests to run:** `cargo test` (integration tests will catch mismatches)
**Blast radius:** Medium — queries break if columns don't match struct

## How to: Add a new service

**Files to create:**
1. `src/services/<name>.rs` — service functions
2. Update `src/services/mod.rs` — add `pub mod <name>`

**Files to change:**
1. `src/handlers/<name>.rs` — handler that calls the service
2. `src/handlers/mod.rs` — register route

**Tests to run:** New integration test in `tests/`
**Blast radius:** Low — isolated addition

## How to: Add an external integration

**Files to change:**
1. `Cargo.toml` — add dependency
2. `src/services/<name>.rs` — integration logic
3. `src/config.rs` — add config fields (API keys, URLs)
4. `.env.example` — document new env vars

**Tests to run:** Unit tests for the integration, full integration suite
**Blast radius:** Low if isolated in a new service

## How to: Modify config/env

**Files to change:**
1. `src/config.rs` — add/modify fields in `Config`
2. `.env.example` — document the variable
3. `src/main.rs` — use the new config field (if startup-related)
4. `src/lib.rs` — add to `AppState` (if needed at runtime)

**Tests to run:** `cargo test` (tests use their own env setup)
**Blast radius:** Low — but missing env vars cause startup failure

## How to: Change the game rules

**Files to change:**
1. `src/domain/grid.rs` — modify `next_state()` or `count_neighbors()`

**Tests to run:** `cargo test --lib domain::grid::tests`
**Blast radius:** Low — pure function, no side effects. But all game outputs change.
