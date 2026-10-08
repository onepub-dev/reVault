# Owned decoder byte storage and block bounds

Date: 2026-10-08. This follows checkpoint `f1dcc2f7` and the
[resumed regression validation](../resume-2026-10-08/README.md). It is a
codec correctness/security change, not another claim of archive/ZIP parity.
No archive format, checksum, signing, padding or worker policy changes.

## Owned byte allocation release

The ordinary decoder's history ring and byte buffers previously freed plaintext
without erasure. `WipingBytes` now exposes slice access and controlled growth,
not a mutable `Vec` facade that could reallocate unseen. Growth allocates/copies
live contents, wipes the entire old capacity, then releases it. Drop wipes the
entire current capacity, including previously cleared/truncated bytes. The
history ring likewise wipes its whole allocation before replacement and release.
Volatile byte writes plus a compiler fence retain those writes before deallocation.

This covers owned history, literal bytes, compressed block bytes (which can
include raw literals), scratch dictionary copies, and dictionaries transferred
into the decoder's private map on replacement/drop. Public `Dictionary` fields
and ownership/move behavior remain unchanged. Caller-owned static workspaces
retain their existing caller wipe policy; reVault's scoped `ZeroizingBytes`
wrapper owns their full-allocation erasure.

Reset and per-frame reuse keep data inside the same bounded owned allocation
until release; this is not a per-block wipe policy. Errors do not forcibly end
a caller-retained decoder's lifetime. reVault's fresh fallback drops its decoder
on return, including errors. The codec has no archive-mode context: it also
processes protected archives after decryption, so unencrypted on-disk data alone
does not determine this shared buffer policy.

The claim does **not** cover entropy tables, sequences/other derived metadata,
caller inputs/outputs or caller-held dictionaries, arbitrary external dictionary
mutation, process abort/OOM, freed copies from outside these ownership paths,
or a complete secure-memory audit. It is not a full decoder-memory wiping claim.

## Separate decoded-block bounds correction

Replacing the mutable byte-vector facade exposed pre-existing malformed-input
paths that could exceed caller-owned static capacity. Literal headers now reject
more than 128 KiB before reserving. Raw/RLE/compressed short inputs return errors
before slicing. Huffman output rejects excess literals before pushing beyond
the declared count. Sequence execution preflights a checked sum of all literal
bytes and match lengths against the 128 KiB decoded-block limit, before history
writes or offset mutation. Multi-block frames may still exceed 128 KiB.

These are bounds corrections, separate from erasure. They do not claim every
malformed-parser path has been audited (for example, four-stream per-stream
balance and other existing parser semantics remain outside this tranche).

## Validation before formatting

- Vendor default-profile decoder unit filter: **17 passed**, no warnings.
- Vendor default-profile workspace suite: **15 passed, 1 ignored**, no warnings.
- Vendor `cargo check --no-default-features`: **passed**.
- Pinned Rust 1.88 release archive compression: **21 passed**.
- Pinned release format suite: **241 passed, 7 ignored**.
- Strict archive library/tests/benches Clippy: **passed**.

Release observers see nonzero **initialized live slices** before erasure and
assert full-capacity zero bytes immediately before deallocation. They never
read uninitialized spare capacity before wiping or inspect freed memory. Tests
cover growth, truncation/reset, dictionary replacement, successful decode,
checksum/truncation failures and concatenated frames with window growth.

Bounds tests cover exact-limit success with byte comparisons, one-over rejection,
checked arithmetic overflow and rejection before history/offset mutation. Owned
and static buffers are covered; static integration tests retain guard bytes and
perform no allocation during the tested decode/error operations. Valid existing
workspace coverage includes larger multi-block frames. Logs and a source manifest
accompany this checkpoint. Post-format verification remains a separate step.

## Declared cost experiment

The same correction includes both release wiping and bounds checks; the measured
cost will not be attributed solely to wiping. A new matched Rust 1.88 codec-only
batch uses 30 alternating pairs after three warmup pairs for fresh ordinary
4 KiB/256 KiB/8 MiB decodes and a 256 KiB static-workspace control. The identical
persisted synthetic encoded/raw bytes are independently inventoried and decoded
bytes compared outside each timer. Frame windows are 4 KiB, 256 KiB and 2 MiB.

The baseline executable was frozen from `f1dcc2f7` before these changes. The
harness uses default `std`/`hash` plus `c-port-validation` for deterministic
fixture generation. It times new/decode/drop lifetimes, excluding caller output
allocation/erasure and the final caller-owned static scratch wipe. Each sample
uses a fresh process; repeated decodes use fresh owned decoders or fresh static
facades and warm resident input. RSS, if reported, is whole-process high water,
not incremental decoder memory. No owned test/build may overlap measurement.
No A3/A4 or whole-archive result follows from this bounded codec comparison.
