# reVault native archive-v1 exporter

Standalone historical exporter for native archive format v1. It emits encrypted
migration-schema-v1 artifacts and preserves existing archive access material.

Protocol version 2 receives the current vault credential and artifact key over
framed stdin. The exporter tries every profile key generation in the migrated
vault, with an existing session-agent content key as an optional fast path.
`LOCKBOX_PASSWORD` is consulted only when the source archive is password-only.
