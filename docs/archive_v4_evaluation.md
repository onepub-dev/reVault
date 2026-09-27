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
| C | Authenticated index, publication and allocator prototype | Independent fragments, atomic file updates and read-only salvage pass internal tests; latest unpacked comparison still fails read/space targets. Public paths and full qualification remain |

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
| A1 | Public CLI append/reuse abort coverage and allocator fault/power-loss tests pass; C atomic file lifecycle, source-change abort and 100-cycle aging preserve ownership, whole-pack erasure and no-change bytes | Full public mutation matrix incomplete |
| A2 | C fresh read-only salvage preserves intact neighbours, uses metadata mirrors and reports lost proofs; production integration and two native recovery failures remain | Production gate failed; candidate integration incomplete |
| A3/A4 | Latest unpacked C is 3.75× ZIP raw / 3.84× compressed (8 MiB). Initial corrected A/B compressed ratios are about 4.8×/5.1×. Bounded A has one passing 30-pair raw-create result; protected/complete matrix remains | A3 failed; A4 not qualified |
| A5 | A 1 GiB creation is about 191 MiB peak RSS; previous unpacked C probe is 14.2 MiB (one measured pair). Allocator aging stabilizes; new pack and 100k-file resources are not yet measured | Full resource/space gate incomplete |
| A6 | Historical reader routing repaired locally; owner identity and missing v4/current Vault fixtures block the full matrix; CLI/binding interoperability still required | Failed |
| A7 | Canonical goals, current plan/scorecard and proposed decisions exist; history is separated; new test-only pack vector exists; complete normative wire spec remains pending | In progress |
| A8 | Separate #322 branch removes native Linux D-Bus build dependency; clean install, headless lifecycle and private logind tests pass | Broader platform/store/CI matrix incomplete |

## Selection discipline

P1 closes when mode assignment, budgets and decision records have named reviewers
and are frozen. Prototype work does not accept a changed guarantee. P3 requires
complete same-contract candidate results, public semantics and resource evidence;
no format is selected by the amount of code already written. Preserve independent
recovery, erasure, padding and authority even when a speed/space target fails.

## Evidence index

Historical conditions and raw data remain authoritative for their own revisions.
The [checkpoint history](archive_v4_history.md) retains earlier commentary.

| Evidence | Scope |
| --- | --- |
| [Baseline reproduction](evidence/v4-baseline-2026-09-26.md) | CLI mirror aborts, native failures and migration blockers |
| [Initial CPU/RSS](evidence/archive-evaluation-2026-09-26/README.md) | A/B/ZIP and GB resource baseline |
| [Bounded A writer](evidence/writer-memory-2026-09-26/README.md) | Streaming staging and paired creation improvement |
| [Recovery proofs](evidence/recovery-commitments-2026-09-26/README.md) | Owner-authorized membership without global payload verification |
| [Publication](evidence/publication-anchors-2026-09-27/README.md) | Mirrored selection, fault and process-death protocol |
| [Initial index](evidence/authenticated-index-2026-09-27/README.md) / [packed index](evidence/packed-index-2026-09-27/README.md) | Rejected one-record leaves and bounded packed-page comparison |
| [Preparation journal](evidence/preparation-journal-2026-09-27/README.md) / [allocation accounting](evidence/allocation-accounting-2026-09-27/README.md) | Reservation, erasure, reuse and aging |
| [Separated mirrors](evidence/separated-mirrors-2026-09-27/README.md) | Failure-region repair and bounded map banks |
| [File extents and encoder fix](evidence/candidate-file-extents-2026-09-27/README.md) | Correctness integration and mandatory dependency release fix |
| [First file comparison](evidence/candidate-file-comparison-2026-09-27/README.md) | Failed reads/space and descriptive 14.2 MiB GB creation probe |
| [Ordered reads](evidence/ordered-file-reads-2026-09-27/README.md) | Paired traversal results and CPU profiles |
| [Normalized buffers](evidence/candidate-buffer-normalization-2026-09-27/README.md) | Latest measured unpacked C, still failing ZIP parity |
| [Independent physical packs](evidence/independent-file-packs-2026-09-27/README.md) | Packing/deletion correctness, damage containment and privacy; no new timings |
| [Candidate file lifecycle](evidence/candidate-file-lifecycle-2026-09-27/README.md) | Atomic updates, source consistency, aging and explicit owner-authorized read-only salvage |
