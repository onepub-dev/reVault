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
| C | Authenticated index, publication and allocator prototype | Independent fragments, atomic file updates and read-only salvage pass internal tests; packed comparison still fails read/space targets. Public paths and full qualification remain |

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
| A1 | Public CLI append/reuse abort coverage and allocator fault/power-loss tests pass; C atomic file lifecycle/source-change abort preserve ownership and erasure; 4,000 mixed operations preserve content/no-change bytes but unpadded growth fails stability; file compaction passes 90 write faults and 16 process deaths | Full public mutation matrix incomplete |
| A2 | C fresh read-only salvage preserves intact neighbours, uses metadata mirrors and reports lost proofs; production integration and two native recovery failures remain | Production gate failed; candidate integration incomplete |
| A3/A4 | Packed C fails ZIP parity. Shared-control reader after 100 metadata cycles is 0.626× ZIP for small plaintext (upper 95% 0.654), but large plaintext remains 2.85–3.36× ZIP and raw range 3.18×. Bounded A has one passing 30-pair raw-create result; full write/production comparison remains | Small-read subcase passed; full A3 failed; A4 not qualified |
| A5 | Packed C has descriptive 14.5 MiB GB creation / 66–67 MiB 100k-file creation RSS; 100k reads take 41–61 s. After compaction: small archive 6.08× ZIP (failed), raw 64 MiB 1.016× (space subcase passed); unpadded aging grows 128 KiB at cycle 421; full matrix incomplete | Full resource/space gate incomplete |
| A6 | Historical reader routing repaired locally; owner identity and missing v4/current Vault fixtures block the full matrix; CLI/binding interoperability still required | Failed |
| A7 | Canonical goals, current plan/scorecard and proposed decisions exist; history is separated; new test-only pack vector exists; complete normative wire spec remains pending | In progress |
| A8 | Separate #322 branch `d8dfa6cf`: clean release install, installed headless lifecycle, private logind, locked/denied/hung Secret Service and existing-keyring interoperability pass | Actual macOS/Windows credential services, binding packages and remote CI remain unqualified |

The [mixed-file aging qualification](evidence/mixed-file-aging-2026-09-27/README.md)
retains the failed unpadded stability assertion and all safety/accounting checks.

