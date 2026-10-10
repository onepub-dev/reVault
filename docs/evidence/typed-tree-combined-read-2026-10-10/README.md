# Combined typed-reader matched comparison

Source `ed4d6c0f`, Rust 1.88.0. This batch includes the accepted guarded-buffer
reuse and bounded fragment join. All six cases still fail ZIP parity; every
tree/ZIP total-time interval is above one. The paired opening diagnostics remain
evidence for their individual changes, but this separate batch does not isolate
their effect against the earlier full-read baseline.

## Protocol and results

The [previous fixed protocol](../typed-tree-matched-read-2026-10-10/README.md)
and original fixtures are retained: 30 measured groups plus three warmups per
case, fresh processes/handles, one pass, warm OS cache, CPU 2, alternating backend
order. Each group contains ZIP, C-default and typed tree. There are 540 measured
and 54 warmup observations. Every observation verifies content outside timing;
executable/source/fixture hashes remain stable. No owned builds or tests overlap
timing. Host load is recorded, not assumed idle. Fixed-seed 10,000-resample paired
log-ratio bootstrap produces the 95% intervals. There was no outcome-driven
resampling or pooling of cases.

All modes here are unsigned plaintext with default padding. ZIP uses Stored for
raw data and Deflate level 6 for compressed data; the candidate uses Zstd level 3
with raw fallback. Full ZIP reads check CRC; a raw range does not verify the full
entry CRC. Candidate checksum, ownership, padding and erasure checks are unchanged.
The small fixture's generator name contains `mixed`, but its 4 KiB files are
compressible patterned bytes.

| Case | Tree/ZIP total [95% CI] | Tree/C-default total [95% CI] | Tree median open/read/total/CPU (ms) | Worker median peak RSS (KiB) |
| --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible stream | 1.159 [1.084, 1.237] | 0.018 [0.017, 0.019] | 1.587 / 2.527 / 4.129 / 4.131 | 10432 |
| 8 MiB random raw stream | 3.359 [3.168, 3.552] | 0.980 [0.956, 1.003] | 0.786 / 5.824 / 6.611 / 6.615 | 9948 |
| 8 MiB patterned compressed stream | 2.600 [2.494, 2.724] | 0.666 [0.645, 0.689] | 0.757 / 3.917 / 4.683 / 4.685 | 10552 |
| 8 MiB raw midpoint 4 KiB | 9.716 [9.253, 10.188] | 0.901 [0.861, 0.944] | 0.833 / 0.056 / 0.887 / 0.888 | 9864 |
| 64 MiB random raw stream | 3.030 [2.841, 3.231] | 0.979 [0.966, 0.992] | 2.472 / 49.474 / 52.022 / 52.010 | 10324 |
| 64 MiB raw midpoint 4 KiB | 23.954 [23.439, 24.473] | 0.712 [0.698, 0.721] | 1.755 / 0.047 / 1.804 / 1.805 | 10332 |

Separate medians need not add exactly. Opening dominates range-read elapsed time;
payload processing dominates large streams. These are inferences from timing,
not a fresh function-level CPU profile. The historical SHA-256 profiles remain
historical. Investigate bounded authenticated index traversal for range opening
and obtain current payload attribution before selecting another stream change.
Do not remove required validation to satisfy a performance target.

## Setup failure, checks and provenance

The first launcher accidentally selected the dense adapter for the C/default
fixture. Every case failed before a complete matched group; only ZIP observations
were produced. The original attempt is retained under
`preparation-and-invalid-attempt/`, with a duplicate excluded subset in
`results/excluded-invalid-attempt/`. None contributes to the table. After the
configuration correction, an untimed smoke verified all three backends; those
observations are also excluded. The same predeclared six-case batch then ran
once to completion. This was correction of setup, not selection based on timing.

Post-format fragment-join (1), traversal (1), credential-open (2) tests and strict
core Clippy pass on this commit. Their exact logs and core/bench build output are
retained in `preparation-and-invalid-attempt/`. The corrected adapter selects the
default C path; only the typed adapter sets `FROZEN_TREE=1`. The summary's legacy
`tree_vs_packed_control_total` key denotes this C/default control.

See [metrics](results/metrics-summary.json), [final audit](results/final-audit.json),
[identities before](results/hash-before.json) and [after](results/hash-after.json),
[host before](results/host-before.json) and [after](results/host-after.json).
Every raw sample is retained under `results/measure/<case>/samples.jsonl`, with
executed build/batch/audit Dart scripts and setup logs. The original build-identity
file contains raw sha256sum output (digest plus filename) in its SHA fields;
the final audit normalizes and independently checks the executable digests.
Binaries and fixture payloads are excluded; their inventories and hashes are
retained. Archived scripts preserve absolute paths and are not installed tools.

Public CLI memory, million-entry scale, PGP and write comparisons, full aging,
migration and complete-format recovery remain unqualified. The two existing
native recovery failures remain unresolved. This experimental test-only reader
is not activated in the public CLI or bindings.
