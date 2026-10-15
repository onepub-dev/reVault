# Dense shared-control metadata update protocol

Date: 2026-09-27. Based on `2f31bd55`; the containing commit connects persisted
ownership, preparation, publication and retirement for bounded file-metadata edits.
This remains an internal architecture experiment, not a selected release format.

## Persisted lifecycle

A new bounded `RV4DENS2` catalogue body adds canonical free/pending intervals to
the compact file and pack tables. The original fresh `RV4COST1` body remains
readable. Decoding bounds integer widths, counts and total metadata before
allocation; writing reserves one wipeable 64 KiB buffer and refuses overflow.
File identities and complete fragment binding fields are checked before encoding.
The selected publication and the decoded graph together must account for every
byte through the sealed length, including inactive inline control slots.

The test-only edit operation renames a file and/or changes its permission bits.
It first recovers any interrupted operation, validates normal-open checks and all
file bytes, and refuses invalid paths, collisions, unsupported metadata sizes and
missing/wrong signing authority before staging the new update. A no-change repeat
leaves persisted bytes and generation unchanged.

New catalogue copies are staged in previously vacant slots or a prepared append.
The compact preparation record becomes durable in both banks before those writes.
New dependencies are read back, synchronized, then published in both banks with
intervening barriers. Only then does recovery derive retirement from the selected
new catalogue and erase the old metadata. Inline and external placement alternate;
subsequent edits reuse the same external allocation. Payload bytes, identities,
wrapped keys and independent fragment protection are preserved.

On abort, recovery validates reservations against the still-selected old graph,
erases those ranges and the unpublished append tail, synchronizes, then truncates
to that publication's sealed length. On commit, it first restores/synchronizes both
new publications and metadata copies, then erases only the new graph's pending
ranges. Journal cleanup is repeatable after interruption. Recovery never trusts a
caller-supplied retirement list or uses journal checksums as live-data erase
authority. Normal open requires an idle journal bound to the selected commit and
zero free/pending bytes; read-only salvage can recover intact files during cleanup.

## Validation and scope

The focused checks cover all 16 protection/compression/padding combinations,
six alternating edits each, exact no-change repeats, name/collision/permission
refusals and independent reopened file-byte comparisons. Fault tests exercise both
inline-to-external and external-to-inline directions:

- 209 returned storage failures.
- 1,254 volatile-write/power-loss cases, using no/partial/full torn writes and
  failed syncs with and without persistence.
- 942 interruptions during recovery itself, covering both abort and commit cleanup.
- 32 separate complete losses of a control or external metadata failure region.
- Forged journal reservations targeting publication, preparation, live metadata,
  payload or out-of-bounds bytes are refused before any recovery write.

These are internal fixtures because no public CLI writes this profile. They do not
replace process-death, real filesystem ordering or cross-platform qualification.

The first edit adds two 64 KiB external control allocations, plus alignment if the
old file ends unaligned. They remain part of the sealed graph even after metadata
returns inline. Shrinking that tail is not silently authorized by relocation.
A retained-corpus `dense-lifecycle` probe measures this cost with a fresh-handle
source-byte comparison after each edit and exact unchanged-repeat hash checks.
It reports size/correctness only, not CPU, RSS or comparable creation timing.

The component limits remain 1,024 files, 4,096 fragments and 64 KiB decoded metadata
fitting one mirrored private envelope. This executor only needs two inline journal
reservations; it explicitly refuses overflow-backed journal sessions. The separate
codec still retains its 2,048-reservation capacity. Neither limit is an accepted
product capacity reduction. Payload additions/replacements/removals, typed public
records, larger catalogue trees, access changes, journal-overflow rotation,
compaction/installation, process death and the resource matrix remain outstanding.
The original C mixed-file aging failure and A3/A4 failures remain unchanged.


[All 184 format tests pass](format-tests.log), with five explicit probes ignored;
[strict Clippy passes](clippy.log). The [focused log](tests.log) retains the exact
fault/region counts above and the persisted catalogue round-trip checks.


Post-hook [dense tests](postformat-tests.log) and [Clippy](postformat-clippy.log)
also pass. The first real-file lifecycle probe at `3b8fdcad`
[failed before its first edit](read-only-probe-failure.log): its update handle used
the read-only file constructor. The follow-up uses `file_for_write`, with a separate
real-file test closing the writer and reopening a reader after both placement
transitions. This probe failure supplies no size result; reruns use a new directory
and a newly frozen executable, preserving the failed artifact.

The [real-file regression](file-tests.log) and [strict lints](file-clippy.log) pass.


## Measured metadata lifecycle size

Frozen `81024436`, executable SHA-256
`ecb4474a57c5cb192dc49ae8d5b71c53583089a5fd3a0600d46b323412c0384b`, ran 100
permission edits and 100 unchanged repeats for each retained 512 × 4 KiB corpus.
[Inputs](sizes/batch.json), [plaintext result](sizes/small-plain.json) and
[encrypted/signed result](sizes/small-protected.json) are retained. Every edit was
closed, reopened through a separate reader and compared with every source byte;
persisted permission bits and generation were also checked. Each unchanged repeat
preserved the complete archive hash.

Both files start at 327,680 bytes, grow to **458,752 bytes on the first edit**, and
stay at that size through all 100 edits. That is 1.935× the retained 237,078-byte
ZIP, exceeding the proposed 355,617-byte small-corpus bound. This is bounded growth,
not a space leak, but the whole-layout size issue is real. Payload mutation and full
aging remain unqualified. [Post-hook file test](file-postformat-tests.log) and
[Clippy](file-postformat-clippy.log) pass for the frozen source.

The next geometry test will permit two 24 KiB private slots in each existing
control bank, retaining the 48 KiB/external route for larger catalogues. This is a
proposal to avoid external growth for small metadata edits, not permission for
in-place overwrite or a reduction of the decoded-metadata bound. It must use the
same graph, journal, publication and erasure checks and repeat interruption tests.
