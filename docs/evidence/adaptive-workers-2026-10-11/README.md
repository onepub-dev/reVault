# Lazy descriptor-aware worker trial — 2026-10-11

Status: complete, 720 verified samples; opt-in policy retained, serial default
unchanged. Base is `1a7242a3`; exact pre-format source, executable and runner
identities are retained in `raw/pre-timing-identities-v2.sha256`. The persistent user goal
continues until remaining usage is below 10% or above 99% after a reset, replacing
the earlier 50% cutoff. Resume reading: 98% remaining.

## Policy under test

Experimental worker value zero selects automatic mode. It initially uses the
ordinary serial codec path and creates no pool. After delivering 16 MiB of a
contiguous compressed run in the current request, it can decode subsequent
compressed chunks with four workers. Each batch has at most 32 chunks. A raw
chunk resets qualification and uses the direct serial path; a saved authenticated
lookahead avoids rereading its metadata. Each new request starts qualification
again, even if a pool survives from a prior request. A one-chunk remainder stays
serial. This trial deliberately gives up the potential four-worker gain at exactly
16 MiB to avoid guessing future work from the archive mode or first chunk.

Encoding comes from authenticated descriptors, not a scan or new wire field.
Existing commitment verification, zeroizing buffers, callback ordering and
bounded batch lifetimes remain mandatory. Optional pool-creation failure still
returns an error in this opt-in trial; default activation would require reviewing
serial fallback for that resource failure.

## Frozen measurements

Compare explicit one worker, explicit four workers and automatic mode in one
fresh executable/source epoch. Each case has four excluded preparatory rounds
and twelve measured paired rounds. Rotate the three variants by round plus case
index and alternate reader TID parity; across twelve rounds each variant occupies
each position twice within each parity. Use the same affinity CPUs 0–4,
cold-file-cache admission, two-read physical-I/O calibration, byte verification
and out-of-timing output drain as the completed crossover experiment. No builds
or tests overlap timing. Preserve every failure and reject incomplete cases.

One file per case, 64 KiB units, default padding, unsigned candidate authority:

| Profile | MiB | Protection/compression |
| --- | --- | --- |
| Pinned source prefix/repetition | 1, 16, 17, 64, 256 | Zstd/plain |
| Pinned source prefix/repetition | 64 | Zstd/encrypted |
| Pinned source prefix/repetition | 64 | Raw/plain |
| Pinned random prefix | 64 | Zstd-enabled/plain, raw fallback |
| Raw first half, source second half | 64 | Zstd/plain |
| Source first half, raw second half | 64 | Zstd/plain |
| Alternating raw/source 64 KiB chunks | 64 | Zstd/plain |
| First 1 MiB of each 16 MiB interval source; rest raw | 64 | Zstd/plain |
| Source first 17 MiB; remainder raw | 64 | Zstd/plain |
| Source final 1 MiB; earlier raw | 64 | Zstd/plain |
| Source final 16 MiB; earlier raw | 64 | Zstd/plain |

These are 15 cases and 720 accepted samples (540 measured, 180 preparatory).
Mixed files take bytes at corresponding logical offsets from the pinned expanded
source and random streams; retain exact source hashes and authenticated raw/Zstd
census for each fixture. The 17 MiB and short compressed-prefix controls expose
startup after qualification when little compressed work remains.

Use the existing 10,000-replicate seeded paired log-ratio bootstrap over six
adjacent two-round parity-balanced blocks, plus within-parity results. Report
automatic/serial and automatic/explicit-four comparisons, CPU, RSS and actual
pool activation. Confidence intervals have no multiplicity correction and are
specific to this host and these cases. Focused tests additionally cover short
cross-chunk ranges, repeated requests, raw transitions, active-batch read bounds,
refusal after qualification, callback failure/unwind and pool reuse.

## Acceptance and next work

For default eligibility, the automatic/serial elapsed ratio's aggregate 95%
upper bound must be at most 1.05 for every raw, fallback, alternating, sparse,
tail-only and 1/16/17 MiB or compressed-prefix boundary control. An upper bound
above 1.05 is uncertain or failing evidence and does not permit a non-inferiority
claim or default activation. A lower bound above 1.05 establishes a material
regression and rejects the policy for that control. Useful improvement requires
an upper bound below one both in aggregate and in each parity stratum for the
64/256 MiB homogeneous compressed cases and the two half-mixed profiles.
These are experimental admission rules, not universal guarantees or corrected
multiple-comparison intervals. CPU increases remain acceptable when elapsed
improves; retain the recorded memory bound.

If boundary evidence rejects the first threshold, make a bounded revision and rerun affected pairs;
preserve the rejected epoch. Do not claim arbitrary mixed-file non-regression,
native/public CLI performance, encrypted-raw crossover, or release qualification.
Continue the performance goal after this milestone, subject to actual usage.

## Completed results

All 720 samples passed independent byte comparison, cold-cache admission,
physical-read gates and source/executable identity checks: 540 measured and
180 preparatory samples. The [paired summary](raw/adaptive-workers-batch/metrics-summary.json)
retains elapsed/CPU/RSS and parity-stratified results for all three comparisons.
Ratios below are automatic/serial; lower elapsed is better.