The [fresh shared-control file image](evidence/dense-file-image-2026-09-27/README.md#measured-persisted-image-sizes)
now measures 327,680 bytes for both retained small plaintext and encrypted/signed
fixtures: 1.382× the 237,078-byte ZIP, with every persisted byte verified after
reopen. Five cases match their earlier projections. This is bounded, immutable,
file-only size evidence; full A5 and A3/A4 remain unqualified for this variant.
[Metadata edits](evidence/dense-metadata-update-2026-09-27/README.md) now exercise
persisted ownership and retirement, but the first edit retains two extra 64 KiB
control allocations. The fresh size result does not qualify a mutable archive. The
[paged-catalogue cost comparison](evidence/paged-catalogue-cost-2026-09-27/README.md)
fails both primary small cases at all four declared page sizes; no page writer
is justified by that budget. Authenticated metadata-tail retirement is the next
bounded protocol experiment. Its [persisted lifecycle evidence](evidence/metadata-tail-retirement-2026-09-27/README.md#retained-persisted-lifecycle-results)
now returns both small archives to 320 KiB after each of 100 metadata edits, with
448 KiB temporary size. This passes only that metadata-size subcase; CPU/write
costs, mixed payload aging and complete public semantics remain unqualified.
[Salvage/locality checks](evidence/dense-salvage-2026-09-27/README.md) preserve
intact neighbours after isolated corruption, but expose larger logical loss per
physical region in denser packs. This trade-off remains part of selection.

## Whole-format eligibility review

The file adapter is not a complete archive candidate. The following differences
must be represented in the next layout decision; file throughput cannot stand in
for them.

| Required public semantics | Current implementation to retain | Candidate C gap |
| --- | --- | --- |
| Files, explicit directories, symlinks and permission bits | [TOC entry model](../rust/revault_lockbox_api/src/toc/toc_entry.rs), [node kinds](../rust/revault_lockbox_api/src/model/node_kind.rs) | The [typed metadata experiment](evidence/typed-filesystem-metadata-2026-09-27/README.md) persists actual public node metadata and validates canonical hierarchy/`0777` permissions; node-aware salvage preserves metadata and file-result status together; public filesystem operations and payload mutation remain unconnected |
| Normal and secret variables, explicit sensitivity changes | [Variable API](../rust/revault_lockbox_api/src/lockbox/variables.rs) | No typed variable records or secure-value API; opaque index values would not establish sensitivity semantics |
| Form definitions, revisions, record references and field validation | [Definitions](../rust/revault_lockbox_api/src/lockbox/forms/definitions.rs), [records](../rust/revault_lockbox_api/src/lockbox/forms/records.rs) | No schema/revision linkage or cross-record validation |
| Mirror ownership, overlap/adoption and deletion policies | [Mirror API](../rust/revault_lockbox_api/src/lockbox/mirrors.rs) | Atomic file updates exist, but no persisted mirror configuration/ownership integration |
| Password/contact access and owner pinning | [Key slots](../rust/revault_lockbox_api/src/keys/key_slot.rs), [key directory](../rust/revault_lockbox_api/src/file_format/key_directory.rs) | Shared file images support explicit keys and wrapped-key bootstrap; access mutation/overflow, labels and public integration remain absent |
| Vault and binding interoperability | Shared production engine and retained migration fixtures | No candidate public constructor, credential open, binding carrier or migration path |

A concrete bootstrap issue: `Transaction::put_key_record` currently uses the same
`Index` as private contents. In encrypted mode that index requires the content
key to decode it. Storing password/contact wrapping slots there would require the
key before the slots could yield it. The generic key-tree ownership tests therefore
do not demonstrate a working access directory. A selected format needs a bounded,
public bootstrap path for existing wrapped-key algorithms, followed by selected
publication/owner authentication before private records are trusted. Public slots
must not expose private names or grant owner signing authority. This is a format
integration requirement, not a proposal for new cryptography. The
[bootstrap component experiment](evidence/credential-bootstrap-2026-09-27/README.md)
now verifies password/Contact unwrapping, owner/MAC selection and refusal to fall
back after rekeying. Wrapped-key bootstrap is connected to fresh shared-control images; access
mutation/overflow and public API integration remain absent. Its inline limit is
not an accepted access-capacity cut.

There is also a hard size contradiction in C's current placement. Its fixed
publication/journal prefix is 262,144 bytes. Even the smallest two-bank allocation
map reserves at least another 131,072 bytes. That **393,216-byte lower bound**,
before any payload or file index, already exceeds the small-corpus proposed limit
of `1.50 × 237,078 = 355,617` bytes. Faster encoding, caching or compaction cannot
make the current control layout pass that case. The measured 1,441,792-byte
compacted result has further metadata/padding costs on top of that floor.

Physical packing also incorrectly couples two distinct access units: the builder
limits the sum of decoded bytes across a pack to 256 KiB, although its fragments
are independently decoded. The 2 MiB small-file corpus therefore needs at least
eight packs; at 64 KiB minimum padding each, payload allocations alone occupy at
least 524,288 bytes. A replacement layout should independently bound each decode,
each physical allocation, member count and retained descriptor memory. Relaxing
aggregate decoded-pack size is only safe when no operation materializes all members
at once; it must retain independent fragment authentication and whole-pack erasure.

Read-only accounting of the compacted small fixture confirms the complete cost:
262,144 fixed + 131,072 allocation map + 524,288 file index + 524,288 payload =
1,441,792 bytes. The payload contains 177,152 stored fragment bytes and 347,136
padding bytes in eight physical packs. There is no leftover free space to trim.
The doubled descriptor catalogue and aggregate pack bound are therefore separate
layout costs alongside the fixed-control floor.

The next layout comparison must budget public bootstrap, owner publication,
reservation/cleanup state, membership and allocation metadata together. Evaluate
sharing physical control regions while preserving separate failure regions for
redundant copies, bounded parsing, privacy padding and write-before-use ordering.
Do not implement a full public adapter around a layout already ruled out by the
unchanged size gate. A different placement is a new bounded candidate experiment;
retain the current C revision as the correctness/performance control. A product
change to the size/damage/privacy contract is an explicit alternative, never an
implicit consequence of these measurements.

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
| [Dense metadata updates](evidence/dense-metadata-update-2026-09-27/README.md) | Bounded rename/permission COW, persisted free/pending state and interruption recovery; payload mutation/overflow absent |
| [Shared ownership](evidence/shared-ownership-2026-09-27/README.md) | Complete bounded physical graph, fresh-reader zero checks and COW obligations; no mutation executor |
| [Dense file image](evidence/dense-file-image-2026-09-27/README.md) | Fresh persisted file-only image, bounded catalogue, bootstrap/ranges and failure cleanup; immutable and unqualified |
| [Compact preparation](evidence/compact-preparation-2026-09-27/README.md) | 4 KiB stub and mirrored overflow preserve 2,048 reservations; writer/cleanup integration remains |
| [Shared-control image](evidence/shared-control-image-2026-09-27/README.md) | Distinct authenticated profile, role-bound placement and private-envelope/bootstrap integration; no complete archive writer |
| [Shared-control ordering](evidence/shared-control-ordering-2026-09-27/README.md) | Abstract interruption/region-loss model; temporary-space and second-publication costs |
| [Credential bootstrap](evidence/credential-bootstrap-2026-09-27/README.md) | Bounded public wrappers, authority selection and rekey no-fallback component tests |
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
| [Packed comparison and scale probes](evidence/packed-file-comparison-2026-09-27/README.md) | Paired CPU/elapsed/space failures and descriptive 100k-file / GB RSS |
