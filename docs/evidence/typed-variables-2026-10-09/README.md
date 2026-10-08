# Selected typed variables and guarded recovery

Date: 2026-10-09. This is the test-only (`cfg(test)`) shared-tree adapter on the experimental
0.5 development branch, not public API activation or format/release selection.
It follows the separately qualified secure segment component (`73b9378f`).

## Selected metadata and value semantics

The `RV4FS002` experimental record marker admits variable namespace 6. Each
canonical VariableName selects a bounded metadata-only segment layout. Values
remain outside ordinary index Entry buffers. The reader requires the exact
union of file-pack and variable-segment extents to equal the authenticated
selected physical ownership graph: no orphan, duplicate or cross-variable
extent ownership is admitted. The complete typed catalogue validates before
any record/value becomes observable. Existing `RV4FS001` file-only images remain
readable; deleting the last variable returns that metadata marker.

Variable names have their own namespace, independent of file names. Normal
getters refuse secrets; secret getters expose an Arc-backed SecretString through
a callback. Normal values can become secret, while secret-to-normal conversion
requires deletion and recreation. Replacement preserves logical identity and
increments revision; no-change updates preserve all stored bytes, allocation
and generation. Revision overflow refuses replacement but does not prevent
deletion. Empty values, split UTF-8 and the public 1 MiB maximum are preserved.

The content key follows existing archive bootstrap: protected archives derive
the page key from their archive key; plaintext archives use the established
public zero master key. Plaintext remains publicly readable. Signed plaintext
normal-open verification includes all selected variables, preserving the eager
whole-content policy; it does not silently fall back to salvage.

## Transaction, memory and recovery boundary

A SecurePayloadPlan shares the existing authenticated reservation/rebind/COW
transaction while keeping encoded pages in SecureVec. Write callbacks borrow
guarded bytes; read-after-write verification uses read_at_secure. Old layouts
retire their exact selected extents. Free/pending-range zero verification also
uses guarded reads, including failures where erasure was incomplete. Existing
ordinary file staging stays separate. Real FileStore/MemoryStore read_at_into,
the experiment's View and Append wrappers, and ExternalStorage's caller fill
route avoid an additional ordinary payload buffer. External transports and
MemoryStore's intentional stored archive image are outside that scoped claim.

The test CrashStore now copies directly into supplied read_at_into buffers;
its former call through read_at would have hidden an ordinary temporary in the
guarded regression. The guarded fault wrapper registers attempted page spans
before delegation, including partial failing writes. It rejects ordinary reads
over live, retired and incompletely staged segment spans throughout transaction
and recovery. Unrelated metadata/index reads may remain ordinary.

Selected variable salvage is separate from filesystem salvage. It validates
selected metadata/ownership first, then validates every complete value in guarded
memory before a scoped callback. It reports unavailable variable names, preserves
intact neighbors, and never searches old roots for deleted values. Callers must
hold a stable source snapshot and stage callbacks until the overall call succeeds.
Filesystem-only salvage retains its explicitly limited scope.

## Bounds and qualification limits

Values remain at most 1 MiB, in at most 16 segments (one segment for empty values).
Padded segment pages retain the existing power-of-two page sizing; adding context
to a 64 KiB part can produce a 128 KiB stored page. Metadata admission retains the
100,000 variable-count and 32 MiB aggregate path budgets; actual capacity is also
bounded by 4,096 ownership records/page-pair limits and 8,192 graph claims.
Staging retains 320 KiB per payload and 64 MiB aggregate limits. These are
experimental admission bounds, not accepted product capacity reductions.

Return to dense metadata explicitly refuses images with variables before writing;
whole-archive migration/export/compaction with variables is not qualified.
Private file and filesystem-metadata operations preserve variable records and
ownership. Forms/references, variable moves and broader public adapters, public binding/CLI
activation and a complete mixed
workload remain future integration. No CPU, incremental RSS, aging, ZIP parity
or architecture-selection gate follows from these correctness tests.

## Resource failure retained

