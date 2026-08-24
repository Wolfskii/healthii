# Release process

1. Update `CHANGELOG.md` and crate/app versions together.
2. CI must be green on `develop`.
3. Tag `vX.Y.Z`.
4. `release.yml` builds the backend image and can push to GHCR.
5. Desktop installers and Expo binaries are platform-specific; produce them on macOS/Windows/Linux as needed.
6. Deploy by image tag, run migrations (startup), confirm `/ready`.
