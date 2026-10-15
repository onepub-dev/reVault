# Candidate file extents and an encoder correctness fix

This checkpoint connects candidate C's authenticated index and allocator to actual
file bytes. It is test-only; public readers/writers do not emit this format. No
architecture selection or performance claim is made by this correctness checkpoint.

## Integrated behavior

Data extent descriptors bind an object identity, ordinal, logical offset/length,
codec and exact stored allocation. The selected index authenticates each descriptor
and its stored-byte digest. Encryption uses an archive-specific derived key and
binds the descriptor, mode and archive identity as associated data. Authentication
precedes decompression and delivery to the caller. Optional size padding is checked
and included in the stored commitment.

The file adapter bulk-builds path records and separate per-extent records. A large
file therefore does not require an ownership envelope exceeding its 512-extent
limit. Open audits missing/orphaned/duplicated extent membership, exact positions,
counts and bounds. Signed plaintext still eagerly decodes and hashes every file
at ordinary open; other modes verify touched data before yielding it. Streaming
and range APIs use bounded callbacks rather than file-sized result buffers. Failed
source reads explicitly abort and reclaim preparations before returning the error.

Raw extents use at most 64 KiB of stored bytes, including nonce/tag. Their logical
boundaries remain 4 KiB-aligned: 64 KiB plaintext or 60 KiB encrypted, with default
padding bringing the latter to 64 KiB. This avoids fetching two allocations for
an aligned 4 KiB request solely because of encryption overhead. Compressed profiles
compare 64/256 KiB logical units and may retain raw data when compression does not
help. A fixed Zstd workspace caps the window at 256 KiB and the output at the
authenticated logical length; there is no allocating fallback decoder. Default
payload padding currently rounds to 64 KiB; its compression and small-file space
cost is an explicit upcoming measurement, not an accepted final policy.

Tests cover all protection/padding/codec combinations and both access units,
independent reopen, empty and short files, whole/range reads, callback/source
errors, a streamed 33 MiB file with more than 512 extents, missing/orphaned chunk
membership, eager signed-open detection, surviving neighbours and authenticated
context substitution. Malformed lengths and excessive Zstd windows are rejected.
The source-error test reopens a real file after releasing the writer lock and
checks both absence of the aborted file and complete zeroed physical accounting.

This is a file-only comparison adapter. Small-file packing, updates/removals with
shared-pack rewriting, all public record types/permissions/access semantics,
independent salvage, compaction and a full A/B/C comparison remain required.

## Encoder defect found during integration

The released `zstd-complete` 0.2.0 encoder reverses the two raw Huffman-weight
nibbles in two serializers. A valid compressed stream can then decode to the
wrong logical bytes without a decode error. This predates the new candidate.
Both the ordinary Rust decoder and the new bounded decoder produced the same
wrong result; independent `zstd` CLI 1.5.7 confirmed it.

Reproducer: generate 65,536 bytes for offsets `65_536..131_072`, with each byte
`((offset / 71 + offset / 251) % 19) as u8`, then encode at numeric Zstd level 3.
The expected SHA-256 is
`e0eceaf70c63d1de0badac66e4e990dec1ead5e16763f100c6f53270b60d5aff`.
The unpatched stream decodes to
`e32cb3e915415b0be6c0b8e0b49838cdc0e685496d03c2068e7ed97afc0786ef`.
Both compressed streams and the before/after hashes are retained here. The corrected
stream decodes to the expected bytes with all three decoders.

The same two-line correction was already present in the local upstream working
tree. This branch copies published 0.2.0 source and applies only that implementation
fix plus its two independent wire-order regression tests. It does not modify or
incorporate the upstream checkout's unrelated ongoing optimizations. Original
crate SHA-256:
`1fbd3540d270b086caa6f53b83aa9d7e12f5ba870029da73b6950fd530610feb`.
Original licenses/notices remain in the vendored copy; `encoder.patch` records the
source change. The workspace dependency stays pure Rust.

Production regressions exercise six numeric levels and commit/independent-reopen
of affected files through the public API in every protection/padding combination.
The complete default suite passes 430 unit tests, all executed integration tests
and ten doctests. The native unit suite has 436 passes and the same two recorded
recovery failures. Those failures are not repaired by the encoder correction.
The candidate format suite passes 113 tests, with four manual resource probes
ignored. All-targets native/external-source Clippy passes. See retained logs.

The upstream wire-order tests pass in a temporary verification copy. The published
crate omits eight compile-time corpus fixtures needed to compile its unit-test
module; those were supplied from the local upstream checkout only in that temporary
tree. They are not runtime dependencies or changes to the vendored implementation.
The two executed wire-order tests themselves use deterministic inline data.

## Release and comparison consequences

Workspace Cargo patches are not inherited by separately published packages.
Before publishing reVault, obtain/publish a corrected dependency release, update
the declared dependency and lockfile, remove the temporary vendor patch, and verify
an isolated packaged/install build. See [patch policy](../../../rust/vendor/PATCHES.md).
Version declarations or this local test result do not establish that release gate.
Already-corrupted encoded payload cannot be reconstructed by fixing the decoder;
verify against source contents where available.

Rebuild candidates A and B with the same corrected encoder before comparing them
with C. Historical performance remains evidence for its recorded source/corpus;
a corrupt output is never an acceptable performance result. The two native recovery
failures, public integration, migration, compatibility and #322 qualification remain
open.


The final native integration/doctest run skips only the two already-failed unit
tests named above; all other executed checks pass. Their separate full native
failure log remains retained. The common-runner adapter passes encrypted-signed
create, stream and range protocol smoke checks with independent verification.
These preflight timings are not performance observations. The final aligned-range
format suite passes 113 tests, with four manual probes ignored.
