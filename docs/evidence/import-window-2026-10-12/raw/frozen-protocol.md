# Parallel import window — frozen public write trial

Current Auto public writes exceed100MB across all four1GiB cohorts. Retained
scoped parallel encoder reuse reduced compressed BulkImport memory but did not
close that target. This next isolated trial bounds submitted-but-not-contiguously
written frames to2×effective jobs. The producer receives/accepts results before
reading/allocating another job when full. The result channel remains unbounded,
avoiding a producer/worker send cycle; only successful contiguous writes release
slots. Existing input-channel bound, workers, scoped encoders, cache/format policy,
durable preparation, authentication, empty inputs and final drain are preserved.

This is a frame-count bound, not total memory: producer scratch, encoder leases,
writer-owned pages, cache and metadata remain separate. Do not claim100MB from
structural accounting. No command-local cache/profile change is included.

Freeze112 fresh processes: two pinned1GiB sources × Interactive single-file and
BulkImport one-file-directory workflows × old/new ×14rounds. First2rounds per
cohort/variant are preparation (16 excluded),96measured remain. Alternate variant
order by round+case+workflow parity. Baseline5b831735 and trialdaa6fb1b are preserved
optimized binaries; oldCLI creates/verifies both. Empty unsigned plaintext Zstd3
archives are created outside timed add. Full add/commit/drop process is timed.

Use six adjacent two-round paired log-ratio blocks,10,000 seeded bootstrap draws
and separate parity views. Seed starts0x6b2175d9 then UTF8 label XOR/multiply16777619
modulo2^32 (zero becomes1). Freeze runner, protocol, analysis/helper, binaries,
source and build provenance. Retain only if ALL four elapsed upper95%bounds<=1.05,
ALL four maximum-new-RSS/maximum-old-RSS<=1.05, AND BOTH BulkImport source cohorts
have median-new-RSS/median-old-RSS<=.95. No post-result alternative usefulness gate,
extra samples or retries. The100MB write requirement remains unchanged/open until
actually qualified. No ZIP/PGP, architecture or release claim.

Same public lifecycle and admission as retained encoder trial: clean short private
session contexts/no credentials; CPU0–4; Auto workers/cache; zero-resident source;
finite GNUtime CPU/time/RSS; public doctor format checks; recursive sole-file listing;
separate public extraction and independent cmp/regular-file/size/mode checks;
archive+output drain outside timing; allowned cleanup; final identity recheck.
Helper-agent RSS excluded.24GiB budget includes all prior contexts and newbinary,
reserves3×source+256MiB before each add and checks actual usage before extraction.
30minute work cap reserves60seconds cleanup; eachcommand<=600seconds and remaining
budget; cleanup kill grace can extend walltime. No builds/tests/edits during timing.

If retained, deterministic out-of-order/window coverage and existing error/abort,
mixed/short/reopen/wiping checks follow. Preserve all failures and baseline evidence;
revert unsuccessful runtime experiments. This is an individual measured change,
not a reason to rerun the full suite after every trial.
