# Public add — current Auto baseline

All 16 processes completed (4 preparation, 12 measured), with identities, sole-file
listing, exact persisted bytes/mode, cold-source admission and owned cleanup
verified. The original analyzer unused-import warning is preserved separately;
the corrected analyzer passed before setup. No runtime change was measured here.

|1 GiB workload/profile|Median elapsed seconds|Maximum RSS bytes|
|---|---|---|
|Source Zstd / Interactive|2.29|279,674,880|
|Source Zstd / BulkImport|2.31|139,513,856|
|Random raw / Interactive|6.68|242,618,368|
|Random raw / BulkImport|7.05|101,543,936|

Every cohort exceeds 100 MB at its measured maximum. These are descriptive current
public-operation observations; unlike historical component figures, they include
CLI add/commit/drop. Empty archive creation is outside timing. The two workflows
select different profile behavior, so their difference is not a cache-only causal
estimate. [Serial diagnostic](../public-add-serial-2026-10-12/README.md) follows.
## Frozen experiment description


Freeze 16 complete CLI add processes: pinned 1 GiB source-Zstd and random-raw
inputs, two public workflows, one preparation plus three measured processes per
case/workflow. Reuse source bytes without copies. No runtime intervention before
this fixed baseline completes; this is descriptive, without ZIP/PGP parity claims.

Use current preserved optimized CLI85caddce with original explicit-file build
provenance and verify its hash. Each sample creates a fresh empty archive outside
timing using public create --encryption none --signing none --compression zstd
--compression-level 3. Workflow A
times one-file add to /payload.bin, selecting Interactive. Workflow B times the
containing-directory recursive add with an exact filename include filter to /data,
selecting BulkImport. The source directory must contain exactly the pinned regular
file, and public recursive listing plus extraction proves one intended stored file.
Public doctor output verifies encryption none, signing none, Zstd3/defaultpadding.
Creation is excluded from both timings. Profile selection also affects frame and
compression behavior, so differences are not cache-only causal estimates.

The original implicit-password proposal was corrected by source inspection before
any execution: add selects CacheOnly without a content-key option and cannot
implicitly create with LOCKBOX_PASSWORD alone. These plaintext workflows exercise
the two import profiles without inventing unsupported CLI credential semantics;
implicit encrypted creation and PGP qualification remain open.

Use clean isolated HOME/XDG/Vault/session contexts, short private sockets with
successful public session roundtrip, disabled platform secret store and no ambient
credentials/configuration. No credentials are used in this plaintext screen.
Stop every owned agent with bounded cleanup and retain errors.

Auto worker/cache policy and default compression remain unchanged; CPU affinity
0–4, record host available parallelism and Auto cap6 without assuming actual worker
activation. Do not enable LOCKBOX_IMPORT_TIMINGS during the baseline. If useful,
one separately labeled untimed diagnostic per workflow/source may follow the fixed
samples, with all bytes still checked; it cannot replace baseline measurements.

Before each timed add require zero-resident source admission after bounded
sync/fadvise/fincore attempts. GNUtime records whole-process elapsed, user/system
CPU and peak RSS, excluding helper-agent RSS. Fresh outputs and archives must not
exist. Require finite positive time/RSS and successful CLI commit/close. Independently
sync archive and parent after completion, and report this drain outside CLI timing;
CLI durability remains its own normal commit contract, not a claim that external
sync cost is included. A separate public extraction must restore the sole stored
file, followed by independent cmp of every byte, regular-file/size/mode checks and
output drain. Record archive hash/size, source hash, listing and verified status.
Remove only successfully verified owned sample archive/output; preserve failures.

Freeze runner/protocol/source/binary identities, verify again at completion, and
keep builds/tests out of timing. Owned24GiB accounting includes prior retained
fixtures/binaries plus next archive, extraction output and temporary headroom.
Allow10minutes per add, bounded30minute work deadline with60second cleanup reserve;
cleanup kill grace can extend total wall time. Stop on any gate failure, no sample
retries or outcome-driven additions. Baseline determines the next measured
intervention; no automatic policy change is authorized by this document alone.

## Evidence preservation

Command logs are losslessly stored in [command-evidence.tar.gz](raw/command-evidence.tar.gz).
Every extracted byte was checked against [the SHA-256 manifest](raw/command-evidence.sha256);
originals remain in ignored local build storage. Sample rows, summaries and
identities remain directly readable. [frozen-protocol.md](raw/frozen-protocol.md)
preserves the pre-result protocol. No corpus or archive payload bytes are committed.
