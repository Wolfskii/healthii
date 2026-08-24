# Deployment

Healthii's API is deployed as a Docker image. Clients are static/Tauri/Expo artifacts.

## Required environment variables

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | At least 32 characters |
| `SESSION_SECRET` | Cookie/session material |
| `HOST` / `PORT` | Bind address (default `0.0.0.0:8080`) |
| `CORS_ORIGINS` | Comma-separated browser origins |
| `STORAGE_PROVIDER` | `local` or `s3` |
| `STORAGE_PATH` | Local disk root |
| `S3_ENDPOINT` `S3_BUCKET` `S3_REGION` `S3_ACCESS_KEY` `S3_SECRET_KEY` | Object storage |
| `RUST_LOG` | Tracing filter |
| `OPENAPI_ENABLED` | Keep `false` in production |

See `.env.example`.

## Database

Run Postgres 16+. The backend applies SQLx migrations on startup. Take logical backups (`pg_dump`) before upgrades.

## Storage

Use S3-compatible storage in production (or a dedicated encrypted volume for `local`). Bucket policies must deny public listing. Document download URLs must be authorized in application code (not yet exposed).

## Reverse proxy and HTTPS

Terminate TLS at a reverse proxy (Caddy, nginx, Traefik). Forward `X-Request-Id` and `X-Forwarded-For`. Enable HSTS. The API itself does not serve certificates.

## Docker

```bash
docker build -f docker/backend.Dockerfile -t ghcr.io/<org>/healthii-backend:0.1.0 .
docker compose up -d
```

`docker-compose.yml` expects `JWT_SECRET` in the environment. Do not publish Postgres ports on the public internet.

## Migrations

Applied automatically at process start. For a dedicated step:

```bash
cargo sqlx migrate run --source backend/migrations --database-url "$DATABASE_URL"
```

## Backups

- PostgreSQL: nightly `pg_dump` plus WAL archiving when available
- Object storage: bucket versioning
- Secrets: never back up `.env` into git

## Rollback

1. Deploy the previous image tag.
2. Do not reverse a migration unless a down-migration was planned. Prefer forward fixes.
3. Restore DB only if a migration corrupted data (tested restore procedure).

## Health checks

- Liveness: `GET /health`
- Readiness: `GET /ready` (Postgres)
- Compose and Kubernetes should use these endpoints

## Release workflow

GitHub Actions `release.yml` builds, tests, tags and can push to GHCR using repository secrets (`GHCR_TOKEN` or `GITHUB_TOKEN`). Wire the actual production host when one exists.
