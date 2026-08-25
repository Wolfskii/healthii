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

Stack traces are never returned. Health values are never written to logs.

## Operations

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | no | Process liveness |
| GET | `/ready` | no | PostgreSQL ping |
| GET | `/metrics` | no | Process gauges (no health data) |

## Authentication

| Method | Path | Description |
| --- | --- | --- |
| POST | `/api/v1/auth/register` | Create account and a default profile |
| POST | `/api/v1/auth/login` | Create session |
| POST | `/api/v1/auth/logout` | Revoke session |
| PUT | `/api/v1/auth/password` | Change password; other sessions are revoked |
| GET | `/api/v1/me` | Current user |
| PUT | `/api/v1/me` | Update display name and timezone |
| DELETE | `/api/v1/me` | Delete account (password required) |

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
  "user": { "id": "...", "email": "ada@example.com", "display_name": "Ada" }
}
```

Send `Authorization: Bearer <jwt>`. A `healthii_session` HttpOnly cookie is also set.

## Profile and overview

| Method | Path | Description |
| --- | --- | --- |
| GET/PUT | `/api/v1/profile` | Height, units, weight goal, allergies, emergency contact |
| GET | `/api/v1/dashboard` | Live widgets from the caller's records |
| GET | `/api/v1/timeline?kind=` | Chronology (`all`, `measurements`, `labs`, …) |
| GET | `/api/v1/search?q=` | Case-insensitive search of the caller's records (min. 2 characters) |
| GET | `/api/v1/charts?metric=&days=` | Time series for a measurement or biomarker |
| GET | `/api/v1/export?format=json\|csv` | Full export of the caller's data |
| POST | `/api/v1/import` | Import a Healthii JSON export into the caller's account (ignores embedded user ids) |
| POST | `/api/v1/import/csv` | Import Healthii CSV, or a Health Connect / Fit vitals CSV |
| POST | `/api/v1/import/apple-health` | Import Apple Health `export.xml` (unzip the export first) |
| GET | `/api/v1/sessions` | Active sessions for this account |
| DELETE | `/api/v1/sessions/{id}` | Revoke one session |
| GET | `/api/v1/activity` | Account activity (sign-in, import; no health values) |
| GET/POST | `/api/v1/notes` | Private health notes |
| DELETE | `/api/v1/notes/{id}` | |

Dashboard widgets are empty until the user has data. Derived BMI is labeled as generated, not a diagnosis.

## Health records

All of these ignore any client-supplied `user_id` and filter by the authenticated principal.

| Method | Path | Description |
| --- | --- | --- |
| GET/POST | `/api/v1/measurements` | Generic vitals (`weight`, `sleep`, `heart_rate`, …) |
| POST | `/api/v1/measurements/blood-pressure` | Systolic + diastolic pair |
| GET/PUT/DELETE | `/api/v1/measurements/{id}` | One measurement |
| GET | `/api/v1/labs/biomarkers` | Known biomarker codes |
| GET/POST | `/api/v1/labs` | Laboratory panels with nested results |
| GET/PUT/DELETE | `/api/v1/labs/{id}` | One panel |
| GET/POST | `/api/v1/documents` | Multipart upload (`file`, `document_type`, optional `title`) |
| GET/DELETE | `/api/v1/documents/{id}` | Metadata only (`storage_key` is never returned) |
| GET | `/api/v1/documents/{id}/file` | Authorized byte stream |
| GET/POST | `/api/v1/workouts` | Sessions and optional exercises |
| GET/PUT/DELETE | `/api/v1/workouts/{id}` | One workout |
| GET/POST | `/api/v1/medications` | Medications and supplements |
| DELETE | `/api/v1/medications/{id}` | |
| GET/POST | `/api/v1/symptoms` | Symptoms |
| DELETE | `/api/v1/symptoms/{id}` | |
| GET/POST | `/api/v1/notes` | Private health notes |
| DELETE | `/api/v1/notes/{id}` | |
| GET/POST | `/api/v1/appointments` | Clinic visits |
| DELETE | `/api/v1/appointments/{id}` | |

Document uploads: PDF, PNG, JPEG, WEBP, 15 MB maximum. Downloads require the same session that owns the row.

JSON/CSV import never includes document bytes. Apple Health import reads `export.xml` only (unzip the Health export on your computer first; body limit 32 MB). Sleep analysis stages are skipped; time in bed / sleep duration is imported as hours. `/import/csv` also accepts a wide Health Connect / Fit vitals CSV (weight, BP, resting HR, glucose, sleep hours, …; steps skipped). Import always creates new rows for the authenticated user and ignores any `user_id` in the file.
