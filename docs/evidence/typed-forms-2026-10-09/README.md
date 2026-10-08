# Selected typed form snapshots

This is a **test-only (`cfg(test)`) shared-tree adapter** for importing and reading
definition revisions and historical captured records. It follows the public
capture-history baseline and prepared guarded staging prerequisite. It does not
activate public forms, replace the released migration-import policy, or select a
format. Deletion, field/revision mutation and selected streamed salvage remain
followthrough work; this snapshot tranche is not a complete form lifecycle.

Experimental `RV4FS003` uses namespaces 7–10 for definition headers, ordered
definition fields, record headers and ordered captured fields. `RV4FS001` and
`RV4FS002` retain their previous meanings. Headers contain only identifiers,
references and bounded descriptors. The established secure-page representation remains uncompressed in every mode;
no compression gain is claimed. Every name, description, label and value has
its own segmented payload, preserving the public per-text 1 MiB limit without
squeezing texts into the index's 49,152-byte value capacity or imposing a whole-form
1 MiB limit. A 4,096 aggregate row cap and existing graph/preparation caps remain
experimental capacity restrictions.

FormLeaf pages have a distinct form prefix and complete 32-byte context binding
parent identity, semantic role and field ID/kind, plus layout UUID/revision,
sensitivity, mode, archive, ordinal/count/offset and page identity. Definition
contexts bind immutable type/revision; record contexts bind stable record UUID,
independently of mutable path/current definition reference. Variable wire bytes
are unchanged. Snapshot texts currently begin at their own revision one; an
independent record mutation revision is not implemented or claimed.

Admission consumes every declared contiguous field row exactly once, rejects
duplicates/orphans/dangling references, checks exact referenced definition and
alias, and authenticates captured kind/sensitivity independently of the current
definition's field list. This preserves the public mixed historical captures,
including removed fields, established in the separate baseline. Accepted type IDs
remain normalized 36-character hex/dash strings. Imported revisions may have
holes; this tranche provides exact-revision lookup, not alias resolution or new
revision creation. Cross-family UUID collisions refuse. The selected physical
payload graph equals precisely the union of file packs, variable segments and all
form text extents; orphan/shared extents are not accepted.

Guarded prepared pages stage and read back one segment at a time. Full verification
reads one text/value at a time and retains signed-plaintext eager-open policy.
Normal public metadata/values may be materialized as ordinary strings; secret
values stay in guarded storage and Arc<SecretString>. The scoped field accessor
allows a reader to verify one captured value without assembling all values.
The full-object getter retains its ordinary requested-object semantics and can
require proportionate guarded memory. Plaintext archives remain publicly readable;
guarded processing adds no at-rest confidentiality.

Definition import is unchanged for identical content and refuses a conflicting
exact revision. Captured-record import refuses existing paths. It preserves the
observed independent form/file namespace, including same-path file/form and a form
with a direct file parent; missing ordinary parents are created atomically with
the snapshot. File/variable mutations preserve existing form identities/extents.
Returning to the filesystem-only dense representation refuses forms before writes.

## Verification scope

The initial connected snapshot test passes all 16 modes with metadata larger than
49,152 bytes, split UTF-8, mixed captures, reference/alias/sensitivity refusal,
independent reopen, byte-identical refusals/no-op and file/variable coexistence.
The guarded admission test distinguishes in-memory metadata-parser controls
from its authenticated archive fixture: complete same-size name/description
extent descriptors are swapped while preserving the ownership union. Semantic
context validation rejects the payload on read, and signed plaintext rejects it
on eager open. Count/oversized input refusals leave archive bytes unchanged.
The guard rejects ordinary allocating read_at over live or attempted form pages;
read_at_into provenance also relies on the reviewed guarded implementation.

An initial compile failed only because the fixture's file reader required a
mutable binding; its transcript is retained. A subsequent Clippy type-complexity
finding was resolved by a named callback type. Neither failure changed the wire
or weakened an assertion. The full serialized release format suite passes **267 tests, 11 ignored**, in
1,258.84 seconds on this pre-final source. Its command, source hashes and log are
retained. Final admission changes then cap canonical namespace values before
collection and remove whole-definition validation/no-change copies in favor of
borrowed, sequential text validation/comparison. Mandatory full content
verification remains before unchanged return. Late-field mismatch/corruption
and oversized row controls qualify these changes separately; the long suite is
not relabeled as testing that later variant. Final affected, aggregate and
post-format results retain separate source identities.

## Initial aggregate probe result

Exactly 16 fresh Linux processes cover all modes under the inherited 8,192 KiB
locked-memory limit. Each writes FileStore snapshots with exact 1 MiB definition
name, description, label, record name, captured label and eight secret values.
One caller-owned Arc<SecretString> is reused for the eight distinct logical
fields; normal synthetic metadata is ordinary input. The writer and caller
objects are dropped before independent reopen. Every secret field is read through
the scoped accessor and every large normal text is verified separately. This
proves the declared bounded staging/open path, not simultaneous capacity of eight
caller secrets or the full-object getter.

All **16/16** declared processes succeed without retries. Each verifies
13,631,586 logical bytes; stored payload measures 13,697,298–13,697,520 bytes
in unpadded modes and 29,097,984 bytes padded. Archive size is 14,548,992 bytes
unpadded and 29,818,880 bytes padded. Maximum observed VmLck endpoint is
**4,076 KiB** under the unchanged 8,192 KiB limit. The final connected suite passes
2 tests (1 explicit aggregate probe ignored), the segment suite passes 3 tests,
and strict archive Clippy passes. Frozen source and executable identities are
retained under `probe-frozen/`.

Phase VmLck/VmRSS and whole-process VmHWM are descriptive endpoints, not incremental
peaks or timing comparisons. Retained guarded arenas affect later phases. No
retry or excluded mode may be hidden. Frozen executable/source hashes, transcripts
and archive identities preserve the tested variant. Binaries and synthetic archive
images stay in /tmp; only small evidence artifacts are committed. Earlier resource
ratios apply to their own frozen variable implementation, not this form adapter.

## Final admission variant

After the complete pre-final suite, final affected checks pass: form segments
3/3, connected forms 2/2 (one explicit probe ignored), variable lifecycle 1/1,
ordinary-vector add/update control 1/1 and strict archive Clippy. The row caps
reject oversized namespace values before collection, and import validation and
exact no-change comparison retain only one text at a time. Late-field mismatch
and corruption cannot be mistaken for unchanged content.

A separate fixed **16/16** final functional batch passes with no retries; its
frozen executable is `9b9c4b1d182b5e921c2fddd3e043cce84508f5924262dfc0b05292cf0ad4c745`.
Logical/archive sizes match the initial batch's declared workload, and maximum
observed VmLck endpoint is again 4,076 KiB. Exact source, outcomes and logs are in
`final-variant/`. The initial batch is preserved; no timing ratio or causal RSS
improvement is inferred between them. Post-format checks are recorded separately
rather than relabeling these frozen pre-format artifacts.

Public APIs/migration, full form lifecycle, region-loss co-loss, native selected
recovery, architecture decisions and complete CPU/RSS/aging/compatibility gates
remain open.
