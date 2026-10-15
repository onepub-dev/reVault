# Guarded buffer reuse for reclaimed-space checks

The experimental ownership verifier now reuses one guarded buffer for an entire
free/pending-space check. Its data capacity is the largest eligible chunk, capped
at 64 KiB. Previously each chunk allocated its own guarded buffer, then entered a
separate read scope to inspect it. The new path reads and checks within one mutable
guard, retains no verified nonzero contents, and drops/wipes the buffer on every
return path. Empty selections allocate no buffer.

Every selected byte is still checked; pending space is included only for the
reclaimed-space operation. No checksum, owner, padding, ownership, admission or
erasure policy changes. Inspection confirms `ExternalStorage::read_at_into` still
enforces bounds, cumulative read budgets, short-read refusal and latched errors;
the old secure-read method also delegated to it under a mutable guard.

## Fixed paired diagnostic

Baseline `866c9aa9`, candidate plus the [retained patch](raw/source.patch), Rust
1.88.0, CPU 2. Each of four file-backed fixtures has three warmup pairs and 30
measured pairs, alternating order. Each fresh process performs 100 warm opens.
The paired unit is mean elapsed time per open within the process; observation
overhead is included. Full contents verify independently before/after each loop.
The fixed-seed 10,000-resample paired log-ratio bootstrap reports 95% intervals.

| Fixture | Candidate/control [95% CI] | Mean control/candidate time per open (ms) |
| --- | --- | --- |
| 512 × 4 KiB compressible | 0.97716 [0.97383, 0.98169] | 0.793785 / 0.775700 |
| 8 MiB random raw | 0.91894 [0.90673, 0.93512] | 0.328176 / 0.301770 |
| 8 MiB patterned compressed | 0.90462 [0.88847, 0.92198] | 0.252928 / 0.228882 |
| 64 MiB random raw | 0.98548 [0.98258, 0.98794] | 1.338895 / 1.319432 |

All four intervals show faster opening: about 2.3%, 8.1%, 9.5% and 1.5% less time,
respectively. All 264 processes and 26,400 opens (including warmups) completed;
content checks passed, frozen identities remained stable, and no owned builds or
tests overlapped timing. No adaptive reruns. This supports retaining the bounded
reuse change, not a ZIP-parity, whole-read or public-CLI speedup claim. The separate
six-case ZIP baseline remains failed; it predates this scoped optimization.

## Validation and limits

Eight ownership tests pass, including a new regression for free and pending chunk
boundaries, short tails, truncation and successful retries after failures. The
single-traversal test, two credential-open tests, 15 external-source integration
controls, strict core Clippy and optimized build pass. The external-source controls
do not constitute a complete public typed-tree remote-read qualification. Baseline
post-format focused checks and Clippy are retained separately.

The [runner report](raw/README.txt), [paired summary](raw/paired/summary.json),
exact source patch, scripts, test/build logs, host context and frozen identities
are retained. Each `raw/paired/<case>/process-samples.jsonl.gz` contains every raw
stage observation and process mean; retention verified lossless decompression
byte-for-byte. Duplicate stdout/stderr, executables and fixture payloads are
excluded. [Launcher failures](raw/launcher-failures.log) reconstruct tool output
from failed no-sample starts; the successful invocation uses the existing Dart
SDK directly via per-process PATH and CI=true, without changing SDK settings.

Index walking/decoding remains the largest 64 MiB opening stage. Scale, full public
integration, ZIP/PGP read/write comparison, complete-format recovery/aging and the
two known native recovery failures remain outstanding. This is still test-only.

Post-format validation on committed `d8cf287e` passes all eight ownership tests,
the focused traversal test, both credential-open tests and strict Clippy. Logs
are in `raw/postformat/`. Timing was not repeated after whitespace-only hook edits;
the reported experiment retains its exact pre-format source patch and identities.
