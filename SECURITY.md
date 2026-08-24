# Security

Health data is the product. Reliability and access control beat extra features.

## Authentication

- Passwords hashed with Argon2id
- JWT signed with `JWT_SECRET` (≥ 32 characters)
- Sessions stored hashed; logout revokes them
- Login rate-limited per client IP (in-memory; replace with Redis/edge limits in multi-instance production)

## Authorization

Every user-owned query must include the authenticated user id from the session, never from the request body or path unless it is verified to match.

Unauthenticated callers receive `401` with `{ "error": { "code": "UNAUTHORIZED", "message": "..." } }`.

## Transport and secrets

- HTTPS in production (reverse proxy)
- Secrets only via environment / GitHub Secrets
- `.env` is gitignored

## Logging

Structured tracing includes timestamp, level, service, request id, route, status and duration.

Never log:

- passwords
- tokens
- medical documents
- blood test values
- private health notes

## Files

Storage keys are sanitized (`..` rejected). Document routes are not public until download is authorized as the owning user.

## Encryption

- In transit: TLS
- At rest: rely on disk/volume encryption and S3 SSE in production; application-level field encryption can be added later for selected columns

## Analytics

No health data in analytics by default. Do not add third-party trackers to client shells without a privacy review.

## Tests

See `backend/tests/authorization.rs` and `backend/tests/api_health.rs`.
