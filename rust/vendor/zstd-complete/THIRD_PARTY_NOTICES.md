# Third-Party Notices

Portions of this package's Rust compressor were derived from and translated
from the Zstandard 1.5.7 compressor implementation, principally `lib/compress/`
at commit `ac66b19e6bd6b83238bf008eecc1298105298532`.

Zstandard offers those sources under either its BSD license or GPLv2. This
package selects the BSD option. Its notice, conditions, and disclaimer form
the package's `BSD-3-Clause` license in `LICENSE`.

The implementation is not represented as clean-room. Automated translation,
analysis, and code-generation do not alter its provenance. Derived areas
include `src/encoding/levels/c_port/`, compressor entropy/table-building code,
and the private generated kernels under `src/kernel/`. The detailed source map
is maintained in `src/encoding/levels/c_port/README.md`.

This package is a fork of Moritz Borcherding's
[`KillingSpark/zstd-rs`](https://github.com/KillingSpark/zstd-rs) (`ruzstd`),
developed with its contributors. Substantial inherited Rust remains,
particularly the decoder, frame parsing, ring buffer, bit I/O, FSE/Huffman
decoding, dictionaries, and `no_std` I/O abstractions. Its copyright and MIT
permission notice are retained in `LICENSES/ruzstd-MIT.txt`. That required
attribution is not a second outbound license choice for `zstd-complete`, which
is distributed under `BSD-3-Clause`.
