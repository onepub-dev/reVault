# Detailed typed-tree opening stages

This refines the test-only observer without changing the normal validation path.
The shared tree reports selected-publication opening, index walking/decoding,
ownership graph construction and reclaimed-space verification separately. The
ordinary reader, salvage and update paths use no-op callbacks where appropriate.
The rejected plaintext-copy experiment remains reverted.

## Observation

Source `7abbf536` plus the retained [patch](raw/source.patch), Rust 1.88.0, CPU 2,
100 warm opens per fixture with independent whole-content checks before/after.
Every case verifies; executable/source/fixture hashes remain stable. Observer
overhead is included, while drop and whole-content verification are excluded.
This is a single-version diagnostic, not a comparison with an earlier binary or ZIP.

Median microseconds:

| Fixture | Publication | Index walk/decode | Ownership | Reclaimed space | Typed validation | Pack padding | Total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible | 105.392 | 525.269 | 2.290 | 106.197 | 360.926 | 24.686 | 1141.004 |
| 8 MiB random raw | 85.442 | 120.907 | 11.666 | 74.781 | 19.361 | 6.451 | 329.635 |
| 8 MiB patterned compressed | 83.817 | 71.922 | 0.740 | 76.087 | 4.375 | 5.911 | 252.674 |
| 64 MiB random raw | 84.291 | 820.733 | 142.808 | 78.031 | 147.887 | 50.411 | 1337.027 |

The [runner report](raw/README.txt) includes handle timing and exact commands;
raw JSON retains every stage. Separate medians need not sum. Timing differences
from earlier diagnostic binaries are not optimization evidence. Index walking
includes authenticated page reads, decoding and staged typed-record visitation;
these timings are not a function-level CPU profile.

Index walking is the largest stage in the 64 MiB case. Publication and reclaimed
space checks together account for nearly half of the 8 MiB raw opening time.
A next candidate is bounded guarded-buffer reuse during free-space checks,
subject to external-source semantics, erasure/failure tests and a fixed paired
comparison. No buffer policy has changed or gain been established here.

## Validation

The shared-tree module passes 17 tests with one ignored resource observation,
including corruption, copy loss, ownership/recovery and interruption. The focused
single-traversal test, two credential-open tests, strict core Clippy and optimized
test build pass. An initially missed no-op callback failed compilation and was
fixed before these checks; its error is retained. A local Dart driver syntax error
was corrected before any fixture measurement began.

Logs, source patch, scripts, frozen identities, host context and raw observations
are retained under `raw/`; binaries and fixture payloads are excluded. Complete
format qualification, ZIP/PGP goals, public activation and native recovery remain
outstanding. No owner, digest, padding, erasure or admission check was weakened.
