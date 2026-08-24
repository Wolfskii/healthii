#!/usr/bin/env bash
set -euo pipefail
if [ "${1:-}" = "create" ]; then
  name="${2:?migration name required}"
  file="$(date +%Y%m%d%H%M%S)_${name}.sql"
  # Prefer sequential numbering already used in backend/migrations
  next=$(printf "%04d" $(( $(ls backend/migrations/*.sql 2>/dev/null | wc -l) + 1 )))
  file="${next}_${name}.sql"
  touch "backend/migrations/${file}"
  echo "Created backend/migrations/${file}"
  exit 0
fi

export DATABASE_URL="${DATABASE_URL:-postgres://healthii:healthii@localhost:5433/healthii}"
cargo sqlx migrate run --source backend/migrations --database-url "$DATABASE_URL"
