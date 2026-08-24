$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
if (-not (Test-Path ".env")) {
  Copy-Item ".env.example" ".env"
}
docker compose -f docker-compose.dev.yml up -d postgres minio
Write-Host "Postgres and MinIO are starting. Run the backend with: cargo run -p healthii-backend"
