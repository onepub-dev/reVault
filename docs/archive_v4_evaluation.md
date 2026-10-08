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

The [selected typed-variable integration](evidence/typed-variables-2026-10-09/README.md)
is a **test-only (`cfg(test)`) adapter**, following the independently checked
[secure segmented-value component](evidence/secure-segments-2026-10-09/README.md).
It connects selected membership/exact payload ownership to guarded value reads,
transactions and salvage without putting values in index entries. The pre-final
variant passes 257 release format tests (8 ignored) with serialized execution
and strict Clippy; final affected and post-format checks pass with separate evidence. Representative
fault controls cover 3,432 transaction cuts and 2,724 interrupted recoveries.

Two concurrent focused runs failed secure allocation at the host's 8,192 KiB
locked-memory limit; isolated 1 MiB functionality and serialized checks pass.
The [borrowed source experiment](evidence/variable-resource-2026-10-09/README.md)
removes one full staging clone; final affected correctness, strict Clippy and one
declared parallel regression pass. Fixed 30-pair/mode evidence reduces every
locked-memory endpoint by 1,028 KiB, with create/no-change CPU ratios 0.939/0.959
across modes. Three read modes show small regressions; whole-process peak RSS
barely changes and incremental peak remains unqualified. All 992 producer and
independent-reader attempts plus 32 reciprocal frozen-version readers pass.
Retained allocator arenas and remaining staging copies are still a qualification
gap. In the two-small-variable fixture, independent 64 KiB loss affects both
variables in unpadded modes and one in padded modes. Public variable/form APIs,
complete variable migration/compaction and CPU/RSS/aging/ZIP gates remain open.
No architecture or release readiness follows from these correctness results.

[Atomic typed variable moves](evidence/variable-moves-2026-10-09/README.md)
add metadata-only name changes without changing guarded value identity/revision
or payload extents. All-mode lifecycle/refusal/selected salvage and 1,512 fault
cuts plus 504 interrupted recoveries pass before and after formatting; public activation
and full-format performance qualification remain separate.

The [October 9 direct tree exporter](evidence/fresh-tree-export-2026-10-09/README.md)
removes fresh packed-C export's dense-intermediate requirement while preserving
source authority/retention and fresh-output cleanup. Seven focused tests cover
all-mode files beyond dense metadata capacity, 93 fault cases and explicit
refusals; the full format suite passes 248 tests (8 ignored) plus strict Clippy.
The streamed 64 MiB raw/protected probe is functional capacity evidence only;
it does not requalify earlier timing, remove experimental count bounds or pass
public API/CPU/incremental-memory/ZIP gates.
Post-format format/probe/Clippy checks passed. Both preserved native recovery
failures reproduce under `native-block-layout`; their assertions remain intact.


The [October 8 owned-byte correction](evidence/decoder-owned-storage-2026-10-08/README.md)
passes 17 vendor decoder unit tests, 15 workspace tests (1 ignored), no-default-
features checking, 21 release compression tests, 241 format tests (7 ignored)
and strict Clippy. Owned byte releases and decoded-block bounds are now tested;
full decoder-derived memory wiping is not claimed. Post-format checks passed.
The fixed codec-only batch regressed 1.040×/1.382×/1.443× for owned 4 KiB/256 KiB/
8 MiB and 1.021× for the 256 KiB static control (all intervals above 1). It includes
both corrections and cannot establish an A3/A4 archive pass. Erasure remains
a correctness requirement; process peak RSS is not incremental decoder memory. A separate fixed
wide-write follow-up retained full-capacity erasure and measured 0.970×/0.770×/
0.741× owned lifecycle time against the byte-wise correction (static control
0.994×). Residual cost against the original unwiped decoder was not directly
remeasured; do not multiply ratios across batches or claim archive parity.


The [October 8 resumed checkpoint](evidence/resume-2026-10-08/README.md)
reproduces 21 compression tests, 241 format tests (7 ignored), strict Clippy and
13 vendor workspace tests (1 ignored), using Rust 1.88.0. This is regression
validation of the accumulated source, not a new performance or release claim.
Historical `/tmp` benchmark binaries/corpora are unavailable; future comparisons
must reconstruct matched controls and record new identities.

