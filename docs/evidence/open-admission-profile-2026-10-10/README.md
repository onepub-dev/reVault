# Opening admission profile

The user resumed performance work with a 50% remaining-usage cutoff. The clean baseline is `1db0cb59`, preserving the later Unicode alias integration. Its freshly built executable is `997632d6c233e4a6289eb7a91976fbfee4038e5987f7fcdb3de266539167be56`. Temporary test-only timers separate publication selection, private manifest reading/decoding, journal admission, publication commitment and authenticated format lookup. All diagnostic source was restored exactly to that baseline before the next trial; no instrumentation remains in the implementation.

## Observations

Rust 1.88.0, CPU 2, one worker, 100 fresh handles per fixture in a repeated-process warm-cache diagnostic. Full content is independently verified before and after each diagnostic process, and each archive's digest is checked before/after. Times include timer/storage-meter overhead and are not fresh-process ZIP comparisons.

| Fixture | Total open (µs) | Select publication | Read manifest | Decode manifest | Journal open | Format lookup |
| --- | --- | --- | --- | --- | --- | --- |
| raw8m-random | 160.4 | 16.7 | 30.0 | 20.3 | 16.5 | 64.3 |
| raw64m-random | 233.3 | 17.1 | 30.2 | 20.4 | 16.8 | 133.6 |
| compressed8m-pattern | 169.8 | 16.5 | 30.0 | 20.3 | 16.5 | 72.3 |
| small512-mixed-compressed | 223.0 | 16.9 | 30.1 | 20.4 | 16.7 | 124.7 |


Format lookup fetches one authenticated index page for raw/compressed 8 MiB and two for raw 64 MiB/small-file fixtures. Verified page reads take about 39/79 µs respectively; cached decoding takes 21–49 µs. Those timers are nested within format lookup, so do not add them to its total. Publication admission includes roughly 17 µs selection, 30 µs manifest reads, 20 µs manifest decoding and 16–17 µs journal opening. These are observational attributions, not isolated causal speedups or function-level CPU profiles.

The compact journal decoder still uses the generic byte-vector wipe wrapper. The next small trial reuses the already tested bulk-wiping `ZeroizingBytes` for its decoded body, retaining full capacity wiping and all parsing/authentication. Larger gains require examining mandatory metadata I/O and hashing; this profile does not authorize skipping checks or changing the format.

## Validation and provenance

Strict Clippy passes, as do four selective-read tests, 19 authenticated-index tests (one manual probe ignored) and 53 publication-anchor tests (one manual probe ignored). Baseline and instrumented logs, exact source patch/snapshots, raw observations, collector and [summary](raw/diagnostic/summary-compact.json) are retained. Diagnostic executable SHA-256 is `8b66a0c56cb619d1a956c301c0278be2cefd44b6efa2e3f3d72449e2b71f11a1`; its source, executable and collector hashes remained stable during collection.

Parent review caught a collector inventory flaw: directory URI basenames were empty, flattening paths and overwriting duplicate names between root and `other` directories. The original before/after hash maps are preserved and do not establish a complete unique-path inventory. Archive digest checks and full-byte verification did run. A [later unique-relative-path audit](raw/later-fixture-audit.json) matches the prior fixed comparison's retained fixture identities, but is not a fresh per-profile before/after inventory. No diagnostic was rerun or selected based on timings.

This evidence identifies follow-up work. ZIP/PGP parity, total CLI RSS, scale, public activation and release qualification remain outstanding.
