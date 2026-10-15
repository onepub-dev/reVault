---
description: "Reclaim unused Lockbox space with verified compaction and automatic tail truncation."
---

# Reclaim Lockbox space

Deleting or replacing content zeros its retired storage after publication and makes that space reusable. It does not necessarily reduce the `.lbox` file size: free ranges can be between live pages, and commit history still occupies storage.

## Compact an archive

With a v4-capable CLI and an identity allowed to write the Lockbox:

```bash
lockbox secrets.lbox doctor --deep
lockbox secrets.lbox doctor compact
lockbox secrets.lbox doctor --deep
lockbox secrets.lbox cat /path/to/important-file
```

`doctor compact` rebuilds live contents, independently verifies the replacement, and atomically replaces the original path. It reports the previous size, replacement size and bytes reclaimed. The last command is an example independent content check; choose a real path in your archive.

Compaction preserves files, directories, symlinks, permissions, variables, form definitions and records, mirror definitions, access slots, archive identity, signing owner, and encryption/signing/compression choices. It discards free space and the old commit history. It does not rotate the content key or change who has access.

Allow disk space for the new archive alongside the old one. Use the actual archive path rather than a symlink. Keep an independent backup if you need the previous archive or commit history: compaction replaces it without retaining a versioned backup. Repeating compaction is safe, but an already compact archive may not become smaller.

An interruption before replacement leaves the original archive; after replacement, the verified new archive is selected. Abrupt termination may leave a temporary file. After an error, reopen and inspect the archive before retrying.

## Automatic truncation

Commits and clean writable-open maintenance can truncate a contiguous free range at the end of the archive after authenticating its ownership. There is no switch to enable this. Interrupted truncation is resumed through transaction recovery.

A run of zero bytes is not sufficient evidence that storage is free. Interior holes remain reusable until compaction; current commits append history, so even a large deletion may not leave a free tail.

For older formats, [migrate the Vault and then the Lockbox](migrating-between-versions.md) first. Migration also rebuilds committed contents without copying abandoned allocations. For interrupted writes, see [recovery](recovery.md) and the detailed [transaction protocol](../develop-with-revault/transactions.md).
