# Conway's Game of Life — User Manual

## Overview

This document summarizes the setup, architecture, and testing walkthrough for the Conway's Game of Life backend project.

---

## 1. Debugging with `rust-lldb`

- **Added** a debugging section to `README.md` covering `rust-lldb` usage.
- `rust-lldb` ships with the Rust toolchain — no `Cargo.toml` dependency needed.
- Covers breakpoints, stepping, variable inspection, backtraces, and test debugging.
- **Committed** as `ac8a3f9` with no Copilot co-authored-by trailer (per user request).

---

## 2. Repository Published

- Pushed to GitHub as a **public repo**: [yablokolabs/game_of_conway](https://github.com/yablokolabs/game_of_conway)
- Created via `gh repo create --public --source=. --push`

---

## 3. Running the Project

### Prerequisites

- Rust 1.75+
- Docker

### Steps

```bash
# 1. Start PostgreSQL
docker-compose up -d

# 2. Create .env
cp .env.example .env

# 3. Run the server
cargo run
# Server starts on http://localhost:3000
```

---

## 4. Testing via Postman

### API Flow

| Step | Endpoint | Body / Notes |
|------|----------|-------------|
| 1 | `POST /api/auth/register` | `{"username":"alice","password":"securepassword"}` |
| 2 | `POST /api/auth/login` | Same body → copy `token` from response |
| 3 | Set header | `Authorization: Bearer <token>` |
| 4 | `POST /api/game/next` | `{"cells":[[0,1,0],[0,1,0],[0,1,0]]}` |
| 5 | `GET /api/history` | View stored game requests |
| 6 | `GET /api/events` | SSE stream (real-time spectator view) |

Import `postman_collection.json` from the repo root.

---

## 5. SQL Verification

### Connect to the database

```bash
docker exec -it game_of_conway-db-1 psql -U conway
```

### Useful queries

```sql
-- List all tables
\dt

-- Verify users
SELECT id, username, created_at FROM users;

-- Verify stored game requests
SELECT id, user_id, grid_size, created_at FROM grid_requests ORDER BY created_at DESC;

-- See full input/output grids
SELECT input_grid, output_grid FROM grid_requests ORDER BY created_at DESC LIMIT 1;

-- Filter by user
SELECT gr.grid_size, gr.created_at
FROM grid_requests gr
JOIN users u ON gr.user_id = u.id
WHERE u.username = 'alice';

-- Count requests per user
SELECT u.username, COUNT(*) as total_requests
FROM grid_requests gr
JOIN users u ON gr.user_id = u.id
GROUP BY u.username;
```

---

## 6. Game of Life Rules (Verified Against PDF Spec)

| Rule | Description |
|------|-------------|
| **Birth** | Dead cell + exactly 3 live neighbors → alive |
| **Survival** | Live cell + 2 or 3 live neighbors → stays alive |
| **Death** | All other cases → dies or stays dead |
| **Boundary** | Cells outside the grid are treated as dead (no wrapping) |

✅ Rules match the coding challenge PDF exactly.

---

## 7. Database Schema

### 2 tables (+ `_sqlx_migrations`)

| Table | Key Columns |
|-------|-------------|
| **users** | `id`, `username`, `password_hash`, `role`, `created_at` |
| **grid_requests** | `id`, `user_id` (FK → users), `input_grid`, `output_grid`, `grid_size`, `created_at` |

---

## 8. Roles & Access Control

### 2 roles

| Role | Constant | Capabilities |
|------|----------|-------------|
| `user` | `ROLE_USER` | Own game requests, own history only |
| `admin` | `ROLE_ADMIN` | Everything above + query any user's history |

### Creating an admin

**Option A — via `.env` (auto-bootstrap on startup):**

```bash
ADMIN_USERNAME=admin
ADMIN_PASSWORD=change-me-in-production
```

**Option B — via SQL:**

```sql
UPDATE users SET role = 'admin' WHERE username = 'alice';
```

---

## 9. Grid Validation

| Input | Result |
|-------|--------|
| 2×2 | ❌ `InvalidSize(2)` |
| 3×3 | ✅ Minimum valid |
| 1000×1000 | ✅ Maximum valid |
| 1001×1001 | ❌ `InvalidSize(1001)` |
| Non-square (e.g. 3×4) | ❌ `NotSquare` |
| Cell value `2` | ❌ `InvalidCellValue` |

---

## 10. SSE Stream Architecture

```
POST /api/game/next
        │
        ▼
  game_service::compute_and_store()
        │
        ├── 1. Grid::new() → validate
        ├── 2. input.next_state() → compute
        ├── 3. grid_repo::save() → persist to DB
        └── 4. event_tx.send(GameEvent) ──► tokio::broadcast channel
                                                │
                          ┌─────────────────────┤
                          ▼                     ▼
                    GET /api/events       GET /api/events
                      (SSE stream)          (SSE stream)
```

- Uses `tokio::broadcast` — lock-free, multi-consumer fanout.
- Keep-alive every 15 seconds to prevent connection timeouts.
- Late subscribers only see events from after they connect.

### Testing SSE

```bash
# Terminal 1 — listen
curl -N -H "Authorization: Bearer <token>" http://localhost:3000/api/events

# Terminal 2 — trigger
curl -X POST http://localhost:3000/api/game/next \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"cells":[[0,1,0],[0,1,0],[0,1,0]]}'
```

### SSE vs WebSocket (Decision)

SSE was chosen because:
- Spec requires **read-only** spectator streaming — no bidirectional need.
- Simpler implementation (Axum built-in `Sse`, no extra crate).
- Auto-reconnect built into the protocol.
- Works through HTTP proxies without special config.

---

## 11. Next Steps / Additional Testing

### Edge cases

- Min/max grids (3×3, 100×100)
- Invalid inputs: 2×2 grid, non-square, cell value `2`, empty body
- Auth errors: expired/missing/malformed token, wrong password, duplicate username

### Concurrency

- Fire multiple `/api/game/next` simultaneously
- Verify SSE receives all events and DB has all rows

### History filters

- Combine `user_id`, `grid_size`, `input_state`, `from`, `to`, `page`, `per_page`
- `input_state` accepts a URL-encoded JSON 2D array (e.g. `[[0,1,0],[0,1,0],[0,1,0]]`)
- Boundary: `per_page=101` (clamps to 100), `page=0` (defaults to 1)

### Automated tests

```bash
# Unit tests (no DB needed)
cargo test --lib

# Full suite (with DB running)
cargo test
```
