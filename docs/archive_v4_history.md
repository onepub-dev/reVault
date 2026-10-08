# Archive v4 implementation history

Dated implementation/evaluation snapshots, retained as evidence. Their “next”
statements describe what was planned at that time and are superseded by the
[current delivery plan](archive_v4_plan.md). The [evaluation scorecard](archive_v4_evaluation.md)
is the current acceptance status; the [project goals](../manual/project-goals.md)
remain the sole goals authority. No historical prototype result selects a format.

## Implementation checkpoints

### Borrowed guarded source checkpoint — 2026-10-09

[Resource protocol and correctness evidence](evidence/variable-resource-2026-10-09/README.md)
freeze a FileStore lifecycle control at `9d00053c` before removing the test-only
secret setter's full guarded source clone. Page API 20, segments 4, serial typed
variables 5 and one declared parallel 5-test regression pass, with strict Clippy.
A comment-only probe lint correction is recorded separately. Post-format freeze
and fixed matched resource sampling remain next; this does not qualify full
concurrency, public activation or an archive resource gate.


### Implementation checkpoint — 2026-09-26

Implementation/evidence owner: Codex. Product/security/release reviewers remain
unassigned; no guarantee-changing decision is accepted yet.

* [Baseline evidence](evidence/v4-baseline-2026-09-26.md): 355 default tests pass;
  native has 361 passes and two recovery failures. Five tests are ignored in
  each configuration. The parallel encoder prototype is preserved as a patch.
  The seven ordinary mirror CLI tests pass, and deterministic post-write source
  change/abort tests pass with both default and native layouts, in appended and
  reused space. The historical-reader routing regression passes all seven inputs.
* Historical fixture routing now uses frozen readers. The full retained matrix
  still fails: old expected records omit owner identity, historical signing keys
  are absent, and current v4/container fixtures are missing. Do not waive owner
  preservation to make that gate green.
* [Security model](security_model.md), [evaluation contract](archive_v4_evaluation.md)
  and proposed decisions [001](decisions/001-v4-signing-and-recovery.md),
  [002](decisions/002-v4-layout-and-access-units.md),
  [003](decisions/003-v4-allocation-history-and-reclamation.md) are now reviewable.
  Resource budgets are concrete proposals, not measured passes.
* #322 uses `issue-322-headless-cli`, based on main. The Linux sleep watcher uses
  the existing pure-Rust zbus dependency. A clean container install, missing-bus
  CLI lifecycle and private-logind regression have passed; macOS/Windows runtime
  and locked/denied desktop-store validation remain open.

Next: resolve the signing contract and freeze evaluation budgets, then build the common comparison runner
and bounded candidate C. No new layout or micro-optimization is activated here.

### CPU and memory checkpoint — 2026-09-26

The common Rust runner now executes fresh-process samples with wall/first-byte
timing, user/system CPU, peak RSS, corpus/executable/source hashes, paired raw
results and untimed independent byte verification. Its deterministic tests and
targeted Clippy pass; all protection/codec combinations passed small range smoke
checks. [Evidence and limits](evidence/archive-evaluation-2026-09-26/README.md).

The 30-pair 8 MiB compressed plain read fails ZIP parity for both layouts (A 4.757×,
B 5.119× ZIP). A separate 1 GiB raw probe exposes a default-layout import peak
of 1,066.91 MiB versus 14.64 MiB native; it uses streaming input, not a GB benchmark
buffer. CPU and memory now have measured baseline evidence, but the matrix, aging,
candidate C, signed-recovery implementation and release gates remain open.

Next implementation work: resolve the independent authorization proof and evaluate
bounded staging/access units with this runner. Preserve the measured regressions;
do not resume encoder micro-tuning or claim a winner from the raw GB case alone.

### Bounded writer checkpoint — 2026-09-26

Candidate A now flushes newly allocated file pages under cache pressure using the
existing durable preparation journal, then permits their eviction despite pinned
metadata. It leaves publication and erasure ordering intact; no wire change.
[Source, validation and measurements](evidence/writer-memory-2026-09-26/README.md).

