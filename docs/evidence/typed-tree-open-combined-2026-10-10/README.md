# Opening stages after the combined improvements

The frozen `ed4d6c0f` core binary is audited against the Rust source at `83ab5c02`;
the intervening commit contains only documentation/evidence. Each of four existing
fixtures runs once on CPU 2, with 100 fresh handles/opens in one process and warm
OS cache. Full payload bytes verify before and after each loop. All source,
executable and fixture identities remain stable. No owned builds/tests overlap.

This is a single-version diagnostic with observer overhead. It does not compare
versions, measure full reads or qualify ZIP parity. Separate stage medians need
not sum to the median total. The following values are microseconds; percentages
divide each stage median by the total median, rather than averaging per-open
shares. All 400 raw observations are retained in the per-case JSON files.

| Fixture | Total | Publication | Index walk/decode | Ownership graph | Reclaimed space | Typed validation | Pack padding |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible | 688.5 | 83.9 (12.2%) | 342.4 (49.7%) | 1.4 (0.2%) | 42.1 (6.1%) | 187.1 (27.2%) | 20.4 (3.0%) |
| 8 MiB random raw | 277.1 | 83.1 (30.0%) | 110.7 (39.9%) | 13.0 (4.7%) | 41.4 (14.9%) | 12.0 (4.3%) | 6.4 (2.3%) |
| 8 MiB compressed pattern | 210.7 | 82.9 (39.3%) | 70.4 (33.4%) | 0.8 (0.4%) | 39.9 (18.9%) | 2.7 (1.3%) | 5.8 (2.7%) |
| 64 MiB random raw | 1178.0 | 83.4 (7.1%) | 755.4 (64.1%) | 149.9 (12.7%) | 48.7 (4.1%) | 79.6 (6.8%) | 50.1 (4.3%) |

The [summary](raw/summary.json) includes the smaller stages and exact values.
[Final audit](raw/final-audit.json) independently normalizes/checks the executable
digest; original hash records retain the initial parser's sha256sum filename
suffix. The executed Dart runners are archived as `.dart.txt`. Per-case JSON
contains the observations and verification results; duplicate stdout/stderr,
binaries and fixture payloads are excluded.

## Source interpretation and next experiment

`index_walk_and_decode` includes authenticated page traversal, typed visitor
decoding and fragment staging. The final fragment-to-file join and semantic
checks occur in `typed_validation`. Each index page invokes `read_verified`,
which checks storage length before each primary/mirror attempt. These are source
observations, not attribution of elapsed time to individual functions/syscalls.

The full graph and catalogue checks still precede exposure; signed plaintext
also retains whole-content owner verification during open. Skipping unrelated
checks or making verification lazy would change detection timing and is outside
this performance experiment. Raw payload checksum cost is not measured here.

Ownership construction is 12.7% of the 64 MiB opening path. Its insertion helper
performs predecessor/successor overlap lookups before an existing final exact
coverage pass. A bounded follow-up will retain immediate length/bounds/count and
duplicate-start checks, and rely on final sorted exact coverage to reject every
gap or overlap before exposing a graph. Test crossing/nested/equal-start/equal-end
claims in both orders, then compare the same four fixed paired opening cases.
Retain it only if correctness and measured results justify it; no benefit is yet
established by this diagnostic. Authenticated traversal remains the larger cost.
