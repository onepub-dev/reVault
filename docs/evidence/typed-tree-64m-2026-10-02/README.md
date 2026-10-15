# 64 MiB transition-built typed-tree comparison

Date: 2026-10-02. Uses the same qualified core as the
[pinned read/aging batch](../typed-tree-pinned88-2026-10-02/README.md), with only
an isolated harness construction path added. It does not change staging limits,
the format, public APIs or migration policy.

The [fresh-export attempt](../typed-tree-single-pass-2026-10-02/large-raw64m-create.log)
failed at its intermediate dense catalogue. This fixture instead starts with an
empty typed tree and runs one already-qualified authenticated update with the
exact retained 64 MiB seeded source. It succeeds within the unchanged 64 MiB
source/staging caps. Independent stream/range byte checks and normal ownership/
reclaimed-space verification pass. The resulting archive is 68,288,512 bytes;
[construction metadata](tree-construction.json), source/control hashes, core
verification, compiler identity and [harness source hashes](transition-harness-source.sha256)
are preserved. Harness sources are copied under [harness/](harness/).

The retained control is the post-compaction packed-C artifact under
`/tmp/revault-compaction-resources-20260927/raw64m`. A read-only cost-model audit
confirms both C and tree use 1,024 raw fragments of 65,536 logical bytes; the
262,144-byte creation hint is not their raw fragment size. Control files and
all sources/archives/executables remain unchanged after measurement.

## Results and chronology

Both cases use 30 paired observations after three warmups, one pass/worker,
CPU 2, fresh processes/handles and warm OS cache. All executables use Rust 1.88.0.
No owned tests/builds overlap timing. Samples, inventories, hashes, host context
and exact sequence are retained under [measurements/](measurements/).

| Case | Tree / C total, paired 95% interval | Tree / ZIP total, paired 95% interval | Tree median total |
| --- | --- | --- | ---: |
| Raw 64 MiB stream | 0.999 [0.967, 1.033] | 3.081 [2.819, 3.365] | 50.807 ms |
| Raw 4 KiB midpoint range | 0.821 [0.798, 0.844] | 15.118 [13.360, 17.139] | 2.256 ms |

The stream result shows no clear speed difference from C; it does **not** prove
statistical parity under the plan's upper-bound criterion. Range total improves
about 18% over C, with read-only ratio 0.119 and open ratio 0.949. Both ZIP cases
fail the proposed parity target; ZIP ranges omit full-entry CRC. Whole-process
RSS is 1.172× C for stream and 1.186× for range, not incremental empty-open RSS.

The stream batch completed before the additional fragment-unit audit arrived.
The audit subsequently confirmed matching units on the unchanged controls.
That batch remains named `raw-stream-before-cost-model`; it was not rerun to
rewrite the chronology. Range timing followed the audit. These are explicitly
transition-built-tree versus post-compaction-C results, not fresh-export or
compaction comparisons. No 64 MiB compressed, cold-cache, public streaming or
whole-format qualification is inferred.
