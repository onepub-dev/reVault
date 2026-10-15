# Typed-tree open stage diagnostic

The six-case ZIP baseline failed, with opening dominating single range reads.
Host policy blocked perf, ptrace and kernel tracing; no host policy was changed.
The [frozen-binary fallback](profile-attempt/README.txt) repeats payload visits
1,000 times after one open. It does not profile repeated opens or attribute CPU
cost to functions. Its range verification scans the full archive outside timing.

The new test-only `open_observed` method shares the complete validation path used
by `TreeImage::open`. Ordinary opens supply a statically empty callback. The
resource probe's `REVAULT_TREE_OPEN_DIAGNOSTIC=100` records 100 warm opens per
fixture, verifies content independently before/after, and checks unchanged bytes.
Observation overhead is included; object drop, metadata-count assertion and
whole-file verification are excluded. Storage handles are reopened each time.
This is a diagnostic, not a ZIP comparison or a fresh-process improvement claim.

## Results

Pinned Rust 1.88.0, source `4db3b974` plus the retained [source patch](stages/source.diff).
The [execution report](stages/README.txt) records exact commands, identities and
checks. Median microseconds across 100 observations:

| Fixture | Handle | Traversal and reclaimed-space validation | Typed validation | Pack padding | Total |
| --- | --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible | 17.850 | 560.108 | 231.833 | 20.760 | 833.246 |
| 8 MiB random raw | 15.365 | 304.445 | 19.285 | 6.200 | 347.460 |
| 8 MiB patterned compressed | 15.455 | 249.139 | 4.471 | 5.941 | 276.149 |
| 64 MiB random raw | 17.455 | 1160.861 | 145.722 | 48.541 | 1375.154 |

Separate medians need not sum. Codec, value-key and owner-verification callback
stages each take below 0.2 microseconds in these plaintext unsigned fixtures;
this says nothing about protected-mode costs. Raw observations retain all stages.

Authenticated traversal dominates all four cases; typed validation is also
material for many small files. The traversal stage includes metadata decoding,
ownership graph construction and reclaimed-space validation, so it cannot yet
identify one function as the bottleneck. No checksum, owner, padding, wiping or
verification-before-exposure requirement was removed.

Focused single-traversal (one test), credential-open (two tests), strict core
Clippy and the optimized test build pass before formatting. Every repeated-open
case verifies, and fixture/source hashes remain unchanged. The separate frozen
diagnostic binary is identified in the report; binaries and payloads are excluded
from the retained repository evidence. This does not qualify the complete format.
