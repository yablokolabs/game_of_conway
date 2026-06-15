# Config, Environment & Secrets

## Config loading

**File:** `src/config.rs`

`Config::from_env()` loads environment variables via `dotenvy`. It is called
once in `main()` at startup. Missing required variables cause an immediate
error with a descriptive message.

## Environment variables

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `DATABASE_URL` | ✅ Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | ✅ Yes | — | HMAC secret for signing/validating JWTs |
| `HOST` | No | `0.0.0.0` | Server bind address |
| `PORT` | No | `3000` | Server bind port |
| `RUST_LOG` | No | `game_of_conway=debug,tower_http=debug` | Log level filter |
| `ADMIN_USERNAME` | No | — | Bootstrap admin user on first startup |
| `ADMIN_PASSWORD` | No | — | Password for the bootstrap admin user |

## Secret handling

- **JWT_SECRET:** Used in `auth::create_token()` and `auth::validate_token()`.
  Stored in `AppState.jwt_secret` (in memory only). Never logged or serialized.
- **DATABASE_URL:** Contains database password. Passed to `PgPool::connect()`.
  Never logged.
- **Password hashes:** Stored in `users.password_hash` column. Never returned
  in API responses (the `User` struct's `password_hash` field is not serialized).

## Config files

| File | Purpose | Committed |
|------|---------|-----------|
| `.env.example` | Template with variable names | ✅ Yes |
| `.env` | Actual runtime values | ❌ No (gitignored) |
| `docker-compose.yml` | PostgreSQL container config | ✅ Yes |

## Docker compose environment

The `docker-compose.yml` defines PostgreSQL credentials:
- `POSTGRES_USER: conway`
- `POSTGRES_PASSWORD: conway`
- `POSTGRES_DB: conway`

These are development-only defaults. Production should use different credentials.
