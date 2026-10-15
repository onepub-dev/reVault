# Rejected fused leaf-validation experiment

The candidate checked leaf ordering while parsing and used consumed body bytes
for the logical-size limit instead of rescanning the completed leaf in
`validate_node`. Record representation, zeroization, count/length/padding checks
and branch validation were preserved. Correctness checks passed, but measured
raw opening regressed. The source and experiment-specific tests were reverted
to `9fcf3cb0`; the [exact patch](raw/source.patch) is retained only as evidence.
Its SHA-256 was checked against the working diff before restoration:
`a492dee45a5c8d9776272aa27472903f9a72e4123c8180ab9f69ae1c3d8f06c3`.

## Fixed paired outcome

The clean baseline and candidate were built with Rust 1.88.0, release and
external-source enabled. Both include the same storage-meter diagnostic
instrumentation. Four existing fixtures, CPU 2, three warmup and 30 measured
pairs each, alternating order, 100 warm fresh-handle opens per process. The paired
unit is mean opening time per process; ratios are geometric paired means with
fixed-seed 10,000-resample log-ratio bootstrap 95% intervals. All 264 processes
and 26,400 opens, including warmups, verify content and retain input identities.
No adaptive reruns or pooling; owned builds/tests completed before timing.

| Fixture | After/before [95% CI] | Interpretation |
| --- | --- | --- |
| 512 × 4 KiB compressible | 1.00864 [0.99888, 1.02286] | Inconclusive; point estimate slower |
| 8 MiB random raw | 1.01380 [1.00882, 1.01938] | 1.4% regression |
| 8 MiB compressed pattern | 0.99935 [0.99295, 1.00444] | Inconclusive |
| 64 MiB random raw | 1.00696 [1.00459, 1.00901] | 0.7% regression |

Only the two raw cases establish regressions at these intervals. A slower point
estimate alone does not establish a small-file regression. There is no supported
performance reason to retain this change. This diagnostic is not a production,
whole-read, memory or ZIP qualification. Instrumentation overhead is present in
both controls and does not establish uninstrumented behavior.

## Checks and provenance

The authenticated-index suite passes 15 tests with one ignored manual probe,
including the added malformed-leaf test. It rejects duplicate/reversed keys,
namespace disorder, wrong counts and logical body overflow that still fits the
physical page. Private malformed records are needed because public CLI and index
encoder paths cannot construct them. Focused single traversal (all 16 modes),
fragment join, both credential-open tests and strict core Clippy pass.

The first runner launches failed to connect to Dart's resident frontend compiler;
the Flutter wrapper also encountered its known read-only cache issue. No timing
had started. Running the existing SDK launcher from `/tmp` with scoped escalation
resolved the setup problem without SDK/settings changes. These facts summarize
tool reports; complete failed-launch stderr was not captured in the retained
runner logs. The fixed batch then ran once.

[Paired summary](raw/paired/summary.json), before/after hashes, host context,
build/test logs and executed Dart runners are retained. Each per-fixture
`process-samples.jsonl.gz` contains all 100 stage observations per process with
warmup/pair labels; lossless compression was verified byte-for-byte during
retention. Binaries, fixture payloads and duplicate stdout/stderr are excluded.
Restoration returns both edited Rust files exactly to the already validated
baseline; no post-format candidate claim is made because it was not retained.

The storage attribution still leaves checksum, decoding and typed staging as the
next cost categories to separate. Do not repeat this leaf-scan or the earlier
body-borrow change without a new hypothesis and fixed comparison. The accepted
graph/fragment/buffer optimizations remain. Full ZIP parity and complete-format
qualification are still unmet.
