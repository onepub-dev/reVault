# Temporary Zstd correctness patch

`zstd-complete/` is the published `zstd-complete` 0.2.0 source, with its original
licenses and notices. The sole implementation change puts the first raw Huffman
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

The same two-line correction already exists as uncommitted work in the local
upstream checkout. This copy applies only that correction to released source;
it does not depend on that checkout or modify its ongoing work.

This workspace patch is a local build dependency fix, not release qualification.
Cargo does not carry workspace patches into separately published crates. Before
publishing reVault crates, publish/obtain a verified corrected Zstd release, update
the direct dependency and lockfile, remove this vendor patch, and verify an
isolated packaged/install build. Do not publish a package that silently resolves
the known-bad encoder. Existing corrupted payload cannot be repaired by changing
the decoder; independently verify source contents when available.
