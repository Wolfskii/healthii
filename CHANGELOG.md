# Changelog

## 0.6.0 — 2026-08-25

- Sleep as a measurement (hours); Apple Health time in bed and Health Connect sleep duration import
- Display name and timezone on the account (`PUT /me`); greeting and sidebar use the name
- Printable emergency summary on Reports (personal card — not a medical ID)
- Dashboard grouped into Body / Care / Records; Ctrl/Cmd+K opens Quick add
- Quick add for sleep and appointments; dialog keeps keyboard focus inside
- Dark mode: System / Light / Dark on web, desktop and the companion; follows the device until you pick one

## 0.5.0 — 2026-08-25

- Private health notes (timeline, search, export/import, dashboard, Quick add)
- Account activity in Settings (sign-in and import events; no health values)
- Backdate measurements; biomarker history on blood tests; appointment “in N days”
- CSV export now includes medications, symptoms, appointments and notes

## 0.4.0 — 2026-08-25

- Import Apple Health `export.xml` (weight, BP, resting HR, glucose, workouts; steps and intraday HR skipped)
- Import Health Connect / Fit vitals CSV through `/import/csv`
- Weight goal on the profile and dashboard; active medications on the dashboard
- Session list and revoke; account deletion with password confirmation

## 0.3.0 — 2026-08-25

- Import Healthii JSON and CSV into the signed-in account (foreign `user_id` values are ignored)
- Change password; other sessions are revoked
- GHCR image publish on `develop`, `main` and version tags
- Production Compose file (`docker-compose.prod.yml`)
- Empty-dashboard start links, insights 30/90/365 day range

## 0.2.0 — 2026-08-25

- User-scoped measurements, blood pressure, labs, documents, workouts, medications, symptoms and appointments
- Live dashboard, timeline, charts, JSON/CSV export and record search
- Authorized document download (Bearer session; `storage_key` never exposed)
- Web and desktop sign-in, quick add, and authenticated file download
- Expo companion sign-in and capture for vitals, symptoms, medications and workouts
- Isolation tests: another user cannot read measurements, labs or search hits

## 0.1.0 — 2026-08-24

- Initial repository foundation
- Axum backend with `/health`, `/ready`, authentication and dashboard shell
- PostgreSQL migrations for users, sessions and audit events
- Docker Compose for development and production-shaped deployment
- React web app, Tauri desktop app, Expo mobile companion
- Shared design tokens, typed API client and dashboard view-model
- GitHub Actions for CI, backend, web, desktop, Docker and release
