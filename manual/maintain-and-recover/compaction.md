---
description: "Understand reusable space in format-3 Lockboxes and deferred compaction work."
---

# Reclaim Lockbox space

Deleting or replacing content cleans up retired storage and makes it reusable.
It does not necessarily reduce the `.lbox` file size: free ranges can lie between
live pages, and commit history still occupies storage.

The current format-3 `0.4.x` line does not expose `doctor compact` or automatic
tail truncation. Those capabilities are part of the deferred format-4
transaction-recovery work on `issue-310-zip-read-performance`.

Use [diagnostics](diagnostics.md) and [recovery](recovery.md) to inspect a damaged
archive or resume pending format-3 cleanup. Migration upgrades supported older
formats; it is not an in-place compaction command or a format-4 downgrade tool.

Keep an independent backup before maintenance and verify important contents
afterward. A run of zero bytes is not evidence that a host-file range can be
removed safely; do not truncate or rewrite archive internals manually.
