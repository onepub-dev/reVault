# Selective authenticated reads: implementation and evidence

The experimental v4 reader now follows the [accepted read contract](../../selective_authenticated_reads.md): authenticate the selected publication, necessary index pages and requested payload units. Ordinary opening does not scan the complete catalogue, ownership graph, unused pack padding, reclaimed space or all signed-plaintext contents. Full verification remains explicit; mutation, deletion, compaction and installation keep the audited path. No persisted format or Vault integrity cache changes.

This deliberately changes when damage in unaccessed data is detected. Selected content remains verified before delivery, with owner-authorized membership in signed modes. This comparison does not claim the selective reader performs the previous reader's complete opening audit faster. Archive-v4 remains test-only on the performance branch; the public CLI and bindings are unchanged.

## Fixed six-case comparison

Candidate source `54adcd84d5f64ed0a34e46da2a8bd39370da5a25`, baseline source `8d06901c` (frozen Rust-identical `9fcf3cb0` executable), Rust 1.88.0. Both adapters read the exact same existing tree archive bytes. The baseline executable SHA-256 is `2cebc6ceee449e5f111f2af2018f2330dd202376188451e2a76aac9d829aefdb`.

Each case uses three warmups and 30 paired triples, alternating ZIP/eager/selective order, one worker on CPU 2, fresh processes and archive handles, with warm OS caches. The normal path has no storage meter or stage observer timing. Each fresh selective handle starts with an empty decoded-page cache; authenticated pages can be reused within that read. Separate correctness tests check repeated lookup reuse, isolation and eviction; this batch does not report a warm-session throughput result. Full exact-content checks occur outside the measured operation. All six cases ran once; identities stayed stable and the batch reports no failures. An initial untimed smoke invocation used the top-level fixture directory instead of its `other` tree directory and failed before timing; the corrected smoke and original correctly configured fixed runner are retained separately.

Ratios below one are faster; intervals are the existing fixed-seed 10,000-resample paired log-ratio bootstrap. Total includes opening and the requested read. Fixtures use unsigned plaintext with size padding. Raw ZIP entries are Stored; compressed ZIP uses Deflate 6 while Lockbox uses Zstd 3 with fallback. Signed/encrypted behavior is covered by correctness tests, not claimed as measured by this timing batch.

| Fixture | Selective/eager total [95% CI] | Selective/ZIP total [95% CI] | Selective median total (ms) |
| --- | --- | --- | --- |
| small512-mixed-compressed-stream | 1.000 [0.986, 1.015] | 1.049 [1.028, 1.069] | 3.232 |
| raw8m-random-stream | 0.969 [0.965, 0.971] | 3.734 [3.623, 3.829] | 5.802 |
| compressed8m-pattern-stream | 0.956 [0.935, 0.980] | 2.459 [2.424, 2.509] | 3.708 |
| raw8m-random-range | 0.662 [0.650, 0.673] | 6.510 [6.288, 6.736] | 0.486 |
| raw64m-random-stream | 1.008 [1.000, 1.021] | 3.756 [3.708, 3.810] | 44.081 |
| raw64m-random-range | 0.551 [0.543, 0.559] | 12.396 [11.985, 12.802] | 0.983 |

The 4 KiB range cases use 33.8% and 44.9% less total time than the eager reader. Full 8 MiB streams improve 3.1% raw and 4.4% compressed. Small-file throughput is inconclusive, and the raw 64 MiB stream interval includes no change (up to a 2.1% regression); no improvement is claimed there. All six comparisons remain slower than ZIP. The range change is useful but does not meet the overall performance goal.

The [summary](raw/metrics-summary.json), per-case summaries, open/read/total observations, hashes, host context and runner are retained. Legacy machine-readable keys `primary`/`packed` refer to the previous eager tree reader in this batch; `other`/`tree` refer to the selective tree reader. They do not compare different archive layouts. Fixture aliases and adapter construction are documented by [the preparation script](raw/prepare.dart.txt). Executables and duplicate fixture payloads are excluded; immutable fixture hashes identify their inputs.

## Validation

Focused tests cover all 16 protection/compression/padding combinations for selected file bytes, cross-chunk ranges, malformed authenticated fragment bindings, successful file updates/deletion, variables and historical form values. Tests distinguish damaged selected data from untouched data, reject selected corruption before callbacks, exercise metadata mirrors, and require explicit audits and writers to reject damaged padding/free space. Credential tests include publication changes and wrong credentials. Authenticated-index tests include cache context isolation, its 16-page bound and exact results after eviction; shared-tree transaction tests also pass. Strict core Clippy passes. Individual logs retain exact filters and results, including earlier corrected fixture/compile failures.

The [source patch](raw/source.patch) includes the ordinary reader, bounded page cache, borrowed traversal retained for audited operations, integration and regression tests. The borrowed traversal has no isolated performance claim. Full CLI memory, million-entry scale, ZIP/PGP read/write parity, native recovery, migration, aging and release qualification remain separate unfinished work. This is focused validation, not a full repository test-suite result.
