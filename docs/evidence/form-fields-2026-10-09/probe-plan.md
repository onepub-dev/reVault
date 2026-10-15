# Fixed cross-record field upgrade aggregate probe

After final correctness and Clippy pass, freeze final release test executable,
source changes/snapshots/compiler/manifests and runner hashes. Exactly16 fresh Linux
processes, modes0..15, one attempt each/no retries; record failures. No concurrent
owned tests/builds. Inherit CPU affinity and8MiB memlock without policy changes.

FileStore fixture: one normal-field definition and nine records, each containing
one distinct1MiB normal value. Build one ordinary synthetic record at a time, then
drop all setup caller objects and close writer. Reopen writer through clone-trap
and guarded allocating-read wrapper. Actual override is a single borrowed1MiB
SecretString of different bytes retained during setter. Secret setter upgrades all
nine records and adds immutable definition revision2, using stored sources for
eight neighbors, caller source for target. Old definition remains selected.

Capture /proc endpoints before setup, after setup/caller drop, after override
allocation, immediately after mutation before independent verification allocations,
and after independent reader/scoped per-record reads and streamed salvage. Drop
writer before opening independent FileStore reader handle in the same process.
Verify all nine value bytes, types, captured labels/references, record metadata,
both definitions and selected ownership. Verify retired old ranges erased via
guarded reads; retain source old definition extents. Stream every event without
retaining all values; one secret callback scope at a time.

Record physical size before/after, logical/selected payload sizes, all endpoints,
complete output and exactlyone FORM_FIELD_AGGREGATE marker perprocess, exitstatus,
archive hash. Require unique modes, one passing test each and stable executable+
runner hashes before/after. Modebits protected1,signed2,compression4,padding8;
secure form pages remain uncompressed. Archive growth not hidden by compaction.

No timing ratios, incremental memory peaks, arbitrary concurrency, all-values
getter capacity, real process-death, released compatibility or formatwidegateclaim.
VmHWM includes setup; VmRSS/VmLck are endpoints and arenas can be retained/reused.
Dropping setup callers does not mean actual override caller is absent. Prior
wholearchive test-scan allocation failure remains retained separately.
