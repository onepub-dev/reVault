# Storage costs within typed opening

This diagnostic refines [the current opening stages](../typed-tree-open-combined-2026-10-10/README.md).
Only the existing test resource probe wraps its file storage with a read-only
meter; ordinary readers do not use it. Every operation delegates to the same
file backend and propagates its result. The meter records lengths, reads,
read-into calls, requested bytes, failures and elapsed storage-operation time,
resetting counters at each existing opening-stage boundary. It prints no content.

The new observation field is `storage_by_stage`. Read time includes the backend's
buffer allocation/copy/I/O; length time includes locking and metadata lookup.
Reported storage durations exclude the meter's counter/JSON bookkeeping, whereas
stage and total durations include instrumentation overhead. Timing a storage
call does not identify system-call or kernel time separately. Other work is a
residual, not a measured pure CPU or decoder subtotal.

## Fixed diagnostic

Baseline source `1bcf7b34` plus the [full patch](raw/source.patch), including the
new meter module. Rust 1.88.0, CPU 2, warm OS cache, four existing fixtures, one
process per fixture with 100 fresh handles/opens. Full contents verify before and
after every loop; metadata checks/drop occur outside the opening timer. All 400
opens succeed; all measured storage operations succeed; source, binary and
fixture hashes remain stable. Tests and builds finish before timing. There is
no paired version comparison, bootstrap speedup claim or adaptive resampling.

| Fixture | Index stage median (µs) | Length / read calls | Requested bytes | Storage length / read median (µs) | Residual from medians (µs) |
| --- | --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible | 483.8 | 3 / 3 | 196608 | 2.72 / 26.45 | 454.6 |
| 8 MiB random raw | 120.5 | 1 / 1 | 65536 | 0.63 / 6.63 | 113.2 |
| 8 MiB compressed pattern | 80.1 | 1 / 1 | 65536 | 0.64 / 6.83 | 72.7 |
| 64 MiB random raw | 751.9 | 6 / 6 | 393216 | 4.31 / 41.69 | 705.9 |

Residual subtracts median length-call and read-call time from median stage
time. It is not the median of per-observation residuals; rounding may differ. Index stages have no read-into calls. Each
fixture's publication selection makes three length and five read calls; reclaimed
space makes two read-into calls totaling 8192 bytes. Padding reads occur for the
small and compressed fixtures. Full per-stage counts and timings are retained.

Length queries consume less than 1% of index-stage time; reads consume about
6–9%. These observations do not justify caching away current length checks.
Most index-stage time remains in checksum verification, decoding, typed visitor
staging and instrumentation. Next separate those costs or test bounded reductions
in per-record allocations, preserving every checksum and validation check.
The earlier plaintext body-borrow experiment regressed and remains rejected;
do not repeat it based on this residual. No new ZIP result follows from this run.

## Verification and evidence

Ownership tests (9), single traversal (1), strict core Clippy and optimized build
pass. The [final audit](raw/final-audit.json) identifies the frozen candidate and
all before/after source/fixture identities. The baseline executable is retained
only as provenance and was not timed in this single-version experiment.
Each per-fixture JSON contains all 100 observations and the verification result.
Executed Dart scripts, source patch, build/check logs and host context are retained;
duplicate stdout/stderr, executable binaries and fixture payloads are excluded.

The wrapper's read-only mutation methods refuse by panicking; the ordinary
write path never uses it. Normal and observed opens retain the same verification
logic. The diagnostic itself is not a public storage adapter, security
qualification, cold-I/O measurement or public CLI memory measurement. The latest
full-read comparison still fails all six ZIP cases; full-format work remains open.

Post-format strict Clippy and single-traversal checks pass on `3a4db99c`.
One-open smoke checks for all four fixtures retain `storage_by_stage`, report no
failed storage calls, and verify full contents. Evidence is in `raw/postformat/`;
these smoke observations are excluded from the 100-open diagnostic above.
