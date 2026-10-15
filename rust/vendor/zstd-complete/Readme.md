# zstd-complete

[![crates.io](https://img.shields.io/crates/v/zstd-complete.svg)](https://crates.io/crates/zstd-complete)
[![API documentation](https://docs.rs/zstd-complete/badge.svg)](https://docs.rs/zstd-complete)
[![CI](https://github.com/bsutton/zstd-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bsutton/zstd-rs/actions/workflows/ci.yml)

`zstd-complete` is a pure-Rust Zstandard compressor and decompressor. It works
with standard `.zst` data and provides Rust-friendly APIs for files, streams,
dictionaries, parallel compression, and memory-constrained applications.

The crate produces standards-compatible data rather than trying to reproduce
the zstd C API or generate byte-identical output.

## Install

```toml
[dependencies]
zstd-complete = "0.2"
```

Rust imports the package as `zstd_complete`.

## Quick start

```rust
# #[cfg(feature = "std")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::io::{Cursor, Read};
use zstd_complete::{
    decoding::MultiFrameDecoder,
    encoding::{encode_all, CompressionLevel, EncoderOptions},
};

let input = b"Zstandard compression from safe Rust";
let options = EncoderOptions::new(CompressionLevel::DEFAULT);
let compressed = encode_all(Cursor::new(input), options)?;

let mut decoder = MultiFrameDecoder::new(Cursor::new(compressed));
let mut decoded = Vec::new();
decoder.read_to_end(&mut decoded)?;

assert_eq!(decoded, input);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

For large inputs, `Encoder<W>` implements `std::io::Write` and keeps memory use
bounded. `MultiFrameDecoder` reads complete archives, including concatenated
frames. Lower-level interfaces are also available when an application needs
direct control over buffers or individual frames.

## Highlights

- Compression levels 1 through 22, negative fast levels, and uncompressed
  frames.
- Streaming compression and decompression with checksums and resource limits.
- Raw and formatted dictionaries, reusable prepared dictionaries, and optional
  dictionary training through the `dict_builder` feature.
- Optional ordered multithreaded compression through the `multithreading`
  feature. The normal single-threaded path remains unchanged.
- `no_std` support with `alloc`, including reusable workspaces and workspaces
  backed by caller-provided byte slices.
- Portable scalar code plus runtime-selected acceleration on supported x86-64
  systems.
- Safe public APIs with typed configuration and `Result` errors.

Default features are `std` and `hash`. Disable default features for the
lower-level `no_std` compressor and decompressor.

## Compatibility and scope

Rust output is tested with zstd C, and this crate is tested against data
produced by zstd C. Compression choices may differ, so compressed bytes and
sizes are not guaranteed to match.

This is a native Rust library, not a drop-in replacement for `zstd.h`. It does
not expose the C ABI, deprecated C functions, experimental numeric parameters,
or internal sequence and entropy APIs. Rust workspaces replace C-style custom
allocators and static contexts.

The full comparison with zstd C 1.5.7 is in the
[support matrix](https://github.com/bsutton/zstd-rs/blob/master/docs/ZSTD_1_5_7_SUPPORT_MATRIX.md).

## Performance and assurance

Compression ratio is close to zstd C on the Silesia corpus. zstd C is generally
faster at levels 1, 3, and 8 in the current public-API benchmark, while results
vary by level and input. See the
[release benchmarks](https://github.com/bsutton/zstd-rs/blob/master/benchmarks/RELEASE_BENCHMARKS_2026-08-30.md)
for the measurements and their limitations.

The public API is safe Rust, although carefully bounded `unsafe` code is used
internally for performance. The project is tested on Linux, Windows, macOS,
AArch64, and wasm, with interoperability, forced-scalar, Miri,
AddressSanitizer, and fuzzing checks in the release process.

The translated compressor has received extensive automated and differential
testing, but it has **not received an independent human line-by-line source
review**. These checks support confidence in the implementation; they are not
a formal proof or warranty.

## Origins and license

This project is a fork of
[`ruzstd`](https://github.com/KillingSpark/zstd-rs). The compressor began as a
translation of zstd C 1.5.7 and has since been reworked around Rust ownership,
errors, streaming, and performance. It is maintained as an idiomatic Rust
implementation of the Zstandard format, not as an ongoing line-for-line port.

`zstd-complete` is distributed under `BSD-3-Clause`, using the BSD option
offered by the Zstandard reference implementation. The inherited ruzstd MIT
notice is preserved in `LICENSES/ruzstd-MIT.txt`; additional attribution and
provenance are in `THIRD_PARTY_NOTICES.md`.
