# Document storage

Allowed types: PDF, PNG, JPEG, WEBP. Maximum 15 MB per upload.

Metadata in PostgreSQL; bytes in the storage abstraction (`local` or S3-compatible). Keys are namespaced by user id and sanitized.

`GET /api/v1/documents/{id}/file` streams bytes only after the session's `user_id` matches the row. JSON list/get responses never include `storage_key`. Clients download with `Authorization: Bearer` (blob fetch), not a public URL.
