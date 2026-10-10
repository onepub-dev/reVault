# Selective authenticated reads

Date: 2026-10-10. Status: product decision accepted for the experimental v4
reader; implementation and measured evidence are tracked in the [v4 plan](archive_v4_plan.md).
This does not activate v4 in the public CLI or bindings, qualify a release, or
change the released format-3 contract.

## Read contract

An ordinary open authenticates the selected publication, bounded control state
and format lookup. It does not certify every byte in the archive. A selected read
authenticates the index pages on its lookup path and the requested payload's
commitment before returning that payload. Parent/child height, count, ordering,
identity, bounds and descriptor checks remain mandatory on the pages consumed.
Signed modes retain owner-authorized membership through the signed root; a
content-key holder's ability to recompute an AEAD tag is not owner authority.
Unsigned plaintext provides accidental-corruption detection, not provenance.

There is no ordinary-open traversal of the complete index, physical ownership
graph, unused pack padding, reclaimed space or all file contents. Signed
plaintext follows the same selective policy. Internal padding covered by an
accessed authentication unit is still authenticated. A compressed fragment must
still be authenticated and decoded as a complete fragment even for a tiny range.
Untouched damaged pages or payloads may be discovered only by a later access or
full audit. A failure on a later streaming chunk does not revoke already returned,
independently authenticated earlier chunks; the failed chunk exposes no bytes.

The storage caller must retain a stable snapshot/read lock for the session.
Publication changes during credential bootstrap are rejected. Every fetched page
and fragment is checked against the selected snapshot; no old publication is
silently selected as a repair for a damaged current root. A valid metadata mirror
may satisfy the same authenticated reference.

## Audit, mutation and disclosure boundaries

Full verification retains complete typed metadata, ownership coverage, padding,
erased-space and payload verification. The explicit audited image remains the
input to update, deletion, compaction, installation and existing credential
preservation paths. Selective read state does not authorize storage reuse or
erasure. Successful deletion and abort still require durable cleanup; recording
a range as free is not a substitute for zeroing it.

Granting access or distributing an entire archive must establish the promised
absence of recoverable deleted or abandoned data, completing cleanup or producing
a verified clean copy as appropriate. This decision does not authorize bypassing
those checks because a selected file read succeeded. New grant/revoke transaction
writers remain separate unfinished work.

Filesystem listing traverses filesystem metadata because it requests that set;
it does not read all file fragments or values. A form-field lookup may load the
selected record and definition's bounded field descriptors because the current
encoding addresses them by ordinal. It reads only the requested field's payload,
not unrelated form payloads. Larger schema/index changes remain separate work.

## Cache and portability

The reader may retain at most 16 authenticated decoded index pages per session.
The cache is bound to archive identity, protection mode, derived index key,
selected root and sealed length; page references include locations, length and
digest. Parent relationships are still checked on a cache hit. Wipeable record
and branch buffers are cleared on eviction/drop. Payload and secret-value bytes
are not added to this cache. This is bounded decoded-page reuse, not a claim that
disk bytes stayed unchanged. Fresh sessions authenticate their fetched pages.

No Vault record, file size or modification timestamp is evidence of integrity.
No persistent integrity cache or new Vault format is introduced. The archive
format is unchanged: this changes verification timing and read architecture.

## Goal alignment and acceptance

This decision serves G3/G4/G7's bounded-memory, selected/range and remote-read
requirements while retaining G5/G6's authority, deletion and transaction
guarantees. It implements G11 by reducing unrelated work, not accepting
unauthenticated returned data. Full-archive certification belongs to an explicit
audit. These detection-timing semantics supersede the earlier requirement to
retain eager ordinary verification during experimentation.

Required evidence includes all protection/codec/padding modes; exact range bytes;
corrupt selected versus unrelated pages and payloads; mirror loss; malformed
parent/fragment bindings; padding/free-space damage accepted by selected reads
but rejected by audits and mutation; credential/publication-switch refusal;
bounded cache isolation; and unchanged transaction checks. First reads and
in-session cache reuse must be reported separately. Full CLI memory, million-entry
scale, ZIP/PGP parity and public activation still need independent qualification.
