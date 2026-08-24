# Backend architecture

Crate `healthii-backend` (`backend/src`). `main.rs` loads config, connects, migrates, serves.

Modules: `config`, `state`, `error`, `telemetry`, `db`, `storage`, `api`, `auth`, `users`, `health`, `dashboard`, plus domain folders for later routes.

Tracing: pretty logs locally, JSON when `RUST_LOG_FORMAT=json`. Request ids via `x-request-id`.
