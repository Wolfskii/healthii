# Health measurements

A single `measurements` table (Phase 2) with a `type` discriminator so new vitals do not need new tables.

Each row: `id`, `user_id`, `type`, `value`, `unit`, `measured_at`, `source`, `notes`, timestamps.

`source` will record `manual`, `apple_health`, `health_connect`, `withings`, `import`, etc.

Never convert and overwrite the stored value. Display conversion belongs in the API/view layer with the original unit preserved.
