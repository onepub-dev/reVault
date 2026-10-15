# Raw frame ownership — retained public write result

All 112 fixed processes completed in 1173.583 seconds: 96 measured and 16
preparation runs. Every persisted output passed the public extraction and byte,
type and mode checks; final identities and owned-agent cleanup passed. The
[frozen summary](raw/metrics-summary.json) met every prespecified retention gate.
Retain the internal Raw/Compressed outcome and transfer owned raw buffers into
prepared frames without copying. Compression decisions, worker settings, caches,
authentication and durable preparation are unchanged.

| Source / workflow | Elapsed ratio, 95% interval | Maximum RSS old → new, MB | Median RSS ratio |
| --- | --- | --- | --- |
| Source Zstd / Interactive | 0.9763 [0.9573, 0.9937] | 151.08 → 148.62 | 1.0020 |
| Source Zstd / BulkImport | 0.9917 [0.9814, 1.0002] | 110.08 → 108.90 | 0.9950 |
| Random raw / Interactive | 1.0026 [0.9935, 1.0153] | 149.81 → 129.84 | 0.9111 |
| Random raw / BulkImport | 0.9902 [0.9572, 1.0214] | 116.44 → 87.79 | 0.8640 |

MB means decimal bytes / 1,000,000. Raw median RSS fell 8.9% and 13.6%,
respectively. All four aggregate elapsed upper bounds and maximum-RSS ratios
passed the 1.05 guards. The raw BulkImport even-round elapsed upper bound is
1.0534, so the same bound is not established for every stratum. Compressed
Interactive shows a modest elapsed improvement, but its median RSS increased
0.2%; no compressed-memory improvement is claimed. Only raw BulkImport has all
measured runs below 100 MB. The overall write-memory target remains open.

The baseline is the preserved cache-policy binary `fcca5f5a`; the measured trial
is `641924c2`, built from checkpoint `ae68a268` plus the three-file ownership
change. [Build provenance](raw/build-new/) and [frozen identities](raw/paired-identities.json)
bind the runtime, executables, protocol, runner, analysis and sources. No samples
were repeated. All 30 focused release test executions passed: 19 default API, 10 native-layout
API and one public CLI test. Counts include overlapping filters; the initial
default raw_ filter selected nine tests, followed by exact focused filters.
Checks establish owned pointer/capacity transfer, borrowed/packed bytes, numeric
compression output, full-capacity wiping, unwind/error cleanup and parallel
abort/reopen. No full suite was rerun; prior broad API timeout limitations remain.

The current-source WASM release build and wasm-bindgen 0.2.128 generation passed.
Only the tracked .wasm binary changed; JS and declarations were identical. Both
portable password and setter/commit/reopen lifecycles passed. Source hashes stayed
stable across generation. The pipeline's full-file hash differs from the measured
epoch only because two post-retention tests were added inside its cfg(test) module;
the provenance records this distinction. See [focused logs](raw/focused/) and
[WASM source/artifact audit](raw/wasm/source-provenance.txt). No release,
architecture or ZIP/PGP acceptance is claimed.

All 3,720 benchmark command files are losslessly packaged in
[command-evidence.tar.gz](raw/command-evidence.tar.gz), with every extracted byte
verified against [the manifest](raw/command-evidence.sha256). Original logs remain
in ignored local storage. No archive or corpus payload is committed.
[Pre-format checkpoint sources](raw/preformat-checkpoint-sources.tar.gz),
[hashes](raw/preformat-checkpoint-sources.sha256) and
[diff](raw/preformat-checkpoint.diff) preserve the validated source before the
tracked hook formats Rust; these are not post-hook source hashes.

The original protocol is preserved verbatim in [frozen-protocol.md](raw/frozen-protocol.md).
The following text records the design and pre-result decision rule.


The retained single-file cache policy reduced peak RSS to about 150 MB but did
not meet the 100 MB write target. Source inspection shows parallel jobs already
own their input, yet raw/fallback encoding clones it into another allocation and
then wipes the original. This trial returns an internal Raw/Compressed outcome,
moving owned raw input directly into the prepared frame. Borrowed raw input still
copies once; compressed output and owned input are protected by full-capacity
zeroizing guards, including rejected output and callback unwind paths.

All compression probes, numeric output decisions, short/high-level/workspace-error
fallbacks, worker/queue/window settings, retained caches, workload profiles,
authentication and durable preparation remain unchanged. Legacy tuple callers
move compressed output without cloning and still materialize raw bytes normally.
Transferred raw buffers may retain spare capacity; this is not a total-memory cap.

Freeze 112 fresh processes: two pinned 1 GiB sources × Interactive single-file
and BulkImport one-file-directory workflows × old/new × 14 rounds. The first
2 rounds per cohort/variant are preparation (16 excluded), leaving 96 measured.
Alternate order by round + case + workflow parity. Preserve optimized baseline
fcca5f5a and the separately built trial. Old CLI creates/verifies both variants;
empty plaintext unsigned Zstd3 archives are created outside timed add. Time the
complete add/commit/drop process. No credentials or phase diagnostics are used.

Use six adjacent two-round paired log-ratio blocks, 10,000 seeded bootstrap draws
and separate parity views. Seed starts 0x6b2175d9, then UTF8 label XOR/multiply
16777619 modulo 2^32 (zero becomes 1). Retain only if ALL four aggregate elapsed
upper 95% bounds are ≤1.05, ALL four maximum-new-RSS/maximum-old-RSS ratios are
≤1.05, AND BOTH random-raw workflow median-new-RSS/median-old-RSS ratios are ≤0.95.
No extra samples, outcome-dependent gate or threshold adjustment. The 100 MB target
remains open until actual complete-process measurements qualify it.

Pin runner, protocol, analysis/helper, old/new executables, build manifests and
all three changed runtime files plus retained CLI/page-cache policy sources.
Use prior verified sources; reject changed identity. CPU 0–4, Auto workers, short
isolated agent contexts and zero-resident source admission match prior trials.
Admit only finite GNUtime CPU/time/RSS metrics, public doctor/list validation,
independent extraction/cmp/regular-file/size/0644 checks, post-timing archive/output
drain, owned-agent cleanup and final identity verification. Helper-agent RSS is
excluded. The 24 GiB owned budget includes prior contexts/binaries and the new
binary, reserving 3×source +256 MiB and rechecking actual usage before extraction.
The 30-minute work cap reserves 60 seconds for cleanup; each command is bounded
by 600 seconds or remaining budget, with cleanup kill grace allowed beyond the
work cap. No source edits, builds or tests during timing.

If retained, focused checks cover actual owned pointer reuse/spare capacity,
borrowed raw copying, packed and short inputs, byte-identical numeric compression,
workspace wiping, callback failure, abort and independent reopen. Revert a failed
trial without new qualification tests. No ZIP/PGP, architecture or release claim.
