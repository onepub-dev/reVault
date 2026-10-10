---
description: "Format-3 transaction publication and resumable cleanup recovery."
---

# Transactions and recovery

The current `0.4.x` line uses Lockbox format 3. Its existing transaction protocol
publishes a new logical state and then cleans up retired storage. The newer
format-4 preparation journal, durable allocation rollback and tail-reclamation
work remain on `issue-310-zip-read-performance`.

## Publication and cleanup

A commit publishes the new logical contents before completing cleanup of retired
pages. A failure after publication can therefore leave the new contents committed
with cleanup still pending. Reopen and inspect persisted contents before retrying
an operation; a commit error does not necessarily mean that nothing changed.

Format-3 recovery uses the published cleanup manifest and its durable progress
checkpoints. It validates the recorded work before zeroing retired storage.
Writable opens perform supported cleanup; read-only opens do not modify the
archive and can report that recovery is required.

## Inspect and resume recovery

```bash
lbx secrets.lbox doctor
lbx secrets.lbox doctor recover --dry-run
lbx secrets.lbox doctor recover
```

The preview distinguishes pending transaction cleanup from the separate salvage
workflow for a damaged archive. Review it before continuing. Salvage writes a
separate recovered archive; it is not a way to recreate a missing source. See
[Recover a damaged Lockbox](../maintain-and-recover/recovery.md).

The Rust APIs expose `transaction_recovery_status`, `recover_transaction`, and
`recover_transaction_controlled`. This line has the `Cleanup` recovery phase.
Controlled recovery can continue or cancel at a durable checkpoint and returns
`NotRequired`, `Complete`, or `Cancelled(status)`. Progress counts describe
manifest ranges, pages and bytes, not user files.

## Format-4 work remains separate

This line does not claim the v4 protocol's recovery of unpublished physical
allocations, interrupted tail truncation, or public verified-compaction command.
Deleting content can make storage reusable without reducing the host file size.
See [Reclaim Lockbox space](../maintain-and-recover/compaction.md).

A format-3 build cannot read or downgrade files already written by a format-4
build. Keep those archives, Vault containers and migration artifacts intact and
use a matching v4 build for access or recovery.

Zeroing cannot erase filesystem snapshots, earlier copies, backups, device-level
remnants or plaintext held outside the archive. Keep independent backups and
verify them with the matching release.
