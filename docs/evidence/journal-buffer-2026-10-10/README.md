# Compact journal decoded-buffer normalization

Retained as a small allocation/wiping cleanup, not real-world performance qualification. The compact preparation journal's two decoded-body constructors now use the existing tested `ZeroizingBytes` bulk-wiping wrapper. Entire allocations, including spare capacity, still wipe on success, errors and unwind. Parsing, checksums, AEAD, sequence/lineage checks and archive bytes remain unchanged; no new unsafe code or dependency is added.

## Fixed microcost comparison

Control is the clean `1db0cb59` executable `997632d6c233e4a6289eb7a91976fbfee4038e5987f7fcdb3de266539167be56`. Candidate is the exact two-line trial with SHA-256 `6e52a248f63699ae842af76c4cf38ed94edfd93dd78829e5c53fd8cb063bec18`. Rust 1.88.0, CPU 2, one worker; three warmups and 30 alternating paired triples per case, fresh processes/handles, identical archives. Content verification is outside timing; ratios use fixed-seed paired log bootstrap intervals.

| Fixture | Candidate/control total [95% CI] |
| --- | --- |
| small512-mixed-compressed-stream | 0.9984 [0.9934, 1.0042] |
| raw8m-random-stream | 0.9938 [0.9901, 0.9975] |
| compressed8m-pattern-stream | 0.9900 [0.9710, 1.0019] |
| raw8m-random-range | 0.9726 [0.9528, 0.9926] |
| raw64m-random-stream | 0.9977 [0.9887, 1.0059] |
| raw64m-random-range | 0.9739 [0.9630, 0.9849] |


Raw 8 MiB stream improves 0.6%; raw 8/64 MiB range totals improve 2.7%/2.6%. Other intervals include no change. All cases remain slower than ZIP. These fixtures live on RAM-backed `/tmp` and fit cache; 4 KiB is the requested range, not the archive size. The 8/64 MiB and 512-file fixtures isolate overhead but are insufficient for realistic throughput/extraction or scale qualification. The user's subsequent direction prioritizes disk-backed gigabyte and many-file workloads. Do not extrapolate these small gains to those workloads or count them as G11 completion.

## Validation and provenance

Strict Clippy and release tests pass: compact journal 10, bulk wiping three, selective reads four and typed credential opening two. The existing wipe tests cover unaligned live/spare capacity and empty/truncated buffers; journal tests cover all protection modes, malformed records, overflow placement, read failures and interruption. No redundant implementation-mirroring test was added for the type substitution.

All six cases have 99 verified sample records including warmups, with stable source/executable and exact per-path fixture hashes. Parent verified binary identities and adapter embedded paths before timing. The first preflight failed because the required `other/source` inventory had not been added; it produced no samples and is retained under `raw/logs/failed-setup-fixture-path.log`. The corrected fixed batch ran once. [Summary](raw/metrics-summary.json), [sample audit](raw/final-audit.json), raw samples, source snapshots/patch, runner, identity records and logs are retained. Generic runner `primary` means the `1db0cb59` control, not the older `f57a12ca` cache baseline.
