# Compressed many-file cache qualification — 2026-10-12

## Qualification failed; worker policy reverted

All 56 fixed processes completed (48 measured, 8 preparation) with exact persisted
tree/bytes/modes, cold-archive, identity and owned-session cleanup checks passing.
Actual archive sizes were 17.55% and 17.07% of logical payload sizes. The frozen
small-case noninferiority gate failed despite a near-neutral point estimate;
this is uncertainty, not evidence of a demonstrated slowdown.

| Workload | New/old elapsed ratio, 95% interval | Maximum old → new RSS, bytes | Frozen gate |
| --- | --- | --- | --- |
| 256 files / 64 MiB | 0.993 [0.911, 1.083] | 119,451,648 → 55,341,056 | Failed: upper bound >1.05 |
| 1,024 files / 1 GiB | 1.003 [0.982, 1.026] | 239,095,808 → 66,461,696 | Passed |

Small-case round-stratum intervals were [0.881, 1.084] and [0.912, 1.125]. The
large even-round interval [0.991, 1.051] also does not establish the stratum-level
5% bound; its aggregate passes the declared gate. See the complete seeded audit
in [metrics-summary.json](raw/metrics-summary.json). No samples were added or
rerun, and no threshold was relaxed after seeing results.

One inspection of the existing pairs found no command timeout, failed cold
attempt or output failure, and no measured mechanism justifying a compression
guard. Reverted the global worker-budget change from `d26e5ce6`, including its new
helper-dependent tests, by restoring exactly the three runtime/test files from
`a881d15d`. Preserve the raw-data improvement as historical evidence, not current
runtime behavior. The earlier single-file ExtractMany 32+8 MiB targets and bounded
credential-open repair remain. No broad API rerun is needed for exact restoration;
the prior broad candidate recovery timeouts remain incomplete.

The following protocol was fixed before timing; original build-source hashes
describe the preserved old/new executables, not the later formatted/test-extended
working tree. Both binaries and all failed-qualification evidence remain intact.


Qualify the retained Auto worker-cache change on compressed data before further
runtime policy changes. Reuse the exact old/new optimized default CLI binaries
from the raw-data trial, preserving their source/build manifests and hashes.
Do not rebuild or modify runtime code for this experiment.

Create fresh public CLI archives from the existing pinned source-Zstd corpus:
256 sequential 256 KiB files (64 MiB), and 1,024 sequential 1 MiB files (1 GiB).
Use plaintext, unsigned, default padding and explicit Zstd compression. Preserve
the original concatenated source hash and every split file hash. Public recursive
import and separate public extraction must verify all bytes, exact tree/types and
0644/0755 modes under umask 0022 before timing. This is the configured compressed
public workload; per-frame fallback remains possible and no all-Zstd census claim
is made. Preserve the prior raw corpus and archives.
Before admitting timing, require each completed archive to be smaller than its
logical payload size; retain both sizes as evidence that compression occurred.

Reuse the reviewed public runner's clean isolated environment, short owned agent
contexts, real session roundtrip, scoped cleanup, zero-resident archive admission,
whole-process GNU time, CPU 0–4, native independent tree comparison and output drain.
The same 24 GiB owned-artifact limit includes both previous raw and new compressed
fixtures, old/new executables and temporary verification output. Stop on any
budget, identity, timeout, output or cleanup failure; retain partial evidence.

Run exactly 56 fresh CLI processes: two preparation and twelve measured rounds
per case and variant, alternating old/new order by `(round + case) % 2`. Use six adjacent
two-round paired log-ratio blocks and seeded 10,000 bootstrap replicates; report
even/odd strata separately. No separate descriptive baseline and no reuse of old
raw observations as the control arm. Freeze identities before timing; tests and
builds remain outside it.

Qualification requires every measured new process RSS <100,000,000 bytes and each
aggregate elapsed-ratio 95% upper bound ≤1.05, plus all byte/identity gates. Report stratum
uncertainty without changing sample counts or rerunning outcomes. Failure does
not erase the raw-data result: investigate once and either revert the globally
applied policy or justify a separately frozen policy revision. Do not leave the
same global policy promoted merely by labelling a compressed-data limitation.
This does not qualify
encrypted data, ReadMostly, native-block-layout performance, every corpus, helper
RSS or release readiness. No additional full API suite for measurement-only work.

## Evidence preservation

Completed command logs are losslessly stored in
[command-evidence.tar.gz](raw/command-evidence.tar.gz); every extracted file was
verified against [the SHA-256 manifest](raw/command-evidence.sha256), and originals
remain in ignored local build storage. Sample rows, complete summaries and
identity manifests remain directly readable. The exact frozen protocol is in
[protocol-before-summary.md](raw/protocol-before-summary.md).
[Restoration evidence](raw/restoration.json) records byte-identical runtime/test
files from the earlier validated checkpoint; no new full-suite result is claimed.
