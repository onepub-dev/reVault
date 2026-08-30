# reVault migration format

Shared encrypted streaming artifact format used by historical reVault exporters
and current migration importers. This crate has no dependency on native vault or
archive APIs.

Archive migration schema 2 adds explicit directory records, portable directory
and symlink permissions, and an optional encrypted archive description. Readers
continue to accept schema-1 records; metadata absent from an already-created
schema-1 artifact cannot be reconstructed.