The new 1 GiB raw creation probe falls from 1,067.14 to 190.43 MiB peak RSS. Thirty
paired 256 MiB raw creation samples show 17.7% less elapsed time and 17.1% less CPU;
median RSS falls from 298.98 to 180.64 MiB. Thirty paired compressed 8 MiB reads
show no material regression. These support one bounded-staging change, not a
layout selection or complete A3/A4/A5 qualification.

Validation: 358 default library tests pass; all eight public mirror CLI tests pass
(one separate large-tree test ignored). Native tests retain exactly the two known
recovery failures, with 364 passing. New pressure/rollback regressions cover all
protection/codec modes and storage-operation fault injection. Targeted Clippy,
cache and runner checks pass. Small-file staging and metadata memory remain open.

Next: independent owner authorization/recovery, then the bounded candidate-C
access-unit comparison. Preserve both original and bounded A results; do not
restart parallel-encoder tuning from this local writer improvement.

### Independent-recovery proof checkpoint — 2026-09-26

The [commitment experiment](evidence/recovery-commitments-2026-09-26/README.md)
implements bounded canonical object commitments, flat hashes, Merkle inclusion
proofs, same-identity updates and an independent byte vector. Real archive-byte
and hybrid-signature tests pass in both layouts and signed protection/codec modes.
They reject owner substitution and unpublished roots while permitting an intact
neighbour to verify independently of damaged content.

Thirty-pair component measurements at 512/100,000 objects establish initial-build,
update, proof-verification CPU and memory costs. At 100k, retained hashes occupy
about 8 MiB and a proof occupies 568 bytes. These exclude persistence and payload
work and cannot qualify a complete candidate. Production recovery is unchanged.

Decision 001 now specifies the proof graph and next persistence experiment:
owner-authenticated mirrored publication records plus authenticated keyed-index
child links. Implement that publication boundary and fault/damage matrix before
claiming A2 or moving to full candidate-C performance comparisons. Preserve eager
normal-open verification. The fixed-position experiment is not a final index.

### Mirrored-publication checkpoint — 2026-09-27

The candidate publication layer now implements two fixed authenticated slots,
owner-pinned signatures/MAC policy, bounded mirrored root references, ordered
synchronization and resumable mirroring. A readable pair is explicitly distinct
from a successful durability result. Object counts stay out of public metadata.
[Implementation, codec and fault evidence](evidence/publication-anchors-2026-09-27/README.md).

Thirteen protocol tests pass, including operation/read failures, torn writes,
failed syncs, actual process death during publication and repair, and independent
byte vectors. Stored publication now supplies authority to the real-page recovery
proof test, which passes in both layouts and signed protection/codec combinations.
Targeted Clippy passes. The code remains isolated from normal archive output.

The next checkpoint below supplies the authenticated keyed-index experiment.
Packed index construction, allocator preparation/cleanup and public-path integration remain. The two native recovery failures,
complete candidate-C comparison and release gates remain open. Do not interpret
component fault coverage as whole-archive crash/recovery qualification.

Update this plan and the evaluation scorecard at milestone boundaries. Every
entry states what changed, which gate it affects, what was actually measured,
remaining failures and the next decision. Keep detailed experiment logs beneath
that summary. Changed baselines invalidate affected comparisons until rerun.

### Authenticated-index and resource checkpoint — 2026-09-27

[The persisted index experiment](evidence/authenticated-index-2026-09-27/README.md)
connects selected owner publication to mirrored, encrypted descriptors with bounded
lookup and copy-on-write insertion, replacement and deletion. Nine focused checks,
independent byte vectors, both real archive-layout recovery tests and Clippy pass.
A no-change repeat appends nothing; damage to unrelated metadata does not prevent
proving the survivor. No new wire encoding is activated in normal archives.

The 100,000-record file-backed measurements show a small working set (3.54/3.72 MiB
peak RSS at build end) but unacceptable append-only construction overhead:
698/751 MB written for 75/81 MB of live nodes, with system CPU dominating. Default
padding on this one-record-per-leaf layout would require 24.41 GiB of live index
copies for that descriptor set. Reject that physical layout as a final choice.

