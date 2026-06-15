# Testing Guide

## Test framework

- **Runner:** `cargo test` (built-in Rust test harness)
- **Async runtime:** `#[tokio::test]` for async integration tests
- **HTTP client:** `reqwest` 0.12 (integration tests)
- **No mocking framework** — integration tests use a real PostgreSQL instance

## Test structure

```
src/domain/grid.rs          # 18 unit tests (inline #[cfg(test)] module)
tests/
  common/mod.rs             # Shared TestApp helper (register, login, register_admin)
  register_login.rs         # 6 integration tests
  game_next.rs              # 7 integration tests
  history.rs                # 5 integration tests
  events.rs                 # 3 integration tests
  rbac.rs                   # 7 integration tests
```

**Total: 46 tests** (18 unit + 28 integration)

## Test commands

```bash
# All tests (requires PostgreSQL)
cargo test

# Unit tests only (no database needed)
cargo test --lib

# Specific game logic tests
cargo test --lib domain::grid::tests

# Specific integration test file
cargo test --test register_login
cargo test --test game_next
cargo test --test history
cargo test --test events

cargo test --test rbac

# Single test by name
cargo test --test game_next next_state_returns_blinker_oscillation
```

## Test inventory

### Unit tests (`src/domain/grid.rs`)

| Test name | What it verifies |
|-----------|-----------------|
| `blinker_oscillates` | Vertical↔horizontal blinker oscillation |
| `block_is_still_life` | 2×2 block remains unchanged |
| `empty_grid_stays_empty` | All-dead grid stays dead |
| `dead_cell_with_three_neighbors_is_born` | Birth rule |
| `alive_cell_with_two_neighbors_survives` | Survival with 2 |
| `alive_cell_with_three_neighbors_survives` | Survival with 3 |
| `alive_cell_with_zero_neighbors_dies` | Death by loneliness |
| `alive_cell_with_four_neighbors_dies` | Death by overpopulation |
| `full_grid_applies_overpopulation` | 3×3 all-alive → corners survive |
| `glider_moves_diagonally_after_four_generations` | Glider pattern across 4 gens |
| `next_state_is_deterministic` | Same input → same output |
| `rejects_empty_grid` | Empty vec rejected |
| `rejects_grid_smaller_than_minimum` | 2×2 rejected |
| `rejects_grid_larger_than_maximum` | 1001×1001 rejected |
| `rejects_non_square_grid` | Non-square rows rejected |
| `rejects_invalid_cell_values` | Value > 1 rejected |
| `accepts_minimum_size_grid` | 3×3 accepted |
| `accepts_maximum_size_grid` | 1000×1000 accepted |

### Integration tests (`tests/register_login.rs`)

| Test | Behavior |
|------|----------|
| `register_creates_user_and_returns_id` | 201 with id + username |
| `register_rejects_duplicate_username` | 409 Conflict |
| `register_rejects_short_password` | 400 Bad Request |
| `login_returns_jwt_token` | 200 with non-empty token |
| `login_rejects_wrong_password` | 401 Unauthorized |
| `login_rejects_nonexistent_user` | 401 Unauthorized |

### Integration tests (`tests/game_next.rs`)

| Test | Behavior |
|------|----------|
| `next_state_returns_blinker_oscillation` | Correct game output via API |
| `next_state_accepts_boolean_grid` | true/false accepted as 1/0 |
| `next_state_requires_auth` | 401 without token |
| `next_state_rejects_grid_too_small` | 400 for 2×2 |
| `next_state_rejects_non_square_grid` | 400 for non-square |
| `next_state_rejects_invalid_cell_values` | 400 for value=5 |
| `next_state_persists_request_to_database` | Row count in DB increases |

### Integration tests (`tests/history.rs`)

| Test | Behavior |
|------|----------|
| `history_returns_past_requests` | Returns submitted grids |
| `history_filters_by_grid_size` | Size filter works |
| `history_filters_by_input_state` | Input state filter works |
| `history_requires_auth` | 401 without token |
| `history_paginates_results` | per_page limits results |

### Integration tests (`tests/events.rs`)

| Test | Behavior |
|------|----------|
| `events_endpoint_requires_auth` | 401 without token |
| `events_endpoint_returns_event_stream` | 200 with text/event-stream |
| `game_request_broadcasts_event_to_subscribers` | Broadcast channel receives event |

### Integration tests (`tests/rbac.rs`)

| Test | Behavior |
|------|----------|
| `register_returns_user_role` | Registration response includes `role: "user"` |
| `user_sees_own_history` | User sees own history without `user_id` param |
| `user_cannot_see_other_users_history` | `user_id` param ignored for regular users |
| `admin_can_query_any_users_history` | Admin can filter by another user's ID |
| `admin_can_see_all_history` | Admin without filter sees all history |
| `login_token_contains_role_claim` | JWT contains `role: "user"` |
| `admin_token_contains_admin_role_claim` | Admin JWT contains `role: "admin"` |

## What to run after specific changes

| Change | Run |
|--------|-----|
| Game rules | `cargo test --lib domain::grid::tests` |
| Auth logic | `cargo test --test register_login` |
| Auth middleware | `cargo test` (all protected endpoints) |
| RBAC / roles | `cargo test --test rbac`, then `cargo test` |
| Game endpoint | `cargo test --test game_next` |
| History endpoint | `cargo test --test history` |
| SSE/events | `cargo test --test events` |
| Database schema | `cargo test` (all integration tests) |
| Any service | `cargo test` |

## Missing test coverage

- No test for expired JWT tokens
- No test for malformed JWT tokens (wrong secret)
- No negative test for grid_size > 1000 via the API (tested in unit tests only)
- No load/performance tests
- No test for SSE keep-alive behavior
- No test for concurrent game submissions
- History time-range filters (`from`/`to`) not tested via API
