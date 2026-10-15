# Ordered file-extent reads

Source `4c236915` changes C's file range reader to one ordered authenticated index
walk. It checks exact contiguous chunk keys and descriptor positions, rejects
missing first/last chunks, and authenticates each payload before delivery. It
introduces no persistent cache and changes no stored bytes. Full-range memory is
bounded by the traversal stack and one decoded extent.

The prior C binary is frozen from `ab4e2bc6`; the [initial comparison](../candidate-file-comparison-2026-09-27/README.md)
remains evidence of its failed read/space targets. This checkpoint improves large
streaming reads but does not establish ZIP parity or solve the small-file layout.

## Paired results

The predeclared eight-case batch has 30 measured pairs after three warm-ups per
case, rotating ZIP / old C / new C. Warm OS cache, fresh child and handle, CPU 2,
one worker, default padding. No owned builds/tests overlapped timing. Every sample
independently reopened and compared all bytes with source after timing. Signed
plaintext still verifies eagerly at open. Raw samples retain CPU, RSS, source,
lockfile and binary hashes; inventories and exact cases are included.

Ratios divide new C by old C (primary), except the last column. Elapsed intervals
are paired bootstrap 95%; full metric intervals are retained in JSONL summaries.
ZIP's protection differs from signed/encrypted cases, so those ZIP ratios are
cost context, not equivalent-work gates. All C public-integration limitations
from the initial comparison remain.

| Case | Elapsed new / old [95%] | CPU new / old | Elapsed new / ZIP |
| --- | --- | --- | --- |
| compressed64 | 0.534 [0.529, 0.545] | 0.534 | 13.077 |
| compressed256 | 0.735 [0.730, 0.743] | 0.735 | 7.114 |
| raw64 | 0.359 [0.357, 0.361] | 0.359 | 6.308 |
| small | 1.008 [0.996, 1.027] | 1.008 | 46.043 |
| compressed-signed | 0.864 [0.862, 0.866] | 0.864 | 16.344 |
| compressed-encrypted-signed | 0.726 [0.725, 0.727] | 0.726 | 9.371 |
| raw-range | 0.978 [0.967, 0.994] | 0.978 | 9.312 |
| raw-encrypted-range | 1.010 [1.005, 1.014] | 1.009 | 13.989 |

Large raw streaming improves about 64%; compressed 64/256 KiB improves about
47%/27%. Signed plaintext open itself is unchanged (ratio 1.002, 95% 0.999–1.006),
while total open-and-read improves about 14%. Small-file and single-range elapsed
ratios remain within roughly 3% of old C; they are not meaningful broad wins.
No result qualifies the production format or closes A3/A4.

## Correctness and reproducibility

The 113 format tests and all-targets native/external-source Clippy pass before
commit; post-hook file tests and Clippy also pass. The >512-extent test checks
byte-identical output and at most two reads at any metadata location (the file
lookup and ordered walk), across multiple index leaves. Authenticated missing
first/last ordinal fixtures fail both ordinary open and a deliberately isolated
range-reader check. Existing mode, corruption, source/callback failure and aligned
range tests remain in force.

The common runner now accepts an optional `PRIMARY_EXECUTABLE` after its existing
`OTHER_EXECUTABLE`, recording the override and both executable hashes. ZIP remains
the runner's own control. The adapter may be compiled with
`REVAULT_CANDIDATE_FROZEN_TEST_BINARY=OLD_TEST_BINARY rustc --edition=2021 candidate_driver.rs -o OLD_DRIVER`.
The ordinary new adapter uses `REVAULT_CANDIDATE_TEST_BINARY=NEW_TEST_BINARY`.
Both create and read protocol smoke runs verified twelve observations and two
distinct C executable hashes. Smoke timings are not performance results.

Rebuild exact binaries as in the initial comparison. For each `batch.json` entry:
set the new test binary and selected extent unit, then run
`taskset -c 2 RUNNER run NEW_ROOT files bytes corpus codec mode 30 access 1 default NEW_DRIVER OLD_DRIVER`.
`RUNNER summarize NEW_ROOT/samples.jsonl` reproduces summaries.

## CPU profiles and next step

Separate synthetic-only profiles ran after statistical measurement. They use the
same frozen new binary and fixtures: 1,000 repeated streaming passes of raw and
compressed 8 MiB, and 30 passes of 512 small files. These are diagnostic repeated
reads, not fresh-handle timing evidence. No samples were lost. Raw perf data,
reports and successful verification logs are retained compressed.

Command pattern: set `REVAULT_CANDIDATE_ROOT` to the case's `other` directory,
`REVAULT_CANDIDATE_PHASE=sample`, `REVAULT_CANDIDATE_ACCESS=stream`, the declared
unit and pass count; run
`taskset -c 2 perf record -q -e cpu-clock:u -F 199 --call-graph dwarf,8192 -o PROFILE -- NEW_TEST_BINARY --exact file_format::candidate_files::resource_probe::candidate_file_resource_probe --ignored --nocapture`.
Reports use `perf report --stdio --no-children --percent-limit 1 --sort symbol`.

Raw read user-CPU samples are about 48% SHA-256 and 48% generic `Zeroizing<Vec<u8>>`
destruction. Compressed reads spend about 52% in generic vector destruction and
9% in SHA-256. Small-file profiles show repeated index verification/decoding and
buffer destruction; payload packing alone will not remove those metadata costs.
These percentages describe sampled user CPU, not end-to-end elapsed attribution.

The current production format already has `ZeroizingBytes`, a tested full-capacity
volatile wiping buffer with wide stores. C accidentally used the slower generic
vector wrapper. Normalize bulk C data/index buffers to that existing abstraction
before interpreting the remaining differences as format costs. Keep key handling,
full-capacity wiping, authentication, padding, codec and wire bytes unchanged.
Do not replace integrity checks to improve the comparison. Repeat a declared
batch after checks; then return to packing, control overhead and public semantics.

Small-file space remains a separate architectural failure. C's fixed separated
control area is substantial even without payload; packing cannot by itself prove
the proposed small compressed archive/ZIP ratio. Any changed space guarantee
must be explicit and reviewed, not inferred from a faster read result.
