# Lazy descriptor-aware worker trial — 2026-10-11

Status: policy and matrix frozen before timing. Base is `1a7242a3`; exact source,
executable and runner hashes will accompany results. The persistent user goal
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
