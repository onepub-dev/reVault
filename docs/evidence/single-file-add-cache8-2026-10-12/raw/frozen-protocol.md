# Smaller single-file add retention — frozen fixed trial

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
