# Admin Auth & RBAC

## ✅ RBAC is implemented

The project uses a **two-role model**: `user` (default) and `admin`.

### Roles

| Role | Value | Description |
|------|-------|-------------|
| User | `"user"` | Default role for all registered users. Can only access own resources. |
| Admin | `"admin"` | Elevated role. Can access any user's resources. Created via bootstrap or direct DB update. |

Role constants are defined in `src/models.rs` as `ROLE_USER` and `ROLE_ADMIN`.

### Authorization rules

| Route | User | Admin |
|-------|------|-------|
| `POST /api/auth/register` | ✅ Public | ✅ Public |
| `POST /api/auth/login` | ✅ Public | ✅ Public |
| `POST /api/game/next` | ✅ Any authenticated | ✅ Any authenticated |
| `GET /api/history` | ✅ Own history only | ✅ Any user's history |
| `GET /api/events` | ✅ Any authenticated | ✅ Any authenticated |

### Key behavior: history endpoint scoping

- **Regular users:** The `user_id` query parameter is ignored. Results are
  always filtered to `auth.user_id` (the caller's own history).
- **Admins:** Can use `user_id` to query a specific user's history, or omit
  it to see all history across all users.

## Implementation files

| File | Role in RBAC |
|------|-------------|
| `migrations/003_add_user_role.sql` | Adds `role VARCHAR(20) NOT NULL DEFAULT 'user'` to `users` table |
| `src/models.rs` | `ROLE_USER` / `ROLE_ADMIN` constants, `role` field on `User` struct |
| `src/auth/mod.rs` | `role` in JWT `Claims`, `AuthUser.role` + `is_admin()`, `AdminUser` extractor |
| `src/error.rs` | `AppError::Forbidden` variant → 403 response |
| `src/repositories/user_repo.rs` | `role` in INSERT/SELECT queries |
| `src/services/auth_service.rs` | Passes role to `create_token()`, registers with `ROLE_USER` |
| `src/handlers/auth.rs` | Returns `role` in `RegisterResponse` |
| `src/handlers/history.rs` | Scopes history by `auth.user_id` for regular users |
| `src/config.rs` | Optional `ADMIN_USERNAME` / `ADMIN_PASSWORD` env vars |
| `src/main.rs` | `bootstrap_admin()` creates admin user on first startup |
| `tests/rbac.rs` | 7 RBAC integration tests |

## JWT claims

```json
{ "sub": "uuid", "role": "user", "exp": 1234567890 }
```

The `role` claim is embedded in the JWT at login time. Changing a user's role
in the database requires re-login to get a new token with the updated claim.

## Auth extractors

### `AuthUser`

Validates JWT and extracts `user_id` and `role`. Available on all protected
routes. Provides `is_admin()` helper.

```rust
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}
```

### `AdminUser`

Delegates to `AuthUser`, then checks `is_admin()`. Returns 403 Forbidden if
the caller is not an admin. Use this extractor on admin-only route handlers.

```rust
pub struct AdminUser {
    pub user_id: Uuid,
}
```

## Admin bootstrap

Set `ADMIN_USERNAME` and `ADMIN_PASSWORD` env vars. On startup, if both are
present and no user with that username exists, an admin user is created
automatically. If the user already exists, bootstrap is skipped.

## How to create additional admins

1. **Via bootstrap:** Set env vars and restart the server
2. **Via direct SQL:** `UPDATE users SET role = 'admin' WHERE username = '...'`

Registration always creates users with the `user` role. There is no API
endpoint for promoting users — this is intentional for security.

## Tests

RBAC behavior is covered in `tests/rbac.rs`:

| Test | What it verifies |
|------|-----------------|
| `register_returns_user_role` | Registration response includes `role: "user"` |
| `user_sees_own_history` | User can query own history without `user_id` param |
| `user_cannot_see_other_users_history` | User's `user_id` param is ignored, sees only own data |
| `admin_can_query_any_users_history` | Admin can filter by another user's `user_id` |
| `admin_can_see_all_history` | Admin without `user_id` filter sees all history |
| `login_token_contains_role_claim` | JWT payload includes `role: "user"` |
| `admin_token_contains_admin_role_claim` | Admin JWT payload includes `role: "admin"` |

## Migration note

Adding the `role` column with `DEFAULT 'user'` is non-breaking — existing
rows are backfilled. However, the JWT `Claims` struct now requires a `role`
field, which **invalidates all existing tokens**. Users must re-login after
this migration.
