# Mirrored publication records: persistence and fault evidence

This implements the candidate publication mechanism from
[decision 001](../../decisions/001-v4-signing-and-recovery.md), following the
[independent recovery commitment experiment](../recovery-commitments-2026-09-26/README.md).
It is a storage layer exercised through real file I/O and process-death tests.
The normal archive writer/reader does not activate this candidate encoding yet.

## Implemented contract

Two fixed 8 KiB records occupy a 16 KiB publication region. Each commits to archive
identity, generation, protection mode, sealed length, logical object-root digest,
previous-publication commitment, and mirrored index/allocation/key-directory root
references. Root references bind both locations, bounded length and stored-byte
digest. Optional absent roots use an all-zero representation. Root allocations
must lie after the publication region, fit inside sealed bounds and not overlap.

Object counts stay in private index metadata. The public record does not add an
explicit object count. Private object identities and descriptors are not carried
in this record. Publication fields still expose generation, length, locations and
digests; the final format's public-field inventory remains a release requirement.

The reader supplies the established archive, mode and authority. Signed modes
require the pinned owner's existing Ed25519 and ML-DSA-65 keys. Encrypted unsigned
mode uses a domain-separated HMAC-SHA-256 verified through the MAC library's
constant-time verification API. Plaintext unsigned mode provides only structural
checks and accidental-corruption detection. It does not establish provenance.

Publication signatures have their own domain. Valid owner signatures on preparation
commitments cannot be substituted. The encoder is private to this protocol, and a
writer must not persist/export publication-domain signatures during preparation.
Any previously exposed valid publication record can be replayed by a storage
adversary; fixed placement is not external freshness state.

The publisher validates the new root copies and the expected prior publication,
synchronizes prepared dependencies, writes/synchronizes one publication slot, then
writes/synchronizes its mirror. It returns a `Published` token only after the final
sync. Reading two matching copies returns a `Selection`, not a durability token.
Mirror repair synchronizes even if both copies are already readable: a previous
final sync may have failed. The future allocator must require successful publication
or repair before erasing allocations needed by the prior generation.

Selection reads only the two fixed slots. It ignores records in prepared tail data,
rejects conflicting equal generations and unlinked adjacent generations, and treats
a slot read error as an error rather than evidence that the other slot is current.
It authenticates the selected generation separately from payload availability.
Truncation below its sealed length does not authorize rollback to older membership;
recovery can use a surviving verified root copy. Mutation/mirror completion requires
intact direct dependencies and refuses truncated/damaged required roots.

## Candidate encoding

This is an experimental codec description, not the selected archive specification.
Integers are little-endian. The magic `RV4PUB01` and codec version 1 distinguish it
from the existing `LBX4HDR` region. It is not written into format-3 archives.

| Offset | Bytes | Field |
| --- | --- | --- |
| 0 | 8 | Candidate magic |
| 8 | 2 | Codec version 1 |
| 10 | 2 | Existing validated protection/compression/padding mode |
| 12 | 4 | Slot length 8192 |
| 16 | 8 | Generation, starting at 1 |
| 24 | 16 | Archive identity |
| 40 | 8 | Sealed length |
| 48 | 8 | Reserved zero; no public object count |
| 56 | 32 | Object-root commitment |
| 88 | 32 | Previous publication commitment; zero only for generation 1 |
| 120 | 56 | Index root: primary/mirror/length (8 bytes each), digest (32) |
| 176 | 56 | Allocation root, same representation |
| 232 | 56 | Key-directory root, same representation |
| 288 | 4 | Authentication payload length |
| 292 | 28 | Reserved zero |
| 320 | Bounded variable | Two ordered hybrid signatures and public keys, a 32-byte MAC, or empty |
| After authentication | To offset 8160 | Zero padding |
| 8160 | 32 | Domain-separated public checksum of the preceding slot bytes |

