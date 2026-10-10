# Ownership graph coverage construction

The [current stage diagnostic](../typed-tree-open-combined-2026-10-10/README.md)
attributes 12.7% of raw 64 MiB opening to graph construction. Its private staging
helper now uses one BTreeMap entry lookup instead of two interval searches plus
insertion. It still checks nonzero length, checked end/bounds, the unchanged
8,192-claim cap and duplicate starts before insertion. A duplicate never replaces
the existing claim. All descendant/control-region constraints remain unchanged.

The final sorted coverage pass, which already ran before this change, requires
the first claim to start at zero, each following claim to start at the preceding
end, and the final end to equal the sealed length. With positive bounded lengths,
this rejects every gap and overlap. No graph is returned before that pass. The
helper is renamed `stage_claim` to make its construction-only role explicit;
all callers are inside `derive_with_descendants`. Insertion order may change
which error wins for multiply malformed input, but cannot expose an invalid graph.

## Fixed paired opening result

Before: frozen `ed4d6c0f` binary, identical Rust source to baseline `83ab5c02`.
After: baseline plus the [exact two-file patch](raw/source.patch). Rust 1.88.0,
CPU 2, four existing fixtures, three warmup and 30 measured pairs each,
alternating before/after order. Each fresh process performs 100 warm opens with
observer overhead; its arithmetic mean opening time is the paired unit.
Fixed-seed 10,000 paired log-ratio bootstrap; no adaptive reruns or pooled cases.
All 264 processes and 26,400 observed opens, including warmups, verify content
and preserve frozen identities. Tests/builds finish before timing.

| Fixture | After/before [95% CI] | Mean before/after opening (ms) |
| --- | --- | --- |
| small512-mixed-compressed | 0.996027 [0.991556, 0.999976] | 0.692256 / 0.689458 |
| raw8m-random | 0.965141 [0.956236, 0.970881] | 0.280294 / 0.270458 |
| compressed8m-pattern | 1.004392 [0.987909, 1.023710] | 0.215151 / 0.216215 |
| raw64m-random | 0.927442 [0.920395, 0.937474] | 1.190081 / 1.104057 |

Retain the change for measured raw 8 MiB and 64 MiB opening reductions of 3.5%
and 7.3%. The small-file effect is about 0.4%, with its upper bound only narrowly
below one. The compressed case is inconclusive: its interval includes both a
1.2% gain and a 2.4% regression. This is not proof of compressed nonregression,
ZIP parity, whole-read improvement or reduced public CLI memory.

## Validation and retained evidence

Nine ownership tests pass. The added regression rejects crossing and nested
claims, equal starts/ends, gaps and overruns in both insertion orders; accepts
adjacent intervals in either order; and checks that duplicate insertion cannot
overwrite an existing claim. Private fixtures are necessary because the public
CLI cannot create this experimental representation. Existing tests cover control
aliasing, descendant ownership, retirement, erasure and transition constraints.
The fragment-join test, single-traversal test, two credential tests, 17 shared-tree
tests (one manual probe ignored), and strict core Clippy pass before formatting.

The [summary](raw/paired/summary.json), frozen source/executable/fixture identities,
build/test logs and executed Dart scripts are retained. Each per-case
`process-samples.jsonl.gz` contains all observations, losslessly compressed and
verified byte-for-byte during retention. Duplicate stdout/stderr, binaries and
fixture payloads are excluded. This test-only change does not activate v4.

Authenticated traversal remains the dominant raw 64 MiB opening stage. The latest
[full-read comparison](../typed-tree-combined-read-2026-10-10/README.md) predates
this change and still fails all six ZIP cases. No new full-read result is claimed.
Public integration, scale, migration, recovery and full qualification remain open.
