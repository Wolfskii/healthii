# syntax=docker/dockerfile:1

FROM rust:bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY backend ./backend

RUN cargo build --release -p healthii-backend

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 healthii

WORKDIR /app
COPY --from=builder /app/target/release/healthii-backend /usr/local/bin/healthii-backend
COPY --from=builder /app/backend/migrations ./migrations

USER healthii
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/health >/dev/null || exit 1

ENTRYPOINT ["healthii-backend"]
