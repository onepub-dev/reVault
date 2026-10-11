# Object-stream reservation trial: rejected

The fixed campaign completed 112 verified public adds in 1161.227 seconds:
96 measured processes and 16 excluded preparations. All identity, independent
persisted-byte/mode, cold-source, archive-size and owned-agent cleanup checks
passed. Archive sizes were positive and identical across variants within each
cohort. The frozen acceptance rule rejected the change, and the single-line
runtime experiment was restored exactly to checkpoint `c4cf3ee0`.

| Input / workflow | Elapsed ratio [95% interval] | Median RSS ratio | Maximum RSS, old → trial (MB) |
| --- | ---: | ---: | ---: |
| source-zstd-1gib / single-file-interactive | 0.9757 [0.8885, 1.0566] | 0.9629 | 110.30 → 110.41 |
| source-zstd-1gib / one-file-directory-bulkimport | 1.0223 [0.9457, 1.1317] | 0.9616 | 98.84 → 92.52 |
| random-raw-1gib / single-file-interactive | 1.0287 [1.0178, 1.0407] | 1.0296 | 87.64 → 90.34 |
| random-raw-1gib / one-file-directory-bulkimport | 1.0168 [0.9913, 1.0351] | 1.1270 | 73.18 → 79.51 |

Ratios are trial/baseline; MB is 1,000,000 bytes. Intervals use the frozen six
adjacent two-round paired log-ratio blocks and 10,000 seeded bootstrap draws.
Both round-stratum views and all individual RSS values remain in the full summary.

The named compressed Interactive usefulness gate required median RSS ratio
at most 0.95; the observed 0.962915 missed it. Its elapsed upper bound 1.056575
also exceeded 1.05, as did compressed BulkImport's 1.131654. These wide intervals
do not prove a compressed slowdown, but fail the predefined non-regression rule.
Raw BulkImport maximum RSS rose 8.65%, exceeding the 5% guard, with median RSS
up 12.70%. Raw Interactive elapsed rose 2.87% [1.78%, 4.07%], still inside the
aggregate allowance; its odd-round upper bound exceeded 5%. No universal speed
or memory improvement is claimed. The 100 MB whole-process write target remains
open; compressed Interactive trial maximum was 110.41 MB.

The preceding diagnostic suggested excess stream capacity, but reducing that
one allocation did not produce a sufficiently useful process-level outcome.
No additional samples or threshold revision were used to rescue this hypothesis.
No qualification tests or WASM rebuild were run for this rejected change.

## Evidence and reproduction

The unchanged [frozen protocol](README.md), [summary](raw/metrics-summary.json),
[112 raw rows](raw/paired-samples.jsonl), [completion](raw/paired-complete.json)
and [identities](raw/paired-identities.json) preserve the exact decision.
The original 47.84-second build log, before/after source manifests, one-line
patch, base revision and binary identity are under `raw/build-new`.
The retained-only test draft was never applied or run.

Baseline release-mode development CLI:
`08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`.
Trial CLI: `dd93a8c63bc431fcb1faea5debf682e8ea9868b2fe648a89daeac0b53e7d59c3`.
Both use the current default permission restoration; output mode 0600 and all
bytes were checked through separate normal CLI extraction. The source remained
frozen through analysis. Rust and binding diffs were empty after restoration.

Verbose command logs are losslessly packaged in `raw/command-evidence.tar.gz`,
with every extracted file verified against `raw/command-evidence.sha256` and
original local files preserved. Phase logs remain directly readable under
`raw/phase-runs`. This is neither a ZIP comparison nor release qualification.
