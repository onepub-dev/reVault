# Smaller single-file add retention — retained result

The fixed trial completed all 56 processes (48 measured, 8 preparation) in
583.582 seconds. Persisted-byte/type/mode checks, post-run identities and owned
agent cleanup passed. Both source cohorts met every frozen gate: aggregate
elapsed upper 95% bound ≤1.05, maximum RSS ratio ≤1.05, and median RSS ratio ≤0.90.
Retain the one-line command-local target reduction from 32 MiB to 8 MiB for
existing-archive single-file adds. No samples were repeated.

| 1 GiB source | Elapsed ratio, 95% interval | Maximum RSS old → new, MB | Median RSS reduction |
| --- | --- | --- | --- |
| Source Zstd | 1.0134 [0.9963, 1.0315] | 149.81 → 122.47 | 21.2% |
| Random raw fallback | 1.0013 [0.9898, 1.0123] | 133.95 → 108.81 | 14.2% |

MB means bytes / 1,000,000. Neither aggregate interval establishes a speedup.
All parity-view upper bounds are below 1.05. Both maxima remain above 100 MB,
so the whole-CLI write target remains open despite raw median RSS below 100 MB.
Workload profile, compression, workers, ownership transfer, authentication and
durable preparation are unchanged. BulkImport is untouched and was not retimed;
no small-file or experimental native-layout performance gain is claimed.

The baseline is preserved binary `641924c2`, the retained ownership implementation.
Trial binary `cccc2f00` was built from `032c8d2a` plus the one-line CLI change.
The target remains soft: dirty pages and active owners can exceed it. Default
layout pressure flushing retains the existing preparation-before-write path.
All seven focused release test executions passed: two default and two native
cache-limit lifecycle tests, the pressure/commit-abort test across protection
modes, and two public CLI add tests covering byte-exact writes, refusal,
replacement, directory import and worker options. No runtime edits followed the
measured one-line change. See [focused evidence](raw/focused/). No full-suite, release, architecture
or ZIP/PGP acceptance is claimed. This CLI-only policy requires no new binding
surface or WASM artifact rebuild.

See [summary](raw/metrics-summary.json), [completion](raw/paired-complete.json),
[identities](raw/paired-identities.json), and [build provenance](raw/build-new/).
All 1,878 benchmark command files are losslessly packaged in
[command-evidence.tar.gz](raw/command-evidence.tar.gz); every extracted file was
verified against [its hash manifest](raw/command-evidence.sha256). Original files
remain in ignored local storage. The validated pre-format CLI source is preserved
in [the source snapshot](raw/preformat-checkpoint-sources.tar.gz), with
[hash](raw/preformat-checkpoint-sources.sha256) and [diff](raw/preformat-checkpoint.diff).
The tracked hook applies formatting at commit; these are pre-hook source hashes.
No archive or corpus bytes are committed. The stream-FD diagnostic recorded in
[the incident note](raw/summary-launcher-diagnostic.txt) came from read-only
tool/launcher inspection after completion, not the benchmark or summary process.

The exact pre-result protocol is preserved in [frozen-protocol.md](raw/frozen-protocol.md).
The following protocol records the original design and fixed decision rule.


The retained owned-buffer transfer leaves default CLI Interactive 1 GiB add
maxima at 148.62 MB (source Zstd) and 129.84 MB (random raw). The command already
sets a 32 MiB page retention target. Its default-layout writer flushes staged
file pages under pressure only after durable preparation. This trial tests a command-local
8 MiB target for the same existing-archive, single non-directory add branch.
No workload profile, compression, worker, queue, ownership, format or
authentication change. Dirty/actively owned memory can exceed the target.
Experimental native-layout writing already discards after flush; no equivalent
native performance gain is presumed.

Preserve the measured raw-ownership binary 641924c2 as baseline. Freeze 56 fresh
public CLI add processes: the same two pinned 1 GiB sources × old/new × 14 rounds.
Exclude the first two rounds per variant/case (8 prep, 48 measured). Alternate
order by round plus case; use six adjacent two-round paired log-ratio blocks,
10,000 seeded bootstrap draws and parity views, with seed 0x6b2175d9 followed by UTF8 label XOR/multiply 16777619 modulo
2^32 (zero becomes 1). The unchanged BulkImport branch is not retimed.

Retain only if BOTH aggregate elapsed upper 95% bounds are ≤1.05, BOTH maximum
RSS new/old ratios are ≤1.05, AND BOTH median RSS new/old ratios are ≤0.90. These
are intermediate benefit criteria, not a replacement for the 100 MB goal.
No extra samples or outcome-dependent threshold adjustment. No small-file
performance, global ReadMostly or native-layout performance claim.

Reuse existing verified corpora; fresh empty plaintext unsigned Zstd3 archive
creation remains outside timing. Time complete add/commit/drop. Use the old
binary for setup and independent extraction, cmp, exact listing, file type/mode,
and doctor checks. Preserve cold-source admission, CPU 0–4, clean environment,
short owned-agent contexts, actual output drain, finite resource metrics,
source/build/executable/runner/protocol/analysis identities, bounded timeout and
cleanup, and combined 24 GiB owned-budget checks. Include new binary and prior
contexts in that budget. No tests or builds overlap timing.

If retained, run focused pressure/preparation, dirty/active-owner, pending
commit/abort/reopen and public add refusal/overwrite lifecycle checks. Reuse
existing public coverage; no test merely asserting the chosen 8 MiB constant.
The public setter and binding surface are unchanged. Revert only the one-line
trial if the fixed rule fails, preserving all evidence.
