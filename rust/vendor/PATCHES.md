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
This does not cover the ordinary vendor decoder's owned internal history:
`RingBuffer` and owned `ReusableVec` retain their pre-existing non-zeroizing
deallocation behavior. Full decoder-memory wiping remains unqualified.

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

## Release boundary

This workspace patch is a local build dependency fix, not release qualification.
Cargo does not carry workspace patches into separately published crates. Before
publishing reVault crates, publish/obtain a verified corrected Zstd release, update
the direct dependency and lockfile, remove this vendor patch, and verify an
isolated packaged/install build. Do not publish a package that silently resolves
the known-bad encoder. Existing corrupted payload cannot be repaired by changing
the decoder; independently verify source contents when available.
