# Compact authenticated index-page cache

The experimental selective reader now stores each cached leaf's private bytes in one bounded wipe-on-drop allocation with record offsets. Previously it allocated separate key/value buffers for every record on the page. The full shared page decoder still validates authentication, lengths, ordering, counts, padding and parent bindings before exposure; only selected rows are copied to callers. Branch links and the 16-page cache/context bounds are unchanged. No archive bytes, checksum algorithm, payload caching or read-verification semantics change. Audited writers retain their existing representation.

## Fixed isolated comparison

The corrected batch compares the compact cache with the selective reader from `54adcd84`/`dc93b097`, Rust 1.88.0, on identical archive bytes. Baseline executable SHA-256: `318291b256dd2b319efdd8532bac8abc3ca675ba9147d2ab0f052b84eb86d216`; candidate: `68a0d28eafb4b97fd698d0703937142f354530e6d8e9a6030d34e1fac0cba80d`. The candidate is the passing unformatted trial whose exact patch and source bytes are retained; commit-hook formatting follows testing and measurement.

Each case has three warmups and 30 paired triples with alternating ZIP/baseline/candidate order, CPU 2, one worker, fresh processes/handles and warm OS caches. The uninstrumented selective cache starts empty per process. Every sample verifies requested and full bytes outside timing; all inputs/source/executable hashes remain stable. Ratios are paired geometric total times with the existing fixed-seed 10,000-resample log-ratio bootstrap. Full-read, stream and range definitions match the [previous comparison](../selective-authenticated-reads-2026-10-10/README.md), including codec/protection limits.

| Fixture | Compact/selective total [95% CI] | Compact/ZIP total [95% CI] | Compact median total (ms) |
| --- | --- | --- | --- |
| small512-mixed-compressed-stream | 0.9641 [0.9588, 0.9695] | 1.0363 [1.0302, 1.0425] | 3.091 |
| raw8m-random-stream | 0.9868 [0.9823, 0.9916] | 3.6795 [3.5791, 3.7642] | 5.759 |
| compressed8m-pattern-stream | 0.9983 [0.9768, 1.0310] | 2.4674 [2.4243, 2.5424] | 3.737 |
| raw8m-random-range | 0.8820 [0.8645, 0.9020] | 6.1106 [5.8596, 6.3502] | 0.443 |
| raw64m-random-stream | 0.9908 [0.9871, 0.9940] | 3.6657 [3.6459, 3.6829] | 43.749 |
| raw64m-random-range | 0.8144 [0.7978, 0.8338] | 10.7303 [10.4219, 11.0530] | 0.823 |

The compact representation reduces total time 3.6% for small files and 11.8%/18.6% for the 8/64 MiB range cases. Raw full-stream improvements are 1.3%/0.9%. The compressed-stream interval includes both improvement and up to a 3.1% regression, so no gain is established for that case. The change is retained for the measured range/small-file benefit with unchanged checks. All six cases still fail ZIP parity; this is not completion of G11 or full-format qualification.

The [machine-readable summary](raw/metrics-summary.json), [final sample audit](raw/final-audit.json), [runner](raw/run-fixed-batch.dart.txt), source snapshots, patch, hashes, per-case raw samples and test logs are retained. The generic runner keys `primary`/`packed` mean the selective-reader baseline here, and `other`/`tree` mean compact cache. Do not compare unrelated batch medians to infer a causal gain.

## Incorrect control preserved separately

The first batch accidentally reused the older eager-reader executable (`2cebc6ce...aefdb`). That batch cannot isolate this cache change. Its complete [classification and artifacts](wrong-control/CLASSIFICATION.txt) are retained separately and excluded from the table and acceptance conclusion. The parent verified the corrected baseline/candidate hashes and embedded adapter paths before the corrected smoke and single fixed batch. No candidate source was changed or rebuilt between those two batches, and no result was selected by timing.

## Correctness and remaining work

Strict core Clippy passes. Authenticated-index tests pass 19 with one ignored manual probe, including new compact-leaf empty/binary/maximum-length records across all 16 modes, existing parent/context corruption checks and eviction. Four selective-read tests, two credential-open tests and all-mode variable/form snapshot lifecycles pass. These are focused checks, not a full repository suite result. The prior [function profile](../selective-read-profile-2026-10-10/README.md) remains useful: stored-byte SHA-256 dominates raw full streaming, while metadata-page work matters for first range reads. Further payload work needs measured evidence and unchanged security guarantees. Public activation, scale, CLI RSS, native recovery, PGP/write and release qualification remain outstanding.
