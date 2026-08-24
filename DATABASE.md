# Database

SQLx migrations in `backend/migrations/`. Never edit a migration that has been applied. Never change production schema by hand.

## Naming

```
0001_initial_schema.sql
0002_add_measurements.sql
0003_add_lab_results.sql
```

## Current schema (0001)

### users

`id`, `email` (unique, stored lower-case), `password_hash`, `display_name`, `timezone`, `locale`, `created_at`, `updated_at`

### sessions

`id`, `user_id`, `token_hash` (SHA-256 of JWT), `user_agent`, `expires_at`, `created_at`, `revoked_at`

Logout sets `revoked_at`. Authenticating requests require a non-revoked, unexpired session whose `user_id` matches the JWT `sub`.

### audit_events

`id`, `user_id`, `action`, `resource_type`, `resource_id`, `request_id`, `ip`, `created_at`

Do not store measurement values, notes or document bytes in audit rows.

## Planned tables

User-scoped tables for measurements, lab tests, lab results, biomarker definitions, documents, workouts, medications, symptoms and appointments. Every row will have `user_id` not null and queries will always include `WHERE user_id = $authenticated`.

Laboratory results stay flexible: biomarker codes are data, not columns.

## Units

Store the original measurement value and unit. Do not overwrite with a converted value. Canonical conversions, when added, must be explicit and reversible.