The publication message is its distinct domain plus bytes 0–287. The MAC adds its
own domain. The chain commitment hashes the publication message, so randomized
signature bytes do not change the logical publication identity. Signature count,
algorithm order, key/signature lengths, reserved bytes and padding are bounded and
validated. Direct root reads are capped at 64 KiB; slot parsing is bounded by 8 KiB.

Independent Python hashlib/hmac/struct vectors reproduce checksum-only and symmetric
records byte-for-byte: [vectors](../../../rust/revault_lockbox_api/tests/fixtures/publication_anchor_v1.json).
The public checksum helper has its own existing domain and length prefix; it is
not interchangeable with bare SHA-256 of a record.

## Executed validation

[Raw successful test/check output](validation.txt) is retained. Tests ran on Linux
with the repository's pinned Rust toolchain. This is not cross-platform qualification.

* Thirteen publication tests pass (including the child-process entry point).
* All four protection modes round-trip across two generations and independent
  reopening; valid publication-shaped tail records do not change slot selection.
* Owner, archive, mode and signed fields are pinned. Wrong keys, malformed lengths,
  short MACs, reserved-byte changes, overlapping/out-of-bounds roots, missing slots,
  equal-generation forks and unlinked generations are rejected.
* Every mutating storage failure in the normal publication path leaves a selected
  old or new generation with intact fixture roots. Slot read failures are tested
  separately because the existing memory-store injector covers mutations only.
* A volatile/durable storage model covers all five write/sync boundaries, failed
  syncs that persist either none or all pending bytes, and eight torn-write prefix
  lengths. The final-sync failure case proves that readable mirrors cannot skip
  the required synchronization during repair.
* Every byte-prefix cut (0–8192 inclusive) is tested for both first-slot and mirror
  writes in checksum mode: 16,386 selections retain a complete old/new fixture root.
  Signed-mode torn writes are covered by the separate boundary fault matrix.
* Actual child processes are killed at five publication boundaries and three
  mirror-repair boundaries. Reopening, root-byte comparison, resumed synchronization
  and another independent reopening all pass. Process death is distinct from the
  simulated power-loss durability model.
* The real-archive proof test now publishes and independently reopens its selected
  commitment through this layer. It passes in default and native layouts, for
  signed plaintext/encrypted data and raw/Zstd. Its synthetic publication store is
  separate from the current archive, which does not implement this encoding.
* Targeted library/test Clippy passes with warnings denied.

```sh
cargo test -p revault_lockbox_api --lib publication_anchor
cargo test -p revault_lockbox_api --lib independent_owner_proof
cargo test -p revault_lockbox_api --features native-block-layout --lib independent_owner_proof
cargo clippy -p revault_lockbox_api --lib --tests -- -D warnings
```

The direct storage/process hooks are unit-test exceptions, not CLI E2E coverage:
no public CLI creates this unactivated format or pauses at its sync boundaries.
Only generated public verification keys cross the test process boundary; private
signing keys remain inside the child. No production credentials are used.

## Remaining integration and limits

The module is currently included under test configuration. The fixed region avoids
publication-record growth, but this is not evidence of bounded whole-archive aging:
index nodes, allocation history and payload retirement still need their protocols.
No new end-to-end speed result is claimed for this layer.

Publication validates direct root bytes, not a complete index traversal or all
payloads. The authenticated keyed index must bind child links, canonical record
semantics and stored extents, implement confidentiality, and enforce its own parser
limits. Its mirrors need a defined physical damage model and reclamation policy.
The allocation layer must durably track preparation, gate erasure on publication,
resume cleanup and account for every mirror. Owner identity must come from a trusted
binding, not from accepting the same untrusted record's self-declared public key.

Normal signed-open verification remains eager. The production native recovery path
still has its two known failures. Full file/mirror/Vault lifecycles, adversarial
content-key-holder tests, migration, compatibility, CPU/memory comparisons and
platform qualification remain required before enabling the complete design.