The [final combined decoder checkpoint](evidence/decoder-final-2026-10-02/README.md)
passes 21 compression tests, 241 format tests (7 ignored), strict Clippy and
before/after source identity checks. Declared-length rejection now retains the
zeroizing output guard. The [isolated inline trial](evidence/decoder-inline-2026-10-02/README.md)
supports a 4.4% compressed plaintext read gain, with other read intervals
including parity and unexplained RSS/open movement explicitly unattributed.
The final combined source completes four matched paired cases with no observed
read/total regression versus the accepted FSE baseline; all byte and inventory
checks pass. Raw-control shifts limit attribution, and no RSS gain is claimed.

The promoted [equivalent FSE-table arithmetic](evidence/decoder-fse-2026-10-02/README.md)
passes 11,188,905 reference comparisons and main compression/format/Clippy checks.
Compared with the checksum-corrected baseline, read time falls 4.5% for compressed
plaintext 8 MiB, 3.9% for protected 8 MiB and 8.7% for small mixed files; the raw
read interval includes parity. This is a measured local decoder gain, not closure
of the remaining large/range ZIP failures or whole-format qualification.

The [full-buffer checksum and failure-wipe correction](evidence/decoder-checksum-2026-10-02/README.md)
is promoted with 20 compression tests, 241 format tests (7 ignored) and strict
Clippy passing in the main worktree. Four matched before/after cases show no
material total regression; small read-only cost rises about 0.9%. This fixes a
pre-existing comparison omission without removing stored-byte authentication.
The overlap-copy experiment remains rejected for lack of target read benefit.
The wipe correction covers fresh output and scoped arena storage. At that October 2 checkpoint, ordinary
vendor-owned decoder history still deallocated without zeroization. The October 8
owned-byte correction closes those specified release paths; full decoder-derived
memory wiping remains outside that scoped guarantee.

The [64 MiB transition-built extension](evidence/typed-tree-64m-2026-10-02/README.md)
adds two matched-compiler retained-control cases without raising source/staging
caps. Stream tree/C is 0.999 [0.967, 1.033], an inconclusive speed difference;
range is 0.821 [0.798, 0.844]. Tree/ZIP is 3.081× stream and 15.118× range,
both failing parity. That batch retained fresh export's dense-intermediate
admission failure and used a different construction route through validated
mutation. The October 9 adapter now removes that admission failure separately;
this historical timing batch is not relabeled as the repaired exporter or a
public streaming claim.

The [pinned Rust 1.88.0 batch](evidence/typed-tree-pinned88-2026-10-02/README.md)
now supersedes exploratory cross-compiler retained-control comparisons. Six
cases complete 30 pairs/three warmups. Plaintext tree/ZIP totals are 0.546× small,
3.567× raw 8 MiB, 2.524× compressed 8 MiB and 5.896× range; only small passes.
Protected comparisons retain separate weaker-ZIP labeling. Process RSS is
1.51–1.58× packed C. The pinned build passes 241 format tests and strict Clippy;
the new manual aging probe passes separately in all 16 modes at 1,000 cycles
each, with the unchanged first-16 completed-size ceiling. Seven normal-suite
ignored probes do not imply seven completed qualifications. Public whole-format
semantics, native recovery and full A3/A4/A5 qualification remain incomplete.

The first October 2 isolated read batches used Rust 1.94.1 because the copied
source omitted the root toolchain pin; retained C/ZIP used 1.88.0. Treat their
cross-control ratios below as exploratory, not layout-only gate evidence. The
original/optimized typed-reader pair is compiler-matched. A pinned 1.88.0 repeat
is being collected; main correctness/Clippy qualification already used 1.88.0.

The [single-traversal typed-open change](evidence/typed-tree-single-pass-2026-10-02/README.md)
passes 241 format tests (6 ignored) and strict Clippy, reducing measured open
time 19–24% and raw-range total time about 20% versus the original typed reader.
The simultaneous ZIP total ratios are 0.560× small, 3.505× raw 8 MiB, 2.519×
compressed 8 MiB and 5.580× raw range. Only the small-file subcase passes;
the larger/range failures and incomplete whole-format gates remain.

