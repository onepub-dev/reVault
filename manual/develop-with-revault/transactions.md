---
description: "Lockbox v4 transaction phases, durable rollback, cleanup and space reclamation."
---

# Transactions and recovery

This page describes the v4 implementation on the development branch. Use the documentation belonging to your installed release when working with older clients. Older Lockboxes require [migration](../maintain-and-recover/migrating-between-versions.md); opening them does not silently upgrade them.

A transaction changes the logical contents of a Lockbox at one publication point. Before publication, recovery restores the previous committed state. After publication, recovery keeps the new state and finishes erasing obsolete storage. A failed command is therefore not proof that its changes were rolled back: an error can occur after publication.

The same protocol covers files, directories, symlinks, variables, form definitions and records, indexes, and access metadata. Mirror updates use these underlying transactions; their host-directory checks do not provide a separate storage recovery mechanism.

## Durable state and free space

The header has two independently validated slots with increasing generations. Header metadata authenticates the selected commit, preparation state, committed file length (`sealed_len`), cleanup progress, and any pending truncation. Payload and metadata pages record their physical allocation extent; encrypted pages authenticate that extent as associated data. Allocation accounting uses the full physical extent, not a compressed payload's apparent size.

The free-space structure is an indexed tree of physical ranges. It is not a chain of deleted file records. The committed free index and the append boundary together provide a conservative reservation journal: every base free range and every byte appended beyond the base length may have been used by an unfinished transaction. This avoids needing to append a journal entry for each allocation.

A clean reusable range must contain zeros. During preparation it may temporarily contain uncommitted data, but durable preparation metadata prevents another open from treating the archive as clean. Ranges awaiting post-publication cleanup are also unavailable for reuse until cleanup is sealed. Zero-looking bytes alone never establish that a range is free.

## Commit phases

| Phase | Durable action | What happens after interruption |
| --- | --- | --- |
| Prepare | Publish preparation in both header slots, syncing each before writing transaction payloads. Preserve the base commit roots, free index and sealed length. | Restore the base state through rollback. |
| Write | Write changed content and metadata using copy-on-write. Reserve control pages before serializing the free indexes. Write the replacement roots, cleanup manifest and commit authentication, then sync storage. | Roll back; unreferenced pages do not become committed because they happen to be readable. |
| Publish | Publish and sync a header selecting the replacement commit. | The new logical state is authoritative. If publication fails ambiguously, reopen to determine which state became durable. |
| Clean | Put both header slots on the new root before erasing old allocations. Validate cleanup ranges against the live inventory, zero them, sync, and checkpoint after each manifest page. | Resume cleanup of the new state; never restore the old root. |
| Seal | Publish completed cleanup and select the post-cleanup free index. | Freed ranges can be reused. Repeating completed cleanup is safe. |
| Trim, when possible | Authenticate a wholly free suffix, publish its shorter target length in both slots, then truncate and sync. | Finish truncation without changing logical contents. |

Copy-on-write includes replacing live objects that share a page with deleted objects. The old page is retired only after surviving objects have replacement storage. Inventory follows authoritative roots and retained commit authentication/history, and retires unreferenced content and superseded control pages. Free-index and manifest pages must not accidentally list their own allocations as reusable.

Cleanup happens **after publication**. A successful commit has completed its required redaction cleanup; interruption between publication and sealing can leave obsolete bytes until recovery completes. Retained commit records explain some continuing growth even when deleted payloads have been erased. Retaining those records does not promise that old file contents remain available for restoration.

## Rollback before publication

Recovery first authenticates the base roots, allocation inventory, free ranges and sealed length. It validates the entire reservation set before erasing anything, including checking that truncation cannot cut through a live allocation when the free list is empty.

It then zeros the base free ranges in bounded 64 KiB buffers, syncing and recording progress at each range. This deliberately includes ranges that the failed attempt might not have used. Next it truncates appended preparation data back to the verified base length, syncs storage and publishes a clean base header. Repeating these actions after another interruption is safe. The previous committed records, identity and signing owner remain authoritative.

This is physical storage rollback, not a rescan of the mirror's source directory. If a source file changes while being imported, the mirror operation can reject the update and abort its transaction; reopening recovers the Lockbox from its own durable state, without depending on that source file still existing.

## Errors, abort and retry

