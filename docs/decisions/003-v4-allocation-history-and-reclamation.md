# 003: Reservations, retained history and reclamation

Status: proposed, 2026-09-26. Author: Codex. Storage/security reviewer: unassigned.
Goals G2/G3/G4/G7; acceptance A1/A5/A7. Existing transaction ordering retained.

## Context

The mirror bug demonstrated that an unpublished operation can still write payload
into real archive allocations. Reverting only the logical TOC is insufficient.
Current preparation journals, free-space reservations and resumable cleanup address
this. Their cost must be assessed against free-range count and archive age, rather
than optimized away without an ownership proof.

## Recommended comparison

Retain conservative base-free-range reservation as the correctness control.
Compare an explicit bounded reservation log whose entries name exact physical
ranges, generation and cleanup state. It must become durable before any range is
written, survive partial writes and recovery interruption, and account for its own
control allocations. A reservation journal cannot reserve space by overwriting
untracked data that it needs to reconstruct after a crash.

Classify retained control/history separately from reusable space and leaked
payload. Define a checkpoint policy that bounds live control traversal and the
cost of aborting a tiny operation; retain only history needed for supported recovery
and verification. Dropping a signed ancestor is not safe until a checkpoint proves
the authority previously supplied by that ancestor. Record the exact retention
bound only after the signing decision specifies required proofs.

Compaction remains source-preserving: copy only authoritative reachable contents,
verify all logical records and access semantics, then install via the platform's
durable replacement protocol. Do not use salvage scanning to copy orphan payload.

## Required experiments

1. Fail a 4 KiB write with 1, 1,000 and 100,000 free ranges; report reservation,
   rollback I/O, syncs and retained bytes separately.
2. Run 100 lifecycle cycles with exhaustive physical accounting; run 1,000 cycles
   for aging. No-change mirrors must not accumulate allocations/history.
3. Interrupt reservations, rollback, cleanup, checkpointing and tail trimming,
   including interruption during recovery; preserve shared-page neighbours.
4. Measure source-plus-replacement compaction headroom and compare all logical
   data after separate reopen. Confirm deletion/aborted payload is absent from
   the completed current archive under the documented erasure boundary.

No new log encoding or history-pruning policy is accepted yet. The default layout
is the control until a candidate has a complete ownership and durability proof.

## Candidate journal evidence — 2026-09-27

The [preparation-journal experiment](../evidence/preparation-journal-2026-09-27/README.md)
implements fixed mirrored reservations and resumable cleanup against the packed
allocation index. It supplies the requested free-range scaling measurements and
interruption evidence, but only journal-level aging. Complete allocation-map
consistency, control-page retirement, mirror placement, public lifecycle accounting
and compaction remain requirements before accepting this decision. Its 392 KiB
of writes and 14 syncs for a failed 4 KiB operation must be evaluated with batching
and the complete production control; the evidence does not justify weakening
publication or erasure ordering.

## Integrated ownership and aging — 2026-09-27

The [allocation-accounting experiment](../evidence/allocation-accounting-2026-09-27/README.md)
adds graph-derived ownership, best-fit reuse, old control-tree retirement and
sorted bulk record replacement. It rejects maps that free live descendants before
recovery performs erasure. The first append-only map policy was rejected after
measured aging: approximately 94 MiB remained after 1,000 tiny padded operations.
An explicitly owned reusable map region stabilized the same workload near 1.08 MiB.
Every operation independently reopened and checked content, zeroed retired ranges
and exact physical coverage.

The control region's unused bytes must be zero, and its size bound includes both
copies of all bulk-built pages. This avoids a recursive self-allocation guess and
accounts for the map's own reserved capacity. The final allocator still needs
batched reservations: long-run CPU increased about 61–63% versus the rejected
append-map variant. These component resource observations do not qualify A4 or
replace full public-record, codec, compaction and platform validation.

## Separated metadata copies — 2026-09-27

The [mirror-separation experiment](../evidence/separated-mirrors-2026-09-27/README.md)
adds distinct aligned failure regions, banked maps and one reservation transition
per reused node pair. In repeated 1,000-operation padded workloads, final size
stabilizes at 1,114,112 bytes after operation five; observed CPU decreases 13.0%
for plaintext and 7.2% for encrypted-signed compared with the prior reusable-map
implementation. This is a descriptive aging comparison, not the A4 statistical
gate. Unpadded tiny archives grow from about 177 KB to 655 KB because of separated
regions and reserved map capacity. Retain that space cost in architecture selection.

Explicit metadata repair audits the entire graph before writes and synchronizes
both publication copies before clearing declared unused space. One aligned 64 KiB
region may be lost without losing metadata membership; payload in that region is
not recreated. Full transaction batching, codec/record integration and compaction
remain open.
