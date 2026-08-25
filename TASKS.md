# Tasks

## Commands

Install JavaScript dependencies:

```bash
npm install
```

Start PostgreSQL and MinIO (Windows, macOS, Linux):

```bash
docker compose -f docker-compose.dev.yml up -d postgres minio
```

Copy environment variables first:

```bash
cp .env.example .env
```

On Windows PowerShell, `Copy-Item .env.example .env`.

Start the development environment (database + API):

```bash
docker compose -f docker-compose.dev.yml up -d postgres minio
cargo run -p healthii-backend
```

Or run API + Postgres together:

```bash
docker compose -f docker-compose.dev.yml up --build
```

Start PostgreSQL only:

```bash
docker compose -f docker-compose.dev.yml up -d postgres
```

Run the backend (from repo root):

```bash
cargo run -p healthii-backend
```

Run the desktop app:

```bash
npm run tauri:dev -w @healthii/desktop
```

Run the web app:

```bash
npm run dev:web
```

Run the mobile companion:

```bash
npm run dev:mobile
```

Run all clients (separate terminals):

```bash
npm run dev:web
npm run tauri:dev -w @healthii/desktop
npm run dev:mobile
```

Run tests:

```bash
cargo test -p healthii-backend
npm test
```

Run backend tests:

```bash
cargo test -p healthii-backend
```

Authorization tests that need a database skip unless `DATABASE_URL` is set:

```bash
export DATABASE_URL=postgres://healthii:healthii@localhost:5433/healthii
cargo test -p healthii-backend --test authorization
```

On Windows (PowerShell):

```powershell
$env:DATABASE_URL="postgres://healthii:healthii@localhost:5433/healthii"
cargo test -p healthii-backend --test authorization
```

Run frontend tests:

```bash
npm run test:web
npm run test:ui
npm run test -w @healthii/desktop
```

Run linting:

```bash
cargo clippy -p healthii-backend --all-targets -- -D warnings
npm run typecheck
```

Run formatting:

```bash
cargo fmt --all
npm run format
```

Check formatting:

```bash
cargo fmt --all -- --check
npm run format:check
```

Run database migrations (also runs automatically on backend start):

```bash
cargo sqlx migrate run --source backend/migrations --database-url postgres://healthii:healthii@localhost:5433/healthii
```

Create a migration:

```bash
# next file: backend/migrations/0002_descriptive_name.sql
```

Prefer sequential prefixes: `0002_add_measurements.sql`.

Build the production Docker image:

```bash
docker build -f docker/backend.Dockerfile -t healthii-backend:local .
```

Start production compose (prebuilt image; requires secrets):

```bash
docker compose -f docker-compose.prod.yml up -d
```

Stop services:

```bash
docker compose -f docker-compose.dev.yml down
docker compose down
```

View logs:

```bash
docker compose -f docker-compose.dev.yml logs -f backend postgres
```

Clean the development environment:

```bash
docker compose -f docker-compose.dev.yml down -v
cargo clean
```

---

## Foundation

- [x] Repository setup
- [x] Backend
- [x] PostgreSQL
- [x] Authentication
- [x] Desktop shell
- [x] Web shell
- [x] Mobile shell

## Health Data

- [x] Measurements
- [x] Weight tracking
- [x] Blood pressure
- [x] Blood tests
- [x] Documents
- [x] Workouts
- [x] Medications
- [x] Symptoms
- [x] Appointments
- [x] Health notes
- [x] Sleep

## Dashboard

- [x] Health overview (shell)
- [x] Timeline (shell)
- [x] Charts
- [x] Trends

## Security

- [x] Authentication hardening (Argon2, JWT sessions, rate limit on login)
- [x] Authorization (session-scoped user, unauthenticated `/me` rejected)
- [x] File security
- [x] Audit logging (table in place; event writes expand with domain routes)
- [x] Security tests (unauthenticated access; revoked session; cross-user isolation)

## Deployment

- [x] Docker
- [x] CI
- [x] CD
- [x] Production environment

## Next coherent slice

1. Live HealthKit / Health Connect / Withings adapters (file import is in).
2. Local-first sync for the desktop sidecar.
