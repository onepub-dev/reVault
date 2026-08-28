# Crash-recoverable redaction transactions

Lockbox format v2 uses a prepare/publish/clean/seal protocol. The current core
API reads and writes only v2; v1 conversion is performed by the external
`revault_migrate_archive_v1` and `revault_migration` crates.

During prepare, every content and metadata change is copy-on-write. The old
published commit is not modified. The replacement commit, commit signature,
pre-cleanup free index, post-cleanup free index, and encrypted redaction
manifest are synced before publication. Publication alternates between two
checksummed header slots with monotonic generations.

After publication, manifest ranges are zeroed idempotently. Each encrypted
manifest page is followed by a storage sync and a durable header checkpoint.
The post-cleanup free index is signed as part of the replacement commit, but it
is not selected until the final clean header is synced. Therefore a range that
still needs redaction cannot be allocated early.

A normal open never performs cleanup. It returns `Error::RecoveryRequired`
with the selected transaction and completed/total page, range, and byte
counters. Call `Lockbox::inspect_transaction_recovery` to inspect it and
`Lockbox::recover_transaction_controlled` to acquire exclusive write access,
resume, report progress, and optionally stop at a durable page checkpoint.
Concurrent recovery returns `Error::RecoveryInProgress`; unavailable writable
storage returns `Error::RecoveryBlocked`. Corrupt authenticated metadata is
reported as corruption and is never used as a zeroing instruction.

The CLI reports the same state with stable recovery exit codes. Run:

```text
lbx archive.lbox recover --transaction --key <content-key-file>
```

Recovery is safe to terminate and invoke repeatedly.

## Bounds

One transaction is limited to:

- 65,536 coalesced physical redaction ranges;
- 262,144 scheduled page-object references in memory;
- 16 encrypted manifest pages and 1,049,088 manifest payload bytes;
- 1 PiB of total scheduled physical redaction bytes.

Malformed counts, chains, totals, overlaps, offsets, and checkpoint counters
are rejected before unbounded allocation or zeroing. Manifest pages reveal no
paths or object metadata outside authenticated encryption.

## Adding recipients

Recipient addition is rejected while recovery or unrelated mutations are
pending, and only one recipient widening may be staged per transaction. For an
existing archive, committing that widening performs a fresh-storage compaction
before the recipient is published. This removes abandoned prepare pages and
previous redacted storage from the archive visible to the new decryptor.
Staged key-directory pages are not selected by header or directory scanning.
