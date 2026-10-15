# Independent fragments in shared physical packs

This correctness checkpoint adds shared physical packing and file deletion to
candidate C. It remains test-only and file-only; no public writer emits it and
no architecture is selected. The existing [normalized comparison](../candidate-buffer-normalization-2026-09-27/README.md)
remains the latest timing evidence, for the previous unpacked adapter. There are
no new packing performance claims in this checkpoint.

## Recovery determines the packing boundary

A provisional design compressed/encrypted several files as one frame. A new
regression proved that damaging its first member made an intact neighbour
unreadable. That design was rejected before commit. The failure log and diagnostic
source snapshot are retained; the snapshot is provenance, not a standalone build
or supported format. The final regression succeeds in plaintext/encrypted,
unsigned/signed and raw/compressed modes.

The selected experiment shares allocation and padding only. Every file fragment
retains its own codec frame, nonce/tag when encrypted, logical identity/ordinal/
offset, and stored-byte digest authenticated by the selected index. A corrupt
fragment does not invalidate another fragment's owner-authenticated commitment.
Fragments never use their own untrusted checksum as membership authority.

The test reads through previously selected authenticated membership after payload
damage. It demonstrates independent-fragment verification, not a completed fresh
salvage API. Explicit read-only recovery, missing metadata/proof reporting and
truncated-tail coverage still need integration. The two production-native recovery
failures remain separate open blockers.

## Experimental wire and privacy rules

* File metadata is 72 bytes with `RV4FIL02`; older test-only `RV4FIL01` records
  are rejected rather than reinterpreted.
* A 144-byte `RV4SLC02` record contains the 64-byte data descriptor, a u32 stored
  offset, four zero reserved bytes, a fragment stored-byte digest and a shared
  padding digest. The ownership envelope names the entire physical pack, including
  its full stored-byte commitment. Shared references must agree on that extent.
* The data descriptor retains file identity, ordinal, logical offset/length,
  codec and exact encoded/stored lengths. Packed data uses separate key/AAD
  domains from the previous standalone codec. File ciphertext is not accepted
  as padding, or vice versa. Existing cryptographic primitives are unchanged.
* Physical packing applies default padding in 64 KiB units. Plaintext gaps are
  zero; encrypted gaps are independently encrypted/authenticated zero plaintext,
  including nonce and tag. Their digest is bound by each member's authenticated
  slice record. Exposed zero gaps must not reveal encrypted encoded lengths.
  If the encrypted gap would be 1–27 bytes, reserve another padding unit so a
  complete 28-byte-overhead padding record fits. No-padding remains explicit.
* Coverage checks reject missing slices, overlap, conflicting pack/padding
  commitments, oversized groups, noncanonical stored lengths and nonempty padding.
  Even removing the final member's metadata while retaining its physical bytes
  fails this check. Raw fragment reads stay within 64 KiB including encryption;
  decoded fragments and group logical contents are bounded by 256 KiB. The largest
  padded compressed allocation is 320 KiB. Limits are checked before payload I/O.
* The fixed [wire vector](../../../rust/revault_lockbox_api/tests/fixtures/packed_file_fragments_v2.json)
  was generated independently with Python's SHA-256 and explicit little-endian
  fields, and is checked against the Rust encoder/decoder. This is an experimental
  vector, not a released compatibility fixture.

All current members use one archive-private access domain. Public/private or
other differently authorized records must not share a pack; those semantics are
not yet integrated. No change is made to released format-3 compatibility or to
production format-4 output.

## Deletion and physical ownership

Deletion identifies every affected pack, removes requested memberships and
rewrites each surviving encoded fragment into a replacement pack. Logical AAD
is independent of physical position, so relocation preserves encryption without
recompression or decrypt/re-encrypt. Each copied fragment is verified first. New
padding is generated, references publish atomically, and complete old allocations
are reclaimed through the existing journal/allocator protocol. Deleted bytes
cannot survive inside a retained neighbour's allocation.

A missing-path repeat returns unchanged bytes and generation. Deleting the last
member retires the entire pack. Source failure and failed preparation retain the
existing explicit abort/cleanup behavior. Bulk buffers reserve capacity before
private bytes are appended; growing a standalone padding buffer transfers through
a wiping guard so its old allocation is cleared before release.

## Validation and remaining work

The complete default suite passes 439 unit tests, all executed integration tests
and ten doctests; nine unit tests and one integration probe are ignored. The format suite passes
122 tests with four manual resource probes ignored. Strict all-targets
native/external-source Clippy passes. Retained post-hook logs confirm the same 122 format tests and strict Clippy pass.
The packed-file deletion fault sweep covers 174 storage-mutation failures.

Tests exercise deletion and no-change repeats across all protection/codec/padding
combinations; every storage-mutation failure in four representative modes recovers
a complete old/new file set, preserves neighbour bytes and verifies full physical
accounting and zeroed retired allocations. These are internal protocol tests:
the public CLI cannot create or corrupt this test-only format. Lower allocator
power-loss tests remain in force; packed-file process-death/public CLI qualification
is not claimed. The damage regression covers an intact neighbour inside the same
physical pack. Padding/context/bounds tests, >512-extent streaming, independent
reopen, ranges and fixed vectors also pass.

Next implement additions/replacements and explicit file recovery, then run the
same-corpus packed/unpacked CPU/RSS/space and lifecycle comparison, including
100k-file resource limits. Metadata lookup and fixed control overhead remain
separate measured design questions. Full public records/access/permissions,
compaction, migration/bindings/platform qualification and reviewed selection gates
are still required. Small-file packing cannot silently relax the ZIP space target.
