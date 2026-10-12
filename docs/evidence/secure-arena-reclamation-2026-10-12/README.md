# Secure arena reclamation under allocation pressure

The repeated full-default integration diagnostic captured an allocation refusal
in `form_segments_empty_utf8_boundary_and_one_mib_all_modes` after 3,395.911
seconds. The error was `SecurityLimitExceeded("weakened secure memory allocation
is disabled")`, with 8,108 KiB locked against an unchanged 8,192 KiB limit.
The platform maps failed `mlock` to that error when weakened allocation is not
allowed. The runner stopped the owned process after the failed result; 380 tests
had passed and 15 were ignored. This was an incomplete run, not a suite pass.
The variable recovery test that failed in the earlier run passed in this capture.
See the preserved [integration diagnostics](../retained-page-integration-2026-10-12/README.md).

The failing unwrap is in the segmented encoder, after initial source allocation.
A full 64 KiB segment plus its 128-byte prefix grows secure capacity to 131,072
bytes; canaries and page rounding require 135,168 bytes of locked backing. That
is larger than the 84 KiB remaining at the captured snapshot. Secure page staging
can request the same class. This is a concrete possible request, not identification
of the exact failed allocation: the original log did not record its size or mode.

The preceding bounded resource profile independently showed that freed slots keep
their arena's locked backing and that repeated size classes reuse it. Many
distinct classes can therefore exhaust the lock budget even with little live
payload. That mechanism does not by itself prove every earlier suite failure.

## Scoped correction

The heap now releases backing only from wholly empty arenas after an allocation
or locking refusal, then retries once with the same allocation policy. It keeps
arena positions, slot metadata, free lists and generations unchanged. Existing
backed slots are preferred over remapping an empty arena. A failed remap leaves
metadata reusable; live arenas and scoped reads retain their backing. Partial
or live arenas are never reclaimed. Existing wipe, canary, guard-page and locking
semantics remain in force; no limits are raised and no weak fallback is added.

## Validation status

The complete secure-memory crate release suite passed 23 tests. The exact
previously failing API form-segment test passed in 2.73 seconds. Source manifests
matched before and after. Three new local-heap regressions cover a bounded retry,
live bytes and canaries, stale generation refusal after remapping, zeroed backing,
failed remapping followed by reuse, and preference for existing backed slots.
Exact commands and outputs are in `raw/focused`.

The isolated natural-pressure comparison used 64 capacities across two passes,
with only one live buffer at a time, deterministic nonzero content checks, and
unchanged 8 MiB locking limits. The old allocator completed 60 allocations before
`WeakAllocationDisabled` at capacity 253,952 bytes: retained backing occupied
8,359,936 bytes and the next arena required 258,048 bytes. The corrected allocator
completed all 128 allocations. Reclamation was observed at pass 1/index 60 and
pass 2/index 55; maximum locked backing was 8,359,936 bytes. This establishes the
benefit under real locking pressure, not an old/new timing comparison: the old
process completed less work.

Old and new standalone copies used identical manifest scaffolding and dependency
resolution. All 20 selected package identities matched the original workspace
lock; that complete lock audit was performed after the profiles and is recorded
as retrospective. Preparation failures and both successful profiles are preserved
in `raw/isolated-pressure-profile`.

The fixed normal-reuse screen completed all 56 processes without retries. Each
fresh process began with zero locked memory, then retained the same positive
locked backing during prewarm, before the measured loop and after it. Content,
allocation policy and the 8 MiB limit were checked in every sample; frozen
identities matched before and after. Two preparation rounds were excluded,
leaving 12 measured old/new pairs per size. The predefined six-block bootstrap
produced these one-sided 95% upper bounds for new/old elapsed time:

| Allocation size | Inner loop | Whole process |
| --- | ---: | ---: |
| 64 bytes | 1.000313 | 1.000399 |
| 65,536 bytes | 1.007462 | 1.007564 |

All four bounds passed the frozen 1.05 noninferiority guard. This supports
retaining the correction without a material ordinary-reuse slowdown in these
two cases; it does not claim an allocation speedup or cover every workload.
The exact schedule, sources, hashes, 10,000-draw analysis and results are in
`raw/hot-reuse-methodology-review`.

Current-source WASM release compilation and generation passed with Rust 1.88.0
and wasm-bindgen 0.2.128. The portable password regression passed, as did the
saved setter lifecycle covering invalid-mode refusal, Bytes(8192), Auto,
commit, free, reopen and byte comparison. Runtime source hashes matched before
and after. Only the generated WASM binary changed; JavaScript and declarations
were unchanged. Exact commands, source and hashes are in `raw/wasm`.

The integration checkpoint uses complete default and native API inventories sequentially, with two test
threads and a predeclared 90-minute runtime bound per configuration after a
separate build. Native runs only if default passes. The longer bound reflects
the earlier capture reaching only 396 results after 56.6 minutes; no additional
filters or ignores will be introduced. Immediate failure output and sampled
memory will be preserved, and a failure or timeout stops the checkpoint.
The default configuration completed successfully in 3,669.243 seconds: 695
passed, zero failed and 18 ignored. All 713 unique outcomes matched the frozen
inventory and terminal libtest summary; source and executable identities were
unchanged. All 734 resource samples had numeric values. Maximum sampled locked
memory was 8,108 KiB with the unchanged 8 MiB limit, and maximum sampled RSS was
515,512 KiB. One observed locked-memory decrease was 8,108 to 5,656 KiB, followed
by eventual reuse/growth to 8,048 KiB in the final sample. These five-second
observations are not continuous peaks or an exact trace of allocation calls.
The suite now completed after reaching the same sampled locked-memory high as
the earlier failed capture.

The native-block-layout configuration also completed successfully in 3,648.358
seconds: 702 passed, zero failed and 18 ignored. All 720 unique outcomes matched
its frozen inventory and terminal summary. Its 779-entry source manifest and
executable identity were unchanged; 730 resource samples were retained. Both
full suites used their complete inventories without added filters or ignores.
The first native build launcher failed before Cargo because its evidence
directory was absent; that preparation failure is preserved separately from
the successful bounded build and suite.

This completes the retained batch's API integration checkpoint. It does not
establish release readiness or every platform's resource behavior. A fresh
current-source protected-write baseline is next; no encrypted-page optimization
has been implemented. Detailed results and samples are in `raw/integration`.
