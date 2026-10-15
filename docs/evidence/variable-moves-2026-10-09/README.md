# Atomic typed variable moves

This is a **test-only (`cfg(test)`) shared-tree adapter**, not public activation
or a selected format. It follows the [guarded source resource checkpoint](../variable-resource-2026-10-09/README.md).
The persisted change is metadata-only: complete guarded content verification
still runs, including signed-plaintext eager-open policy and no-op requests.
No payload authentication is bypassed to obtain a no-read claim.

The adapter validates duplicate and missing sources, unique destinations,
occupied destinations only when their variables also move, and the complete
final parent/child namespace before persistent writes. Cycles and chains rename
rows together. Value UUID, sensitivity, revision (including `u64::MAX`), length,
segment descriptors/digests and payload ownership remain unchanged. A missing
source is refused even when its destination is itself. Empty and identity-only
requests return unchanged without allocation/publication changes. Files retain
their independent namespace.

All **16 modes** pass lifecycle/no-op/refusal fixtures: swaps, normal and secret
renames, rename to a same-named file, child-to-parent rename when the old name
vacates, occupied/duplicate/missing/conflicting destinations, signer refusal,
independent reopen/read and exact unchanged payload bytes. Secret normal-getter
refusal persists after rename. Either control-bank loss salvages only current
names. Corrupted value content refuses even an empty move without modifying the
image. Maximum-revision values move forward/back without incrementing revision.

Eight representative modes (`0,1,2,3,12,13,14,15`: encryption/signing/compression/
padding bits) pass **1,512 transaction cut cases** and **504 interrupted recovery
cases**. Each operation is cut with zero/97/full-byte persistence and both sync
persistence outcomes; three deterministic transaction positions enumerate every
recovery operation with zero/97/full-byte persistence. Recovery yields the
complete old or complete new name-to-layout mapping, preserves the neighbor
variable/file and all payload bytes, is repeatable, and leaves no nonzero
abandoned tail when the old state wins.

The storage guard rejects ordinary `read_at` allocations over variable spans,
writes into any live payload span, new variable-page staging and truncation
through live payloads. It permits `read_at_into` for required authentication;
the wrapper alone does not establish destination memory provenance. That scoped
guarantee also relies on the separately reviewed guarded segment/readback paths.
No full plaintext value is newly staged merely to rename metadata.

An initial lifecycle fixture used relative grouped names, which `VariableName`
correctly rejects. Its failed transcript is retained; correcting the fixtures
to absolute grouped paths passes the lifecycle test (6.39 s). The fault test
already passed in that first run. Maximum-revision coverage (0.22 s) and strict
archive Clippy pass after the correction. Post-format verification at `ab0fc021`
passes both move tests (the same 1,512 transaction and 504 interrupted-recovery
cases), the maximum-revision test and strict Clippy. Full logs and before/after
source hashes are retained. No source-name validation bug is
claimed: the public name grammar excludes the punctuation counterexample
considered during review.

Forms/references, broader public adapters, whole-archive variable migration/
compaction, native selected recovery and complete CPU/RSS/aging/compatibility
qualification remain open. This tranche adds no performance claim.
