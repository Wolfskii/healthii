# API

Base URL (development): `http://localhost:8080`

Version prefix: `/api/v1/`

OpenAPI UI (when `OPENAPI_ENABLED=true`): `http://localhost:8080/docs`

## Errors

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid measurement value"
  }
}
```

Stack traces are never returned.

## Operations

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | no | Process liveness |
| GET | `/ready` | no | PostgreSQL ping |
| GET | `/metrics` | no | Process gauges (no health data) |

## Authentication

| Method | Path | Description |
| --- | --- | --- |
| POST | `/api/v1/auth/register` | Create account |
| POST | `/api/v1/auth/login` | Create session |
| POST | `/api/v1/auth/logout` | Revoke session |
| GET | `/api/v1/me` | Current user |

Register body:

```json
{
  "email": "ada@example.com",
  "password": "at-least-10-characters",
  "display_name": "Ada"
}
```

Login and register return:

```json
{
  "token": "<jwt>",
  "token_type": "Bearer",
  "expires_in": 43200,
  "user": {
    "id": "...",
    "email": "ada@example.com",
    "display_name": "Ada",
    "timezone": "UTC",
    "locale": "en",
    "created_at": "...",
    "updated_at": "..."
  }
}
```

Send `Authorization: Bearer <jwt>`. A `healthii_session` HttpOnly cookie is also set.

## Dashboard

`GET /api/v1/dashboard` returns empty widget shells for the authenticated user. Widget payloads fill in when measurement data exists.

## Planned

```
GET/POST /api/v1/measurements
GET/POST /api/v1/labs
GET/POST /api/v1/documents
GET/POST /api/v1/workouts
GET/POST /api/v1/medications
GET/POST /api/v1/symptoms
GET/POST /api/v1/appointments
```

All of these will ignore any client-supplied `user_id` and filter by the authenticated principal.