`Lockbox::abort()` discards an active uncommitted transaction and restores the last published state. It recovers through a cloned storage handle and clears stale page and compression caches. It is not an undo command for a published commit.

A recoverable pre-publication `commit()` error restores the state from before that commit attempt, which can still include staged changes. That differs from aborting the whole transaction. If publication or restoration fails ambiguously, the handle refuses further writes until it is reopened; do not keep retrying writes on a failed handle.

After a process crash, the caller does not need to have called `abort()` for rollback to work. The durable preparation record is what makes the next recovery possible. After an error, reopen, let recovery complete, and inspect the persisted result before deciding whether to repeat an application operation.

## Opening and controlled recovery

Write-capable opens finish pending authenticated recovery automatically while holding exclusive access. Explicit read-only opens do not modify storage and return `Error::RecoveryRequired` when recovery is needed. A normal CLI read workflow may arrange recovery before obtaining its read-only handle; this does not change the explicit read-only API contract.

Rust exposes `inspect_transaction_recovery`, `recover_transaction`, and `recover_transaction_controlled`. Status and progress identify `TransactionRecoveryPhase::Rollback`, `Cleanup`, or `Truncate`. Controlled recovery returns `NotRequired`, `Complete`, or `Cancelled(status)` and accepts cancellation at a durable checkpoint. Range/page counters refer to reservation ranges during rollback and manifest work during cleanup; truncation reports its own byte progress. They are not counts of user files.

Concurrent recovery can report `RecoveryInProgress`; unavailable writable recovery storage can report `RecoveryBlocked`. Corrupt metadata is an error, never permission to erase or truncate guessed ranges. Recovery does not require promoting an uncommitted key-directory page found by scanning.

The CLI detects the required phase automatically:

```bash
lockbox secrets.lbox doctor recover --dry-run
lockbox secrets.lbox doctor recover
lockbox secrets.lbox doctor --deep
```

Run the second command after confirming the preview identifies a pending transaction. When no transaction is pending, `doctor recover` follows its separate salvage workflow and writes a recovered archive. See [recovery](../maintain-and-recover/recovery.md).

## Tail truncation and compaction

Automatic reclamation removes only a contiguous authenticated free suffix at the actual end of the file. Before truncating, both synced header slots record the new `sealed_len` and `trim_origin_len`, which authenticates the old free-index extent. Opening validates that suffix and clips it from the reusable ranges. The next ordinary commit writes a fresh free index and clears the historical trim extent.

Interior holes cannot be removed by truncation. Current commits append history records, so a large deletion often creates reusable interior space without shortening the file. Explicit [compaction](../maintain-and-recover/compaction.md) rebuilds live state in fresh storage and discards the old history.

File-backed compaction writes a same-directory temporary archive, syncs it, independently reopens it, checks physical ownership, compares metadata and secrets, and streams every live file byte for comparison. Only after verification does it atomically rename the replacement over the original. The handle is rebound to the new file before syncing the parent directory; failure at that point prevents further writes until reopen. Host file permissions are preserved, symlink source paths are refused, and a stale handle cannot replace a different file that has taken over its path.

An interruption before rename leaves the original; after rename the new archive is selected. Abrupt process termination can leave a temporary file. Compaction preserves identity, format options, access slots, signing owner, live records, and mirror definitions, but it is not a backup and does not preserve the old commit chain. Adding a recipient to an existing archive also uses a fresh rewrite before making the new decryptor visible.

## Limits and security scope

Redaction manifests are bounded to 65,536 coalesced ranges, 262,144 scheduled page-object references, 16 manifest pages, 1,049,088 manifest payload bytes and 1 PiB of scheduled redaction bytes. Counts, chains, offsets, overlaps, extents and checkpoint totals are checked. Secure variable/form pages retain their fixed 128 KiB physical allocation when accounting for erasure and reuse.

Zeroing protects the updated archive's reusable space. It cannot erase filesystem snapshots, previous copies, device-level remnants, backups or plaintext held elsewhere. Compaction and migration likewise cannot erase copies outside their replacement archive.

## Binding compatibility

See [API maintenance and v4 compatibility](apis/revault-api.md#transaction-maintenance-and-v4). The Rust maintenance APIs and recovery phases are not new C ABI operations; foreign facades receive automatic recovery through their existing native writable-open operations.
