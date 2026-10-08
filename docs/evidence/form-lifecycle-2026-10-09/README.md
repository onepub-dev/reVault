# Selected form deletion and streamed salvage

This is a test-only (`cfg(test)`) extension of the selected shared-tree form
snapshot adapter. It adds record deletion and bounded streamed salvage; public
activation, field edits, definition revision creation and migration qualification
remain unfinished. No format or release has been selected.

Deletion verifies the selected graph, owner authorization and all selected
contents before removing exactly one record and its captured rows. Its name,
labels and values retire through the existing guarded prepared transaction with
zero new payload pages. Definitions remain, even when their last record is
removed. Missing records return NotFound; refusal leaves the image unchanged.
Recovery selects complete old or new membership and erases retired payloads or
abandoned metadata tails according to that selection.

Salvage authenticates complete selected membership, exact definition references
and physical ownership before emitting any event. It never falls back to an older
definition or root. Each whole definition/record is first validated one text or
value at a time. Damaged stored content suppresses that object's events and reports
its exact identity; unavailable definitions also suppress all records requiring
that exact revision. Independent objects remain eligible, including in signed
plaintext mode where ordinary eager open correctly refuses damaged content.

Delivery re-reads authenticated text into DefinitionStart/DefinitionField/End and
RecordStart/CapturedField/End events. A start contains at most two normal strings,
and a captured event contains one normal value or at most one Arc<SecretString>
in guarded storage. Caller
retention can consume proportionate memory and is outside the bounded-processing
claim. Normal metadata is ordinary text; plaintext archives remain publicly
readable and gain no at-rest confidentiality from guarded processing.

The caller must keep storage stable and stage all events until both the object's
End and successful completion of the whole call. Sink, I/O and secure-memory
failures remain fatal, including after partial delivery; they are not reported as
unavailable archive content. Semantic InvalidInput/InvalidOperation errors from
stored value validation become CorruptRecord, while operational errors propagate.
The selected commitment is checked before starts/ends and before success, and each
second-pass payload is checked against its selected digest/context. These checks
do not detect arbitrary in-place mutation after a page's last read or provide a
lock for an arbitrary Storage implementation.

## Verification

The strengthened forms module passes **5 tests, 1 ignored**, including **1,704**
transaction cases and **564** interrupted recoveries. Both classification controls
and all 3 form-segment tests pass. Two constructor borrow-only Clippy fixes then
pass the normal/delivery checks and strict Clippy; the longer matrix remains
explicitly tied to its preceding source manifest. Initial compile failures
(two fixture/edit typos, then missing test-wrapper trait derives) and the lint
failure are retained alongside successful results. Internal fixtures are
necessary because the public CLI
cannot construct this experimental representation. All-mode normal fixtures
reconstruct and compare every definition/captured field, verify missing-record
refusal byte identity, deletion erasure, neighbor retention and nonresurrection
after either control bank loss. One-byte record damage preserves its sibling;
exact-definition damage suppresses only its dependents. Both private roots lost
produce zero callbacks. These controls do not establish independent 64 KiB-region
isolation; the previously documented co-loss trade-off remains.

The fault matrix includes signed plaintext and protected unsigned modes. Every
transaction operation is cut at zero, 97 and full bytes with both modeled sync
persistence outcomes. Representative interrupted recoveries are cut and resumed;
all final states verify complete membership, contents, erasure and tail cleanup.
This is modeled returned-failure/durable-storage evidence, not OS process-death
qualification.

Delivery faults demonstrate an I/O error after RecordStart and selected control
loss after a captured field, neither producing RecordEnd or whole-call success.
Sink errors likewise propagate after partial staging. Exact classification controls
retain SecurityLimitExceeded and Io errors rather than silently losing objects.

A separate fixed **16-process** aggregate run passes every mode without retries.
Frozen binary SHA-256 is
`7586ee55e600f375280df11dd8ba43e7ce5e643c52a6a096c29adef20dda6ef5`
and remained stable. It extends the existing FileStore fixture with eight 1 MiB secret
fields to validate streaming delivery one field at a time under the inherited
8 MiB locked-memory limit. One caller secret is reused as synthetic input; normal
metadata remains ordinary, and full-object getter/concurrency capacity is not
claimed. VmLck/VmRSS phase endpoints and whole-process VmHWM are not incremental
peak measurements or performance acceptance. Earlier snapshot/resource results
remain tied to their original frozen source variants.

The fixture verifies 13,631,586 logical bytes per mode. Archive sizes are
14,548,992 bytes (modes 0–7) and 29,818,880 bytes (8–15). Maximum recorded VmLck
endpoint is 4,076 KiB under 8,192 KiB; after-salvage endpoints range from 3,680
to 4,076 KiB. Mode bits are protected=1, signed=2, compression=4 and padding=8.
The secure form-page representation remains uncompressed even when the archive
compression mode bit is set. Detailed endpoints and all archive hashes are in
[qualified outcomes](qualified/outcomes.json). No timing ratios follow.

Commands, compiler/package identities, source hashes, frozen tracked patch and
exact untracked `.rs.txt` snapshots are retained here. Frozen binaries and
synthetic archives remain under `/tmp/revault-form-lifecycle-2026-10-09-final-qualified/`;
only bounded text artifacts are committed. The `.rs.txt` suffix protects exact
source evidence from the Rust formatting hook; provenance source paths are unchanged.
Post-format qualification is recorded separately after the implementation checkpoint.
