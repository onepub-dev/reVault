# 002: Physical layout and access units

Status: proposed, 2026-09-26. Author: Codex. Storage/performance reviewer: unassigned.
Goals G3/G4/G7; acceptance A1–A5/A7. No default-layout change.

## Problem and recommendation

Native indexed frames separate 16 KiB authentication blocks from multi-MiB
compression frames. This helps raw access but does not make Zstd independently
restartable. Performance work has accumulated special paths while compressed
first reads and protected writes still miss recorded gates.

Compare A (existing pages), B (native indexed frames) and C (a compact object index
pointing to independently encoded extents, with packed small entries). C must
use the same transaction, wiping, authority and recovery guarantees. Test 64 KiB
and 256 KiB independently compressed extents against existing frame sizes, using
the fixed evaluation corpus. Stop after that bounded sweep before further tuning.

Treat compression restart size, authentication unit, padding, packing and allocator
unit as separate but coupled choices. One descriptor model must cover raw/Zstd and
all protection modes. Small packed files need individual membership identities;
removing one entry must preserve neighbours and retire the old complete allocation.

## Choices to measure

| Choice | Benefit | Cost/risk |
| --- | --- | --- |
| Existing pages | Retains tested mutation model | Whole-page/frame work and control overhead |
| Indexed native blocks | Efficient bounded raw verification/ranges | Complex caches/descriptors; codec boundary still dominates compressed ranges |
| Independent compact extents | Potentially simpler read path and bounded decompression | More tags/index entries, weaker compression at small units, packing rewrite costs |

Measure metadata opening, copies, decompression, remote requests, recovery and aged
mutation, not only a raw memcpy loop. Keep single-worker defaults during selection.
The saved parallel encoder prototype is preserved evidence, not a prerequisite.

## Acceptance

A2 independent recovery is mandatory. Evaluate A3/A4 and the proposed resource
budgets in the evaluation contract. No production winner until the same revision
meets correctness and performance. If every candidate fails, report the specific
trade-off and revise an explicit requirement rather than selecting by sunk effort.


## Independent physical packing experiment — 2026-09-27

[The file-pack checkpoint](../evidence/independent-file-packs-2026-09-27/README.md)
shares allocation and padding while retaining independent per-fragment codecs,
AEAD context and owner-bound stored-byte digests. A provisional shared codec/AEAD
frame was rejected by an intact-neighbour damage regression. Encrypted padding
is itself authenticated zero plaintext; visible zero gaps would leak encoded size.

Whole-pack deletion verifies and relocates surviving encoded fragments, publishes
all references together and erases the old allocation. Coverage rejects hidden
deleted bytes, overlapping members and incompatible shared commitments. This
remains a private-file-only experiment, not acceptance of decision 002. The
[file lifecycle and salvage](../evidence/candidate-file-lifecycle-2026-09-27/README.md)
now pass internal tests; public semantics, expanded failure/resource qualification,
[packing measurements](../evidence/packed-file-comparison-2026-09-27/README.md) and
[source-preserving compaction](../evidence/candidate-compaction-2026-09-27/README.md)
now quantify those trade-offs. Neither qualifies the current layout.


## Whole-layout feasibility checkpoint

The [current structural review](../archive_v4_evaluation.md#whole-format-eligibility-review)
rules out selecting current C unchanged. Its fixed controls/map and its aggregate
pack-size bound each independently exceed the primary small-file size budget.
The encrypted generic key tree also cannot bootstrap the content key.

The next single layout hypothesis is a compact typed catalogue with file-local
fragment descriptors, one shared descriptor per physical pack, independently
bounded fragments, and two shared 64 KiB control regions. Evaluate actual public
bootstrap and reservation/retirement ownership alongside the private catalogue.
A default bounded metadata cache must retain source identity, generation and
security-domain checks; it must not conceal mutation or bypass authentication.
Keep eager signed-plaintext open and normal padding verification in the comparison.

First run the [read-only cost model](../evidence/whole-layout-cost-model-2026-09-27/README.md)
on retained real encoded fragments. It reconstructs omitted binding fields,
retains both plaintext and stored-byte digests, and tests bounded metadata encoding.
If even that model cannot fit, reject the geometry before implementing a writer.
If it fits, implement and fault-test bootstrap, copy-on-write metadata, journal
overflow, inline-metadata retirement and ownership before any speed claim. Typed
public records and migration remain required; a file-only size model cannot prove
a full format. This is still a proposed decision, with no active wire change.


## Shared-control ordering checkpoint

The [executable ordering model](../evidence/shared-control-ordering-2026-09-27/README.md)
requires staging new metadata externally, publishing both copies, then retiring
old inline metadata. Returning to compact inline placement needs another complete
publication before the temporary external copies can be erased. The compacted
size projection is therefore neither a peak-space nor an update-cost estimate.
The abstract model passes interruption and separate region-loss checks; actual
wire encoding, journal overflow, graph ownership and byte-level crash tests remain.
It does not authorize in-place overwrite or select a compaction policy.


## Damage locality checkpoint

[Read-only recovery tests](../evidence/dense-salvage-2026-09-27/README.md)
distinguish isolated fragment corruption from whole-region loss. On a deliberately
highly compressible 512-file stress corpus, one destroyed 64 KiB region loses
64 files in C and 512 in the denser image; one changed byte loses one dense file.
The independent restart/decode bound is not a bound on logical bytes lost per
physical failure region. Compare retained primary corpora before deciding whether
to accept this trade-off or constrain packing; no new guarantee is selected here.


## Mutable control-space checkpoint

[Measured metadata lifecycles](../evidence/dense-metadata-update-2026-09-27/README.md#measured-metadata-lifecycle-size)
grow the small image from 320 to 448 KiB after its first edit, then reuse that
space. The fixed 48 KiB private slot fills each bank's entire private area, forcing
new metadata outside the prefix until retirement. Test two 24 KiB private slots
within each bank for smaller catalogues, with the existing larger/external route
when necessary. This changes allocation geometry, not fragment decode bounds or
publication/erasure ordering. No format decision or gate change is implied.
