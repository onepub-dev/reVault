# Single authenticated traversal for typed open

Date: 2026-10-02. Follows the measured open regression in the
[typed-tree baseline](../typed-tree-read-2026-10-02/README.md).

**Compiler scope:** both original and optimized typed readers in this paired
batch used Rust 1.94.1, so their direct comparison is compiler-matched. The
retained ZIP runner used 1.88.0; ZIP ratios below include that confound. Main
correctness qualification used the repository's pinned 1.88.0. The isolated
snapshot initially omitted the root toolchain pin. A subsequent matched-pin
batch is required for stronger retained-control claims; these samples remain
unchanged. The first full isolated-library Clippy run under 1.94.1 later exposed
seven existing-code lint failures; main 1.88.0 strict Clippy had passed.

Typed open previously traversed the authenticated index once to construct
ownership and again to decode typed records. It now stages typed records during
the ownership traversal, completing graph, reclaimed-space, semantic and fragment
validation before the image is returned. Failed opens discard staged records.
Signed whole-content verification and payload integrity checks are unchanged.
The read-only salvage route also shares its authenticated traversal; it does not
expose staged records before complete membership validation.

The [five-file overlay](source-overlay.sha256) matched the main worktree when
copied. The [isolated core manifest](single-pass-core-source.sha256) and
[verification](single-pass-core-verification-local.log) pin the measured core.
The same baseline harness is used. [Focused checks](tests.log),
[241 format tests](format-tests.log) (6 manual probes ignored), and
[strict Clippy](clippy.log) pass. The focused test covers all 16 modes, equivalent
logical results, fewer reads and rejected damage. The isolated harness also
passed its [build](single-pass-harness-build.log) and
[independent byte-verification smoke](single-pass-harness-smoke.log).

## Paired result

The four declared cases each use 30 pairs after three warmups with fresh handles,
one worker, CPU 2 and warm OS cache, preserving the original method. Both typed
readers consume the exact same archive bytes, with alias/hardlink proofs retained.
In this batch `primary` means the original typed reader and `other` the optimized
typed reader; it no longer means packed C. Every source/archive/binary check
passed. Samples, manifests and the complete metric table are under
[measurements/](measurements/).

Ratios below are optimized/original elapsed time. Smaller is better.

| Case | Total ratio, paired 95% interval | Open ratio | Median total | Median process peak RSS |
| --- | --- | ---: | ---: | ---: |
| 512 × 4 KiB mixed compressed | 0.933 [0.922, 0.946] | 0.764 | 4.839 ms | 6,360 KiB |
| 8 MiB seeded raw | 0.974 [0.966, 0.980] | 0.782 | 6.094 ms | 5,892 KiB |
| 8 MiB patterned compressed | 0.971 [0.964, 0.979] | 0.807 | 4.055 ms | 6,672 KiB |
| Raw 4 KiB midpoint range | 0.798 [0.789, 0.809] | 0.784 | 0.578 ms | 5,884 KiB |

Open improves about 19–24%, with a 20% total range-read improvement. Read-only
intervals include parity, consistent with an open-path change. Whole-process RSS
ratios are 0.945–0.953, about 5% lower; these are not incremental empty-open
measurements. The optimized binary hash is
`537e74683294d2ae377ab15e2ae369f001670c4af9c707d573592e35fe92549e`.

The simultaneous ZIP total ratios remain:

| Case | Optimized / ZIP, paired 95% interval |
| --- | --- |
| Small files | 0.560 [0.553, 0.569] |
| Raw 8 MiB | 3.505 [3.453, 3.547] |
| Compressed 8 MiB | 2.519 [2.486, 2.548] |
| Raw range | 5.580 [5.416, 5.748] |

The large/range ZIP failures remain explicit. ZIP ranges omit whole-entry CRC;
the protection comparison is limited. This optimization does not establish full
A3/A4/A5 qualification, public/native activation or complete aging. The next
bounded diagnostic attributes remaining payload cost without changing crypto,
codec, verification-before-exposure, padding or wiping guarantees.

## Remaining payload attribution

The [read attribution](profiles/read-attribution.txt), raw reports and call graphs
retain two 1,000-pass user-CPU profiles and separate 10-pass syscall traces of
the frozen optimized reader. Both successful profiles exited zero, with 877 raw
and 627 compressed samples and no lost samples. Traces verify exact bytes.
Profiles include probe setup/open and post-timer verification, and ambient Java
LocalTrialS3/HMB processes were recorded. These are diagnostics, not another
fresh-handle timing batch or an isolated-device throughput measurement.

Raw reads spend 89.62% of sampled user CPU in hardware-accelerated SHA-256.
Each 8 MiB visit reads exactly 8 MiB in 128 archive calls: no duplicated payload
read was found. Compressed visits read 59,919 stored bytes in 32 calls for 8 MiB
logical output. Compressed sampled CPU includes copying (20.41%), Zstd sequence
decoding (18.34%), XXHash (16.27%) and FSE-table construction (10.53%). Copy stacks
include decoder history/output and the common consumer, not just application
staging. Removing benchmark copying would invalidate the controlled comparison.

The profile supports no further high-value application-level change that keeps
the existing integrity/codec contract. Do not bypass stored-fragment checksum,
padding, wiping or decoded-stream checksum to claim parity. Decoder internals or
a persisted integrity/codec change require a separate scoped experiment or
architecture decision. Further retained-corpus coverage can proceed unchanged.

One initial raw profiling setup omitted `REVAULT_CANDIDATE_UNIT` and failed with
`NotPresent` before producing samples. `profiles/raw.data.old` preserves that
failed capture; its original stdout/exit files were overwritten. It is not a
completed profile. The corrected raw profile and compressed profile are the
successful evidence above. Sibling host-context files record the before/after
environment; they were not initially inside the profiles directory.

## 64 MiB coverage attempt

An existing padded plaintext 64 MiB seeded-raw packed-C/ZIP control was found at
`/tmp/revault-compaction-resources-20260927/raw64m`. Creating the corresponding
typed-tree fixture failed with `SecurityLimitExceeded("shared image needs catalogue overflow")`
in the exporter's intermediate dense image. The [failure log](large-raw64m-create.log)
is retained. No timings ran and the retained controls were unchanged. This is an
export-path admission blocker, not a measured 64 MiB read result or proof that
the tree reader itself cannot represent the corpus. No alternative control or
new export path was substituted to make the batch pass.
