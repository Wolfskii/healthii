# Authentication

Register and login issue a JWT whose `jti` is a `sessions` row. The password never leaves Argon2 hashes. Logout sets `revoked_at`.

Clients send `Authorization: Bearer` or rely on the `healthii_session` cookie. CSRF for cookie use should be addressed before browser cookie-only sessions are the primary mode (desktop and mobile can use Bearer).

Rate limiting applies to login. Registration still needs product-level abuse controls (captcha/email verification) before a public internet deployment.
