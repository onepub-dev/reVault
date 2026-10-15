# Post-security extraction worker crossover — 2026-10-11

Status: completed measurement milestone, with 640 verified extractions and no
failed samples. Base source is security checkpoint `a191020a` plus the retained
untimed fixture-census patch. The user explicitly resumed optimisation; the prior
50%-remaining usage cutoff applies. The final account reading reports 2% used
(98% remaining) in the seven-day window. No automatic worker policy is selected.

## Frozen matrix

Measure one-file extraction at 1, 16, 64, 256 and 1024 MiB for each workload:

| Input | Compression | Encryption | Purpose |
| --- | --- | --- | --- |
| Repeated pinned source bundle | Disabled | None | Cheap serial control |
| Repeated pinned source bundle | Zstd level 3 | None | Decompression crossover |
| Repeated pinned source bundle | Zstd level 3 | ChaCha20-Poly1305 | Combined work crossover |
| Deterministic random stream | Zstd level 3 with raw fallback | None | Incompressible plaintext boundary |

Use the preceding protected-extraction corpus identities, constructing exact
prefixes/repetitions without changing historical inputs. Build fresh fixtures
with the current executable, 64 KiB units and default padding. The endpoint is
the test-only candidate tree reader, not the native public CLI; native graph
repairs do not establish candidate security qualification.

Compare explicit one and four workers with four excluded preparatory rounds
and twelve measured paired rounds per case. Reader TID parity alternates each
round; variant order flips every adjacent two-round block, with case parity
offset. Each variant has six measured samples in each parity stratum, and each
stratum contains three rounds of each variant order. This avoids confounding
execution order with parity in the two-variant comparison.
Retain affinity CPUs 0–4. Thread-pool construction remains inside extraction
timing. There is no ZIP variant: these are within-reader crossover comparisons,
not refreshed ZIP or PGP parity claims.

Use cold archive page cache, established by per-file sync, eviction and zero
residency before each admitted sample. Permit at most three eviction attempts
100 ms apart, retaining each check. Calibrate each archive with two cold
sequential reads; require positive physical traffic agreeing within 10% and
at least half the smaller calibration count in extraction. Any admission failure
stops the case; partial cases do not become accepted comparisons.

Time archive open, output creation, writes, flush and close, then independently
reopen and compare every output byte. Drain output outside timing before the
next sample. Record elapsed time, process CPU, RSS, physical reads, parity,
source/executable/fixture identities and failures. Builds and tests must not
overlap timed samples. Keep generated artifacts on the healthy `/home` filesystem
under a new experiment root; do not overwrite earlier evidence.

Report paired geometric-mean four/one ratios with 10,000 seeded bootstrap
replicates over adjacent two-round blocks, plus parity strata. Confidence
intervals are host/workload-specific with no multiplicity correction. Increased
CPU is an accepted trade-off only when elapsed time improves. Retain the serial
default unless adjacent sizes and both parity strata support a coherent policy;
uncertain crossover results justify a measurement-only milestone. Any retained
reader change requires fresh affected measurements and focused integrity checks.

## Results

Four workers improve both compressed-source profiles from 16 MiB through 1 GiB.
Their aggregate and both parity-specific 95% intervals lie below one at all four
sizes. At 1 MiB, serial reads win. Raw source and incompressible plaintext fallback
also favor serial reads at every measured size. Compression enabled in the archive
options is therefore insufficient to choose workers.

Four-worker / one-worker paired elapsed ratios [95% interval]; below one favors
four workers:

| MiB | Source raw/plain | Source Zstd/plain | Source Zstd/encrypted | Random fallback/plain |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1.177 [1.047, 1.313] | 1.192 [1.127, 1.267] | 1.192 [1.124, 1.288] | 1.220 [1.150, 1.296] |
| 16 | 1.181 [1.146, 1.216] | 0.614 [0.588, 0.639] | 0.595 [0.561, 0.631] | 1.140 [1.120, 1.159] |
| 64 | 1.234 [1.223, 1.246] | 0.776 [0.743, 0.816] | 0.791 [0.760, 0.820] | 1.156 [1.129, 1.182] |
| 256 | 1.075 [1.068, 1.082] | 0.853 [0.828, 0.876] | 0.863 [0.849, 0.878] | 1.086 [1.065, 1.109] |
| 1024 | 1.092 [1.081, 1.103] | 0.897 [0.891, 0.904] | 0.847 [0.831, 0.860] | 1.042 [1.036, 1.050] |

In the larger compressed cases, the elapsed reduction is 10.3–40.5%, with
25.4–105.7% more CPU. The user accepts that trade-off; CPU growth alone does not
reject the gains. The crossover lies somewhere between the tested 1 and 16 MiB
points, not at a precisely measured threshold. The GiB results are fresh-epoch
comparisons within this batch, not paired estimates of the security patch's cost.

The authenticated fixture census confirms every random/plain chunk used raw
fallback, every source/raw chunk was raw, and every source/compressed chunk used
Zstd. Each profile has 16, 256, 1,024, 4,096 and 16,384 chunks at the five sizes.
All outputs passed independent byte comparison, all archive residency checks
passed, and all process-level reads met the calibrated physical-I/O requirement.
Some small-case global device-sector deltas are zero; those counters remain
corroboration only and are not substituted for the process-level admission gate.
Maximum extraction-stage RSS is 14,636 KiB (14.29 MiB), both across all 640
samples and across the 480 measured samples. This includes the test process,
not a complete public CLI invocation.

The default remains one worker. A next bounded experiment should select workers
lazily from authenticated descriptors and cover mixed compressed/raw chunks,
short ranges and repeated reads. A mode flag or first-chunk guess would extend
beyond this homogeneous-file evidence. There is no native reader policy change,
archive format change or new full-suite run in this milestone. Existing focused
parallel-reader checks passed 2/2 before timing, covering all-mode ordering,
errors, pool reuse and bounded input batches.

## Retained evidence and limits

The [frozen protocol](raw/protocol-before-measurements.md),
[fixture census](raw/fixture-census.tsv), [source identities](raw/cases.json),
[calibration](raw/physical-read-calibration.json) and
[pre-timing identities](raw/pre-timing-identities.sha256) retain the exact setup.
The [paired summary](raw/worker-crossover-batch/metrics-summary-v2.json) includes
CPU, RSS, physical reads and parity strata. The original summary is preserved;
v2 only clarifies the requested-variants gate label. The
[round-order audit](raw/worker-crossover-batch/round-order-audit.json) checks all
640 results against their JSONL rows, the fixed case matrix and balanced order
within each parity. The current-source executable was
built with Rust 1.88.0, locked/offline release settings and `external-source`;
no historical executable was reused as current. The source patch and hashes are
the tested pre-format epoch; commit-hook formatting follows validation.

All per-case verbose records are preserved in the
[sample archive](raw/sample-evidence.tar.gz), with a
[per-file SHA-256 manifest](raw/sample-evidence-sha256.txt) and
[verified package record](raw/sample-evidence-package.json). The package was
extracted and every file compared with its original hash before moving the
original directories to ignored local storage. To rerun the summary/audit,
extract the archive under `raw/worker-crossover-batch`; it restores the recorded
case-relative paths. No failed or excluded admission records were discarded.

These are one-file candidate endpoint measurements on one host, with repeated
source content and deterministic random data. They do not qualify native/public
CLI performance, mixed files, encrypted raw crossover, ZIP/PGP parity, hardware
cache coldness, durable completion, the total CLI RSS goal or a release. The
candidate's separate integrity and architecture qualification remain open.