The immediate next implementation task is a packed, preferably key-ordered index
with bulk construction, compared against this measured baseline using the same
publication/authorization contract. Preserve default padding and confidentiality;
do not optimize by silently dropping them. Then integrate durable accounting and
reclamation of prepared/retired pairs, separated mirror placement, and complete
candidate-C/public-path tests. The production recovery failures and A3–A8 gates
remain outstanding.
H1 continues independently and is not blocked by that sequence.
The saved parallel encoder experiment is optional evidence to
revisit after selection, not the next task by default.


### Packed-index checkpoint — 2026-09-27

The [packed index comparison](evidence/packed-index-2026-09-27/README.md) replaces the
rejected one-record layout with ordered pages, bulk construction, local split/merge
updates and bounded range/salvage traversal. Fourteen focused checks and both real
archive-layout proof tests pass; code remains isolated from production output.

For 100,000 descriptors, 30 fresh-process observations per mode show approximately
21.4–21.6 MB total index file size, zero discarded construction pages and 48–61 ms
median construction CPU. The largest observed process RSS across construction and
replacement samples was 4,584 KiB. Padded live nodes occupy 20.625 MiB. These are
component measurements over sorted input; they do not prove whole-archive gates.
Single-record updates write more bytes than the previous tree, so page-level batching
and reclamation remain part of the design rather than optional micro-optimizations.

Next: persist allocation/preparation/cleanup ownership for complete and partial node
writes, gate old-state erasure on synchronized mirrored publication, and separate
mirror failure regions. Integrate batched archive operations and data extents, then
run the complete candidate comparison. Preserve the current security, padding,
eager-open and release-compatibility contracts. Production recovery and release
qualification remain unfinished; H1 continues on its independent branch.

### Preparation-journal checkpoint — 2026-09-27

The [journal experiment](evidence/preparation-journal-2026-09-27/README.md) adds
mirrored durable reuse reservations, append-tail rollback, publication-gated
retirement and resumable cleanup. It remains test-only. Fault tests include torn
writes, interrupted recovery, actual file-backed process death and repeated aborts.

Thirty fresh-process measurements per case cover 1, 1,000 and 100,000 free ranges
in all four protection modes. Reservation and rollback add one index-page read at
100,000 ranges; writes and sync counts remain fixed, with zero retained growth.
Maximum process RSS is 5,640 KiB. The fixed 392 KiB/14-sync cost of a failed 4 KiB
operation still needs batch integration and comparison against the control.

Next: derive and audit the complete physical allocation map from reachable data
and control pages; integrate reusable allocation, retirement of the allocation
map itself and batch updates. Then separate mirror failure regions and implement
data extents/public paths for the full A/B/C comparison. The journal alone does
not prove complete archive ownership or lifecycle aging. Production recovery,
CPU/read/write gates, migration, bindings and #322 platform qualification remain
open; retain the compatibility contract and existing verification semantics.

### Allocation-accounting checkpoint — 2026-09-27

The [integrated allocator experiment](evidence/allocation-accounting-2026-09-27/README.md)
derives complete byte ownership from the candidate record/index graph, including
shared payload, both metadata copies, key indexes, discarded preparation writes
and the allocation map itself. Recovery checks the full graph before erasure.
Sorted bulk record replacement and exact no-change handling are implemented.
This remains test-only; typed public records and data codecs are not integrated.

Aging exposed an unsuitable append-only allocation-map policy: tiny padded
workloads grew to about 94 MiB after 1,000 operations. Reusable, explicitly owned
map regions stabilized those same cases at about 1.08 MiB after operation 29.
All 5,600 before/after lifecycle operations independently reopened, verified content
and accounted for every byte; completed retired ranges were zero. The revised
runs stayed below 6 MiB RSS, but long-run operation CPU increased by about 61–63%.
This is a space-reuse result, not an A4 pass; preserve the raw trade-off.

