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
