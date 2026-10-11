# Direct raw-body serialization trial: rejected

All 112 fixed public imports passed independent byte/mode, identity, cold-source,
unchanged archive-size and owned-agent cleanup checks in 1127.86 seconds. There
were 96 measured processes and 16 excluded preparations. The frozen decision
was rejection: the speed and compressed-memory benefits did not override either
raw-workload maximum-RSS failure. The page.rs experiment was restored exactly
to retained checkpoint `15c90e5e`; the lead independently verified an empty Rust
and bindings diff after the executor's documented restoration.

| Input / workflow | Elapsed ratio [95% interval] | Median RSS ratio | Maximum RSS, old → trial (MB) |
| --- | ---: | ---: | ---: |
| source-zstd-1gib / single-file-interactive | 0.9029 [0.8837, 0.9289] | 0.9380 | 106.85 → 103.53 |
| source-zstd-1gib / one-file-directory-bulkimport | 0.8979 [0.8692, 0.9323] | 0.9161 | 97.48 → 87.09 |
| random-raw-1gib / single-file-interactive | 0.8563 [0.8282, 0.8845] | 1.0600 | 83.78 → 90.02 |
| random-raw-1gib / one-file-directory-bulkimport | 0.8598 [0.8308, 0.8886] | 0.9472 | 71.23 → 77.46 |

Ratios are trial/baseline; MB is 1,000,000 bytes. The frozen analysis uses six
adjacent two-round paired log-ratio blocks and 10,000 seeded bootstrap draws.
All aggregate and even/odd elapsed upper bounds were below 1.0. The observed
9.7–14.4% time reductions were meaningful evidence, but not grounds to change
the acceptance rule after seeing the result.

Compressed Interactive median RSS fell 6.20%, meeting the named 5% usefulness
gate; its maximum remained 103.53 MB, so it did not complete the 100 MB target.
Compressed BulkImport median RSS fell 8.39% and maximum fell 10.66%.
Raw Interactive maximum RSS rose 7.46% and median rose 6.00%; raw BulkImport
maximum rose 8.76% even though its median fell 5.28%. Both maximum ratios
exceeded the frozen 1.05 guard. These are process-level observations; they do
not identify allocator, queue or lifetime behavior as the cause of the peaks.

The direct body removed two intermediate buffers, but the remaining
body-to-stored-body-to-final-page allocations still overlap in the source.
Any further experiment needs a distinct, reviewed ownership change and fresh
comparison, rather than excluding raw inputs or relaxing their guards.
No additional samples, qualification tests or WASM rebuild were run for this
rejected change; the conditional regression drafts were never applied.

## Reproduction

The unchanged [frozen protocol](README.md), [summary](raw/metrics-summary.json),
[raw rows](raw/paired-samples.jsonl), [completion](raw/paired-complete.json) and
[identities](raw/paired-identities.json) retain the exact fixed decision.
`raw/build-new` preserves the original 49.07-second build log, source patch,
before/after source manifests, base revision and exact restoration evidence.

Baseline release-mode development CLI:
`08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`.
Trial CLI: `507b37e718cc06056bf8aa35229dffc028d3dcf3274886a5fe6339816944afd8`.
All output was independently extracted using the retained normal CLI with
default stored permission restoration (0600). Archive sizes were constant
within each cohort across both variants. No wire change is claimed or retained.

Verbose command logs are losslessly packaged in `raw/command-evidence.tar.gz`,
with all extracted bytes checked against `raw/command-evidence.sha256` and
original local files preserved. Phase logs remain directly readable. This is
not a ZIP comparison, encrypted-write qualification, or release qualification.
