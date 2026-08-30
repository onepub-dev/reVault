# reVault native archive-v1 exporter

Standalone historical exporter for native archive format v1. It emits encrypted
migration-schema-v2 artifacts and preserves existing archive access material,
including empty directories and portable directory and symlink permissions.

Protocol version 3 receives the current Vault credential, artifact key, and
current-Vault contact key records over framed stdin. Passing the key records
allows contact-only archives to migrate after the Vault has already moved to a
format the historical exporter cannot read. An existing Session Agent content
key remains an optional fast path. `LOCKBOX_PASSWORD` is consulted only when
the source archive is password-only.
