# Document storage

Allowed types: PDF, PNG, JPG, JPEG, WEBP.

Metadata in PostgreSQL; bytes in the storage abstraction (`local` or S3-compatible). Keys are namespaced by user id and sanitized.

Downloads must verify the authenticated user owns the row before streaming bytes. That HTTP surface is intentionally not mounted in Phase 1.
