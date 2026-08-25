# Roadmap

## Phase 1 — Foundation

Repository, Rust API, PostgreSQL, Docker, authentication, React desktop/web shells, Expo mobile shell, CI, documentation.

## Phase 2 — Core health data

Generic measurements, weight tracking, dashboard numbers, timeline.

## Phase 3 — Medical data

Lab tests, biomarkers, reference ranges, documents and authorized file access.

## Phase 4 — Lifestyle

Workouts, medications, symptoms, appointments.

## Phase 5 — Charts and export

Charts, trends, JSON/CSV export, print/PDF, search, live clients, JSON/CSV import.

## Phase 6 — Advanced (current)

Import from Apple Health / Health Connect, local-first sync, Withings OAuth on the backend. CD publishes the API image to GHCR; `docker-compose.prod.yml` is the self-hosted production stack.

## Localization

English first. Keep user-facing strings out of domain logic so Swedish, Polish and German can be added later.

## Units

User preference for metric/imperial. Preserve original values. Convert laboratory units only with explicit, scientifically valid mappings.
