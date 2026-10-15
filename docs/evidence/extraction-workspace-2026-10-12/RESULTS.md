# Worker decoder workspace and last-use retirement: retained

The fixed trial completed 112 verified public extractions in 739.149 seconds: 96 measured and 16 excluded preparation runs. Every frozen aggregate timing and maximum-RSS guard passed, along with both large-cohort median-memory gates. Output bytes, exact trees and modes, cold archive admission, output draining, unchanged identities and owned-agent cleanup passed. No additional samples or changed thresholds were used.

| Plaintext workload | Median seconds old → new | Paired elapsed ratio (95% interval) | Peak CLI RSS MB old → new |
| --- | --- | --- | --- |
| Raw, 256 files / 64 MiB | 0.630 → 0.545 | 0.908 (0.857–0.967) | 212.68 → 59.75 |
| Raw, 1,024 files / 1 GiB | 3.705 → 3.350 | 0.935 (0.895–0.9997) | 254.32 → 68.30 |
| Compressed, 256 files / 64 MiB | 0.455 → 0.400 | 0.897 (0.810–1.004) | 119.69 → 43.36 |
| Compressed, 1,024 files / 1 GiB | 1.475 → 1.550 | 1.02689 (0.99779–1.04987) | 237.49 → 48.91 |

MB means 1,000,000 bytes. The large compressed aggregate upper bound was **1.04986893**, narrowly below the frozen 1.05 limit; its median was slower. Its two round-stratum upper bounds were 1.06179 and 1.05520. Raw-large stratum zero also had upper bound 1.08484. Thus the aggregate rule passed, but per-stratum 5% noninferiority and universal speedup are not established. The small compressed cohort's interval also includes 1.

Large-cohort median RSS ratios were 0.26525 raw and 0.20570 compressed, below the required 0.80. Maximum candidate CLI RSS across all runs, including preparation, was 68,296,704 bytes. The small compressed preparation peak was 43,384,832 bytes versus measured 43,356,160 bytes; other preparation-inclusive cohort maxima equal their measured maxima. The largest conservative CLI-plus-owned-agent bound was **76,599,296 bytes (76.60 MB)**, including preparation. This is a sum of per-process maxima, not observed simultaneous memory or unique physical memory. Boundary child checks are not a complete transient census. These plaintext fixtures establish no universal 100 MB, encrypted many-file or arbitrary topology guarantee.

## Mechanism and scope

The earlier [last-use-only candidate](../extraction-last-use-2026-10-12/RESULTS.md) was rejected for compressed slowdown and reverted. Its separate [diagnostic](../extraction-last-use-diagnostic-2026-10-12/RESULTS.md) observed increased minor faults and `madvise` calls with unchanged read/write counts. This candidate combines reviewed last-use retirement with one existing scoped decoder workspace per eligible extraction worker. It does not retrospectively retain the earlier failed trial or prove the exact syscall cause.

Eligibility is native `CacheLimit::Auto` plus `ExtractMany` parallel workers. Plans include every frame and segment page reference; cache ownership is retired only after successful file output and last local use. Dirty, staged, discard and zeroed markers, external owners, parent caches and explicit policies remain unchanged. Workspace scratch is lazy for compressed input, retains existing bounded eligibility and fallback behavior, and is fully wiped on growth and scope exit, including errors and unwinding. No global allocator tuning, format, authentication, compression, job grouping or worker-count change was made.

The actual dependency's size probe confirmed `required_size(window, 0) = window + 1,593,527` bytes on this target: maximum 5,787,831 bytes per worker for a 4 MiB window, up to 23,151,324 bytes across four scopes, excluding decoded outputs and other allocations. A second stale registry rlib lacked the API; that preparation compile failure is preserved and was not a candidate failure. No timed samples preceded the successful static/setup/freeze sequence.

Reference executable SHA-256 is `4142dd043db7ccd608f5eaccdf3d9673b76b5879fb0a729e2527207c0924d4d6`; candidate is `e15c98b2e938513b8dbad83bed10a494921544c5f8bf0503cd891a1f0120f3cc`. The release build took 39.29 seconds, with 800 tracked Rust/Cargo/config identities unchanged before and after. Original measured sources are preserved as text before retained-only test additions and hook formatting.

## Qualification status

All 74 focused test executions passed: 36 each for default and native-layout API configurations, plus two public CLI lifecycle checks. The API counts include overlapping filters: two last-use tests, one protected compressed parallel extraction, six decoder-workspace tests, seven extraction-module tests and 20 page-cache tests per configuration. They cover shared frames/pages, pending markers, external ownership, every protected output byte, authentication/cache behavior, decoder fallback/refusal and error/unwind cleanup. CLI checks cover read-only extraction/refusal/overwrite and the jobs option. A preparation redirection typo launched no Cargo process; corrected invocations passed. Source manifests matched after normalizing their relative-path prefixes.

The pinned Rust 1.88 release WASM build and wasm-bindgen 0.2.128 generation passed. The portable password test passed (one test), as did the saved setter invalid-mode/Bytes8192/Auto/commit/free/reopen byte lifecycle. Source hashes matched; JavaScript and type declarations were unchanged. Only the generated WASM binary changed, to SHA-256 `6f09826a1c66a6c58234412c5aacd5b27381d41f803b1556db773e248cc62270` (4,685,338 bytes). The build emitted 89 existing Vault API warnings. Exact commands, sources and logs are in [raw/wasm](raw/wasm/).

No additional full suite was run for this individual performance experiment. The prior full default/native integration checkpoint predates this change and the retained encrypted-page optimization; it is not a full-suite pass for the final source. This checkpoint does not qualify a production release.

The user requested completion of this experiment followed by a pause. No subsequent optimization or measurement campaign is authorized by this report. Frozen [protocol](PROTOCOL.txt), [summary](raw/metrics-summary.json), [samples](raw/paired-samples.jsonl) and [build evidence](raw/build-new/) remain unchanged.
