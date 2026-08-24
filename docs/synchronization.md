# Synchronization

Not implemented. Desktop has a JSON `LocalCache` in the Tauri sidecar.

Future shape:

1. Pull user-scoped collections with `updated_at` cursors
2. Push optimistic local writes with server validation
3. Conflict policy: server wins on medical documents; last-write-wins on manual measurements unless a merge is defined

Do not ship ad-hoc bidirectional sync in a feature PR without a design note here.