The [typed-tree read comparison](evidence/typed-tree-read-2026-10-02/README.md)
passes only the small-file ZIP subcase (0.602× total time). Raw 8 MiB (3.627×),
compressed 8 MiB (2.596×) and raw 4 KiB range (7.002×) remain failed ZIP cases.
All four use 30 pairs/three warmups with retained controls and byte verification.
Raw/range tree open and process RSS regress against packed C; read-only work is
faster. This is a local plaintext padded batch, not full A3/A4/A5 qualification.

The [dense payload dispatch checkpoint](evidence/typed-dense-payload-2026-10-02/README.md)
passes 240 format tests (6 ignored) and strict Clippy, with 1,620 focused
interruption cases, capacity-refusal byte preservation and 192 sampled large
overflow faults across dense/tree starting layouts. Bounded all-mode aging
passes; complete whole-format aging, public semantics and native recovery are
not implied. Controlled read comparison follows this scoped mutation tranche.

The [vacant-payload reuse checkpoint](evidence/typed-payload-reuse-2026-10-02/README.md)
passes 237 format tests (6 ignored) and strict Clippy, with bounded all-mode
32-cycle stabilization and reused-write/overflow fault evidence. This is not the
complete 1,000-cycle whole-format workload or a fixed cross-fixture size bound.

The [typed payload mutation checkpoint](evidence/typed-payload-2026-10-02/README.md)
passes 233 format tests (6 ignored) and strict Clippy. It adds bounded tree-only
file addition/replacement/removal with verified neighbour preservation and whole
retired-pack erasure. Appending changed packs still requires payload-reuse/aging
work; its single resource observation is not a write-performance or full A5 pass.

The [in-place typed transition checkpoint](evidence/typed-transitions-2026-10-02/README.md)
adds automatic dense/tree routing and return-to-inline with exact initial-size
restoration on the bounded all-mode fixture. Final format regression passes 228
tests (6 ignored), with strict Clippy. Its 4,572 transition fault cases and serial
resource observation are component evidence, not public/native activation or
whole-format aging, transient-space or performance qualification.

The later [typed-tree checkpoint](evidence/typed-tree-2026-10-02/README.md)
passes 5 focused tests, 225 format tests (6 manual probes ignored) and strict
Clippy. It adds typed metadata mutation and independently authenticated read-only
salvage to the candidate tree, including all-mode membership/copy-loss tests and
1,128 growth interruptions in four modes. This does not activate native recovery;
the retained native failures and full qualification statuses remain unchanged.

October 2 local checkpoint: the [raw-tree transition/reuse adapter](evidence/shared-tree-transitions-2026-10-02/README.md)
passes focused fault tests, 220 format regressions and strict Clippy. Metadata
and journal arenas can be reused under authenticated preparation, with bounded
all-mode size observations retained. This advances component integration only;
none of the full-gate statuses below changes. The first C mixed-aging fix
failed and was removed. A subsequent [encoding-bounded map-bank change](evidence/compact-map-bank-2026-10-02/README.md)
passes the unchanged stability gate, with higher retained unpadded size. Both
native recovery failures were reproduced. The
serial CPU/RSS observation had concurrent HMB tests and is not paired performance
qualification. Source hashes distinguish this uncommitted checkpoint from the
September 27 file-only timing baselines.

