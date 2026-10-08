# Secure segmented-value component

Date: 2026-10-09. This checkpoint is a **component**, not connected public
variable support. Selected metadata/physical ownership, transaction/recovery
and typed read integration are the required next step on the same branch.
No format has been selected or released.

## Existing public baseline and prerequisite

The public variable/form pipeline already round-trips all 16 encryption,
signing, compression and padding modes. The new baseline uses public APIs to
create secret values, commit, reopen, read through scoped SecretString access,
reject normal getters/downgrades, repeat unchanged commits, replace/reopen, then
delete and explicitly recreate a normal variable. The existing all-mode normal
variable/form/file padding lifecycle control also passes.

The public PageCache deliberately routes plaintext secure objects through the
generic cleartext encoder, while protected objects use the secure encrypted
encoder. Calling the latter directly with plaintext mode is not that public
route; no speculative public reader/sizing repair is included. Plaintext mode
uses the established nonsecret zero master key for internal key derivation;
guarded memory never adds at-rest confidentiality to a plaintext archive.

An empty-value test exposed SecureVec's requirement for an allocation even when
copying a valid zero-length range from a fresh empty vector. The compatible
page-api fix validates checked range bounds, returns success for a valid empty
range, then requires source allocation only for actual copying. Its tests cover
empty clone/append, empty range at a nonempty source's end, unchanged destination
capacity/content, and invalid offsets/overflow. This changes no persisted format.

## Guarded staging and segmentation

A test-only page staging helper constructs plaintext stored pages inside
SecureVec, preserving the existing CLEAR_TEXT flag, raw 16-byte body header,
single-object framing, public checksum and exact unpadded extent. Protected
pages reuse the existing secure encoder; its ordinary Vec holds ciphertext
only before transfer to guarded staging. Public PageCache routing is unchanged.

Segments hold at most 64 KiB of value bytes; values retain the public 1 MiB
maximum, with at most 16 segments. An empty value has one header-only segment.
The 96-byte segment context binds archive UUID, value UUID, revision,
sensitivity, mode, total length, segment count/ordinal/canonical offset and
page identity. The secure page also authenticates page identity/sequence.
Descriptors contain only metadata, including stored extent lengths and digests;
the complete 1 MiB descriptor set is at most 824 bytes, below the reused index's
49,152-byte value limit. No decrypted value is placed in an index Entry.

Readers check canonical count/length/placement and nonoverlap before reading,
use Storage::read_at_secure, verify stored digests, decode into guarded memory,
and check the full expected page/context before secure-range assembly. UTF-8
and public value validation happen on the complete value, allowing a code point
to cross a segment boundary. Returned errors do not expose partial values.

Padded pages retain the existing 128 KiB secure-page allocation; a full 1 MiB
value therefore stages 2 MiB of page bytes. This is a bounded experiment, not a
space/CPU/incremental-memory acceptance result. Descriptor possession alone
does not establish current membership or authorize physical reads; the next
typed-tree integration must prove those independently.

## Component checks before formatting

- Public secret-variable/form 16-mode lifecycle baseline: **passed**; existing
  16-mode padding lifecycle control: **passed**.
- Secure page staging: all 16 modes, 0/1/64 KiB payloads; plaintext output is
  byte-for-byte equal to the established writer, including unpadded size and
  checksums; all stored pages reopen with the established secure decoder.
- Three segmentation tests pass across all 16 modes: empty/one-byte values,
  a UTF-8 code point crossing 64 KiB, exact 1 MiB, one-over rejection, malformed
  modes/metadata, UUID/revision/sensitivity/mode/length changes, duplicate and
  overflowing extents, wrong archive/key and corruption.
- Equal-sized full segments with different contents are permuted after layout
  validation succeeds. A different variable's same-sized complete descriptor
  has a verified stored digest but is rejected by page/context identity.
- A guarded Storage fixture refuses ordinary read_at and confirms that payload
  reads use read_at_into through guarded allocation. MemoryStore intentionally
  retains its existing archive image, which is plaintext in plaintext mode;
  this is not a claim that the whole process contains no ordinary archive bytes.
- Full release format suite: **252 passed, 8 ignored**. Page-api release suite:
  **19 passed**. Strict archive and page-api Clippy: **passed**.

Known synthetic ordinary Vec values occur only in test/reference fixture
construction. Source hashes and logs are retained. The initial invalid relative
variable-name fixture and the reproduced empty-range failure are retained too;
neither is hidden as a successful run. Formatting/post-format checks remain
separate checkpoint steps.

## Required integration boundary

The selected typed reader must prove an exact union of file packs and unique
variable segment extents against the authenticated ownership graph. It must
preserve eager signed-plaintext verification, exact no-change behavior, secret
getter/downgrade rules, namespace checks, retirement erasure and interrupted
recovery. Staging, read-after-write and pending/free zero checks must avoid
ordinary plaintext scratch for secret segments. Return-to-dense must refuse
unsupported variable records rather than discard them. Public activation,
forms/revisions, migration and full-format qualification remain separate work.

## Component checkpoint after formatting

Commit `73b9378f` passed the tracked formatting hook. Post-format release format
checks again passed **252 tests, 8 ignored**; the public 16-mode secret
variable/form lifecycle and all **19** page-api tests passed. Strict archive
Clippy (library/tests/benches) and page-api Clippy (library/tests) passed. The
`postformat-*` logs record those runs. Connected typed-variable integration is
a subsequent tranche and is not attributed to these component results.
