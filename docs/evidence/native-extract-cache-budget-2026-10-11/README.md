# Native ExtractMany cache budget trial

## Completed result — 2026-10-12

Retain the native Auto ExtractMany cache change. Corrected epoch 2 completed
320 accepted CLI processes (256 measured, 64 preparation), verified identities
after timing, and stopped all sixteen owned helper agents. Every measured trial
CLI stayed below 100,000,000 bytes peak RSS. All eight aggregate elapsed-time
95% intervals met the frozen upper-bound limit of 1.05. Point estimates range
from 0.966 to 0.995 times the old elapsed time; this is an aggregate decision,
not a claim that every timing stratum excludes a 5% regression.

| Workload | Layout | Trial/old elapsed, 95% interval | Maximum old → trial RSS, bytes |
| --- | --- | --- | --- |
| Source 64 MiB | A | 0.976 [0.958, 1.004] | 91,684,864 → 35,291,136 |
| Source 64 MiB | B | 0.968 [0.941, 0.996] | 80,617,472 → 49,000,448 |
| Source 1 GiB | A | 0.976 [0.969, 0.983] | 236,343,296 → 77,238,272 |
| Source 1 GiB | B | 0.995 [0.991, 0.999] | 151,138,304 → 49,168,384 |
| Random 64 MiB | A | 0.966 [0.912, 1.024] | 145,858,560 → 68,186,112 |
| Random 64 MiB | B | 0.975 [0.921, 1.033] | 79,015,936 → 43,380,736 |
| Random 1 GiB | A | 0.982 [0.973, 0.992] | 236,617,728 → 75,231,232 |
| Random 1 GiB | B | 0.969 [0.954, 0.988] | 144,457,728 → 43,610,112 |

Random/raw 64 MiB even-round strata have upper bounds 1.05885 (A) and
1.07721 (B), so per-stratum 5% noninferiority is not established. The frozen
aggregate rule passes. At 1 GiB, maximum process RSS falls about 67–70%.
The complete audited results are in [metrics-summary.json](raw/metrics-summary.json).
The original protocol and corrected environment amendment remain below.

This establishes the one-file plaintext `extract --to` cases only. Cache targets
are per handle; parallel many-file extraction can multiply caches. Pending writes
and active shared owners can exceed retention targets. ReadMostly, credential
unlock, helper-agent memory, encrypted workloads and overall G11 qualification
remain separate. Authentication, wiping and explicit cache policies are unchanged.


This is the next bounded optimisation experiment after the October 11 native/public extraction baseline. It changes only native `CacheLimit::Auto` retention for `ExtractMany`: 32 MiB for decoded pages/native frames and 8 MiB for the separate decoded compression-frame cache. These are clean-cache retention targets, not total-process memory limits. Pending writes and active shared owners keep their existing guarantees. Explicit cache policies, WASM, and transitions wholly between other workload profiles retain current behavior.

The baseline must finish and its identities and results must be verified before applying the trial. Preserve the baseline executable files and all existing evidence. Use the same four immutable plaintext unsigned archives per layout: 64 MiB and 1 GiB source/Zstd, and 64 MiB and 1 GiB random/raw. Do not rebuild fixtures or include candidate/ZIP cases in this experiment.

The four variants are A-old, A-trial, B-old, and B-trial. A is the public default issue303-protocol build; B explicitly enables native-block-layout. The trial binaries differ from the respective baseline only by the reviewed native Auto ExtractMany cache patch and any required build metadata. Pin both old/new executable hashes, source patch, manifests, toolchain, runner, protocol, and archive/source identities before timing.

Every sample launches a separate public CLI process with `archive extract --to NEW_DIRECTORY`, CPU affinity 0–4, isolated HOME/XDG/Vault/agent directories, and platform secret storage disabled. The environment does not inherit ambient credentials or configuration. The CLI selects ExtractMany. Default permissions, the exact single-file output tree, and persisted bytes are independently verified after process exit. GNU time reports total process elapsed time, user/system CPU and peak RSS. Output is drained with bounded sync calls outside the process timer before removal and the next sample.

Run four excluded preparation rounds and sixteen measured rounds for each case and variant: 320 total processes, 256 measured and 64 preparation. At case index c, round r and position p, choose variant `(p + floor(r / 2) + c) mod 4`. Each measured variant occupies every position twice in each even/odd round stratum. These are order strata, not claims about the actual CLI reader TID parity. Preserve the exact observed command, order and all raw time logs. No extra timings, tests or builds overlap this batch.

