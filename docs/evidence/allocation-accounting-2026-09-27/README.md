# Candidate allocation accounting and reusable control maps

2026-09-27 local date. Source base `8d62ebdf`, with the candidate allocator and
index ownership walk. Experimental code remains under `cfg(test)`; no production
archive output or compatibility-line declaration changes.

## Implemented ownership contract

The candidate now derives physical ownership from the selected authenticated
record graph. It accounts for the fixed publication/journal region, both copies
of every logical-index, key-index and allocation-index page, payload allocations,
reusable ranges, retired ranges and reserved control slack. Sorted coverage must
cover every byte through the selected sealed length exactly once. Gaps, partial
overlaps, overlapping free/live claims and duplicate ownership of control pages
are errors. Shared payload packs are counted once only when their bounds and
stored-byte commitments match exactly.

A bounded `RV4OWN01` record envelope declares whole stored allocations separately
from typed record metadata. It permits at most 512 extent references and 49,152
bytes total. Payload bytes are already encoded/protected when handed to this
layer; this is not the final file/codec/record wire specification. The allocator
checks that new record references name existing owned payload or extents written
by the current transaction. Typed file, slice, codec, access and other semantic
relationships still need their public-format integration.

Transactions use best-fit reusable ranges, durably reserve exact subranges before
writing, and retain a receipt for every new allocation. The next map is derived
from the final reachable graph and those receipts. This includes discarded
intermediate paths and payload, not just objects removed from the old index.
Unchanged paths retain ownership; old allocation-map descendants are reclaimed
only after mirrored publication. Recovery audits the complete selected graph
before allowing journal cleanup to erase anything. A map that falsely retires a
live descendant is rejected without changing storage.

Complete sorted record replacement uses the streaming packed-index builder rather
than constructing and discarding a path for every inserted record. Individual
updates still use path copying. Exact single-record repeats and absent removals
leave the selected generation and file length unchanged.

## Rejected append-only map placement

The first integrated allocator appended every allocation map. Its old pages were
correctly wiped and reusable, but map placement itself could never consume them.
The lifecycle run exposed continued file growth: after 1,000 padded operations,
the file occupied 98,848,768 bytes in plaintext and 98,848,796 bytes in encrypted
signed mode, despite ending with no live payload. All space was accounted for;
roughly 98 MB was reusable rather than unowned. This is an unsuitable placement
policy for the intended archive.

The implementation now reserves one reusable control region before building the
new allocation map. An upper bound derived from actual leaf/branch encoding and
padding allows the region to be reserved before its map exists. That map declares
the region in namespace 242. The audit locates every map page inside it and
requires all unused bytes to be zero, so the reservation cannot hide old payload.
The old region is retired through the next map like other obsolete control state.
If no reusable region fits, preparation appends a zeroed region owned by its tail
journal. No arena is reused until the old publication has released it.

This bounds growth in the measured workload. It is not a universal fragmentation
bound; source-preserving compaction and larger workload qualification remain open.

## File-backed lifecycle experiment

Each design ran ten fresh processes: 100 operations for all four protection modes
with padded and unpadded indexes, then 1,000 operations for padded plaintext and
padded encrypted-signed modes. Both designs together completed 5,600 operations.
The eight-operation sequence adds data, repeats unchanged, replaces it with larger
and smaller content, adds a shared alias and key record, removes each owner, then
repeats an absent removal. Every operation independently reopens the file, reads
and compares the expected content, checks absence, audits complete physical
coverage, and verifies all reusable/retired bytes are zero. Each 1,000-operation
run includes 250 no-change operations with no generation or file growth.

The payload fixture supplies raw bytes or synthetic AEAD-protected bytes. Padding
settings apply to the index/control pages. This does not qualify the final file
codec, payload padding, compression, secret-variable policy or public CLI paths.

The process runs on CPU 2 with the host's existing filesystem/cache state. No owned
builds or tests overlapped measurement. Operation timers include begin, mutation,
commit and cleanup; independent verification is outside those timers. CPU is user
plus system time, and RSS is the process lifetime high-water mark including setup
and verification. No archive-sized in-memory clone is used in this file probe.
There are no warmups or paired confidence intervals: these are deterministic aging
checks with resource observations, not A3/A4 acceptance measurements.

