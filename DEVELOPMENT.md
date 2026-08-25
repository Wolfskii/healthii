# Development

## Prerequisites

- Rust (stable, 1.80+)
- Node.js 20+
- Docker
- Optional: [Task](https://taskfile.dev) (`Taskfile.yml`), Tauri system deps, Expo / Xcode / Android Studio for device builds

## First run

```bash
cp .env.example .env          # PowerShell: Copy-Item .env.example .env
npm install
docker compose -f docker-compose.dev.yml up -d postgres minio
cargo run -p healthii-backend
npm run dev:web
```

API: `http://localhost:8080/health`  
OpenAPI: `http://localhost:8080/docs`  
Web: `http://localhost:5173`  
Postgres (dev compose): `localhost:5433` (container still listens on 5432)  
Desktop Vite: `http://localhost:1420` via `npm run tauri:dev -w @healthii/desktop`  
Mobile: `npm run dev:mobile`

## Environment

All backend variables are listed in `.env.example`. Never commit `.env`.

Vite apps read `VITE_API_BASE_URL` (default `http://localhost:8080`).
Expo reads `EXPO_PUBLIC_API_BASE_URL` (Android emulator default `http://10.0.2.2:8080`).

## Tests

```bash
cargo test -p healthii-backend
npm test
```

Database-backed API tests need Postgres and `DATABASE_URL`.

## Formatting and lint

```bash
cargo fmt --all
cargo clippy -p healthii-backend --all-targets -- -D warnings
npm run typecheck
npm run format
```

## Desktop notes

Tauri bundles native code in `desktop/src-tauri`. That crate is **not** a workspace member of the root `Cargo.toml` so Docker backend builds stay small. Format/check it with:

```bash
cargo fmt --manifest-path desktop/src-tauri/Cargo.toml
cargo check --manifest-path desktop/src-tauri/Cargo.toml
```

macOS and iOS builds require those operating systems. Windows CI should not be expected to produce signed Apple artifacts.

## Platform limits

- This Windows development host can run the API, web build, desktop frontend build, `cargo check` for the Tauri crate, Expo typecheck, and Docker.
- Full Tauri installers, macOS `.app` bundles, iOS simulators and signed Apple/Play Store artifacts require the matching OS and certificates.
- HealthKit / Health Connect need Expo **development builds**, not Expo Go.


Expo Go is enough for the Phase 1 shell. HealthKit and Health Connect require a **development build**, privacy policy URLs, and store review strings (already sketched in `mobile/app.json`).
