# Raw frame ownership — frozen public write trial

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
