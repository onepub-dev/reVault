# Extraction last-use retirement: rejected

The fixed trial completed 112 verified extractions: 96 measured and 16 preparation runs, in 742.32 seconds. Input identities, exact output tree, bytes, stored modes, cold-input admission, output draining and owned-agent cleanup passed. The frozen [protocol](PROTOCOL.txt) and [summary](raw/metrics-summary.json) remain unchanged.

The candidate retired each worker's clean page and decoded-frame cache entries after their last planned use. Although memory fell substantially, compressed extraction became slower. Both compressed timing intervals lie wholly above 1. The frozen all-cohort timing guards therefore rejected the candidate; memory gains do not override those guards.

| Plaintext workload | Median seconds old → new | Paired elapsed ratio (95% interval) | Peak CLI RSS MB old → new |
| --- | --- | --- | --- |
| Raw, 256 files / 64 MiB | 0.665 → 0.580 | 0.887 (0.849–0.928) | 211.51 → 60.08 |
| Raw, 1,024 files / 1 GiB | 4.235 → 3.815 | 0.897 (0.864–0.927) | 254.43 → 68.02 |
| Compressed, 256 files / 64 MiB | 0.350 → 0.445 | 1.301 (1.275–1.330) | 120.24 → 40.03 |
| Compressed, 1,024 files / 1 GiB | 1.490 → 1.805 | 1.205 (1.142–1.253) | 234.99 → 46.03 |

MB means 1,000,000 bytes. Preparation-inclusive peak RSS equals the measured maximum in each cohort. RSS in this table covers the CLI process; separately recorded helper bounds are not simultaneous process-tree measurements. These results do not qualify encrypted or other workloads.

All maximum-RSS guards passed, as did both large-cohort median-memory usefulness gates (new/old 0.267 raw and 0.193 compressed). The compressed elapsed upper bounds 1.330 and 1.253 exceeded 1.05. No samples were added or replaced after the result.

Exactly `lockbox/extraction.rs` and `storage/page_cache.rs` were restored to retained checkpoint `0c10fb30`. The measured patch, build logs and source identities remain in [raw/build-new](raw/build-new/). Reference binary SHA-256 is `4142dd043db7ccd608f5eaccdf3d9673b76b5879fb0a729e2527207c0924d4d6`; rejected candidate is `2f56a9986cae3c50b36a2c71e3308fe96e39a018b9e60a454d9682e02cbf783e`. Conditional tests were not applied. No qualification tests or WASM rebuild were run for this rejected change.

The first static-analysis attempt found two malformed Dart getter declarations. Their syntax was corrected before setup or timing, and the failed preparation log was preserved. The successful installed runner is bound by the frozen identities; no timed data preceded that correction.

Existing CPU records provide a diagnostic lead, not a causal conclusion: compressed 1 GiB median user CPU decreased from 3.215 to 3.045 seconds while system CPU rose from 1.185 to 2.400 seconds. The next bounded diagnostic will distinguish allocation/mapping, page-fault and I/O/scheduling effects using preserved binaries. It will not relax the guards or retain the policy only for selected cohorts. The many-file 100 MB target remains open in the retained runtime.