| Profile | Elapsed ratio | 95% interval | CPU ratio |
| --- | ---: | --- | ---: |
| mixed-alternating-64k-64mib | 1.0066 | 0.9994–1.0145 | 1.0064 |
| mixed-raw-first-source-second-64mib | 0.9058 | 0.8810–0.9231 | 1.2180 |
| mixed-source-1mib-per-16mib-64mib | 1.0046 | 0.9942–1.0163 | 1.0051 |
| mixed-source-final-16mib-64mib | 1.0014 | 0.9944–1.0117 | 0.9953 |
| mixed-source-final-1mib-64mib | 0.9962 | 0.9886–1.0051 | 0.9969 |
| mixed-source-first-17mib-64mib | 1.0047 | 0.9780–1.0246 | 1.0414 |
| mixed-source-first-raw-second-64mib | 0.8913 | 0.8826–0.8984 | 1.1526 |
| random-zstd-fallback-plain-64mib | 0.9881 | 0.9788–0.9975 | 0.9870 |
| source-raw-plain-64mib | 0.9833 | 0.9683–0.9957 | 0.9837 |
| source-zstd-encrypted-64mib | 0.7984 | 0.7840–0.8115 | 1.5527 |
| source-zstd-plain-16mib | 0.9962 | 0.9870–1.0048 | 0.9961 |
| source-zstd-plain-17mib | 1.0335 | 1.0175–1.0490 | 1.0887 |
| source-zstd-plain-1mib | 0.9881 | 0.9675–1.0071 | 0.9947 |
| source-zstd-plain-256mib | 0.8803 | 0.8562–0.8985 | 1.9402 |
| source-zstd-plain-64mib | 0.8081 | 0.7854–0.8344 | 1.5696 |

Every control's aggregate upper interval is at most 1.05. All five useful
compressed/half-mixed profiles have upper intervals below one in aggregate and
both reader-parity strata. Automatic mode improves elapsed time by 9.4–20.2%
for those profiles. These are the frozen host-specific gates, not a universal
non-regression guarantee.

The 17 MiB homogeneous control is 3.35% slower on average (aggregate upper
1.0490); its parity-zero upper interval is 1.0681. Thus aggregate control
non-inferiority passes, but all-parity control non-inferiority is not established.
Qualification deliberately forgoes the large explicit-four-worker gain at 16 MiB.
No threshold revision is justified by the prespecified acceptance rule.

Retain automatic mode as an explicit experimental choice (`workers=0`), with
serial still the default. Four focused tests pass: two existing parallel-read
tests and two automatic-policy tests covering active-pool refusal, callback
failure/unwind, pool reuse, per-request/raw reset and bounded payload lookahead.
No full suite was repeated for this experiment; broader validation belongs to a
batched integration checkpoint. The initial fixture-preparation failure and
superseded pre-timing runner identity remain evidence, not accepted samples.

## Next milestone and limits

The user goal continues; the post-batch account reading is 4% used (96% remaining).
Next compare current default native, explicitly enabled native-block layout,
typed-tree candidate and ZIP through a matched sink, then separately measure a
public CLI extraction including startup, permissions, credential costs and total
RSS. CLI fixtures and persisted results must use separate public invocations.
Reconcile current migration/security evidence with the architecture scorecard.
Do not extend this into repeated threshold tuning before that comparison.

No architecture is selected. This candidate policy does not activate native
recovery, establish public CLI/PGP/write competitiveness, qualify all candidate
security paths, or satisfy cross-version release compatibility. Default promotion
also needs optional-pool resource failure to preserve a serial fallback.

## Reproducing the evidence

The [round/fixture audit](raw/adaptive-workers-audit.json) checks every retained
round against its sample JSONL, exact workload/census/source identities, the
three-way rotation and expected pool activation. Every variant occupies every
position twice within each measured parity stratum. Peak extraction RSS is
14,656 KiB across all 720 rows and 14,628 KiB across the 540 measured rows.

The final measured runner is `raw/adaptive-workers-runner.dart`, SHA-256
`d24e6558302dec44c93419014fcd2c821e076f0189cac61f43e37ffc1be7f7ae`.
`pre-timing-identities-v2.sha256` identifies the final pre-timing review boundary;
earlier identity files and runner snapshots are explicitly superseded. The frozen
protocol remains `raw/protocol-before-measurements.md`. Source hashes and
`raw/source-diff-1a7242a3.patch` describe the tested **pre-format** epoch; the commit
hook formats Rust only after validation. Do not equate pre-format hashes with
the formatted committed files.

Verbose per-case evidence is preserved losslessly in
[`raw/sample-evidence.tar.gz`](raw/sample-evidence.tar.gz):
6340 files, 1637615 compressed bytes,
SHA-256 `321d75a1248465cdec7f70d7df81976fcf0701baffe43958dbada70fbbd59a9e`. Every extracted file was verified against
`raw/sample-evidence-sha256.txt` before original directories were moved to the
ignored `rust/target/adaptive-workers-preserved-samples/originals` backup.
Extract the tar under `raw/adaptive-workers-batch` before rerunning
`tools/summarize-adaptive-workers.dart` (choose a new output filename) and
`tools/audit-adaptive-workers.dart`. Top-level summaries, setup logs, frozen
identities and audit results remain directly reviewable.

The two focused test logs and tested source patch are stored as `.gz` files to
preserve exact whitespace. Their pre-timing hashes refer to the decompressed
bytes. Restore these three files with `gzip -dk` before rerunning the audit; the
compressed carriers retain the original log/patch content without normalization.
