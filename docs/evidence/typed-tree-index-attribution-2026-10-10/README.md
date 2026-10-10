# Authenticated traversal function attribution

Temporary test-only instrumentation on `0fd6ae24` measures verified index-page
reads, node decoding and visitor callbacks. A nested timer measures SHA-256
checks inside RootRef reads. The full [patch](raw/source.patch) and timing module
are retained, but all five instrumented source files have been restored/removed
after checking their hashes against the passing audit. Normal source is exactly
the validated baseline. No profiling overhead is retained in the reader.

## Method and result

Rust 1.88.0 optimized external-source test binary, CPU 2, warm OS cache. One
process per existing fixture, 100 fresh handles/opens each. Full contents verify
before and after the loop; all 400 observations and source/fixture identities
pass. Tests and builds complete before timing. This is one instrumented version,
not a paired speedup experiment, statistical comparison or ZIP qualification.
Both the prior storage meter and new function timers add overhead. Their values
must not be compared directly with earlier uninstrumented times as regressions.

| Fixture | Index stage median (µs) | Verified page calls / µs | Decode calls / µs | Visitor calls / µs | Nested checksum calls / µs |
| --- | --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible | 438.5 | 3 / 117.0 | 3 / 126.5 | 1030 / 144.7 | 3 / 93.0 |
| 8 MiB random raw | 160.3 | 1 / 38.6 | 1 / 60.6 | 387 / 42.9 | 1 / 31.0 |
| 8 MiB compressed pattern | 94.4 | 1 / 38.4 | 1 / 45.7 | 37 / 5.6 | 1 / 30.8 |
| 64 MiB random raw | 965.3 | 6 / 234.0 | 6 / 247.8 | 3080 / 345.6 | 6 / 186.0 |

Verified page time includes storage length/read operations and the nested root
checksum timer; never add that checksum column again. Decode includes body
copy/decryption, entry allocation and structural validation. Visitor time
includes page/ownership collection and typed record decoding/staging, including
callback-owned value disposal. It is not just a file-content callback. Unmeasured
traversal, relationship checks, timer/counter bookkeeping and stage-JSON work
remain outside these function subtotals. Medians need not add to stage medians.
Failure counters record Result errors, not a false checksum comparison; this run
had no errors or damage and is not a corruption-path timing qualification.

These observations support investigating per-record decoding/visitor allocations
before another tiny scan change. For raw 64 MiB the three disjoint function
groups take roughly 234, 248 and 346 microseconds respectively. Checksums remain
mandatory. The [failed fused-leaf trial](../typed-tree-leaf-validation-rejected-2026-10-10/README.md)
and earlier body-borrow failure remain rejected. A borrowed record view would be
a new hypothesis: it must preserve complete page validation before exposure,
ordered membership/count/upper-bound checks, error cleanup and wipe lifetimes;
it must not weaken whole-content verification or silently change ordinary read
semantics. No such reader is implemented or accepted by this diagnostic.

## Evidence and limits

Strict core Clippy, 14 authenticated-index tests (one ignored manual probe), and
single traversal across 16 modes pass. [Final audit](raw/final-audit.json) contains
exact source/patch/binary identities and all stage summaries. Each per-fixture
JSON retains every observation, including storage and nested function metrics.
Executed Dart scripts, commands, build/check logs and host context are retained.
The independent baseline binaries listed in provenance were not timed here.
Binaries, fixture payloads and duplicate stdout/stderr are excluded. The archived
timing-module source has `.txt` appended to preserve its measured bytes through
the Rust commit hook. No post-format instrumented result is claimed.

The full-read ZIP scorecard still fails all six cases. Public integration, scale,
recovery, migration and complete-format qualification remain outstanding.
