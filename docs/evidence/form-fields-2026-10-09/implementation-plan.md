# Connected typed form field mutation plan

Scope: test-only shared-tree normal/secret field setters, including the public
cross-record automatic secret upgrade established at7cee4820. No public activation,
general schema-edit API, wire selection or independent record mutation counter.
Prerequisite8cec9211 provides selected-source preparation; postformat checks pending.

## Admission and logical transition

Normalize record path and field ID using existing public validators. Validate
caller text/size and selected latest field kind before constructing a candidate.
Resolve the record's type, then the maximum selected definition revision. Missing
record/field refuses; full verify_all remains mandatory before successful mutation
or no-change. Invalid requests cause no persistent writes. Normal input to latest
Secret refuses; historical Secret capture does NOT independently forbid normal
input when the latest schema removed/recreated the field normal. Reuse public
kind rules, including scoped SecretString validation; expose a narrow crate-visible
borrowed text validator only if needed, preserving behavior and public tests.

For Secret input to latest normal kind, checked_add definition revision, copy all
immutable definition metadata to fresh text UUIDs/revision1 with new type/revision
contexts, and make the target field Secret. Retain old definitions and their pages.
Advance definition ref and alias on EVERY same-type record, even those lacking
this captured field. Matching captures adopt latest label and Secret kind; preserve
other captures. Normal captured values become secret; historical secret values
retain bytes/layout when destination context already matches. Target override is
planned directly from caller, never by staging an intermediate conversion. Other
types remain unchanged. For ordinary/repeated-secret setting without schema upgrade,
only target record ref/alias and target capture follow the public setter behavior.

## Reuse, identity and no-change

Record UUID remains stable. A changed existing text retains its UUID only with a
checked text revision increment; overflow refuses before writes. New captures get
fresh text UUIDs/revision1. New definition texts always use fresh UUIDs because old
revisions remain selected. Context includes role, parent/type/revision where
applicable, field ID, kind/sensitivity; equal bytes do not permit reuse across a
context change. No-change requires matching ref/alias, capture kind/sensitivity,
context, current label and caller bytes after full verification. Compare texts
sequentially in guarded scopes; no all-values materialization or allocation inside
active read scope. Repeated equal secret setting may be byte-identical in this
adapter; public baseline promises logical no-change, so qualify this optimization.
A changed label but equal value rewrites only the label when value context matches.

## Mixed prepared sources and single publication

Build a metadata-only candidate and worklist with one item per changed text:
Stored{old_layout,destination} or Caller{borrowed normal/SecretString,destination}.
Source layouts needed for copied latest labels come from the selected OLD latest
definition, not newly prepared pages. Deduplicate exact source extents, admit them
through SelectedSource factory before first source preparation. All needed old
values are already semantically verified; prepare_stored additionally retains its
per-text/segment validation. No generic Storage clone.

A new mixed batch owns PreparedSecurePages, global ordinal ranges, rebind IDs and
StoragePayloadCallback<'a> dispatchers. Stored callbacks own immutable layouts/key
and use supplied ReadView; caller callbacks borrow the immutable input and ignore
ReadView while calling existing prepare_source rendering. The global dispatcher
maps page index to local ordinal, passes the view by reference-only copy and ends
all source scopes before writer render/write/readback. Empty selected-source list
is valid for caller-only mutation. No callback may read uncommitted new pages.

Build retirement union from exactly replaced old texts (not retained definitions
or unchanged sources). Deduplicate and reject accidental reuse. Rebind every new
layout to checked reservations, run existing canonical record/ownership admission,
and publish one rewrite_prepared_storage_payload_records transaction. First-pass
failure leaves bytes unchanged; second-pass failure uses existing recovery. Old
sources remain live through publication even when retired in the new state.
Existing4096 row/graph/preparation and64MiB staging experiment bounds remain; no
aggregate1MiB cap. Retain memory-policy failure evidence; no allocator changes.

## Fixed validation scope

1. All16 modes: independent reopen after normal creation/replacement, repeat,
   secret setting/upgrade, missing capture, label-only change, mixed histories,
   absent-field and unrelated-type records. Match public baseline complete records.
2. Equal-byte normal-to-secret must rewrite context; equal-byte historical-secret
   to latest-normal must likewise rewrite context. Label-only changes preserve
   unchanged value extents. Explicit latest normal with historical secret capture replacement; latest
   Secret normal-input refusal. Captured removed fields remain. Invalid caller,
   missing field/record, one-over1MiB, definition/text revision overflow refuse with
   exact archive bytes unchanged. Exact1MiB and split UTF8 succeed.
3. Layout assertions: stable record/text IDs, checked revisions, unchanged extent
   reuse only when context matches; new definition texts unique; all old definitions
   retained; retired ranges erased. Guard ordinary allocating reads over secure
   spans and underlying Storage clone; permit required guarded authentication.
4. Bounded>8MiB cross-record upgrade using selected stored values, setup caller objects
   dropped before mutation (the actual override remains borrowed through mutation), one scoped independent read at a time. Freeze exact16
   fresh-process functional probes after correctness; endpoint VmLck/RSS/HWM and
   archive growth reported with setup/arena caveats, no timing ratio/peak claim.
5. Representative8 modes include signed plaintext and protected unsigned. Every
   returned-write cut and selected interrupted recovery must yield complete old or
   new definition/ref/capture sets, target override, unchanged neighbors, retired
   erasure and abandoned staging cleanup. Repeated recovery; moved/deleted paths
   and streamed salvage preserve selected membership. No process-death claim.
6. Existing public3 form-history tests, selected-source/segment/prepared controls
   and strictClippy. Tests precede hook; postformat affected checks follow. Retain
   exact failures/source variants. No resource comparisons to earlier frozen paths.
