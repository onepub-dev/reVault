# Shared-control first-read comparison

Date: 2026-09-27. Based on `809b6f9a`. Freeze the containing source and executables
before measurement. This is a bounded file-only whole-layout comparison following
[successful metadata-tail lifecycle size checks](../metadata-tail-retirement-2026-09-27/README.md).
It does not select the archive format or qualify the missing public semantics.

## Declared batch

Compare retained packed C against shared-control full-catalogue images after 100
metadata edits and return-inline cycles. ZIP uses the same source bytes. Run the
following finite six cases once, with 30 paired observations after three warmups:

- 512 × 4 KiB mixed bytes, compressed plaintext, whole-file reads.
- The same small corpus, encrypted and owner-signed, whole-file reads.
- 8 MiB seeded incompressible raw plaintext, whole-file reads.
- 8 MiB patterned compressed plaintext, whole-file reads.
- 8 MiB patterned compressed encrypted/signed, whole-file reads.
- The same raw plaintext 8 MiB archive, one 4 KiB range at its midpoint.

Use one pass, CPU 2 affinity, warm OS cache and a fresh process/handle per sample.
Alternate ZIP/C/dense and dense/C/ZIP order. No owned build, fixture creation or
test overlaps timing. Record host load; do not claim cold-I/O or load isolation.
The common Rust runner computes paired log-ratio geometric means and fixed-seed
10,000-resample 95% intervals. Keep every sample and input inventory.

The common resource probe now uses one timing/copy/verification loop for both C
and the dense reader. Open and read CPU/wall time are separate; verification,
archive hashing and source reads occur outside timers. Process peak RSS is
reported alongside its pre-open baseline. It is not the incremental-above-empty
A5 memory qualification. The parent runner is native Rust, avoiding a Python
launcher high-water floor. Every sample separately opens and verifies all source
bytes; range samples additionally verify the requested range.

`compare-existing` admits matching C/dense corpus and protection options; only
ZIP may have reduced protection. It checks all source inventories before and
after sampling and pins archive/public-owner hashes. Dense images have retained
password wrappers while the C fixture has explicit-key access: wrapper validation
remains in dense normal-open time, while password derivation is outside this
explicit-key comparison. ZIP ranges omit full-entry CRC, as in the earlier batch.
Protected-vs-ZIP ratios are descriptive, not equivalent-security A3 claims.

## Admission and interpretation limits

These images still admit at most 1,024 files / 4,096 fragments and a 64 KiB decoded
catalogue. They do not implement directories, links, variables, forms, mirror
ownership, access mutation/overflow, a large metadata hierarchy or a public
migration path. A faster reader does not establish whole-format eligibility.
The batch is not A4's comparison against the same-protection production baseline,
a write/update benchmark or a complete A3/A5 workload matrix. Preserve existing
failed cases and do not extend sampling until a result happens to pass.

The new runner's tests check protection/corpus comparability as well as the
existing reproducible paired interval and chunk-independent corpus generator.
Both C and dense resource paths are smoke-tested with exact-byte verification;
those smoke timings are not included in the measurement batch.