| Gate | Current evidence | Status |
| --- | --- | --- |
| A1 | Public CLI append/reuse abort coverage and allocator fault/power-loss tests pass; C atomic file lifecycle/source-change abort preserve ownership and erasure; October 2 bounded map-bank candidate passes all 4,000 mixed operations and the unchanged stability gate; file compaction passes 90 write faults and 16 process deaths | Full public mutation matrix incomplete |
| A2 | C fresh read-only salvage preserves intact neighbours, uses metadata mirrors and reports lost proofs; production integration and two native recovery failures remain | Production gate failed; candidate integration incomplete |
| A3/A4 | Packed C fails ZIP parity. Shared-control reader after 100 metadata cycles is 0.626× ZIP for small plaintext (upper 95% 0.654), but large plaintext remains 2.85–3.36× ZIP and raw range 3.18×. Bounded A has one passing 30-pair raw-create result; full write/production comparison remains | Small-read subcase passed; full A3 failed; A4 not qualified |
| A5 | Packed C has descriptive 14.5 MiB GB creation / 66–67 MiB 100k-file creation RSS; 100k reads take 41–61 s. After compaction: small archive 6.08× ZIP (failed), raw 64 MiB 1.016× (space subcase passed). Historical unpadded aging grows 128 KiB at cycle 421; October 2 bounded map banks stabilize by cycle 100 but retain 1,507,859 bytes, above the old final 1,376,256; full matrix incomplete | Full resource/space gate incomplete |
| A6 | Historical reader routing repaired locally; owner identity and missing v4/current Vault fixtures block the full matrix; CLI/binding interoperability still required | Failed |
| A7 | Canonical goals, current plan/scorecard and proposed decisions exist; history is separated; new test-only pack vector exists; complete normative wire spec remains pending | In progress |
| A8 | Separate #322 branch `d8dfa6cf`: clean release install, installed headless lifecycle, private logind, locked/denied/hung Secret Service and existing-keyring interoperability pass | Actual macOS/Windows credential services, binding packages and remote CI remain unqualified |

The [mixed-file aging qualification](evidence/mixed-file-aging-2026-09-27/README.md)
retains the historical failed unpadded stability assertion and all safety/accounting checks.
The October 2 change passes that same gate; neither historical evidence nor the
storage-cost trade-off is discarded.

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

The [prepared staging prerequisite](evidence/prepared-staging-2026-10-09/README.md)
passes 32 fresh all-mode FileStore probes with one/twelve 1 MiB values and affected
variable, ready-vector and production correctness checks. Maximum observed VmLck
endpoint is 2,788 KiB under 8,192 KiB; no peak, concurrency, timing or A3/A4/A5
qualification follows. Two-pass encoding and additional pre-write checks are new
work not covered by the frozen borrowed-source ratios. Forms remain pending.

The [typed form snapshot adapter](evidence/typed-forms-2026-10-09/README.md)
connects full-size metadata/values and exact references to selected ownership in
test-only code. The pre-final format suite passes 267 tests (11 ignored), and final affected
checks/Clippy pass; separately frozen initial/final 16-mode batches of fresh
13.6 MB FileStore snapshots reopen and verify under the unchanged 8 MiB memlock
policy, with a maximum 4,076 KiB endpoint. Full multi-secret object assembly,
concurrency, mutation and public activation remain unqualified;
no performance gate follows from these functional endpoint probes.

The [form deletion/streamed salvage followthrough](evidence/form-lifecycle-2026-10-09/README.md)
passes all-mode selected membership and exact-definition damage controls, 1,704
transaction cuts and 564 interrupted recoveries. Sixteen fresh aggregate streams
verify all eight 1 MiB secret values individually, with a maximum 4,076 KiB VmLck
endpoint. Post-format affected checks and strict Clippy pass at `9ec6c89f`. Operational failures remain fatal after possible partial delivery;
whole-call staging and stable storage are required. This is functional bounded
processing and modeled recovery evidence, not performance or process-death qualification.

The [test-only atomic form moves](evidence/form-moves-2026-10-09/README.md)
preserve payload identity/bytes and exact old/new parent sets across 1,488 modeled
transaction cuts and 504 interrupted recoveries. All-mode functional controls
and strict Clippy pass. This supplies no new resource/performance qualification.

The [selected-source prerequisite](evidence/selected-source-2026-10-09/README.md)
passes 3,288 modeled transaction cuts, 1,020 interrupted recoveries and 16 fresh
13,631,586-byte aggregate replacements with a maximum 4,076 KiB locked-memory
endpoint. Archive sizes increase to 28,311,552/58,916,864 bytes; no performance,
concurrency, aging or complete-format gate is closed.

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
| [Shared ownership](evidence/shared-ownership-2026-09-27/README.md) | Complete bounded physical graph, descendant placement/retirement proofs, fresh-reader zero checks and COW obligations |
| [Shared overflow reader](evidence/shared-tree-reader-2026-09-27/README.md) | Selected private manifest, authenticated index traversal and complete allocation ownership; typed records, transition writer and journal overflow remain pending |
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
