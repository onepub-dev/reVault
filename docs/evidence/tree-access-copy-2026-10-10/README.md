# Retained credential directories during tree compaction

The private shared-tree extent copier and complete-copy resume adapter now retain
existing bounded public credential directories. They validate the source's public
root and canonical bootstrap encoding before destination mutation. The directory
bytes, slot IDs, wrapping algorithms, salts and wrapped keys remain exact. Its
original directory generation remains valid beneath newer authenticated anchor
generations; compaction does not create new credential state or unwrap/re-wrap keys.

The successor initializer writes both directory mirrors with private metadata and
idle preparation, reads back both mirrors, synchronizes dependencies, and only
then publishes the successor anchors. Complete-copy resume compares exact source
and candidate directory bytes in addition to lineage, payload extents and typed
rows. A valid successor carrying different credentials is refused unchanged.

Credential tests cover all eight encrypted signing/compression/padding modes:
password and hybrid contact reopen after repeated compaction, exact directory
retention, unknown-slot/wrong-password refusal, independent loss of either public
directory mirror, and unchanged source bytes. Sixteen injected partial writes
(first or second public directory in each mode) return failure and erase/truncate
the owned destination. Both public-directory copies damaged before compaction
refuse with zero destination mutations. File-backed resume independently reopens
credentials and all logical fixture families.

The changed-wrapper refusal fixture intentionally signs or MACs a replacement
with the same generation and predecessor, same payloads, and a different valid
password wrapper for the same content key. The alternative password successfully
opens that fixture first. This proves admission checks credential equality rather
than relying only on structural parsing or successor lineage. No public CLI can
produce that experimental state; the helper exists only in tests and verifies its
original and rewritten publications. No production signing policy is weakened.

Final validation uses Rust 1.88.0, release external-source and serial tests. Focused
credential tests, six scoped compaction controls, the complete resume suite and
strict core Clippy pass. The initializer/bootstrap controls previously passed eight
tests on the same production source. Logs retained here identify exact filters;
the expensive historical compaction fault matrix and large aggregate fixture were
not repeated. This does not claim physical power-loss, peak-memory, CPU, aging or
whole-format qualification.

The current bootstrap experiment still permits only a 4 KiB directory and at most
48 slots. Overflow, access grant/revoke/rotation transactions, public API integration
and full resource/compatibility qualification remain open. Fresh conversion from
the older candidate file image still refuses access roots; this change applies to
already selected shared-tree archives. Automatic partial temporary ownership and
cleanup remain separate work. No public format or API is activated.

## Preserved native recovery blockers

Both known failures reproduce independently with external-source,native-block-layout:
truncated-tail recovery yields zero rather than three intact files at
api_tests.rs:3364; multiframe damaged recovery reports two rather than one partial
files at block_frame_tests.rs:317 in signed plaintext/raw/default-padding mode.
Raw and backtrace logs are retained. Neither assertion nor owner authentication
was changed. Inspection shows native signed recovery currently requires a full
normal-open snapshot; signed plaintext verification hashes the entire archive's
contents. Missing or damaged unrelated content therefore prevents that proof.
A safe fix needs authenticated selected membership that survives unrelated loss;
skipping full verification alone would weaken owner guarantees. These failures
remain architecture/recovery blockers, not passes of the shared-tree component.
