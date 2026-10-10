# Rejected path-buffer ownership move

The two-line candidate moved the already-owned zeroizing path buffer into staged
file/directory records instead of cloning it. It preserved representation,
validation and error cleanup, removing one allocation/copy per path. All focused
checks passed, but three measured workloads regressed. The source was restored
exactly to `f99f4e3f`; the [patch](raw/source.patch) is retained as evidence only.
Its hash matched the working diff before restoration.

## Fixed paired result

Source `f99f4e3f` is Rust-identical to the frozen `9fcf3cb0` baseline. Rust 1.88.0,
optimized external-source test binaries, CPU 2, warm cache, four existing fixtures,
three warmup and 30 measured pairs, alternating before/after order. Each fresh
process performs 100 fresh-handle opens. Both binaries include the same storage
meter instrumentation; these results do not qualify uninstrumented public reads.
The paired unit is mean opening time per process; fixed-seed 10,000-resample
paired log-ratio bootstrap yields the 95% intervals. All 264 processes verify
content; source/executable/fixture identities stay stable. No adaptive reruns,
pooled cases or owned builds/tests during timing.

| Fixture | After/before [95% CI] | Outcome |
| --- | --- | --- |
| 512 × 4 KiB compressible | 0.98142 [0.97984, 0.98303] | 1.9% faster |
| 8 MiB random raw | 1.02354 [1.02024, 1.02673] | 2.4% slower |
| 8 MiB compressed pattern | 1.03161 [1.02456, 1.03813] | 3.2% slower |
| 64 MiB random raw | 1.01508 [1.00957, 1.02046] | 1.5% slower |

Retaining only the favorable small-file result would conceal the three clear
regressions. The experiment is rejected. It proves neither that moving a buffer
is intrinsically slower nor why these binaries differ; compiler layout and
instrumentation effects are unmeasured. The allocation elimination is evident
from source, but does not establish a whole-reader performance gain.

Filesystem lifecycle, growth/deletion/permissions/copy-loss checks across all 16
modes pass, as do traversal across 16 modes, fragment join, both credential-open
tests and strict core Clippy. No new test duplicates the two-line implementation;
existing lifecycle checks independently validate persisted content and failures.

The [paired summary](raw/paired/summary.json), frozen identities, build/check logs,
executed Dart scripts and all raw observations are retained. Each per-fixture
`process-samples.jsonl.gz` contains 66 process records with 100 observations each;
lossless compression was checked byte-for-byte during retention. Binaries,
fixture payloads and duplicate stdout/stderr are excluded. No candidate formatting
or post-format qualification is claimed because the candidate was reverted.

The accepted buffer/fragment/graph improvements remain in the restored reader.
The [function attribution](../typed-tree-index-attribution-2026-10-10/README.md)
supports a more substantial decoder-to-visitor allocation investigation. A future
borrowed-record design must preserve page validation before exposure, membership
bounds, complete graph checks and wipe lifetimes. This experiment does not select
that design. Full ZIP parity and complete-format qualification remain unmet.
