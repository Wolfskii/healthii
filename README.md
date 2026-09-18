# Healthii

<p>
  <a href="https://www.buymeacoffee.com/wolfskii">
    <img src="https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20coffee&emoji=&slug=wolfskii&button_colour=FFDD00&font_colour=000000&font_family=Cookie&outline_colour=000000&coffee_colour=ffffff" alt="Buy me a coffee" />
  </a>
</p>

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

Health records, live dashboard, charts, notes, sleep, export/import (Apple Health export.xml and Health Connect / Fit CSV), search and authorized document download are implemented. Sign in on web, desktop or the Expo companion against a local API. Live HealthKit / Health Connect / Withings sync is still later work.

## License

MIT — see [LICENSE](LICENSE).
