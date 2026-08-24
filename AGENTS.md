# Agent instructions

Healthii is a privacy-first personal health platform. Future coding agents must treat health data as extremely sensitive.

## What this repository is

A monorepo with applications at the repository root (not under `apps/`):

- `desktop/` — Tauri + React desktop workspace
- `web/` — React + Vite web application
- `mobile/` — Expo / React Native companion
- `backend/` — Rust Axum API
- `packages/` — shared TypeScript contracts, tokens, API client, dashboard view-model, React UI

## Before changing code

1. Read this file.
2. Read the relevant document in the repository root or `docs/`.
3. Read `TASKS.md` and implement the smallest coherent slice.
4. Inspect existing modules. Do not duplicate an implementation that already exists.
5. Add or update tests, run format/lint/tests, update docs and `TASKS.md`, then summarize.

## Architecture

```
UI  →  API client  →  backend/domain  →  PostgreSQL / object storage
```

- Do not put database logic in React components.
- Do not duplicate health calculations across desktop, web and mobile.
- The Rust backend is the source of truth for business rules.
- Desktop may cache JSON locally (`desktop/src-tauri/src/cache.rs`) for a future sync engine. Do not invent a sync protocol in a drive-by change.

## API

- Versioned at `/api/v1/`.
- JSON errors: `{ "error": { "code": "...", "message": "..." } }`.
- OpenAPI is generated with utoipa (`/docs` when `OPENAPI_ENABLED=true`).
- Authentication: Argon2 passwords, JWT sessions, `Authorization: Bearer` or `healthii_session` cookie.
- Never trust a client-provided `user_id`. Scope every query to the authenticated user.

## Health data rules

- Never expose another user's health data.
- Never log passwords, tokens, medical documents, blood test values or private notes.
- Never commit secrets. Configuration is environment-only (see `.env.example`).
- Never silently change schemas; add a SQLx migration.
- Never add a dependency without considering maintenance and security.
- Always add tests for security-sensitive behavior.
- Always update documentation when architecture changes.
- Prefer small, focused modules.
- Keep API contracts backwards-compatible where possible.
- Healthii is a tracking tool, not a clinician. Do not present uncertain conclusions as diagnosis.
- Distinguish recorded data, laboratory reference ranges, user-defined info and app-generated insights.

## Commands

From the repository root:

```bash
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d postgres minio
cargo run -p healthii-backend
cargo test -p healthii-backend
cargo fmt --all
cargo clippy -p healthii-backend --all-targets -- -D warnings
npm install
npm run dev:web
npm test
```

Migrations live in `backend/migrations/` and run on backend startup.

## Tasks

Track work in `TASKS.md`. Check boxes only for work that is actually done.
