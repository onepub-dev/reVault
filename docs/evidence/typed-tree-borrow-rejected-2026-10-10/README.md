# Rejected plaintext index borrowing experiment

Decision: **reverted**. Removing the duplicate plaintext index body allocation
passed correctness checks but regressed every predeclared open-time case. The
implementation remains the committed `65d2d845` reader; only evidence is retained.

The [exact rejected patch](raw/source.patch) borrowed plaintext from the existing
verified page and kept encrypted decoded storage owned and wiped. It changed no
checksum, owner, padding or graph-validation rule. A smaller allocation count did
not translate into faster measured opening; this experiment does not identify the
machine-level cause of the slowdown.

## Fixed comparison

Rust 1.88.0, CPU 2, same fixtures/harness, immutable before/after binaries; four
cases with three warmup and 30 measured process pairs each. Each process performs
100 warm opens, with independent full-content checks before/after. The paired
unit is its mean elapsed time per open, not each nested observation. Order
alternates; a fixed-seed 10,000-resample log-ratio bootstrap supplies the intervals.
Observation overhead is included. No adaptive reruns or overlapping owned builds.

| Fixture | Candidate/control [95% CI] | Decision |
| --- | --- | --- |
| 512 × 4 KiB compressible | 1.02285 [1.01977, 1.02710] | Regression |
| 8 MiB random raw | 1.01816 [1.01449, 1.02282] | Regression |
| 8 MiB patterned compressed | 1.01696 [1.01212, 1.02206] | Regression |
| 64 MiB random raw | 1.03540 [1.02620, 1.05185] | Regression |

All 264 processes verified; all 26,400 nested opens completed. This includes
warmups. Binary, source and fixture hashes stayed unchanged through measurement.
These diagnostic ratios do not change the separate failed ZIP baseline.

## Validation and provenance

The full authenticated-index module passes 14 tests with one ignored resource
probe; the focused traversal and two credential-opening tests and strict Clippy
also pass. An initial incorrectly named test filter matched zero tests and was
corrected before the experiment. Baseline post-format checks pass independently.

The [runner report](raw/README.txt), [paired summary](raw/paired/summary.json),
exact patch, Dart orchestration, build/test logs, host context and frozen identities
are retained under `raw/`. Each `raw/paired/<case>/process-samples.jsonl.gz` holds
all raw stage observations and process means, losslessly compressed and checked
byte-for-byte after decompression during retention. Duplicate stdout/stderr,
fixture payloads and executables are excluded. Original temporary artifact paths
in reports are provenance, not portable paths to the retained copies.

Next: separate metadata traversal, ownership construction and reclaimed-space
validation in the diagnostic before selecting another optimization. Complete
format, public API, scale, native recovery and ZIP/PGP qualification remain open.
