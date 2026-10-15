# First file-only A/B/C comparison

Candidate C is rejected in its current form for read performance and small-file
space use. This is architecture screening, not a selected or release-qualified
format. All emitted bytes were independently reopened and compared with source.

Source: `ab4e2bc6eb1f0387b4a6ebf2118b6675e032808d`. A is the bounded default writer;
B enables `native-block-layout`; C is the test-only file adapter. All three use
the same corrected encoder documented in the [correctness checkpoint](../candidate-file-extents-2026-09-27/README.md).
The source was clean and unchanged throughout this batch. Binary hashes and the
predeclared cases are retained. Each raw JSONL contains source/lockfile/runner
hashes, CPU/kernel/memory/load/affinity, corpus inventory hash and per-sample
verification, CPU, elapsed time, RSS and archive size. C also records its test
executable hash. No owned builds or tests overlapped measurement.

## Read results

Warm OS cache, fresh child and handle, one worker pinned to CPU 2, plaintext
unsigned and default padding. Thirty measured paired observations follow three
warm-ups; ordering rotates among ZIP, A/B and C. Ratios are elapsed open-and-read
(candidate divided by baseline), with paired bootstrap 95% intervals. Values
above one are slower. The 8 MiB cases use one file; small uses 512 × 4 KiB files.

| Case | C / ZIP | C / primary A or B |
| --- | --- | --- |
| compressed-c64-a | 24.452 [24.349, 24.547] | 5.087 [5.062, 5.113] |
| compressed-c256-a | 9.639 [9.487, 9.733] | 2.024 [2.015, 2.033] |
| compressed-c256-b | 9.619 [9.534, 9.680] | 1.878 [1.871, 1.883] |
| raw-c64-a | 17.024 [16.871, 17.184] | 1.715 [1.712, 1.718] |
| small-c256-a | 44.776 [44.494, 45.222] | 92.905 [92.544, 93.270] |

A itself remains slower than ZIP for the large cases: 4.76–4.81× for compressed
8 MiB and 9.92× for raw 8 MiB. B is about 5.12× on its compressed case. No measured
large-read candidate establishes the proposed ZIP gate. C's 256 KiB compressed
profile substantially improves on its 64 KiB profile but remains slower than A/B.
CPU ratios closely follow elapsed ratios; raw summaries retain both.

Archive bytes include padding and control state, before compaction (C has no
compaction implementation yet). Fixture creation order is ZIP, primary A/B, C.

| Case | ZIP bytes | Primary bytes | C bytes |
| --- | --- | --- | --- |
| compressed-c64-a | 110296 | 312704 | 9175040 |
| compressed-c256-a | 110296 | 312704 | 2883584 |
| compressed-c256-b | 110296 | 315776 | 2883584 |
| raw-c64-a | 8388736 | 8659328 | 9175040 |
| small-c256-a | 237078 | 1056128 | 34734080 |

The small-file C result is especially poor: about 93× A's read duration and
33.1 MiB stored versus A's 1.0 MiB and ZIP's 0.23 MiB. Small payloads are not packed;
each currently consumes a 64 KiB allocation. Repeated metadata lookups add a
separate CPU cost. These results cannot be averaged away by a large-file win.

## Large-file resource probe

One measured pair after three warm-ups; descriptive only, not an A4 confidence
gate. One 1 GiB seeded-random raw file, plaintext/default padding, fresh creation,
then independent verification. Trial archives are removed outside timing.

| Implementation | Elapsed seconds | CPU seconds | Peak RSS MiB |
| --- | --- | --- | --- |
| ZIP | 6.704 | 1.659 | 4.34 |
| A | 14.720 | 5.490 | 190.91 |
| C | 9.795 | 2.783 | 14.20 |

This supports bounded streaming as a useful direction. It does not establish
protected write parity, cold-I/O performance, 100k-file resources or full format
readiness. The runner's single-observation bootstrap endpoints are identical;
they are not a meaningful confidence interval.

## Limits and next experiment

C retains authenticated membership, encryption/signing modes, padding, erasure,
publication and preparation recovery. It still omits public key bootstrap,
permissions, non-file records, payload packing, mutation integration, independent
file salvage and compaction. Its public owner key is loaded outside timed open.
A/B perform their normal public bootstrap. Even favourable C numbers cannot yet
qualify equivalence or select the format. ZIP is unsigned/unencrypted and uses
Deflate for compressed cases; codec and protection costs are not interchangeable.
Cold-cache control, I/O/sync/decoded-byte accounting and protected cases remain.

The immediate implementation hypothesis is an ordered extent-index traversal per
range. The current adapter restarts authenticated index lookup for every extent,
repeatedly decoding the same metadata pages. Reuse the existing bounded range
walker, check exact contiguous ordinal membership, and preserve authentication
before delivery. Test page-read counts and corruption/error behaviour, then run
a separately declared paired comparison against this frozen binary. This does
not solve small-file packing or select a cache policy; those remain explicit
architecture work, not reasons to weaken the contract.

## Reproduction

From `rust`, build A with
`cargo bench -p revault_lockbox_api --features external-source --bench archive_evaluation --no-run`
and B with `--features native-block-layout,external-source`. Copy the respective
Cargo-reported executables before rebuilding another variant. Build C with
`cargo test -p revault_lockbox_api --release --features external-source --lib --no-run`.
Compile `revault_lockbox_api/benches/evaluation/candidate_driver.rs` with
`rustc --edition=2021 ... -o DRIVER`.

For each `batch.json` entry, set `REVAULT_CANDIDATE_TEST_BINARY=C_TEST_BINARY` and
`REVAULT_CANDIDATE_UNIT=unit`; run the indicated A/B executable under
`taskset -c 2` with arguments
`run NEW_ROOT files bytes corpus codec plain samples access 1 default DRIVER`.
Run `A_EXECUTABLE summarize NEW_ROOT/samples.jsonl` after completion. Retained
inventories and case descriptions reproduce sources without retaining GB payloads.
