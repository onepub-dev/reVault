# Whole selected-tree extent compaction

This test-only (`cfg(test)`) adapter copies a stable selected shared-tree archive
into an owned empty destination, preserving files/filesystem metadata, normal and
secret variables, all selected form definition revisions and historical captures.
It does not activate public APIs, install a replacement path or select a format.
Same archive UUID, mode, key and owner are required. Access roots are explicitly
refused; no access state is silently dropped.

Publication is an authenticated successor: checked source generation plus one,
with the selected source commitment as `previous`. Existing fresh export retains
generation one and zero predecessor. The shared initializer preserves dependency
readback/sync, bound idle preparation, and both synchronized anchor publications;
the successor path rechecks the source immediately before publication. Rebuilt
index/catalogue/anchor state is freshly authenticated. Copied payload pages retain
their encoded bytes, identities, nonce, revision, context, length and digest.

This is extent relocation. Pack interiors, fragment-relative offsets and selected
historical definitions remain intact. The adapter omits old unselected/free physical
ranges, rebinds each exact selected extent, builds a new ownership union and admits
typed row/path limits plus the existing 4,096 ownership-record bound before staging.
It does not promise every later index/page resource limit can be checked without
building; a staging failure must cleanly discard the owned destination.

Nonempty destinations, invalid authority/key/signer, generation exhaustion,
unsupported access roots and corrupt selected content refuse before destination
mutations, including cleanup operations. After the first attempted staging write,
any error invokes bounded zeroing, sync, truncate-to-zero and final sync. Cleanup
failure preserves both errors and never establishes successful erasure or durability.
The source is borrowed; neither storage backend is cloned by the adapter.

The new relocation and complete extent readback use at most 64 KiB guarded chunks,
including tails larger than one chunk. Streaming checks reuse the existing checksum
domain and encoded total-length prefix. Secret variable/form ranges use guarded
reads throughout. Existing file-codec preflight/final verification retains its
ordinary wipe-on-drop buffers; this work does not claim to remove those buffers.
Backend-held plaintext archive bytes are outside the no-extra-copy claim.

## Functional and failure checks

All 16 modes pass mixed filesystem/variable/form copying, exact typed metadata rows
after reversing placement, independently reopened logical reads, stable source
bytes and successor lineage. One-MiB secret variable and historical form values
exercise multichunk stored extents. Complete source/destination extent ranges are
registered before copying; the guard observes the first append through actual
shared-anchor publication and checks exact source/readback byte totals. Secure
ranges remain protected outside that phase. Clone traps cover both adapter sides.

These fixtures shrink from 3,080,192 to 2,424,832 bytes unpadded and 8,060,928 to
7,536,640 bytes padded. This is a fixture result, not a universal shrinking, mixed
aging or size gate. Both source and destination coexist during copying.

The returned-failure matrix passes 1,896 cases: all 316 successful mutation
positions across eight representative modes, prefixes zero/97/full and both
modeled sync-persistence outcomes. Every returned failure leaves volatile and
modeled durable destination empty, with unchanged source bytes. This exercises
cleanup after a returned failure, not process death or replacement installation.

Additional controls pass tail corruption and read I/O errors in a second 64 KiB
chunk, explicit cleanup zero/sync/truncate/final-sync failures, and source selection
changes after first append, after dependency sync and after both anchor writes.
The last source-change control requires the final post-verification source check.
Synthetic external commits remain intact while the owned destination is discarded.
Stable storage remains a precondition: commitment checks cannot detect arbitrary
in-place edits after the last content read.

Preflight controls independently authenticate a maximum-generation fixture and
prove zero destination mutations on exhaustion. The fixture-only generation helper
authenticates its original source and rebuilds both anchors and bound idle stubs
with existing encoders; it is compiled only for tests. Actual password-slot access
fixtures independently reopen credentials and the complete typed graph before
the adapter's explicit refusal, again with zero destination mutations.

Unchanged initializer regressions pass four tests. Fresh-tree export passes seven
tests and 93 returned-failure cases; its separate manual 64 MiB probe is ignored.
Final combined checks pass six whole-tree controls and 13 page codec tests. Strict
Clippy for library/tests/benches passes on the same final source. No CPU, incremental peak RSS, concurrency, physical-disk durability,
whole-region loss or full-format acceptance follows from these results.

## Larger FileStore functional fixture

The corrected fixture completes all 16 modes with nine live one-MiB secret variables
and one one-MiB captured form value: 10,485,760 selected large-value bytes, plus
normal variable, filesystem and remaining form/definition state. Setup also creates
and deletes a tenth large variable. One synthetic guarded caller is reused during
setup and dropped before compaction; it does not model ten independent caller
allocations. Both writers close before independent FileStore handles reopen and
verify every value separately. No collection of all secret values is materialized.

| Modes | Source bytes | Destination bytes |
| --- | --- | --- |
| 0–7, unpadded | 13,041,664 | 10,813,440 |
| 8–15, padded | 26,017,792 | 23,265,280 |