Next: separate metadata/publication/journal mirror failure regions, integrate
batched reservations and updates, and implement the 64/256 KiB data extent sweep.
Use the complete candidate for A/B/C comparison before selection. Public codec,
padding, access/record semantics, native recovery, compaction, migrations, bindings
and headless platform qualification remain open. Do not substitute more encoder
microbenchmarks for those architectural tasks.

### Separated-mirror checkpoint — 2026-09-27

The [separated-mirror experiment](evidence/separated-mirrors-2026-09-27/README.md)
places every metadata pair in different aligned 64 KiB failure regions, including
publication, journal, key/index nodes and banked allocation maps. Explicit repair
authenticates the full ownership graph before repairing copies or reclaiming
space. It preserves payload bytes and refuses to infer membership from older
history. Experimental control encodings advance to version 2 to reject the old
adjacent-slot geometry; no production output changes.

Default and native format suites pass 104 tests. Coverage includes 148 single-region
damage cases, 223 repair mutation failures and 1,784 repair power-loss cases.
Two-copy authority loss fails closed; missing descendant proofs remain unavailable
while unrelated records can be salvaged. The damage guarantee is one aligned
region, not an arbitrary 64 KiB span or protection against payload loss.

The 5,600-operation comparison preserves complete accounting and no-change
behavior. Padded 1,000-operation archives stabilize at 1,114,112 bytes after
operation five. Observed mutation CPU decreases 13.0% (plaintext) and 7.2%
(encrypted-signed), with candidate peak RSS at most 5,984 KiB. Unpadded tiny
archives grow from about 177 KB to 655 KB. These are descriptive component results,
not the statistical performance gates or full candidate-C qualification.

Next: integrate real data extents/codecs and typed record/access semantics so C can
run the complete A/B/C workload matrix. Retain batched update/reservation costs,
raw-range and compressed-decode limits as measured design constraints. Do not
activate C or substitute these component results for public eager verification,
compaction, the two native recovery failures, migration or platform qualification.

### File-extent integration checkpoint — 2026-09-27

The [file adapter](evidence/candidate-file-extents-2026-09-27/README.md) now connects
candidate ownership/index records to bounded raw/Zstd data extents and streaming
file/range reads. Per-extent membership supports files exceeding the earlier
512-extent record envelope; signed plaintext retains eager ordinary-open checking.
This remains test-only and file-only, with small-file packing and public record/
access integration still outstanding.

Integration exposed an existing Zstd encoder defect: reversed raw Huffman-weight
nibbles can silently change decoded content. Independent reference decoding proves
the fault and the two-line correction. A temporary vendored published-source patch
makes local builds reproducible. A corrected dependency release and isolated
packaged-build verification are now explicit prerequisites for release. Rebuild
both A and B with the correction for further performance comparisons.

Next: run the 64/256 KiB file workload sweep, including CPU/RSS and the default
padding cost; implement shared small-file packing and complete mutation/public
semantics before treating C as eligible for architecture selection.


### File-only comparison checkpoint (2026-09-27)

The [first A/B/C file comparison](evidence/candidate-file-comparison-2026-09-27/README.md)
is complete for its declared six-case batch. All controls use the corrected
encoder. C is not selected: read targets and small-file space use fail, despite
bounded GB streaming memory. Next replace repeated per-extent index lookup with
an authenticated ordered traversal, test its bounded reads and error behaviour,
and compare against the frozen C binary in a new declared batch. Then address
payload packing and public record/access integration. Do not interpret component
or GB resource wins as closing P2, A3/A4 or release qualification.


### Ordered read checkpoint (2026-09-27)

[Ordered traversal evidence](evidence/ordered-file-reads-2026-09-27/README.md)
shows 64% less raw streaming elapsed time and 27–47% less compressed streaming
elapsed time versus the first C adapter, with unchanged signed-open guarantees.
All eight declared cases are retained; small files and ZIP parity still fail.
Profiles identify a comparison mismatch: C uses generic byte-vector wiping while
A already uses the tested full-capacity wide-store buffer. Reuse that abstraction,
verify unchanged safety and wire behavior, and run a bounded follow-up batch.
Then resume payload packing, control-space costs and full record/access design;
do not treat this buffer normalization as selecting a format.


