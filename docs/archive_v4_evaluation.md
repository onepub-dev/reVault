# Archive v4 evaluation contract and scorecard

2026-09-26. Owner: Codex (implementation and evidence collection). Product,
storage/security and release reviewers: unassigned. Status: concrete proposed
measurement contract; budgets below require review before the P2 comparison is
frozen. No candidate has been selected or qualified for release.

The [goals](../manual/project-goals.md), [security contract](security_model.md)
and [delivery plan](archive_v4_plan.md) define purpose, guarantees and sequence.
[Baseline evidence](evidence/v4-baseline-2026-09-26.md) records newly run checks.
Historical microbenchmarks remain evidence about their own revisions.

## Comparable candidates

| Candidate | Control/prototype | Eligibility |
| --- | --- | --- |
| A | Current default data pages plus v4 transactions | Baseline; default library suite passes. Release migration matrix fails |
| B | Same source with `native-block-layout` | Experimental; two reproducible recovery failures. Not eligible for activation |
| C | Bounded pack/extent prototype | Not implemented. Must preserve A's durability and all required protection modes to compete |

Do not count a prototype lacking signatures, padding, wiping or crash recovery as
a faster implementation of the same contract. A cost-only prototype may reject a
hypothesis but cannot establish a release winner. Keep at most one primary layout
hypothesis active per experiment; stop after its predeclared measurement batch.

## Primary workloads and statistics

Primary local read cases: 512 files × 4 KiB, individual 1/8/64 MiB files, and a
pinned source-tree corpus. Each uses compressible, seeded incompressible and mixed
bytes. Record corpus generator/version, seed and SHA-256 inventory before timing.
The real corpus should be a clean Git archive of `0f9a137e`, excluding retained
binary fixtures and generated assets; freeze its exact inventory in the runner.

Measure open separately from read, plus total open-and-read. A handle is fresh for
each first-read observation. ZIP and Lockbox produce byte-identical outputs;
verification occurs outside timing. ZIP codec, compression level and CRC checking
are recorded; signed Lockbox costs are not interchangeable with ZIP CRC costs.
Default privacy padding is the primary case. No-padding is a separate result.

For A3 the primary comparable mode is unencrypted and unsigned; report signed
plaintext separately with its eager verification cost. Product review must accept
this mode assignment before freezing the contract: “unencrypted” in the goal
alone does not resolve signed-open costs. Measure whole file, extraction and raw
4 KiB/64 KiB ranges. Compressed-range amplification is a separate gate and must
not be hidden inside a whole-file result.

Use 30 paired observations per case after three untimed warm-up pairs. Alternate
AB/BA order. Compute per-pair log duration ratios; report geometric mean ratio
and a paired bootstrap 95% interval with a fixed seed and 10,000 resamples.
Retain samples, not just rounded summaries. Increase samples only under a recorded
new experiment; do not keep sampling until an interval happens to pass.

* Statistical parity: upper 95% ratio bound <= 1.00 against ZIP. This stringent
  interpretation is proposed for A3; a 5% equivalence band would be a changed gate.
* At least 10% faster: upper 95% duration ratio bound <= 0.90. This means elapsed
  time, not a silently substituted throughput percentage.
* Open/write nonregression A4: upper 95% ratio bound <= 1.05 against candidate A,
  for the same protection, codec, padding, cache and durability policy.
* No pooling a failing primary case into an aggregate that conceals its failure.

Separate warm OS cache/fresh handles, controlled cold I/O, and repeated decoded
cache reads. If cold-cache control is unavailable, label the result unmeasured.
No owned builds or other tests may overlap timed runs. Record CPU, RAM, storage,
OS/kernel, toolchain, source/lockfile hashes, features, thread count and load.
The first matrix uses one worker; evaluate bounded parallelism separately.

## Mutation, aging and resource budgets

These are initial engineering budgets, not measured claims or established product
promises. Freeze or amend them with reasons before selecting an architecture.

