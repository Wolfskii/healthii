# Database

SQLx migrations in `backend/migrations/`. Never edit a migration that has been applied. Never change production schema by hand.

## Naming

```
0001_initial_schema.sql
0002_health_records.sql
0003_profile_goal.sql
0004_health_notes.sql
```

Prefer sequential prefixes for new files.

## 0001 — accounts

### users

`id`, `email` (unique, stored lower-case), `password_hash`, `display_name`, `timezone`, `locale`, `created_at`, `updated_at`

### sessions

`id`, `user_id`, `token_hash` (SHA-256 of JWT), `user_agent`, `expires_at`, `created_at`, `revoked_at`

Logout sets `revoked_at`. Authenticating requests require a non-revoked, unexpired session whose `user_id` matches the JWT `sub`.

### audit_events

`id`, `user_id`, `action`, `resource_type`, `resource_id`, `request_id`, `ip`, `created_at`

Do not store measurement values, notes or document bytes in audit rows. `GET /api/v1/activity` returns action labels and timestamps only.

## 0002 — health records

Every table is scoped with `user_id` (or a parent that is). Queries always include `WHERE user_id = $authenticated`.

### profiles

One row per user: `height_cm`, `blood_type`, `allergies`, `medical_history`, `emergency_name`, `emergency_phone`, `unit_system` (`metric` \| `imperial`), `weight_unit` (`kg` \| `lb`), `goal_weight`, `goal_weight_unit` (`kg` \| `lb`, optional).

### measurements

`type` discriminator (`weight`, `blood_pressure_systolic`, `resting_heart_rate`, `sleep`, …), `value`, `unit`, `measured_at`, `source`, `notes`. Original value and unit are preserved. Sleep is stored as hours (`h`) when imported; manual entries may use `h` or `min`.

### lab_tests / lab_results / biomarker_definitions

Panels on a date; results are rows with `biomarker_code` (data, not columns). `status` is `low` \| `normal` \| `high` \| `unknown` (or `critical` only if stored explicitly — never auto-diagnosed).

### documents

Metadata in PostgreSQL (`filename`, `mime_type`, `size`, `storage_key`, `checksum_sha256`, `document_type`). Bytes live in the storage backend. JSON responses omit `storage_key`.

### workouts

`workout_type`, timing, optional distance/calories, `exercises` JSONB.

### medications / symptoms / appointments

Name, schedule or severity, timestamps. All user-scoped.

### health_notes

Private journal: `title`, `body`, `noted_at`. User-scoped. Never written to logs.

## Units

Store the original measurement value and unit. Do not overwrite with a converted value. Canonical conversions, when added, must be explicit and reversible.
