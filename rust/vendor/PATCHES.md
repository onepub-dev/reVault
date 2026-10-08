# Local Zstd correctness and performance patches

`zstd-complete/` is the published `zstd-complete` 0.2.0 source, with its original
licenses and notices. The original encoder correction puts the first raw Huffman
weight in the high nibble in both serializers in
`src/huff0/huff0_encoder/weights.rs`. Two independent wire-order regression tests
accompany it. No local encoder optimizations from the upstream working tree are
included.

The released encoder can emit a valid Zstd stream that decodes to incorrect bytes.
The reproducer is 65,536 bytes at offsets 65,536..131,072 of
`((offset / 71 + offset / 251) % 19) as u8`, encoded at level 3. Both the ordinary
Rust decoder, bounded Rust decoder and zstd CLI 1.5.7 produce the same incorrect
content from the unpatched stream. This is an encoder defect, not a decoder
acceptance difference or a candidate archive-layout defect.

The same two-line encoder correction already exists as uncommitted work in the
local upstream checkout. This copy does not depend on that checkout or modify
its ongoing work. The following decoder changes were added locally on 2026-10-02;
no upstream encoder optimizations were imported.

## Full-buffer frame checksum comparison

With the `hash` feature enabled, `src/decoding/frame_decoder.rs` now compares
each completed frame's stored checksum with the value already calculated by the
decoder, before success or advancing to the next concatenated frame. Previously
the full-buffer API exposed those values without comparing them. A distinct
`ChecksumMismatch` error is defined in `src/decoding/errors.rs`. Workspace tests
cover corruption in the first, middle and last frame and valid frames without
checksums. Streaming/incremental APIs retain their separate existing contract;
this patch does not promise that those APIs withhold bytes until frame completion.

The reVault consumer separately retains a full-length zeroizing destination on
fresh fallback decode errors, ensuring partial output is wiped. That change is
outside this vendor directory. Both consumer wrappers also reject declared-length
mismatches before transferring output out of the zeroizing guard. See the
[checksum/wiping evidence](../../docs/evidence/decoder-checksum-2026-10-02/README.md).
That October 2 correction did not cover ordinary decoder-owned internal history.
The subsequent October 8 owned-byte correction below closes specified release
paths; full decoder-memory wiping remains unqualified.

## FSE decoding-table arithmetic

`src/fse/fse_decoder.rs` computes equivalent table entries using the next-state
log/shift formula for valid power-of-two state tables, avoiding repeated integer
division and distribution arithmetic per entry. The prior calculation remains
the fallback for zero/out-of-domain inputs. Exhaustive tests compare 11,188,905
valid combinations against the original calculation, plus boundary/fallback
cases. Table order, format, checksum behavior, memory bounds and wiping are
unchanged. See the [paired experiment](../../docs/evidence/decoder-fse-2026-10-02/README.md).

The same file also adds `#[inline]` to `FSEDecoder::update_state`, retaining its
operations and bounds checks. The [isolated paired trial](../../docs/evidence/decoder-inline-2026-10-02/README.md)
measures a 4.4% compressed plaintext read benefit; other read intervals include
parity. Unexplained process RSS and raw open-time shifts are not claimed as gains.

An overlapping-copy experiment was rejected for lack of demonstrated read
benefit and is not included in this vendor copy.

## Owned byte release and decoded-block bounds — 2026-10-08

Owned history, literal/block byte buffers and decoder-owned dictionary contents
now wipe full capacity before replacement/deallocation. Controlled byte storage
prevents hidden Vec reallocations. Caller-owned workspace policy and the public
Dictionary API remain unchanged. Entropy/sequence metadata, caller-owned data
and process abort are outside this scoped claim. No per-block wipe or mode-based
bypass is introduced; protected archives also pass decrypted data to this codec.

Separate literal-size/count and checked sequence-output bounds reject decoded
blocks beyond 128 KiB before static storage can grow. Short literal inputs return
errors before slicing. Valid multi-block frames remain supported. Tests observe
known live bytes before erasure and zeros before free, plus owned/static error
paths, canaries and allocation-free static decoding. See the
[owned-storage evidence](../../docs/evidence/decoder-owned-storage-2026-10-08/README.md)
for exact limits, qualification and the fixed combined cost comparison.
The initial byte-wise volatile wipe had a material lifecycle cost; aligned
64-byte volatile stores with byte prefix/suffix now retain the identical
full-capacity policy and fence. MaybeUninit partitions avoid reading spare
capacity; guard coverage spans every offset 0–63 and length 0–192. The separate
fixed comparison against the byte-wise correction is recorded with that evidence.

## Release boundary

This workspace patch is a local build dependency fix, not release qualification.
Cargo does not carry workspace patches into separately published crates. Before
publishing reVault crates, publish/obtain a verified corrected Zstd release, update
the direct dependency and lockfile, remove this vendor patch, and verify an
isolated packaged/install build. Do not publish a package that silently resolves
the known-bad encoder. Existing corrupted payload cannot be repaired by changing
the decoder; independently verify source contents when available.