Before each process, sync and evict only its owned archive, requiring fincore resident bytes zero with at most three attempts. CLI processes do not expose the component reader-TID or physical-read observations; do not claim those admission guarantees here. Retain rejected attempts and stop on incomplete capture, timeout, nonzero exit, malformed/nonfinite metrics, changed identities, output mismatch or failed drain. One process has a maximum 180-second work timeout with a 15-second termination grace, and each attempt is shortened to the remaining batch budget. The full batch has a 60-minute cap. Preserve partial evidence on failure. Retain the existing 24 GiB owned-artifact limit and free-space headroom, including old/new binaries and the next output file.

For each layout and workload, compare trial/old elapsed time using paired round log ratios and a seeded 10,000-replicate percentile bootstrap over eight adjacent even/odd two-round blocks. Also report each round stratum using eight paired observations. Report geometric mean ratios, 95% intervals, median throughput/CPU, every measured peak RSS and the maximum. These local decision intervals are not comprehensive public endpoint, storage-device, workload or architecture qualification; there is no multiplicity correction.

Retain the trial only if every affected layout/workload preserves byte/permission/identity gates, each measured trial process stays below 100,000,000 bytes RSS (GNU time KiB converted with 1024), and every aggregate trial/old elapsed interval has upper bound at most 1.05. Report any stratum whose upper bound exceeds 1.05; do not claim per-stratum noninferiority in that case. A statistically supported slowdown greater than 5% in any workload rejects this policy. If the aggregate intervals remain uncertain, do not silently increase sampling or tune thresholds: preserve the failed/uncertain trial and make a separately justified next decision.

Only after a useful measured result, run focused cache/authentication and shared-frame reuse tests. Cover reversible Auto profile transitions, explicit Bytes/Disabled behavior, and preservation of dirty staged data; avoid a full suite for this individual experiment. Broader integration occurs after batching retained runtime changes. This trial does not close password/contact bootstrap memory, the separate explicit single-file ReadMostly CLI path, full format-4 recovery qualification, or release readiness.

## Environment correction before accepted timing

The first cache trial was canceled independently of its measured results after discovery that every declared agent socket path was 117–121 bytes. Linux Unix-domain socket paths permit at most 107 pathname bytes plus the terminating NUL. The public CLI attempts secret-activity registration; failed agent startup waits three seconds and its error is currently swallowed. That unrelated wait can mask extraction regressions. The owned process exited 130 with 126 rows, no completion marker, and no remaining workload children. Those rows are incomplete, invalid timing evidence and are not used for a retention decision. The original runner, identity manifest, protocol and partial logs remain preserved.

The earlier native/public baseline has a narrower issue: its source/Zstd 64 MiB agent socket path is 108 bytes, whereas the other case paths fit. Its component results remain qualified as recorded; the public source64 elapsed observations include this confound. Its RSS observations establish a lead but do not replace corrected matched timings.

Epoch 2 changes only the harness environment to short, isolated case/variant directories under `revault-native-public-20261011/e2/cNvN`. Before freezing, each of the sixteen contexts runs the public `session --format json` command, requires `agent/running=yes`, a successful public list protocol roundtrip and an actual owned Unix socket, and records the full path byte length. It uses an explicit private agent log. Merely preparing a directory or a successful swallowed-error command is insufficient. The regular activity registration and sleep protection remain enabled. All sources, binaries, archives, sample counts, ordering, time/RSS criteria and gates are unchanged.

Owned agent processes remain available through the trial, and public `session stop` cleans up only these contexts afterward. The process-RSS observations cover the measured public CLI process; persistent helper-agent RSS is not included and remains a qualification limit. The 320-process epoch uses `identities-v2.json` and `batch-v2`; it never replaces or resumes the invalid partial epoch.

## Raw sample packaging

Per-sample logs and rows are losslessly stored in [sample-evidence.tar.gz](raw/sample-evidence.tar.gz). Every extracted file was compared against [the SHA-256 manifest](raw/sample-evidence.sha256); original local files are also preserved under ignored build storage. Extract the archive from `raw/` before rerunning the summarizer. Identity manifests, completed summaries, frozen protocols and validation logs remain directly readable. Source hashes describe the measured pre-format epoch; commit-hook formatting does not change the tested behavior.

## Focused validation

Seven release-mode filters passed: 24 test executions covering 23 distinct tests. They cover authenticated cache hits and disabled admission, dirty payload preservation, reversible profile changes, explicit policies, shared-frame reuse, native index/decoded budgets and oversized streams. The initial test-only inspector API compile typo and successful retry are both preserved in [validation](raw/validation/result.json), with exact commands and logs. No full suite ran for this individual experiment; broader integration remains scheduled after batching retained changes. The tested pre-format source snapshot and hashes are retained there; the tracked hook performs final formatting.
