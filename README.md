# Healthii

Privacy-first personal health management. Your entire health history, organized in one private place.

Healthii is a **personal health operating system**, not a fitness tracker and not a clinic EHR. It helps you keep doctor reports, laboratory results, measurements, workouts, medications, symptoms and appointments together — for years.

> Healthii does not provide medical diagnosis or replace professional medical advice.

## Applications

| App | Stack | Role |
| --- | --- | --- |
| `desktop/` | Tauri 2 + React + Vite | Primary workspace |
| `web/` | React + Vite + React Router | Full experience in the browser |
| `mobile/` | React Native + Expo | Fast capture companion |
| `backend/` | Rust, Axum, SQLx, PostgreSQL | Source of truth for health data |

## Quick start

```bash
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d postgres minio
cargo run -p healthii-backend
npm install
npm run dev:web
```

See [DEVELOPMENT.md](DEVELOPMENT.md) and [TASKS.md](TASKS.md) for every command.

## Status

Phase 1 foundation: repository, API health/readiness, authentication, Docker, three client shells, design system, CI and documentation. Domain records (weight, labs, documents, …) are intentionally not fully implemented yet.

## License

MIT — see [LICENSE](LICENSE).
