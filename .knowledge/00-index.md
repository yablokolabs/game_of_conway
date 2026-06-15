# Cluely Knowledge Base — Conway's Game of Life Backend

This pack documents the architecture, auth flow, routes, services, models,
and testing of the `game_of_conway` Rust backend. Generated from the codebase
at commit `e0ca842` on 2026-06-10.

## Files in this pack

| # | File | Focus |
|---|------|-------|
| 01 | [01-system-overview.md](01-system-overview.md) | What the project does, tech stack, request flow |
| 02 | [02-repository-map.md](02-repository-map.md) | Folder structure, who owns what |
| 03 | [03-runtime-and-entrypoints.md](03-runtime-and-entrypoints.md) | Server startup, Docker, CLI, local dev |
| 04 | [04-authentication-flow.md](04-authentication-flow.md) | JWT login flow, middleware, identity extraction |
| 05 | [05-admin-auth-and-rbac.md](05-admin-auth-and-rbac.md) | Admin/RBAC — two-role model (user/admin) |
| 06 | [06-routes-controllers-and-api.md](06-routes-controllers-and-api.md) | All endpoints, auth requirements, handler map |
| 07 | [07-services-business-logic.md](07-services-business-logic.md) | Service layer, side effects, dependencies |
| 08 | [08-database-models-and-storage.md](08-database-models-and-storage.md) | Tables, migrations, ORM, query patterns |
| 09 | [09-config-env-and-secrets.md](09-config-env-and-secrets.md) | Env vars, config loading, secrets |
| 10 | [10-dependency-map.md](10-dependency-map.md) | Internal module graph, external crates |
| 11 | [11-change-impact-guide.md](11-change-impact-guide.md) | How-to for common changes with blast radius |
| 12 | [12-critical-files.md](12-critical-files.md) | Top 20 most important files with context |
| 13 | [13-testing-guide.md](13-testing-guide.md) | Test framework, structure, what to run when |
| 14 | [14-common-live-call-questions.md](14-common-live-call-questions.md) | Q&A cheat sheet for meetings |
| 15 | [15-glossary.md](15-glossary.md) | Project-specific terms and concepts |

## Best questions to ask during a call

- "How does the game engine work and where is it?"
- "How does user authentication flow through a request?"
- "Is there admin auth or RBAC?" (Yes — two-role model: user/admin)
- "What happens when a grid is submitted?"
- "How is request history stored and queried?"
- "How does the SSE spectator stream work?"
- "What would break if you changed the auth middleware?"
- "How would you add a new role or permission?"
- "What tests cover the auth flow?"
- "How do you run the project locally?"

## Key facts

- **Language:** Rust (edition 2021)
- **Framework:** Axum 0.8
- **Database:** PostgreSQL via SQLx
- **Auth:** Argon2id + JWT bearer tokens
- **Admin/RBAC:** Two-role model (`user`/`admin`). Users see own history; admins see all.
- **Tests:** 38 total (18 unit, 20 integration)
- **LOC:** ~1,600 lines of Rust
