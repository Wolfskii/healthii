#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [ ! -f .env ]; then
  cp .env.example .env
fi
docker compose -f docker-compose.dev.yml up -d postgres minio
echo "Postgres and MinIO are starting. Run the backend with: cargo run -p healthii-backend"