| Padded 1,000-operation case | Map placement | Final bytes | Operation CPU, s | Operation elapsed, s | Peak RSS, KiB |
|---|---|---:|---:|---:|---:|
| Plaintext | Append only | 98,848,768 | 6.313 | 51.984 | 5904 |
| Plaintext | Reusable region | 1,134,592 | 10.173 | 58.561 | 5660 |
| Encrypted + signed | Append only | 98,848,796 | 13.865 | 71.721 | 5800 |
| Encrypted + signed | Reusable region | 1,134,620 | 22.575 | 92.841 | 5768 |

Reusable-region file size stopped growing at zero-based cycle 28 (the 29th
operation) in both 1,000-operation runs. Unpadded 100-operation runs stabilized at
cycle 10, ending at 177,383/177,643 bytes. Padded 100-operation runs ended at the
same sizes as the corresponding 1,000-operation runs. Peak RSS across the revised
design's ten processes was 5,900 KiB.

CPU cost increased substantially in the long runs, by about 61–63%. The extra map
reservation and audit work is real and must remain visible. Elapsed observations
are noisy, including an append-only 100-operation encrypted-signed unpadded run of
44.510 seconds; no sample was removed. The result establishes improved space reuse,
not a CPU speedup or write nonregression. Batched reservation/publication work and
the complete A/B/C comparison remain required before selecting this design.

The final padded plaintext ledger contains 147,456 fixed bytes, 131,072 bytes each
for logical index, key index and allocation map, 331,745 published FREE bytes and
262,175 published PENDING bytes. All PENDING bytes have already been wiped: the
completed journal records cleanup, while the published retirement labels persist
until the next map. They are eligible for checked reuse. The encrypted-signed
case differs by 28 retired payload bytes. No live payload or unowned bytes remain.

## Validation

The native/external-source release format suite passes 98 tests, with 3 manual
resource probes ignored. Integrated allocator tests cover 333 injected mutation
failures (plain unpadded and encrypted-signed padded), 2,004 simulated power-loss
cases (plain unpadded), lifecycle checks across all protection/padding combinations,
shared packs, no-change repeats, reuse and forced append, abandoned payload,
fragmented free space, and retirement of multi-page allocation maps. The size bound
is compared with actual encoded bulk writes through a second branch level at
893,953 records. Nonzero control slack and a falsely retired live descendant are
rejected before recovery writes. See `validation.txt` for final checks.

The existing journal's process-death tests run in that suite; this does not claim
a new allocator-specific process-death matrix, public CLI lifecycle qualification
or platform-specific durable replacement validation. Two production native recovery
failures remain unresolved. No lower-level component result clears those gates.

## Evidence and reproduction

`append-map-aging.jsonl.gz` and `arena-aging.jsonl.gz` retain every cycle and process
total. `summary.json` contains all ten totals per design, final ledger counts and
last growth cycles. `binary.sha256` identifies both preserved executables.
`measured-*.rs.txt` snapshots describe the rejected append-map experiment;
`arena-*.rs.txt` snapshots describe the revised experiment. Both include the exact
unformatted measured implementation and test source. Later additions are geometry,
zero-slack and pre-erasure audit tests. The final hook-formatted code retains the
measured transaction behavior. `measured-run-series.rs.txt` preserves the runner
before hook formatting; `run_series.rs` is its maintained equivalent.

```console
cd rust
cargo test -p revault_lockbox_api --release --lib allocation_map -- --nocapture
# Preserve the printed release test executable before another build.
rustc --edition 2021 -O ../docs/evidence/allocation-accounting-2026-09-27/run_series.rs -o /tmp/revault-allocation-series
taskset -c 2 /tmp/revault-allocation-series /tmp/revault-allocation-arena-resource
```

Next integration work is independently separated mirror placement, batched
reservations/updates and the 64/256 KiB data-extent comparison. Public record
families, codec/padding/access semantics, eager verification and independent
salvage must then use this ownership protocol. Migration, bindings, compaction,
large-scale CPU/RSS gates and #322 platform qualification remain outstanding.
