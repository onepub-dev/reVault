# Bounded construction and disk-backed extraction — 2026-10-10

Both previously failing capacity cases now create successfully and pass independent
reopening and exact source-byte verification: one 1 GiB random file and 10,000
files of 64 KiB each (625 MiB). Earlier [failed attempts](../large-capacity-2026-10-10/README.md)
remain retained. No fixture was reduced to obtain a passing result.

## Implementation

Fresh export preflights authenticated source metadata before retaining its audit
and repacking collections. The experimental typed catalogue uses a 24 MiB
admission budget, tree traversal staging 4 MiB and physical ownership graph
8 MiB. Conservative per-record charges include vector slack, maps and temporary
joins; paths have separate charges. These modeled 36 MiB ceilings are not a
measurement of total process RSS or a replacement for its acceptance gate.
Variables and forms use the same reader/writer accounting, with refusal before
publication. Dense-inline format limits remain unchanged.

Fresh index construction streams namespace/key-ordered records into the existing
index builder. It removes the second full serialized record vector and sort;
only a file-ID permutation and the builder's page allocation receipts remain.
Construction metadata is dropped before the independent final full audit.
All source authority, fragment authentication, descriptor binding, padding,
ownership, full payload verification and destination cleanup remain intact.
No dependency, unsafe block or wire encoding is added or changed.

## Fixed extraction comparison

| Workload | Lockbox seconds | ZIP seconds | Lockbox / ZIP [95% CI] | Lockbox MiB/s | Peak worker RSS, Lockbox / ZIP (MiB) |
| --- | ---: | ---: | --- | ---: | ---: |
| raw-random-1gib | 1.475 | 0.986 | 1.475 [1.402, 1.546] | 694.413 | 10.246 / 8.098 |
| raw-mixed-10000x64k | 1.453 | 0.894 | 1.603 [1.535, 1.669] | 430.045 | 10.613 / 10.285 |


Both paired confidence intervals are entirely above 1.0: Lockbox takes about
47% longer for the 1 GiB case and 60% longer for the many-file case. Neither
passes ZIP parity. The construction+verification workers peak at 25,076 and
25,048 KiB RSS respectively (about 24.5 MiB); these are capacity/resource
observations, not comparable creation-performance samples.

Seconds and throughput are medians; the ratio is the paired geometric mean with
a fixed-seed, 10,000-resample 95% bootstrap interval. There are three excluded
warm-up pairs and 30 measured alternating ZIP/Lockbox pairs for each workload,
one CPU worker pinned to CPU 2, fresh child/handle/output directory every time.
Every output file is reopened and compared byte-for-byte with its source outside
the timed/resource snapshot. Source inventories, ZIP/tree archives and executable
identities are checked before and after the batch. Raw samples, logs, commands,
provenance and [summary including CPU time](raw/metrics-summary.json) are retained.

This measures actual filesystem writes/flush/close, including output-directory
creation, with **no fsync durability guarantee**. It is warm-cache, disk-backed
Btrfs with host compression enabled, not controlled cold-device I/O. The ZIP
reader reaches EOF and checks CRC; Lockbox retains selected metadata and stored
fragment authentication. Both use raw, unsigned, unencrypted archive contents.
CPU figures are process CPU time, not all background filesystem work. Peak RSS
is the maximum observed GNU-time child-process high-water mark, including
verification; the inner pre-verification resource snapshots are also retained.

The retained many-file generator was named `mixed`, but alternates pattern/random
only every 256 KiB within each file: these 64 KiB files are patterned. This is
a many-file workload, not evidence about broadly mixed real-world contents.
The random 1 GiB fixture remains incompressible at the archive layer.

## Correctness checks and scope

Strict Clippy passes. Release checks cover metadata budget overflow/refusal,
fresh export beyond the old file/fragment limits, source/destination refusal and
cleanup, independent reopened payload corruption, physical ownership, selective
reads, malformed fragment joins, forms, refusal before publication and extraction
verification/overwrite protection. The shared benchmark sink is tested for wrong
bytes despite a correct byte count. Evaluation integration checks also pass.
The initial visibility/benchmark-helper lint errors and a zero-match filter are
retained; corrected nonzero checks passed before measuring. No full repository
suite or public CLI qualification is claimed.

Source packed-image construction still retains metadata rows; repacking and full
audits still use bounded inventories. Mutation/compaction retain narrower
operation limits and materialized rows. TB-scale/million-entry support needs
further streaming work. The measured endpoints are test-only v4 components;
they do not activate public v4, pass the 100 MB total CLI RSS gate, establish
encrypted/PGP performance or qualify release readiness. These larger cases now
provide practical capacity and extraction evidence for the next optimization.

Post-format verification of commit `4544ec69` also passed: strict Clippy,
metadata budgets (2 tests), fresh export (8 tests; one manual probe ignored),
extraction (6 tests) and evaluation integration (6 tests). These used Rust 1.88.0
with `external-source`; tests used release mode. The [post-format logs](raw/postformat/clippy.log)
are retained alongside the original pre-format benchmark evidence.
