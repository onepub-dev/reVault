# Post-security extraction worker crossover — 2026-10-11

Status: protocol frozen before measurements. Base source is security checkpoint
`a191020a`. The user has explicitly resumed optimisation; the prior 50%-remaining
usage cutoff applies. No automatic worker policy has been selected.

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

Pending current-source build and measurements. No performance or release gate is
closed by freezing this protocol.
