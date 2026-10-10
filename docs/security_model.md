# Archive security and recovery contract

Date: 2026-09-26. Scope: requirements for the v4 design evaluation, with observed
implementation limits explicitly identified. This is not a security certification
or a claim that experimental native blocks meet the contract. Product authority
is [Project goals](../manual/project-goals.md); delivery status is in the
[v4 plan](archive_v4_plan.md). Protocol changes require the decision process below.

## Assets, actors and boundaries

Protect file bytes and names, directory/symlink metadata, variables, form schema
and values, access records, owner signing keys and Vault credentials. An archive
recipient holding its content key is not automatically an authorized writer.
Owner authorization must therefore remain meaningful even against a recipient
who can recompute symmetric authentication tags. Password guessing resistance,
key wrapping and hybrid signatures retain their existing cryptographic policy;
layout experiments may not replace them to improve a benchmark.

Assume an attacker can read/copy archives, truncate or replace storage, edit
arbitrary bytes, provide hostile lengths/offsets/compressed streams and replay
an older authentic copy. A storage provider or network peer is untrusted. An
ordinary storage failure can stop any write, sync, truncate or rename. Injected
I/O errors and actual process death are separate test conditions.

The local OS, cryptographic implementation and authorized process are trusted
while handling plaintext. The archive cannot protect plaintext already exported
by an authorized recipient or a compromised process. An attacker who possesses
the owner's signing key can authorize new content. No standalone archive can
prove it is the latest copy without external freshness state.

## Protection modes

| Mode | Confidentiality | Integrity / authorization required |
| --- | --- | --- |
| Encrypted, signed | Content and private metadata protected under the archive keys | Symmetric authentication plus owner-authorized committed membership; a read-only key holder must not forge owner-authorized data |
| Encrypted, unsigned | Content and private metadata protected | Symmetric authenticity against parties without the key; no distinct owner-write guarantee against key holders |
| Plaintext, signed | None for public payload | Owner-authorized content and metadata, independent of who can rewrite storage |
| Plaintext, unsigned | None | Structural checks and accidental-corruption detection only; checksums do not establish provenance |

Signing and encryption are independent dimensions. Tests must cover all four,
with both codecs and padding policies. The table is the required interpretation,
not evidence that each experimental path implements it. Public format selectors,
archive identity, public access information, total size and access patterns may
leak information; padding reduces selected size leakage, not all traffic analysis.
Inventory exact public fields in the selected wire specification before release.

## Committed membership and ordinary reads

A reader must bind an object to the authenticated archive identity, selected
commit generation, logical identity/path, kind, permissions, length, ordering,
codec and physical descriptor. It must reject cross-archive substitution,
reordered/duplicated extents, stale replacements, deleted entries and unpublished
preparation. A self-consistent frame checksum is not a membership proof.

The [accepted selective-read decision](selective_authenticated_reads.md) changes
experimental v4 detection timing: ordinary reads authenticate the selected root,
required index pages and requested chunks, including signed plaintext. Unrelated
damage is detected on access or full audit. Owner-authorized membership remains
mandatory; full audits and destructive operations retain their wider checks.
Released format-3 behavior is unchanged. Recovery must preserve authorization.

The previously evaluated native `CommitAuth` path signs a commit-root digest;
the commit root includes metadata offsets. That eager path additionally checks
a whole-content digest during signed-plaintext open. The selective shared-tree
reader instead authenticates requested fragment commitments through its selected
root, as specified in the decision above. For encrypted signed data, review how key-holder edits to
metadata and payload are bound to owner authority: an AEAD tag alone cannot prove
that authority. This is an explicit audit item, not an asserted exploit.

## Transaction recovery and physical reclamation

Transaction recovery selects a complete old or new committed generation after
interruption. It is distinct from salvage of damaged data. The maintained
[transaction protocol](../manual/develop-with-revault/transactions.md) controls
publication, preparation reservations, durable cleanup and tail trimming.

Every physical range must be live, retained control/history, zero and reusable,
or durably tracked work. Track the complete padded allocation, not just logical
payload length. Shared allocations require preserving/re-encoding neighbours
before retirement. No-change updates must not repeatedly allocate preparation
or history. Recovery itself must be interruptible and resumable.

Completed deletion/abort must remove retired or abandoned recoverable payload
from the current archive before that space is declared reusable. It does not
promise erasure of copies, backups, snapshots or device-level remnants. Sharing
must not expose prior abandoned/deleted bytes through the current archive.

## Damage recovery

Recovery reports distinguish an intact authorized object, a partially recovered
object with authenticated ranges, and an object whose authority cannot be proved.
Never promote the latter to a successful authorized recovery. Do not infer a
commit from the largest sequence in scanned, unauthenticated metadata.

An intact object should survive unrelated payload damage when its publication
and membership proofs survive. If the selected proof of commitment is destroyed,
report that limit. Falling back to a previous valid generation must be explicit:
it can resurrect a subsequently deleted record, so cannot silently represent the
current committed state. Copying the entire file from an old signed snapshot has
the same rollback limitation even if every signature verifies.

Current native recovery opens a reconstructed archive snapshot before authorizing
an entry. Whole-content verification or missing tail control records can therefore
prevent recovery of intact neighbours. The two reproduced native failures remain
blockers. [Decision 001](decisions/001-v4-signing-and-recovery.md) evaluates how to
separate membership proof from verification of unrelated contents.

## Resource and secret-handling boundaries

Check overflow, bounds, overlap, identity, count and decoded-size limits before
allocation. Cap parser work, decompression, queues, cached plaintext and parallel
workers; run hostile-input cases on native and WASM. A claimed file size is not
permission to allocate that many bytes. Verify authentication before releasing
bytes to callers, including partial/range APIs.

Do not put production credentials in benchmarks, fixture logs, command arguments
or unencrypted migration intermediates. Minimize secret lifetime and preserve
existing secure-memory policies. Desktop credential services are optional;
explicit-credential workflows must not require a desktop session. Suspend cache
clearing and unattended credential behavior are platform capabilities requiring
separate tests, not archive-format properties.

## Decisions and evidence required before release

1. Review owner authorization in all modes, especially malicious content-key holders.
2. Select an independent recovery proof with explicit proof-loss and freshness limits.
3. Validate the accepted selective-read contract, including unchanged mutation, erasure and sharing boundaries.
4. Review encodings, domain separation, nonces, proof substitution and resource bounds.
5. Execute corruption, malicious mutation, rollback, crash and cross-version vectors.

Implementation owner for this draft: Codex. Product/security reviewer: unassigned.
No cryptographic protocol change is activated by this document. The related
[evaluation contract](archive_v4_evaluation.md) records the evidence still needed.