### Buffer normalization checkpoint (2026-09-27)

The [bounded follow-up batch](evidence/candidate-buffer-normalization-2026-09-27/README.md)
reuses the existing production secure buffer without changing wire bytes or safety
checks. C now reads the designated 8 MiB raw/compressed cases in about 3.75/3.84×
ZIP duration: still failed targets. Stop this tuning sequence and implement the
shared-pack ownership/mutation experiment next, preserving whole-pack retirement,
authenticated slice coverage and recovery. Fixed control-space overhead and public
record/access integration remain separate blockers; no candidate is selected.


## Earlier evaluation commentary


The [CPU/memory baseline](evidence/archive-evaluation-2026-09-26/README.md)
retains exact commands, raw paired samples, hashes, measurement scope and the GB
probe. It is initial evidence for the comparison, not a selection of candidate B
or qualification of a release. Creation/read memory trade-offs now explicitly
constrain the candidate-C and bounded-staging designs.

The [bounded writer evidence](evidence/writer-memory-2026-09-26/README.md) retains
its exact source patch, CPU/RSS samples and safety checks. Candidate A has changed;
future architecture comparisons must identify whether they use original or bounded A.

The [recovery commitment experiment](evidence/recovery-commitments-2026-09-26/README.md)
adds a tested proof model and component CPU/memory costs. The subsequent
[publication experiment](evidence/publication-anchors-2026-09-27/README.md) now persists
and authenticates the selected anchor with fault/process-death coverage. The
[authenticated index](evidence/authenticated-index-2026-09-27/README.md) connects that
root to mirrored private descriptors. Its 100k-entry CPU/RSS/space measurements
reject one-record-per-leaf physical storage: small resident memory does not offset
excessive append-only construction and padded space. The subsequent
[packed-page comparison](evidence/packed-index-2026-09-27/README.md) removes that
construction waste: 100k descriptors use about 21.4–21.6 MB, with 48–61 ms median
construction CPU over 30 fresh processes per mode. Maximum observed process RSS is
4,584 KiB. It retains default padding and authority but increases single-record
write amplification; batching, allocator/reclamation and full archive gates remain.
The normal archive still needs selected index, allocator and public-path integration.

P1 closes only when the proposed mode assignment, budgets and decisions have
named reviewers and are frozen. P2 begins with an instrumented common runner and
bounded candidate-C prototype after that contract. P3 requires all candidate
results, not a preference for whichever branch contains the most code. Preserve
independent recovery and ownership constraints even if no candidate meets speed
budgets; revisit the specific trade-off explicitly.


The [first file-only A/B/C comparison](evidence/candidate-file-comparison-2026-09-27/README.md)
uses the corrected encoder across all candidates. C fails read and small-file
space targets: 256 KiB compressed C is 9.64× ZIP / 2.02× A, and 512 small files
read 92.9× slower than A. Its 1 GiB creation probe uses 14.2 MiB RSS versus A's
190.9 MiB, but that single measured pair is descriptive only. Preserve both the
resource improvement and failed cases. Next evaluate ordered extent traversal;
payload packing and complete public semantics remain architecture blockers.


[Ordered C reads](evidence/ordered-file-reads-2026-09-27/README.md) remove repeated
per-extent index traversal. New C is 6.31× ZIP for 8 MiB raw and 7.11× for 256 KiB
compressed units; the same-revision old/new comparison shows useful streaming
improvements but unchanged small-file weakness. All modes retain authentication;
signed plaintext open stays eager. Profiles require normalizing C's bulk memory
wiping to the existing production abstraction before further layout interpretation.


[Normalized bulk buffers](evidence/candidate-buffer-normalization-2026-09-27/README.md)
remove C's avoidable generic-vector wiping cost using production's existing tested
abstraction. Raw/compressed 8 MiB C/ZIP duration ratios are now 3.75/3.84, with all
checks retained; A3 still fails and no new paired C/A A4 result is claimed. Next
address shared-pack ownership and physical deletion, separately from control-space
and metadata-access costs. Keep historical checkpoints as evidence, not current
gate status or a reason to continue isolated buffer tuning.
