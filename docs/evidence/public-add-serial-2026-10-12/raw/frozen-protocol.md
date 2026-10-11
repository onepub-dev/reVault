# Public large-file add — serial-path diagnostic

The frozen Auto baseline completed16 processes and showed measured peak RSS above
100 MB in all four cohorts (source Interactive279.7MB/Bulk139.5MB; random
Interactive242.6MB/Bulk101.5MB). Freeze16 new descriptive processes with explicit
--jobs1: two sources × two workflows × one preparation plus three measured.
The public option selects serial preparation, so this observes combined serial
versus parallel implementation/workspace effects, not isolated worker count or
precise allocation ownership. Comparisons with the prior batch are descriptive,
not paired speed or noninferiority evidence. No runtime edits/builds are involved.

Reuse the exact85caddce binary and pinned sources from the completed baseline,
with a separate runner, identities, context/output root and evidence. There is
no separate repeated setup-import phase: original setup proves format/lifecycle,
and each of these16 processes independently verifies its new archive and output.
All gates/budgets/cleanup below remain; explicit jobs1 replaces Auto workers only.
The four preparation processes are excluded from descriptive summaries. No
outcome-driven reruns, policy changes or broad tests are part of this diagnostic.

## Shared baseline contract (worker option overridden above)

# Public large-file add — descriptive baseline

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