Mode bits are protected1, signed2, compression4, padding8. Form/variable pages remain
uncompressed in all modes; this is not a compression comparison. Final functional
images and a synthetic public-key sidecar remain under
`/tmp/revault-whole-tree-copy-functional-189934-3746568383224254782/`.
Independent artifact verification confirms all 16 unique modes and all 32 image
sizes and length-bound digests, adding raw SHA-256 hashes in
`aggregate-qualified/verified-artifacts.json`. Earlier compile/setup failures and
partial results remain separate. This is a functional test, not a fixed timing or
resource sample batch. `/tmp` is tmpfs on this host: FileStore reopen/sync behavior
does not establish physical-disk power-loss durability or I/O performance.

## Retained development failures

The first streaming draft used raw SHA-256 and failed source extent verification:
existing archive checksums bind a domain and total length. The correction factors
the unchanged initializer into `strong_checksum_hasher`; fixed empty/abc vectors
computed independently with Dart crypto and the retained old one-shot construction
verify wire equivalence, chunk boundaries and length binding. No checksum policy
or stored representation changed. Temporary stage diagnostics were removed.

A new fixed-vector test initially referenced an unavailable `hex` dependency;
standard Rust formatting corrected that compile failure. The first copy-phase guard
watched separated-layout `RV4PUB02` rather than shared-layout `RV4SHR01`, incorrectly
classifying final file verification as copying. Its retained backtrace identifies
`Codec::load` through `verify_all`; the corrected phase uses actual shared publication.

The first access fixture overlaid keys on an aged graph that still explicitly
claimed those control ranges as free. Credential reopen succeeded but typed ownership
correctly refused the overlap. Starting from a fresh ownership graph corrected the
fixture. The first tail fixture used an unprefixed string instead of canonical
VariableName equality, and one adjacent test condition failed strict Clippy; both
were corrected without changing the copy algorithm. All failures and source hashes
remain retained. Prior copy/test source snapshots were reconstructed only after
matching each original SHA-256 manifest and are stored as `.rs.txt` artifacts.

The aggregate fixture first referenced an unavailable `tempfile` dependency; it now
uses existing getrandom/std APIs. Its next run passed modes 0–7 before a setup guard
failed in mode 8 during variable deletion, before compaction/destination creation.
The backtrace shows authenticated index traversal over a prior form-value range:
the long-lived fixture guard retained that range after retirement, while a later
transaction legitimately reused it as metadata. The corrected fixture refreshes
guard membership only at successful setup boundaries, after independent selected
reopen/verification and explicit guarded zero/truncation checks of every removed
tracked range. Live/attempted guards remain active within transactions; complete
planned source/destination copy guards remain unchanged. No allocator or archive
algorithm was modified for this fixture correction.

## Reproducibility

Evidence directories retain original logs, HEAD, status and source hashes. Exact
copy/test snapshots use `.rs.txt` suffixes to preserve bytes outside formatting
hooks. `final-controls/` additionally retains tracked source diff, final source
snapshots and compiler identity; no synthetic archive images or binaries are
committed. Tests use pinned Rust 1.88.0, release `external-source`, serial test
execution. Artifact verification is an executable Dart+dcli script, following the
current repository script instructions; older retained evidence is unchanged. The
local verifier project uses dcli 10.3.0 and crypto 3.0.7. Host memlock remains
8,192 KiB; successful functional checks do not establish arbitrary concurrency
or incremental-memory bounds.

Final affected commands from `rust/`:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib whole_tree_compaction -- --nocapture --test-threads=1 --skip whole_tree_compaction_returned_failures_durably_discard_owned_destination --skip whole_tree_compaction_large_selected_state_file_store_all_modes
cargo test -p revault_lockbox_api --release --features external-source --lib file_format::page -- --nocapture --test-threads=1
cargo clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
```

The separately retained matrix and aggregate runs select their skipped names with
the same release/features/serial flags. They were not repeated for the final combined
controls; Clippy is retained from the same final source's aggregate-qualified run.
The tracked hook formatted five Rust files in checkpoint `047a0f7a`.
Post-format verification passes six whole-tree controls (including all 16 modes),
four shared-anchor tests, 13 page tests and strict Clippy. `postformat/` retains
the exact source hashes and logs. Compilation took 67 seconds; test runtimes
were 47.79, 0.79 and 0.34 seconds respectively. The fault matrix, aggregate
fixture and fresh-export regressions were not repeated after formatting.

## Remaining work

Connect this verified whole-tree result to filesystem atomic installation and
process-death handling before claiming complete compaction. Existing file-only
installation is not a resumable temporary-copy protocol. Access mutation/overflow,
mirror/public filesystem policies, native selected-membership recovery, architecture
selection and complete lifecycle/resource/CLI-binding-Vault compatibility gates
remain open. Copying cannot hide prior aging or damage-locality failures.
