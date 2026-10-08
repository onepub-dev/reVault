# Typed-tree controlled read comparison

Date: 2026-10-02. The candidate core follows the
[240-test mutation/recovery checkpoint](../typed-dense-payload-2026-10-02/README.md).
This is a bounded read experiment, not native activation or release qualification.

## Identity and method

The [core manifest](final-core-source.sha256) identifies 28 changed source paths;
its [verification log](final-core-verification.log) records the isolated source
check. The [harness manifest](harness-source.sha256) identifies the three adapter
files, copied under [harness/](harness/). They add canonical-path typed export and
read sampling to the existing common probe. The isolated harness passed its
[build](harness-build.log) and [all-mode byte-verification smoke](harness-smoke.log).
The empty Clippy output represents only the standalone driver check with
`clippy-driver -D warnings`, not full isolated-library Clippy. Main-source strict
library Clippy is separately recorded in the mutation checkpoint.

**Compiler caveat discovered after measurement:** copying only `rust/` omitted
the root `rust-toolchain.toml`. The isolated typed reader used Rust 1.94.1, while
the retained packed-C resource and ZIP runner used Rust 1.88.0 (confirmed from
ELF compiler comments). Consequently these ratios do not isolate layout from
compiler effects. Preserve this exploratory batch; a pinned 1.88.0 repeat is
required before interpreting differences as layout-only results.

The [binary manifest](binaries-before.sha256) pins the typed resource executable,
driver, common runner and two packed-C controls. The earlier 237-test snapshot
remains preserved separately and is **not** the candidate measured here. See the
[batch scope](batch-scope.txt) for complete interpretation of legacy runner labels:
`primary` means retained packed C and `other` means the new typed tree. Historical
source/layout labels in runner output do not identify the candidate core.

Each plaintext, unsigned, default-padded case has 30 measured pairs after three
warmups, alternating ZIP/C/tree and tree/C/ZIP, one worker/pass, CPU 2, fresh
process and handle, warm OS cache. Exact bytes are verified outside timing.
Source inventories and archive/public-key hashes are checked before and after;
the runner reported all five executable hashes unchanged. No task-owned build
or test overlapped timing. Recorded host load fell from 1.16 to 1.03; persistent
HMB collectors remained. Cold I/O and complete host isolation are not measured.

## Results

Ratios are typed-tree duration divided by control duration, including normal
open. Lower is better; intervals are paired fixed-seed bootstrap 95% intervals.
All observations are retained in [measurements/](measurements/), with the complete
[metric table](measurements/paired-results.tsv).

| Case | Tree / ZIP, 95% interval | Tree / packed C, 95% interval | Tree median total | Tree median peak RSS |
| --- | --- | --- | ---: | ---: |
| 512 × 4 KiB mixed compressed | 0.602 [0.596, 0.607] | 0.02655 [0.02630, 0.02681] | 5.211 ms | 6,732 KiB |
| 8 MiB seeded raw | 3.627 [3.590, 3.667] | 0.9843 [0.9782, 0.9910] | 6.125 ms | 6,176 KiB |
| 8 MiB patterned compressed | 2.596 [2.573, 2.618] | 0.6542 [0.6500, 0.6588] | 4.141 ms | 6,968 KiB |
| Raw 4 KiB midpoint range | 7.002 [6.740, 7.264] | 0.9028 [0.8884, 0.9204] | 0.724 ms | 6,196 KiB |

Only the small-file case passes the proposed ZIP parity/10%-faster subcase.
Large raw, compressed and range cases still fail ZIP parity. ZIP range reads
omit full-entry CRC; this range comparison has weaker protection than Lockbox.
No aggregate may conceal these failures. RSS is whole-worker lifetime high
water, about 1.34–1.41× packed C, not incremental memory above empty-open.

The raw tree open is 1.212× C [1.198, 1.225]; the range open is 1.225× C
[1.202, 1.250]. In contrast, raw read-only time is 0.963× C and range read-only
time is 0.201× C. Compressed open is 0.200× C while compressed read-only time is
0.963× C. This identifies tree-open work as a useful narrow diagnostic without
claiming it explains the large payload gap. The new experiment will preserve
these baseline samples and all authentication, padding and wiping checks.

This batch does not establish A4 against the same-protection production default,
the full A3 workload matrix, mutation speed, full aging or public semantics.
Signed/encrypted reads, migration and the two native recovery failures remain
outside this measured batch.
