# Compressed many-file cache qualification — 2026-10-12

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
