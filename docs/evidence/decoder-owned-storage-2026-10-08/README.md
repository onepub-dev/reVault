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

## Validation before and after formatting

- Vendor default-profile decoder unit filter: **17 passed**, no warnings.
- Vendor workspace suite (default before formatting, release afterward): **15 passed, 1 ignored**, no warnings.
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
accompany this checkpoint. All six checks passed again after the formatting hook
in commit `efe496bd`; the post-format logs are retained here.

## Matched codec lifecycle cost

The same correction includes both release wiping and bounds checks; the measured
cost is not attributed solely to wiping. The matched Rust 1.88 codec-only
batch used 30 alternating pairs after three warmup pairs for fresh ordinary
4 KiB/256 KiB/8 MiB decodes and a 256 KiB static-workspace control. The identical
persisted synthetic encoded/raw bytes are independently inventoried and decoded
bytes compared outside each timer. Frame windows are 4 KiB, 256 KiB and 2 MiB.

The baseline executable was frozen from `f1dcc2f7` before these changes. The
harness uses default `std`/`hash` plus `c-port-validation` for deterministic
fixture generation. It times new/decode/drop lifetimes, excluding caller output
allocation/erasure and the final caller-owned static scratch wipe. Each sample
uses a fresh process; repeated decodes use fresh owned decoders or fresh static
facades and warm resident input. RSS is whole-process high water,
not incremental decoder memory. No owned test/build may overlap measurement.
No A3/A4 or whole-archive result follows from this bounded codec comparison.


The candidate `efe496bd` regressed in every measured case. No extra samples or
policy changes were made in response. Ratios below are candidate/baseline time;
intervals use paired bootstrap resampling of log ratios (10,000 replicates).

| Case | Time ratio (95% interval) | Baseline/candidate median process peak RSS |
| --- | --- | --- |
| 4 KiB owned | 1.040 (1.038–1.041) | 3,278 / 3,370 KiB |
| 256 KiB owned | 1.382 (1.380–1.385) | 4,102 / 4,184 KiB |
| 8 MiB owned | 1.443 (1.434–1.457) | 25,848 / 25,950 KiB |
| 256 KiB static | 1.021 (1.019–1.023) | 4,112 / 4,168 KiB |

Owned decoder lifetime erasure remains a correctness requirement despite this
cost; the static control also contains the bounds checks. These data do not
isolate causal contributions or measure public archive reads. The RSS deltas
are whole-process observations, not an incremental-memory acceptance result.

System `libzstd.so.1` 1.5.7 independently decompressed all three saved encoded
files and compared exact raw bytes before timing. Every measured decode also
compared output bytes outside its timer. Before/after source, executable and
corpus inventories match. Each case has three warmup pairs and exactly 30
measured pairs. Samples use a single allowed CPU; affinity, host, initial/final
load, repetitions and exact summaries are retained in `cost/measurements/`.

Harness compilation took 18.73 s for baseline and 18.17 s for candidate, outside
timing. Compiler details and builds are retained separately. An initial runner
preflight failed while serializing Python host metadata, before collecting any
samples. Its artifacts are preserved separately; the corrected runner completed
once. No owned build or test overlapped the timed run.

`cost/main.rs.txt` preserves the exact Rust harness without hook reformatting.
The manifest's absolute path identifies the measured checkout; adapt that path
only when reproducing elsewhere. Executables and raw corpora remain under
`/tmp/revault-wipe-cost`, with their SHA-256 identities retained here. Encoded
fixtures, the deterministic raw generator, all pair records, runner, lockfile,
toolchain and source commit identities are retained in this directory. Local
binary availability is not promised for later environments.