The first concurrent focused run passed its guarded transaction/recovery test,
lifecycle and max-revision controls, but the maximum-value test failed with
`weakened secure memory allocation is disabled`. The host locked-memory limit
was **8,192 KiB**. The original fixture retained both 1 MiB and 1 MiB + 1 input
handles across its entire 16-mode loop, alongside concurrent tests. SecureVec
capacity growth can reserve 2 MiB for the one-over value. The secure heap retains
arenas after slots are wiped/freed, so dropping handles permits reuse but does
not establish that locked pages return to the OS. Fixture lifetimes were reduced;
a second concurrent run still failed the maximum-value test, while its other
four tests passed. The allocator, security policy and format implementation
were not weakened.
This remains resource evidence, not proof of arbitrary concurrent 1 MiB support.

The initial compile error applied `?` to a metadata Vec return value. A second
fixture error used a noncanonical persisted name without re-sorting raw records,
so the raw writer correctly refused ordering before the typed reader was reached.
Both failures and corrected runs are retained; no failed run is relabeled a pass.

## Verification

The pre-final implementation passed the complete release file-format suite:
**257 passed, 8 ignored**, with `--test-threads=1` in **1,153.30 seconds**.
The maximum/empty test passed alone in a fresh process (**6.26 seconds**) at the
same 8,192 KiB locked-memory limit. The selected ownership/salvage and lifecycle
controls passed independently, and strict archive Clippy passed. Serial functional
success does not erase the two retained concurrent allocation failures.

Five focused tests cover all 16 modes for variable lifecycle, first-write/read,
no-change/downgrade and namespace refusals, coexistence with file and filesystem
metadata changes, empty/exact-maximum/one-over values, selected corruption and
salvage, and a separate authenticated maximum-revision fixture in unsigned plaintext. The representative crash
matrix spans eight modes, including signed plaintext and unsigned protection:
**3,432** transaction cuts and **2,724** interrupted recovery cases passed.
Transaction cuts cover every persistent operation, prefixes 0/97/full and both
failed-sync persistence outcomes. Interrupted recovery exhausts operations and
three prefixes at five deterministic transition cuts, with failed sync persisted;
it is not the complete transition × recovery × sync-outcome cross product.
Guarded reads also verify unchanged neighboring variable and file bytes.

A one-byte segment corruption loses one value while retaining its neighbor.
Independent 64 KiB region loss in the two-512-byte-value fixture loses **both**
variables in all eight unpadded modes; all eight padded modes lose **one** and
retain the neighbor. This is a measured locality trade-off in this fixture,
not general region-loss isolation or whole-format recovery qualification.
Deletion remains absent after either control-bank loss. Losing both selected
private roots fails before any salvage callback; no older membership is used.

After that full-suite run, final reviewed changes made the value-key field
conditional on actual variables (avoiding unused KDF work on file-only opens)
and added explicit retired/abandoned-span erasure checks after interrupted
recovery resumes. Guarded free/pending checks remain unconditional because old
secret history cannot be inferred from an empty current variable set. No unchanged
file-only performance claim follows. Final affected checks and formatting status
are recorded separately; source hashes identify the final pre-format variant.

## Final pre-format checks

The final source passes all **five** variable tests with serialized execution
(**268.96 seconds**), including the additional post-resume erasure assertions:
3,432 transaction faults, 2,724 interrupted recovery cases and all 16 region-loss
outputs. The file-only open/read control passes; the fresh exporter module passes
**7 tests, 1 ignored**, retaining its 93 failure cases. Strict archive Clippy
(library/tests/benches) passes. These are the `final-*` logs, following the
conditional value-key adjustment. Formatting and post-format checks follow as a
separate checkpoint step; the 19-minute full suite was not relabeled as this
final variant or needlessly repeated for the narrowly affected source.

## Post-format checkpoint

Commit `803bc37d` passed the tracked hook (11 Rust files formatted). Post-format
affected checks pass: all **5** variable tests with serialized execution
(**269.54 seconds**), the file-only open/read control, the fresh exporter module
(**7 passed, 1 ignored**, retaining 93 faults), and strict archive Clippy.
The variable matrix again reports **3,432** transaction cuts and **2,724**
interrupted recoveries, with all 16 region-loss outputs. Post-format logs and
source hashes are retained separately. No source edits were made during these
runs, and no public activation or resource gate is inferred from their success.
