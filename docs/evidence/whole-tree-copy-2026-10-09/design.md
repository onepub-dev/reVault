# Selected-tree extent compaction: concrete bounded design

Follow definition checkpoint 7201cfae and its post-format checks. Test-only adapter,
not public activation, release migration or format selection. Copy a stable selected
shared-tree image into one caller-owned empty destination, same archive UUID, mode,
content key and owner. Preserve every selected filesystem object, variable, form
revision and captured record. Relocate live extents; do not repack pack interiors,
remove retained definition revisions, rekey or change compression/padding policy.
Access roots are unsupported and explicitly refused, never omitted silently.

## Publication lineage

Use authenticated successor semantics, consistent with existing separated-layout
relocation: generation = selected.generation.checked_add(1), previous = selected
shared commitment. Refuse exhaustion before destination mutation. Generation1 and
previous0 remain exclusive to fresh export. This successor may be read independently
without the old image physically present; previous names authenticated lineage, not
a source-dependent read requirement. No installation/rollback-resistance claim.

Factor shared initializer internally into common staging with explicit publication
identity. Existing initialize retains generation1/previous0 unchanged. A narrow
successor initializer accepts the authenticated predecessor and revalidates source
selection immediately before initializing; require same UUID/mode, absent access
root, checked generation and predecessor commitment. Preserve existing private
catalogue encoding, both dependency readbacks, initial idle preparation stub bound
to new commitment, dependency sync, first anchor/sync, second anchor/sync. Validate
new anchor with existing Shared rules. New index, catalogue and anchor are freshly
authenticated; only copied payload pages avoid resealing.

## Preflight and placement

Function owns T directly; source is borrowed &S, with stable-storage/read-lock
precondition. No Storage::clone, including for empty MemoryStore destination.
Before any destination mutation OR cleanup: require len0; open/authenticate source
TreeImage; check current selected commitment; require signer/authority for writing;
refuse actual access root; checked successor generation; verify_all logical content,
reclaimed zeros and exact catalogue/graph union. Wrong key/owner, malformed source,
nonempty destination, unsupported access and exhaustion therefore have zero
append/write/truncate/sync calls on destination. Existing guarded normal-open paths
are preserved, including eager signed-plaintext verification.

Plan a BTreeMap oldstart -> (exact old Extent, new Extent), in graph payload order.
New payloads begin after zero control prefix; all additions use checked u64 arithmetic.
No sharing/overlap admitted: graph and catalogue exact union already enforce unique
ownership. Preserve len/digest, change only start. Precompute final data end/alignment,
reject impossible shapes/counts before writes. Rebind in-memory catalogue before
staging: each Pack.extent via exact old triple lookup; fragment relative offsets,
used/padding digest, identities and member order unchanged. Each variable Layout
and every forms.texts_mut Layout gets its ordered list of mapped extents via its
existing rebind validator; UUID/context/revision/ordinal/length/digest unchanged.
Clear old catalogue.vacant only in destination candidate; no old free/pending proof
may authorize source writes. Generate tree_records to admit all typed row/name caps
before staging. Build namespace0 payload claims for the exact new union plus final
alignment-zero claim. Preserve ALL typed namespaces and exact old logical objects.
Index metadata pairs supply their normal absent-key vacant claims on open.

## Staging, verification and cleanup

Only after all preflight succeeds enter owned staging/cleanup boundary. The first
attempted append counts as mutation even if returned failure says no bytes written.
Append control zeros with offset/len checks. For each full source extent, copy using
at most64KiB guarded buffers via read_at_secure and scoped append callback. Stream
SHA256 across all encoded source bytes, require exact expected extent digest. Check
each append's returned offset and physical end; storage failures/partial appends
remain errors. Read back EVERY whole copied extent through guarded chunks, validate
length and independent full stored digest before index/publication. This verifies
tails of extents larger than64KiB, not only first chunk. No plaintext ordinary Vec
or spool. Existing backend/archive caches remain outside scoped-copy guarantee.

Append checked alignment zeros; build sorted index/manifest. Recheck source selected
commitment before successor initialization, then authenticate/initialize successor
as above. Independently open TreeImage on a borrowing View of destination and run
verify_all (including full pack/logical/secure text checks). Assert exact successor
lineage/physical end and recheck source selection after verification. Stable-source
precondition is essential: commitment checks do not detect arbitrary in-place edits
after last content read. Return sole owned destination only after all checks.

On ANY staging-or-later error, call existing bounded discard on that same destination:
zero64KiB chunks, sync, truncate0, sync. Preserve primary failure and cleanup failure
in returned error if cleanup fails; do not claim erased/durable success when either
operation fails. Preflight errors do not call discard(empty), which would mutate via
sync/truncate. Source never receives a mutation call. No clone to recover ownership.

## Qualification

All16 modes: mixed files/directories/symlinks/permissions, normal+secret variables,
all historical definitions and mixed/removed captured fields; changed/deleted state
and >8MiB aggregate selected text verified one scoped value/event at a time. Compare
exact logical objects/IDs/layout context and copied encoded bytes, source identity,
old/new generation+previous; destination ownership union no old free/pending extents.
Report old/new physical size and simultaneous source+destination cost. Extent-only
compaction can preserve unused pack interiors; no universal shrinking guarantee.

Guard fixture explicitly registers COMPLETE old and planned new extent ranges before
chunked append; PAGE_MAGIC discovery alone would miss tails. Reject ordinary read_at
for secure variable/form spans throughout, and all source/destination payload spans
during the observable new relocation/readback phase; existing file-codec preflight/
final verification keeps its ordinary wipe-on-drop buffers. Trap source/destination
Clone. Reviewed guarded routes establish buffer provenance; guard alone is not a
universal proof.
Include >64KiB extents and tail corruption/readback failure controls.

Refusals: nonempty unchanged, wrong authority/key, valid actual access-slot source,
generationMAX authenticated source, corrupt selected content/metadata, sourcecommit
switch before publication/after verification. Require zero destination mutations for
preflight errors. Exhaustive returned write/append/sync prefixes across representative
modes with source unchanged and successful discard empty; explicit readback faults,
cleanup zero/sync/truncate failure preserve both errors. Independently reopen all
successful results. No process-death or region-loss claim from these returned faults.
Existing initialize/fresh export controls must remain valid after common helper refactor.

After this checkpoint, integrate with filesystem AtomicFileReplacement and all-record
verification, then process-death installation checks. Existing compact_path is file-only
atomic installation, NOT a reusable resumable temporary-copy protocol; any resume
support needs separate design. Access/mirror/public filesystem policy, native recovery,
performance/aging and compatibility gates remain open. No fixed CPU/resource batch
without a separately frozen hypothesis and plan.