| Area | Proposed budget / method |
| --- | --- |
| Mirror lifecycle | Public CLI create, repeat, add, larger/smaller replacement, remove, threshold refusal, changed-source abort; independent reopen and exact-byte verification |
| Leak/accounting | Zero unowned ranges and zero retired payload after completed reclamation; unchanged repeat adds zero allocated bytes after initial stabilization; 100 cycles required |
| Aging | 1,000 mixed update cycles; classify growth as payload, padding, retained control/history, reusable or pending; no unexplained growth accepted |
| Raw small range | At most 64 KiB payload fetched to return an aligned 4 KiB range, excluding separately counted cold metadata |
| Compressed random range | At most 256 KiB decoded for a 4 KiB request in the interactive profile; whole-file codec cost separately reported |
| GB streaming | At most 256 MiB incremental peak RSS over empty-open baseline, with one worker; no allocation proportional to total file length |
| Parser/index at 100k entries | At most 256 MiB incremental peak RSS; bounded, documented counts/lengths; reject malicious sizes before allocation |
| Worker policy | One worker baseline; explicit total in-flight encoded/decoded budget <= 256 MiB in parallel experiments; WASM one-worker path |
| Recovery | Linear work in scanned bytes plus indexed records; no full archive reopen per recovered file. Proposed local floor: 50 MiB/s on the benchmark host, reported separately for crypto/codec modes |
| Compaction | Source retained until verified replacement; peak extra space <= replacement size + 256 MiB, excluding existing backups; report actual required headroom before starting |
| History/reservations | Report cost versus transaction count and free-range count; failed 4 KiB write tested against 1, 1k and 100k free ranges; select a bounded policy in decision 003 |
| Storage overhead | Raw 64 MiB corpus <= 1.10 × ZIP size after compaction; 512 × 4 KiB <= 1.50 × ZIP; protection/padding metadata included. Other modes reported separately |

The proposed 256 KiB compressed access unit is deliberately incompatible with
claiming that a 16 KiB authentication block makes a multi-MiB Zstd frame seekable.
A candidate can fail the proposed budget; record the failure instead of changing
the workload. Review this trade-off before writing a replacement codec layout.

For each operation record end-to-end/first-byte duration, CPU, peak RSS, archive
size, decoded/read/written/zeroed bytes, allocations, syncs and remote request
counts. A sum of overlapping stage medians is not total elapsed time.

## Correctness and conformance matrix

Cross encryption on/off × signing on/off × raw/Zstd × default/no padding ×
append/reused/shared allocations. Exercise files, symlinks, directories, normal
and secret variables, form schemas/records, access metadata and mirror ownership.
Inject failures and process death at reservation, write, root/auth, sync,
publication, cleanup, checkpoint and truncate. Repeat interruption during recovery.

Damage tests must separately destroy an unrelated payload block, subject block,
TOC, frame descriptor, selected auth record, inactive header slot and selected
header slot. Test substitutions by an attacker with the content key, across
archive IDs, commits, paths and frames. Assert authority and report categories,
not merely recovered byte counts. Internal corruption/accounting inspection is
permitted where no public CLI operation can create or observe that condition.

Migration keeps immutable old bytes and verifies logical records, owner identity,
access slots, options and later mutations. Never generate old fixtures with the
new writer. Missing fixture credentials are not permission to replace an owner.
CLI/binding version pairs must read and write both directions within `0.5.x`,
including Vault containers. `0.4.x` remains format 3 and is isolated from this work.

## Current scorecard

| Gate | Current evidence | Status |
| --- | --- | --- |
| A1 | Default/native transaction tests pass; deterministic public CLI abort after physical writes passes for append and reuse in both layouts; full P4 matrix remains | Baseline established; qualification incomplete |
| A2 | Two native recovery tests fail; signed recovery depends on whole snapshot validity | Failed |
| A3/A4 | Initial 8 MiB compressed read: A 4.757× ZIP, B 5.119× ZIP. Bounded A passes 30-pair 256 MiB raw create nonregression (17.7% less elapsed, 17.1% less CPU); read comparison unchanged | A3 still fails; A4 passes one write case, full matrix incomplete |
| A5 | Bounded file-page staging reduces default 1 GiB create peak from 1,067.14 to 190.43 MiB in the new paired probe; small-file/index/aging matrix remains | Measured large-file creation improved; full resource gate incomplete |
| A6 | Historical reader routing repaired locally; snapshot owner coverage and missing v4/current Vault fixtures block full test | Failed |
| A7 | Goals, plan, security/evaluation drafts and three proposed decisions exist; normative wire spec remains historical | In progress |
| A8 | Independent #322 branch removes native Linux D-Bus build dependency; clean install, headless lifecycle and private logind tests pass | Linux fix verified; broader platform/store matrix incomplete |

## Decision gates

The [CPU/memory baseline](evidence/archive-evaluation-2026-09-26/README.md)
retains exact commands, raw paired samples, hashes, measurement scope and the GB
probe. It is initial evidence for the comparison, not a selection of candidate B
or qualification of a release. Creation/read memory trade-offs now explicitly
constrain the candidate-C and bounded-staging designs.

The [bounded writer evidence](evidence/writer-memory-2026-09-26/README.md) retains
its exact source patch, CPU/RSS samples and safety checks. Candidate A has changed;
future architecture comparisons must identify whether they use original or bounded A.

P1 closes only when the proposed mode assignment, budgets and decisions have
named reviewers and are frozen. P2 begins with an instrumented common runner and
bounded candidate-C prototype after that contract. P3 requires all candidate
results, not a preference for whichever branch contains the most code. Preserve
independent recovery and ownership constraints even if no candidate meets speed
budgets; revisit the specific trade-off explicitly.
