# C Compressor Port Source Map

## Publication direction (supersedes the original faithful-port objective)

This directory records provenance and the experiments that produced the
compressor; it is not a promise to preserve C structure or output decisions.
Zstandard 1.5.7 was the disclosed source, algorithmic reference, and parity
oracle. The maintained product is an idiomatic Rust implementation of the
Zstandard format. Measured Rust-specific improvements may diverge from C, exact
compressed-byte parity is not public behavior, and recurring audits of future
C releases are explicitly out of scope.

`zstd-complete` is distributed under the selected Zstandard `BSD-3-Clause`
option. The inherited ruzstd copyright and MIT permission notice remains in
`LICENSES/ruzstd-MIT.txt` as required attribution for substantial pre-existing
decoder and supporting Rust code; it is not a second outbound package-license
choice. See `THIRD_PARTY_NOTICES.md`. Refactoring or moving derived code does
not remove its provenance.

Future structural work should extract cohesive Rust-owned objects and complete
producer-to-consumer transactions, not small helpers chosen merely to resemble
C functions. Because compiler boundaries and layout materially affect CPU,
each such refactor requires same-binary controls and a clean post-removal
benchmark.

Authoritative C source: the local `zstd-sys` checkout matching `Cargo.lock`.

Do not re-discover or re-clone the C implementation for this port unless the
local `zstd-sys` checkout is missing. Use other local zstd checkouts only for
deliberate comparisons.

This module is a staged Rust port of the upstream zstd C compressor. Keep new
code split by the same behavioral boundaries as the C implementation, while
using Rust ownership and types instead of transliterating pointer-heavy C.

Historical restart objective: the work originally pursued a faithful C port
until size and CPU were comparable. That objective is complete as a porting
phase and is retained below only to explain experiment decisions. Resume from
the publication/API/portability direction in `AGENTS.md` and the pause report.

Local C source:

- Use the local `zstd-sys` source matching `Cargo.lock` as the authoritative C
  source tree for this porting work:
  `/home/bsutton/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zstd-sys-2.0.16+zstd.1.5.7/zstd`.
- `/tmp/zstd-reference` may exist in some sessions as a scratch checkout, but
  `/tmp` can be cleaned. Recreate it from the same upstream version before doing
  broad line-by-line parity audits that need an editable copy.

Primary C references:

- `lib/compress/clevels.h`: compression level parameter table.
- `lib/compress/zstd_compress.c`: frame/block orchestration, parameter
  adjustment, block compressor selection, one-shot API behavior, and dictionary
  loading via `ZSTD_compress_insertDictionary()` and
  `ZSTD_loadDictionaryContent()`.
- `lib/compress/zstd_fast.c`: level 1/2 fast match finder.
- `lib/compress/zstd_double_fast.c`: double-fast match finder.
- `lib/compress/zstd_lazy.c`: greedy, lazy, lazy2, and btlazy2 search.
- `lib/compress/zstd_opt.c`: btopt, btultra, and btultra2 parser.
- `lib/compress/zstd_compress_literals.c`: literal block compression.
- `lib/compress/zstd_compress_sequences.c`: sequence entropy encoding.
- `lib/compress/zstd_compress_superblock.c`: superblock path.

Porting rule: add parity tests at the module boundary before wiring the module
into the active encoder path.

Reviewability note: target-compressed-size code is split by role. Keep
`target_block.rs` focused on candidate ordering and fallback policy, keep
single-sub-block emission helpers in `target_single.rs`, and keep multi-block
budgeting/emission in `target_multi.rs` and `target_multi_basic.rs`.
Optimal parser code is also split by role: keep `opt_parser.rs` focused on
entry-point dispatch and the parse loop, keep forward dynamic programming in
`opt_parser/forward.rs`, and keep match-collection dispatch/LDM integration in
`opt_parser/matches.rs`.
Optimal match-finder code is split similarly: keep `opt_match.rs` focused on
entry points, bounds, and repcode helpers; keep match table storage in
`opt_match/table.rs`; keep the request descriptor in `opt_match/request.rs`;
and keep the binary-tree walk/update loops in `opt_match/tree.rs`.
Keep optimal price-state behavior in `opt_price.rs` and price/weight/scaling
formula helpers in `opt_price/weights.rs`. Keep greedy block orchestration in
`greedy_block.rs` and the raw/special-block shortcut in
`greedy_block/special.rs`.
Unsafe code should stay centralized. `unaligned.rs` owns the intentional
unaligned scalar reads, `dfast_table.rs` owns the DFast table-access invariant,
`fse_encoder/table.rs` owns validated FSE compression-table access, and
`x86.rs` owns the x86 prefetch/SSE2 intrinsics. Private modules under
`crate::kernel` own the generated Huffman, row, Fast, DFast, and sequence-store
transactions, including their bounded raw tables and primitive interfaces.
Former unsafe call-site modules (`row_match.rs`, `fast_helpers.rs`,
`dfast_helpers.rs`, `hash_chain_match.rs`, and `match_count.rs`) now forbid
unsafe code locally.

Current parity notes:

- Rejected August 30 complete Fast/DFast entropy-sequence handoff: the existing
  statistics pass converted matcher-owned 12-byte records in place into
  12-byte code-ready records consumed by all reverse emission modes. It added
  no allocation, sidecar, record growth, marker, or traversal, and higher
  strategies retained their original records. Exhaustive legal-length tests,
  759 library tests (5 ignored), control oracles, exact focused output, and
  decoding passed. Same-binary instruction changes at levels 1/3/5/8/16 were
  `+0.8187%`/`+1.0365%`/`+0.0021%`/`+0.0471%`/`+0.0871%`. Packing and
  unpacking cost more than the retained compact on-demand LL/ML lookup, so the
  complete candidate and control were removed. Artifacts begin with
  `perf-z000033-fast-entropy-sequence-handoff-`.

- Rejected August 30 compact FSE symbol transform: normalized probability and
  `delta_find_state` were temporarily narrowed to their proven `i16` ranges,
  shrinking the shared transform from 12 to 8 bytes while retaining `i32`
  calculation semantics. Focused FSE oracles and output passed. Preserved-
  binary instruction changes at levels 1/3/5/8/16 were `-0.2547%`/
  `-0.3552%`/`+0.3275%`/`-0.0016%`/`-0.3235%`; the level-5 regression repeated
  at `+0.3183%` and `+0.2610%` with a matching branch increase. The candidate
  was reverted. Artifacts begin with `perf-z000033-compact-fse-transform-`.
  A follow-up kept C's two hot fields full-width in an 8-byte transform and
  moved only probabilities into a separately recycled `i16` vector. It still
  changed instructions by `-0.4585%`/`-0.3241%`/`+0.3571%`/`+0.0536%`/
  `-4.0657%` at levels 1/3/5/8/16 and raised level-5 branches about `0.81%`.
  That variant was reverted as well; artifacts begin with
  `perf-z000033-split-fse-transform-`.

- Rejected August 30 packed FSE transform load: C loads its two full-width hot
  transform fields as one 64-bit unit, so Rust temporarily used one unaligned
  `u64` read plus endian-specific extraction from the unchanged 12-byte
  record. Instructions changed `+0.2100%`/`-0.0110%`/`+0.1164%`/`+0.0718%`/
  `-5.3270%` at levels 1/3/5/8/16. Level 3 was neutral and three guards
  regressed, so the candidate and added unsafe read were removed. Retain the
  compiler's separate field loads. Artifacts begin with
  `perf-z000033-packed-fse-transform-load-`.

- Kept August 30 compact length-code lookup representation: the 64-entry LL
  and 128-entry ML small-value tables now store an explicit 8-byte record
  containing `u32` additional bits and `u8` code/bit-width fields. The former
  tuple representation occupied 16 bytes per entry because its bit width was
  a `usize`. Helper return types and all formulas remain unchanged, with the
  bit width widened only after loading. A compile-time assertion fixes the
  compact layout.

  Preserved-binary instruction changes at levels 1/3/5/8/16 were
  `-0.6041%`/`-0.3970%`/`-0.2200%`/`-0.1527%`/`-5.5166%`. Three additional
  level-16 pairs improved `5.5851%`, `5.5445%`, and `5.5598%`; branches,
  misses, and cycles also improved. The larger high-level win comes from the
  same tables being hot throughout optimal pricing and parsing. All 1,095
  normal, target-2048, and prepared-CDict candidate/control rows were exact
  and decoded. Artifacts begin with `compact-length-codes-`.

- Rejected August 30 Huffman select-once unroll: moving the runtime table-log
  switch outside the four-stream BMI2 loop preserved output but changed
  level-1 instructions by `+0.0044%`, regressed cycles about `1.02%`, and grew
  the selected generated symbol from about 5.9 KiB to 8.0 KiB. The complete
  candidate and control were removed. Do not repeat dispatch-only Huffman
  specialization without an inner-loop work reduction.

- Kept August 30 Fast/DFast post-count statistics transactions: the hot native
  sequence-record walks now only classify and increment LL/ML/OF count bins.
  DFast derives total, most-frequent, and maximum-symbol metadata from its
  three bounded `u32` lanes afterward; Fast derives total and most-frequent
  from its existing `CodeCounts` arrays afterward. This removes three running
  maximum updates and three total updates per sequence without changing table
  selection, count widths, or emitted bytes.

  DFast's removed same-binary control produced level-3 candidate/control
  instruction counts of `939,787,271`/`966,011,665`, a `2.7147%` improvement;
  levels 1/5/8/16 were neutral. Fast's three paired 50-run level-1 samples
  improved instructions by `1.7289%`, `1.9743%`, and `2.0391%`; branch counts
  were neutral and two of three cycle samples improved. Both changes passed
  exact candidate/control comparison across 365 normal, 292 target-2048, and
  292 prepared-CDict rows. Artifacts begin with `dfast-post-count-stats-` and
  `fast-post-count-stats-`.

- August 30 SIMD audit: the retained four-lane literal histogram already has
  an SSE2-vectorized lane merge in release assembly. Its hot counting loop is
  a data-dependent byte-to-bin scatter, for which AVX2 and NEON provide no
  direct operation. No SIMD candidate was introduced there. The fresh profile
  instead justified the post-count sequence-statistics transactions above.

- Latest kept August 28 matcher-produced repeat-state transaction: every
  native stored-sequence strategy now hands the matcher's final repeat offsets
  directly to deferred entropy state. C updates repeat history during match
  storage; Rust previously replayed every stored record during table selection
  to calculate it again. Const-specialized Fast and compact DFast count walks
  now omit that update, while Greedy/Lazy/optimal omit their separate replay.
  Accepted compressed blocks commit the provisional state; raw/RLE blocks
  discard it. `RUZSTD_TUNE_C_MATCHER_OFFSET_HANDOFF=0` selects the former
  replay path as the same-binary control.

  Eager/deferred all-strategy oracles and explicit rejection tests prove exact
  output, entropy decisions, and histories. The full workspace/static/release
  gate passed with 730 library tests and 5 ignored, and all 511 normal, 292
  target-2048, and 292 prepared-CDict candidate/control rows decode and match.
  Lunx candidate/control instruction medians at levels 1/3/5/8/16 are
  `549,077,573`/`556,712,965`, `951,451,925`/`965,767,611`,
  `2,005,049,592`/`2,019,122,704`, `2,747,793,891`/`2,760,470,050`, and
  `7,840,892,862`/`7,840,892,936`: `-1.3715%`, `-1.4823%`, `-0.6970%`,
  `-0.4592%`, and effectively zero. Branches improve at every affected level,
  bytes are exact, and Lunx recommends KEEP. Counter artifacts end in
  `matcher-offset-handoff-ab-{candidate,control}`; broad artifacts begin with
  `matcher-offset-handoff-`.

- Latest kept August 28 DFast C-width sequence-count transaction: the compact
  LL/ML/OF histogram remains C's `unsigned`/Rust `u32` through table-log
  selection, last-symbol removal, and both fast and slow FSE normalization.
  This halves the DFast count workspace and eliminates the machine-word
  conversion before `ZSTD_buildCTable()`; Fast and higher strategies are
  unchanged. An exhaustive width-equivalence oracle covers all relevant
  alphabet maxima, 97 generated distributions, table logs 5/6/8/9, and both
  low-probability policies. A transaction oracle compares RLE, predefined,
  and encoded mode bytes/table descriptions plus final repeat history.

  Full workspace/static/release gates and all 1,095 decode-verified rows passed
  exactly. Lunx candidate/control instruction medians at levels 1/3/5/8/16
  were `549,079,461`/`549,079,053`, `952,320,973`/`952,398,373`,
  `2,005,005,008`/`2,005,005,745`, `2,747,795,258`/`2,747,794,121`, and
  `7,840,893,729`/`7,841,519,196`. The level-3 target improves `0.0081%` and
  every guard is neutral; branches at level 3 improve `0.075%`. Lunx
  recommended KEEP. The temporary control and old DFast path were removed,
  then full validation and the broad matrices passed again. Counter artifacts
  begin with `perf-z000033-c-dfast-u32-counts-ab-`; broad artifacts begin with
  `c-dfast-u32-counts-`. This closes a faithful data-width boundary but only
  marginally reduces the remaining level-3 CPU gap.

- Rejected August 28 Fast/DFast in-record sequence-code handoff: the existing
  forward statistics walk cached LL/ML codes in unused high bits of the same
  12-byte matcher record, and the reverse BMI2 emitter consumed them through
  C's bit-count tables. This added no allocation, extra traversal, or record
  growth and used `RUZSTD_TUNE_C_FAST_CACHED_SEQUENCE_CODES=0` as its
  same-binary control. Exhaustive legal-length and all-table/RLE/mixed oracles,
  full gates, and all 1,095 decode-verified candidate/control rows passed
  exactly.

  Lunx candidate/control instruction medians at levels 1/3/5/8/16 were
  `564,591,339`/`556,837,879`, `978,107,169`/`964,867,598`,
  `2,015,067,161`/`2,015,069,056`, `2,756,980,136`/`2,756,979,135`, and
  `7,840,892,680`/`7,841,519,764`. The intended levels regressed
  `+1.3924%`/`+1.3722%`; guards were neutral. Although branches fell about
  `1.61%`/`1.45%`, cycles also rose, so marker extraction, masking, and code
  lookup outweighed avoided classification. Lunx recommended REVERT and the
  entire candidate was removed. Artifacts end in
  `cached-sequence-codes-ab-{candidate,control}`; broad artifacts begin with
  `cached-sequence-codes-`. Do not retry this representation without a larger
  producer-to-consumer boundary that writes final code-ready values directly.

- Rejected August 28 complete Fast/DFast sequence-orchestration transaction:
  one strategy-gated function owned table-mode selection, the mode byte,
  LL/OF/ML descriptions, reverse sequence emission, the legacy-decoder guard,
  and conversion into deferred FSE/history updates. The former split path was
  the same-binary control under
  `RUZSTD_TUNE_C_STORED_SEQUENCE_TRANSACTION=0`. An exact oracle covered Fast
  and DFast, matcher and replay histories, predefined/RLE/encoded mixtures,
  bytes, tables, offsets, and fallback state. Full gates and all 1,095
  decode-verified candidate/control rows passed exactly.

  Lunx candidate/control instruction medians at levels 1/3/5/8/16 were
  `549,087,386`/`549,083,111`, `951,513,231`/`951,458,873`,
  `2,005,008,430`/`2,005,054,582`, `2,747,840,859`/`2,747,797,413`, and
  `7,841,518,709`/`7,841,519,032`: `+0.0008%`, `+0.0057%`, `-0.0023%`,
  `+0.0016%`, and effectively zero. Cycles and misses moved noisily, but the
  stable instruction/branch counters showed no benefit. Lunx recommended
  REVERT and the complete candidate was removed. Artifacts end in
  `stored-sequence-transaction-ab-{candidate,control}` and broad artifacts
  begin with `stored-sequence-transaction-`. Do not repeat a wrapper-only
  ownership move; the retained selector ABI and two record traversals must be
  eliminated or replaced for this boundary to save CPU.

- Rejected August 28 C high-bit LL/ML conversion on the current retained
  baseline: C's exact `highbit` plus base-subtraction formulas replaced the
  piecewise large-length ranges and shrank the BMI2 emitter symbols by about
  38%. Exhaustive legal-domain tests, full gates, and all 1,095 decode-verified
  candidate/control rows passed exactly. Lunx candidate/control instruction
  medians at levels 1/3/5/8/16 were `550,140,436`/`549,075,424`,
  `953,314,756`/`951,441,134`, `2,001,577,576`/`2,004,634,190`,
  `2,744,692,098`/`2,747,826,672`, and
  `7,754,986,391`/`7,840,890,331`: `+0.194%`, `+0.197%`, `-0.152%`,
  `-0.115%`, and `-1.097%`. Although branches fell at every level, the only
  two slower-than-C levels regressed in instructions. Lunx recommended REVERT
  and the candidate was removed. Post-revert full validation passed. Counter
  artifacts begin with
  `perf-z000033-c-highbit-length-codes-ab-`; broad artifacts begin with
  `c-highbit-length-codes-`. Keep the piecewise Fast/DFast conversion unless a
  larger producer/consumer representation removes more work than the arithmetic
  substitution adds.

- Latest kept August 28 nested Huffman-weight FSE workspace lifetime: normal
  Fast/DFast generated Huffman construction now borrows the frame's existing
  `FSETableBuildScratch` for weight-table normalization, compact table
  construction, and serialization. After the two-state weight stream is
  emitted, `FSEEncoder` returns its temporary table to the same pool before
  LL/OF/ML construction, matching C's sequential workspace lifetime without
  adding another frame allocation. Target, higher-strategy, target-size,
  prepared-CDict, and non-scratch paths are unchanged. The same-binary
  allocating control is
  `RUZSTD_TUNE_C_REUSE_HUFFMAN_WEIGHT_FSE_SCRATCH=0`.

  Oracles prove exact normal and FSE-unusable descriptions, stable state and
  transform allocation addresses, complete Huffman tables, and literal
  output. The full workspace/static/release gates passed with 728 library
  tests and 5 ignored. All 511 normal, 292 target-2048, and 292 prepared-CDict
  candidate/control rows decode and match exactly. Lunx candidate/control
  instruction medians at levels 1/3/5/8/16 are
  `556,703,513`/`556,807,992`, `965,705,303`/`966,922,450`,
  `2,024,934,721`/`2,024,953,200`, `2,765,732,915`/`2,765,707,852`, and
  `7,841,518,959`/`7,841,498,235`: `-0.0188%`, `-0.1259%`, `-0.0009%`,
  `+0.0009%`, and `+0.0003%`. Bytes are exact, guards are instruction-neutral,
  and Lunx recommends KEEP. Counter artifacts end in
  `huffman-weight-fse-scratch-ab-{candidate,control}`; broad artifacts begin
  with `huffman-weight-fse-scratch-`.

- Latest kept August 28 Fast/DFast Huffman output-table lifecycle: frame-owned
  `HuffmanBuildScratch` now retains one released `HuffmanTable`, completing
  C's current/next Huffman ownership model alongside the existing reusable
  tree workspace. The generated builder refills the owning code vector and an
  exact in-place raw/FSE serializer refills the description vector. Accepted
  new tables recycle the superseded current table; raw/RLE block rejection
  recycles provisional tables; and repeat selection recycles the freshly
  built unselected table. Target, higher-strategy, and non-scratch paths are
  unchanged. `RUZSTD_TUNE_C_RECYCLE_FAST_HUFFMAN_TABLES=0` is the same-binary
  allocating control.

  Exact allocation and lifecycle tests prove stable code/description
  addresses, identical table fields and bytes, accepted replacement,
  rejection, and repeat-selection behavior. Lunx candidate/control medians at
  levels 1/3/5/8/16 are `556,786,463`/`557,223,747`,
  `966,780,303`/`969,754,487`, `2,024,824,672`/`2,023,896,012`,
  `2,765,603,524`/`2,764,705,446`, and
  `7,840,148,575`/`7,835,273,783`: `-0.0785%`, `-0.3067%`,
  `+0.0459%`, `+0.0325%`, and `+0.0622%`. Bytes are exact, the low-level
  branches/misses improve, every guard is below `0.063%`, and Lunx recommends
  KEEP. Validation passed 725 library tests with 5 ignored, all workspace and
  release/static gates, and all 1,095 broad candidate/control rows. Counter
  artifacts end in `huffman-table-recycling-ab-{candidate,control}`; broad
  artifacts begin with `huffman-table-recycling-`.

- Rejected August 28 generated literal/Huffman ownership transaction: the
  existing Huffman codegen unit owned C's combined histogram, fast table-log
  choice, canonical construction, weight conversion, and serialized table
  description, with reusable scratch for the counts and four histogram lanes.
  Exact oracles covered both sides of the 1,500-byte histogram threshold, RLE
  and incompressible exits, count metadata, codes, description bytes, scratch
  reuse, and complete output. All 1,095 normal/target/prepared-CDict rows were
  byte-identical and the full workspace/static/release gates passed. Lunx's
  candidate/control instruction medians at levels 1/3/5/8/16 were
  `562,309,180`/`557,131,220`, `970,986,143`/`969,035,796`,
  `2,023,140,921`/`2,023,140,261`, `2,763,982,083`/`2,763,983,031`, and
  `7,831,310,928`/`7,831,312,369`. The intended levels regressed
  `+0.9294%` and `+0.2013%`; guards were neutral, and branches rose about
  `2.785M`/`2.312M`. Lunx recommended REVERT, so the source and switch were
  removed. Artifacts begin with `generated-literal-table-`; do not repeat a
  dynamic scratch histogram that returns counts to parent costing. The next
  viable boundary must either retain the fixed histogram loop shape or own the
  remainder of literal costing, selection, and emission as well.

- Latest kept August 28 Fast/DFast FSE output-table lifecycle: the frame-owned
  `FSETableBuildScratch` now pools up to three superseded or rejected sequence
  tables and refills their state-table and symbol-transform vectors in place.
  This ports C's alternating `prevEntropy`/`nextEntropy` ownership lifetime in
  addition to the already-retained cumulative/spread workspace. It does not
  change the compact table representation, normalization, NCount description,
  or encoding transitions. The pool chooses the smallest allocation with
  sufficient state/alphabet capacity and otherwise uses the original safe
  initialized builder.

  Recycling follows the deferred stored-entropy transaction: accepted blocks
  recover uniquely owned superseded LL/ML/OF tables, while raw/RLE rejection
  recovers provisional replacement tables without changing current frame
  state. Shared dictionary tables fail `Rc::try_unwrap()` and are dropped
  safely. Higher strategies, target/split mode, prepared records, and estimates
  do not use the pool. `RUZSTD_TUNE_C_RECYCLE_FAST_FSE_TABLES=0` is the
  same-binary control.

  Lunx candidate/control 20-run instruction medians at levels 1/3/5/8/16 are
  `557,111,340`/`557,312,198`, `968,898,134`/`970,183,740`,
  `2,023,043,129`/`2,022,839,697`, `2,763,825,223`/`2,763,760,684`, and
  `7,831,467,509`/`7,831,467,791`: `-0.0360%`, `-0.1325%`, `+0.0101%`,
  `+0.0023%`, and effectively zero. All longitudinal shifts are below `0.11%`,
  bytes are exact, and Lunx recommends KEEP. Allocation and transaction
  oracles cover exact rebuilt tables, stable vector addresses, commit, and
  rejection. Validation passed 720 library tests (5 ignored), all workspace,
  codegen, and documentation suites, strict Clippy, formatting, release builds,
  `git diff --check`, and all 511 normal, 292 target-2048, and 292
  prepared-CDict candidate/control rows. Counter artifacts end in
  `fse-table-recycling-ab-{candidate,control}`; broad artifacts begin with
  `fse-table-recycling-`.
- Latest kept August 28 deferred native entropy-state transaction: normal
  Fast, DFast, Greedy/Lazy, and optimal `StoredSequence` paths now construct
  LL/ML/OF repeat-table updates and the next offset history provisionally.
  They commit that state only when compressed-block acceptance succeeds, like
  C's block-state confirmation boundary; raw and RLE fallbacks discard it
  without cloning and restoring the current frame tables. Target-size,
  split-block, and prepared-sequence paths retain their established rollback
  behavior. `RUZSTD_TUNE_C_DEFER_STORED_ENTROPY_COMMIT=0` selects the eager
  snapshot/restore control in the same binary.

  Keep the pending transaction separate from `CompressedBlockResult` and keep
  eager/deferred emission in distinct non-inlined frames. The first logically
  correct version enlarged the shared result ABI and caused longitudinal
  level-5/8 regressions of about `0.206%`/`0.139%`, even though its same-binary
  comparison looked neutral. The retained isolated shape reduces that movement
  to `+0.0501%`/`+0.0345%`. A two-block all-strategy oracle proves no table or
  offset mutation occurs before commit and exact eager/prepared state afterward.

  Lunx candidate/control 20-run instruction medians at levels 1/3/5/8/16 are
  `557,000,243`/`557,002,105`, `968,516,014`/`968,509,421`,
  `2,020,995,238`/`2,020,996,998`, `2,762,128,560`/`2,762,151,208`, and
  `7,825,918,544`/`7,826,378,091`, changes of `-0.0003%`, `+0.0007%`,
  `-0.0001%`, `-0.0008%`, and `-0.0059%`. Bytes are exact and Lunx recommends
  KEEP. Validation passed 718 library tests (5 ignored), all workspace/codegen
  and documentation suites, strict Clippy, formatting, release builds, `git
  diff --check`, and all 511 normal, 292 target-2048, and 292 prepared-CDict
  candidate/control rows. Counter artifacts end in
  `deferred-entropy-commit-isolated-ab-{candidate,control}`; broad artifacts
  begin with `deferred-entropy-commit-isolated-`.
- Latest kept August 28 per-frame FSE table-construction workspace: normal
  Fast/DFast frame state now owns one `FSETableBuildScratch`. The retained
  initialized builder reuses its cumulative-range and normalized-symbol-spread
  vectors sequentially across LL, OF, and ML and then across source blocks,
  matching `FSE_buildCTable_wksp()`'s caller-owned temporary lifetime. Returned
  `FSETable` vectors, repeat history, normalization probabilities, and NCount
  serialization are unchanged. Target-size, Greedy/Lazy, and optimal paths do
  not use the new workspace; raw/RLE fallbacks preserve it. The same-binary
  control is `RUZSTD_TUNE_C_REUSE_FAST_FSE_BUILD_SCRATCH=0`.

  A two-pass oracle compares every fresh/reused state-table and transform field
  across dense, sparse, zero-containing, and different-log alphabets and
  verifies retained capacity. Lunx's three-sample 20-run candidate/control
  instruction medians at levels 1/3/5/8/16 are
  `556,984,110`/`557,193,901`, `968,426,353`/`970,252,196`,
  `2,019,982,933`/`2,019,967,112`, `2,761,176,403`/`2,761,176,530`, and
  `7,825,922,719`/`7,826,382,998`. Changes are `-0.0377%`, `-0.1882%`,
  `+0.0008%`, `-0.000005%`, and `-0.0059%`: the intended low levels improve
  and all guards are neutral. Output bytes match exactly and Lunx recommends
  KEEP.

  Validation passed 718 library tests (5 ignored), all workspace/codegen/doc
  suites, strict workspace Clippy, formatting, release builds, `git diff
  --check`, and exact candidate/control decode comparison across 511 normal,
  292 target-2048, and 292 prepared-CDict rows. Counter artifacts are
  `benchmarks/tmp/perf-z000033-fast-fse-build-scratch-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`;
  broad artifacts begin with `fast-fse-build-scratch-`. This differs from the
  rejected fixed-short normalization and single-write workspace experiments:
  it only elides temporary allocations around the retained initialized table
  builder and does not change shared table or normalization representations.
- Latest kept August 28 per-frame Huffman construction workspace: normal
  Fast/DFast frame state now owns and reuses `HuffmanBuildScratch` across
  blocks, matching C's compression-context workspace lifetime. The workspace
  is threaded through normal stored-SeqStore block emission and into the
  retained generated tree/rank/code transaction; target-size, Greedy/Lazy,
  and optimal paths retain their existing ownership. Raw and RLE fallbacks
  preserve the workspace for the next block. The same-binary control is
  `RUZSTD_TUNE_C_REUSE_FAST_HUFFMAN_SCRATCH=0`.

  A two-pass oracle proves reused and fresh construction emit identical bytes
  while the generated node allocation retains capacity. Lunx's three-sample
  20-run candidate/control instruction medians at levels 1/3/5/8/16 are
  `557,111,143`/`557,439,523`, `968,289,290`/`970,551,646`,
  `2,019,403,338`/`2,019,404,496`, `2,760,884,813`/`2,760,885,471`, and
  `7,822,204,166`/`7,821,746,940`. The intended levels improve by `0.0589%`
  and `0.2331%`; levels 5 and 8 are identical in practice, and level 16's
  `+0.0058%` is noise. Branches also fall by about `60K` and `450K` per
  20 runs at levels 1 and 3. Lunx recommends KEEP.

  Validation passed 717 library tests (5 ignored), every workspace/codegen
  suite, strict workspace Clippy, formatting, release builds, `git diff
  --check`, and exact candidate/control decode comparison across 511 normal,
  292 target-2048, and 292 prepared-CDict rows. Counter artifacts are
  `benchmarks/tmp/perf-z000033-fast-huffman-scratch-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`;
  broad artifacts begin with `fast-huffman-scratch-`.
- Rejected August 28 complete generated Huffman-weight FSE transaction: after
  the retained generated tree/rank/weight boundary, `ruzstd-huff0-codegen`
  temporarily also owned the complete specialized `HUF_compressWeights()`
  path: C normalization (including slow fallback), compact FSE CTable
  construction, safe-buffer NCount serialization, two-state interleaved FSE
  emission, end marker, and raw/FSE selection. The retained parent serializer
  remained available through
  `RUZSTD_TUNE_C_GENERATED_HUFFMAN_WEIGHT_FSE=0` as a same-binary control.

  An initial broad run exposed a sparse-alphabet NCount zero-run transcription
  error on `corpus_z000050` level 16; the permanent oracle was expanded to
  cover that case and the error was fixed before any performance run. The
  corrected candidate passed 718 workspace tests (5 ignored), strict workspace
  Clippy, formatting, release builds, `git diff --check`, and exact
  candidate/control decode comparison across 511 normal, 292 target-2048, and
  292 prepared-CDict rows.

  Lunx's unattended three-sample 20-run instruction medians at levels
  1/3/5/8/16 were candidate/control `563,334,040`/`557,428,953`,
  `1,009,884,093`/`970,475,629`, `2,058,275,252`/`2,019,387,175`,
  `2,799,173,920`/`2,760,868,680`, and
  `8,054,474,210`/`7,822,096,548`: regressions of `+1.0593%`, `+4.0607%`,
  `+1.9257%`, `+1.3874%`, and `+2.9708%`. Branches also increased at every
  level. Lunx recommended REVERT, and the generated serializer, switch, and
  serializer-only oracles were removed. Artifacts are
  `benchmarks/tmp/perf-z000033-generated-huffman-weight-fse-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`;
  broad artifacts begin with `generated-huffman-weight-fse-`. Keep the retained
  parent FSE/raw serializer callback. Do not repeat this owning serializer port
  without a materially different in-place representation or generated-code
  shape that removes the measured overhead.
- Latest kept August 28 combined generated Huffman construction/description
  boundary: `ruzstd-huff0-codegen` now owns the complete C-shaped bucket sort,
  two-queue tree merge, maximum-height redistribution, canonical-code pass,
  and bits-to-weight consumption as one caller-scratch transaction. The
  generated function immediately invokes the retained exact FSE/raw weight
  serializer once and returns the finished code table plus table description;
  unlike the rejected builder-only boundary, it does not return or allocate an
  intermediate weights vector on the active path. Dense, sparse, tied, skewed,
  constrained-log, and infeasible-alphabet cases are checked against the local
  compact-node builder. `RUZSTD_TUNE_C_GENERATED_HUFFMAN_TABLE=0` selects that
  retained local builder in the same binary.

  Lunx's three-sample 20-run candidate/control instruction medians at levels
  1/3/5/8/16 are `557,434,210`/`557,508,345`,
  `970,468,134`/`971,003,794`, `2,019,378,725`/`2,019,456,436`,
  `2,760,800,008`/`2,760,547,400`, and
  `7,821,713,922`/`7,819,736,195`. Levels 1 and 3 improve by `0.0133%` and
  `0.0552%`; level 5 is effectively neutral, and level 8/16 move by only
  `+0.0092%`/`+0.0253%`. This is a modest CPU improvement and a larger
  structural port, not a major performance step. Bytes are exact and Lunx
  recommends KEEP. Artifacts are
  `benchmarks/tmp/perf-z000033-generated-huffman-described-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`.
  Validation passed 716 library tests (5 ignored), every workspace/codegen
  target, strict workspace Clippy, formatting, release builds,
  `git diff --check`, and exact candidate/control comparison across all 511
  normal, 292 target-2048, and 292 prepared-CDict rows. Broad artifacts end in
  `generated-huffman-described-{candidate,control}.csv`.
- Latest kept August 28 optimal native stored-sequence entropy boundary:
  normal BtOpt, BtUltra, and dictionary-routed BtUltra2 blocks now retain the
  parser's 12-byte C `{litLength, matchLength, offBase}` records through
  literal gathering, C-cost table selection, repeat-history replay, and final
  sequence emission. No-dictionary, external-dictionary, and attached-
  dictionary paths are covered. Target-size and post-split modes deliberately
  retain prepared records because their split/estimate transactions consume
  that representation. The final full-block leading-repcode preference has a
  native stored-record implementation with an exact prepared-equivalence
  oracle. `RUZSTD_TUNE_C_OPT_NATIVE_SEQUENCE_STORE=0` disables only this new
  path for same-binary attribution.

  Lunx's three-sample 20-run candidate/control instruction medians at levels
  1/3/5/8/16/19/22 are `557,503,369`/`557,505,925`,
  `970,977,956`/`970,955,188`, `2,019,415,422`/`2,019,394,120`,
  `2,760,486,533`/`2,760,485,865`, `7,819,629,390`/`7,819,069,594`,
  `20,695,715,385`/`20,695,715,971`, and
  `33,326,521,011`/`33,328,020,673`. All deltas are within `0.008%`: the
  tranche is CPU-neutral, not a performance win. Bytes are exact in both arms,
  and Lunx recommends KEEP because no guard or affected strategy regresses
  materially while this completes the normal native sequence lifecycle.
  Artifacts are
  `benchmarks/tmp/perf-z000033-opt-native-stored-ab-{candidate,control}-l{1,3,5,8,16,19,22}-{1,2,3}.stat`.
  Validation passed 715 library tests (5 ignored), every workspace/codegen
  target, strict workspace Clippy, formatting, release builds,
  `git diff --check`, and exact candidate/control comparison across 511 normal,
  292 target-2048, and an expanded 511-row prepared-CDict matrix through level
  22. Broad artifacts end in `opt-native-store-{candidate,control}.csv`.
- Latest kept August 28 Greedy/Lazy native stored-sequence entropy boundary:
  normal Greedy, Lazy, Lazy2, and BtLazy2 blocks now retain the matcher's
  12-byte C `{litLength, matchLength, offBase}` records through C-cost table
  selection and final sequence emission instead of expanding them into
  16-byte prepared records. The direct lifecycle covers no-dictionary,
  external-dictionary, and attached-dictionary compression; target-size mode
  deliberately retains prepared records for its split/estimate transaction.
  Sequence storage is recycled into `GreedyMatchState` after compressed, raw,
  and RLE decisions. A separate
  `RUZSTD_TUNE_C_GREEDY_NATIVE_SEQUENCE_STORE=0` control leaves the already-
  retained Fast/DFast native path enabled for causal same-binary A/B tests.

  Lunx's three-sample 20-run candidate/control instruction medians at levels
  1/3/5/8/16 are `557,501,764`/`557,501,890`,
  `970,955,186`/`970,978,110`, `2,019,393,304`/`2,041,364,903`,
  `2,760,507,264`/`2,781,490,603`, and
  `7,819,066,651`/`7,819,629,814`. The affected levels 5 and 8 improve by
  `1.0763%` and `0.7544%`; levels 1, 3, and 16 are neutral. Expected bytes are
  exact in both arms. Lunx recommends KEEP. Artifacts are
  `benchmarks/tmp/perf-z000033-greedy-native-stored-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`.
  Validation passed 714 library tests (5 ignored), every workspace/codegen
  target, strict workspace Clippy, formatting, release builds,
  `git diff --check`, and exact candidate/control comparison across all 511
  normal, 292 target-2048, and 292 prepared-CDict rows. Broad artifacts end in
  `greedy-native-store-{candidate,control}.csv`.
- Latest kept August 28 direct native stored-sequence entropy boundary: normal
  Fast/DFast blocks carry their 12-byte native `{litLength, matchLength,
  offBase}` records directly through table selection and sequence emission,
  gathering only literals instead of materializing 16-byte prepared records.
  No-dictionary and external-dictionary paths are covered; target mode and
  higher strategies remain on the prepared path. DFast recycles its sequence
  store after emission. A two-block oracle proves exact prepared/stored bytes,
  FSE state, and repeat history for both strategies. Release builds use one
  codegen unit and named generated sections; because LLD may still shift
  absolute addresses, `RUZSTD_TUNE_C_NATIVE_SEQUENCE_STORE=0` provides a
  same-binary prepared-path control.

  Lunx's three-sample 20-run candidate/control medians at levels 1/3/5/8/16
  are `557,482,949`/`569,154,719`, `970,829,668`/`990,327,690`,
  `2,041,258,492`/`2,041,281,119`, `2,781,407,521`/`2,781,407,398`, and
  `7,819,628,022`/`7,819,628,043`: gains of `2.0507%` and `1.9688%` at the
  affected low levels with the three guard levels neutral. Focused bytes are
  exact and Lunx recommends KEEP. Artifacts are
  `benchmarks/tmp/perf-z000033-stable-direct-stored-ab-{candidate,baseline}-l{1,3,5,8,16}-{1,2,3}.stat`.
  Validation passed 718 library tests (713 passed, 5 ignored), all workspace
  targets, 7 documentation tests, strict workspace Clippy, formatting,
  release builds, `git diff --check`, and all 511 normal, 292 target-2048,
  and 292 prepared-CDict rows against the same-binary prepared control. Broad
  artifacts end in `stable-direct-stored.csv` and `stable-prepared-ab.csv`.
  Refresh paired C level-1/3 attribution next, then continue below the already-
  cheaper matchers in shared table statistics or Huffman construction.
- Rejected August 28 complete generated Huffman construction boundary: the
  `ruzstd-huff0-codegen` crate temporarily owned the safe equivalent of C's
  complete `HUF_buildCTable_wksp()` transaction, including bucket sorting,
  tree merge, height redistribution, and canonical code construction.
  Equivalence tests covered dense, sparse, tied, skewed, constrained-log, and
  fallback distributions. Full tests and strict gates passed, and all 511
  normal, 292 target-2048, and 292 prepared-CDict rows matched the retained
  local-builder control exactly. Lunx's same-binary level-1/3/5/8/16 medians
  regressed by `+0.0440%`, `+0.1837%`, `+0.0915%`, `+0.0778%`, and `+0.0300%`.
  Lunx recommended REVERT and the candidate was removed. Artifacts are
  `benchmarks/tmp/perf-z000033-generated-huffman-build-ab-{candidate,control}-l{1,3,5,8,16}-{1,2,3}.stat`;
  broad artifacts contain `generated-huffman-{candidate,control}`. Preserve
  the local compact-node transaction. If revisited, combine construction with
  table-description or emission consumption rather than returning an owning
  code vector across the crate boundary.
- Superseded first August 28 direct native stored-sequence entropy attempt:
  normal
  Fast/DFast blocks carried their 12-byte native `{litLength, matchLength,
  offBase}` records directly through table selection and sequence emission,
  gathering only literals instead of materializing 16-byte prepared records.
  No-dictionary and external-dictionary paths were covered; target mode and
  higher strategies remained unchanged. A two-block oracle proved exact
  prepared/stored entropy bytes and state for Fast and DFast. Full workspace
  tests, strict Clippy, formatting, release builds, and all 1,095 broad
  normal/target/prepared-CDict rows passed exactly. Lunx medians at levels
  1/3/5/8/16 were `556,962,702`, `971,055,368`, `2,079,735,297`,
  `2,661,157,140`, and `7,571,408,346` instructions: `-1.8030%`, `-1.7093%`,
  `+0.0231%`, `+0.0225%`, and `+0.2584%` versus the retained baseline. REVERT
  because level 16 crossed the 0.2% guard. The low-level data-layout change is
  valuable, but its in-crate duplicate entropy boundary perturbs the retained
  optimal path. Retry only with stable section/codegen isolation. Counter
  artifacts end in `c-direct-stored-sequence-entropy-{1,2,3}.stat`; rejected
  broad artifacts end in `direct-stored-sequences.csv`.
  After removing the candidate, a fresh local rebuild produced single samples
  of `570,260,413`, `993,541,796`, and `7,562,092,283` instructions at levels
  1, 3, and 16 instead of reproducing the historical retained layout bands;
  bytes and all validation remained exact. Refresh a three-sample rebuilt
  baseline or stabilize generated-section placement before the next A/B.
- Rejected August 28 persistent Fast/DFast SeqStore code sidecar: a separate
  reusable LL/ML/OF code vector was populated once per block and used for both
  table selection and reverse emission. It passed the complete correctness
  suite and left all 1,095 broad rows byte-identical, but Lunx measured
  level-1/3/5/8/16 medians of `602,736,817`, `1,033,824,923`,
  `2,083,818,368`, `2,665,496,977`, and `7,551,920,134` instructions. Those
  are `+2.0631%`, `+1.6863%`, `-0.1285%`, `-0.0926%`, and `-0.0715%` versus
  the retained DFast-compact baseline. The candidate was removed because the
  extra fill/traversal and memory traffic lose decisively at levels 1 and 3.
  Do not retry an additive code-array sidecar unless the codes replace an
  existing representation. Artifacts end in
  `c-fast-persistent-sequence-codes-{1,2,3}.stat`.
- Rejected August 28 isolated direct-output sequence transaction: a temporary
  no-std primitive-ABI crate owned the complete 64-bit all-table
  `ZSTD_encodeSequences_body()` transaction and wrote through one pre-sized
  overlapping-word cursor instead of the shared `BitWriter`. Cursor boundary
  tests and an end-to-end writer oracle proved byte identity; full workspace
  tests, strict Clippy, formatting, release builds, and broad parity gates
  passed. Excluding the two Cargo manifest fixtures changed by adding the
  crate, 497 normal, 284 target-2048, and 284 prepared-CDict rows were exact.
  Lunx's level-1/3/5/8/16 medians were `592,630,216`, `1,021,427,448`,
  `2,085,005,722`, `2,666,630,126`, and `7,556,124,985` instructions:
  `+0.3517%`, `+0.4669%`, `-0.0716%`, `-0.0502%`, and `-0.0158%` versus the
  retained DFast-compact baseline. REVERT; the separate cursor/ABI boundary
  loses at the two affected levels. Artifacts end in
  `c-fast-direct-sequence-output-{1,2,3}.stat`; rejected broad artifacts end
  in `c-fast-direct-sequence-output.csv`.
- Rejected August 28 DFast matcher-to-prepared-SeqStore fusion: normal and
  target no-dictionary DFast temporarily wrote literals and fully prepared
  `[ll, ml, rawOffset, offBase]` records inside the isolated matcher, removing
  the intermediate stored-sequence vector and replay pass. Fast, Greedy/Lazy,
  optimal, and external-dictionary paths were untouched. Full tests, strict
  checks, release builds, and all 1,095 broad rows passed exactly. Lunx's
  level-1/3/5/8/16 medians were `590,495,708`, `1,016,884,681`,
  `2,086,155,352`, `2,667,671,783`, and `7,560,202,653` instructions, or
  `-0.0098%`, `+0.0200%`, `-0.0165%`, `-0.0111%`, and `+0.0381%` versus the
  retained baseline. REVERT: all changes were noise and the primary level-3
  target slightly regressed. Artifacts end in
  `dfast-fused-matcher-seqstore-{1,2,3}.stat`; rejected broad artifacts end in
  `dfast-fused-matcher-seqstore.csv`.
- Rejected August 28 DFast C-width normalization transaction: the compact
  three-lane count workspace and complete fast/slow/fallback normalizer used
  `u32` end to end, reducing the generated selection frame from `0x8f8` to
  `0x4f8` bytes and its memset from `0x800` to `0x400`. Exhaustive table-state
  equivalence, full tests, strict checks, and all 1,095 broad rows passed.
  Lunx's level-1/3/5/8/16 medians were `568,362,555`, `988,325,721`,
  `2,073,887,535`, `2,656,619,576`, and `7,558,736,002`, or `+0.2069%`,
  `+0.0389%`, `-0.2581%`, `-0.1480%`, and `+0.0906%` versus the retained
  BMI2 baseline. REVERT: level 3 did not improve and level 1 crossed the guard.
  Artifacts end in `c-dfast-u32-normalization-{1,2,3}.stat`; rejected broad
  artifacts end in `c-dfast-u32-normalization.csv`.
- Rejected August 28 shared FSE single-write construction transaction: the
  normalized-symbol spread and compact state-table build used `MaybeUninit`
  production-log workspaces, while logs below 5 retained the initialized
  algorithm because their masked spread step is not a full permutation.
  Independent equivalence tests covered logs 5 through 12 and the low-log/RLE
  fallback; full tests, strict checks, release builds, focused bytes, and all
  1,095 broad rows passed exactly. Lunx's level-1/3/5/8/16 medians were
  `569,369,696`, `990,905,808`, `2,082,211,386`, `2,663,397,028`, and
  `7,568,714,122` instructions, regressions of `+0.3845%`, `+0.3000%`,
  `+0.1422%`, `+0.1067%`, and `+0.2227%` versus the retained BMI2 baseline.
  REVERT: every level lost. The initialized builder was restored; eliminating
  its zero-fill alone is not useful under the current ownership/generated-code
  shape. Artifacts end in `c-fse-single-write-construction-{1,2,3}.stat`;
  rejected broad artifacts end in `c-fse-single-write-construction.csv`.
- Rejected August 28 Fast/DFast C selection-validity transaction: the general
  Fast and compact DFast table selectors used the maximum code for predefined
  support and trusted `FSE_repeat_valid` as C's full-alphabet invariant,
  removing up to six populated-alphabet scans per block. Debug builds kept the
  scans as equivalence assertions. Full tests, strict checks, release builds,
  focused bytes, and all 1,095 broad rows passed exactly. Lunx's
  level-1/3/5/8/16 medians were `569,976,634`, `991,748,654`,
  `2,082,731,768`, `2,665,049,573`, and `7,571,856,649` instructions,
  regressions of `+0.4915%`, `+0.3853%`, `+0.1672%`, `+0.1688%`, and
  `+0.2643%` versus the retained BMI2 baseline. REVERT: the old scans were
  restored because the changed local control-flow shape loses at every level.
  Artifacts end in `c-fast-selection-validity-{1,2,3}.stat`; rejected broad
  artifacts end in `c-fast-selection-validity.csv`.
- Rejected August 28 isolated Fast/DFast FSE-statistics transaction: a new
  `no_std` generated unit owned code derivation, wide or compact counting,
  offset-history advancement, C heuristic selection, optimal table log, and
  normalization behind a primitive-array ABI. Reference tests covered both
  strategies, every mode, and up to 2,050 sequences; 718 library tests, all
  workspace/codegen suites, strict checks, release builds, and all unchanged
  broad rows passed exactly. Shared Fast+DFast level-1/3/5/8/16 medians were
  `569,663,528`, `982,920,767`, `2,084,932,503`, `2,665,082,931`, and
  `7,571,882,723`, or `+0.4363%`, `-0.5082%`, `+0.2731%`, `+0.1700%`, and
  `+0.2646%` versus the retained BMI2 baseline. DFast-only medians were
  `570,224,688`, `982,758,015`, `2,084,672,060`, `2,664,924,776`, and
  `7,570,931,764`, or `+0.5352%`, `-0.5247%`, `+0.2605%`, `+0.1641%`, and
  `+0.2520%`. REVERT: the level-3 improvement is real, but the added linked
  unit shifts the whole binary enough to breach the cross-level guard. The
  crate and wiring were removed. Artifacts end in
  `c-isolated-fast-fse-statistics-{1,2,3}.stat` and
  `c-isolated-dfast-fse-statistics-{1,2,3}.stat`; corresponding rejected broad
  artifacts use the same stems with `.csv`.
- Rejected August 28 existing-DFast-unit FSE-statistics transaction: the same
  complete DFast code/count/history/policy/optimal-log/normalization boundary
  was moved into the already-retained `ruzstd-dfast-codegen` crate, returning
  primitive mode tags and normalized arrays while `ruzstd` retained concrete
  table ownership. Generated/reference equivalence covered every mode at 1,
  2, 80, 512, and 2,050 sequences. Full workspace validation and all 1,095
  broad rows passed exactly. Lunx's level-1/3/5/8/16 medians were
  `570,224,582`, `980,766,081`, `2,082,061,987`, `2,662,484,665`, and
  `7,566,164,786`, or `+0.5352%`, `-0.7263%`, `+0.1350%`, `+0.0724%`, and
  `+0.1889%` versus the retained BMI2 baseline. REVERT: existing-unit
  placement strengthens the real level-3 win and keeps the other guards below
  0.2%, but level 1 still pays the same 0.54% whole-binary layout penalty.
  Do not retry before Fast placement can be held stable or an equal-size
  resident region can be replaced. Counter artifacts end in
  `c-dfast-existing-unit-fse-statistics-{1,2,3}.stat`; rejected broad
  artifacts end in `dfast-existing-unit-fse-stats.csv`.
- Kept August 28 dynamic BMI2 entropy port: a cached CPUID decision now mirrors
  C's runtime BMI2 dispatch for Fast/DFast all-table sequence emission and the
  complete four-stream Huffman emitter. Both have separate BMI2-targeted and
  portable generated functions; the Huffman hot stream loops are inlined into
  the feature-specific boundary. Disassembly confirms BMI2 variable shifts,
  masking, rotates, and multiplication. Lunx's level-1/3/5/8/16 medians are
  `567,188,864`, `987,941,835`, `2,079,254,880`, `2,660,557,934`, and
  `7,551,897,701` instructions, improvements of `3.9564%`, `2.8268%`,
  `0.3472%`, `0.2778%`, and `0.0718%` over the retained DFast-compact
  baseline. Focused bytes and all 1,095 broad rows are exact. Full workspace
  tests, strict Clippy, formatting, release builds, and `git diff --check`
  pass. Artifacts end in `c-dynamic-bmi2-entropy-{1,2,3}.stat` and
  `c-dynamic-bmi2-entropy.csv`. KEEP; this is the current low-level baseline.
- Kept August 28 DFast compact sequence-statistics transaction: strategy 2
  now counts LL/ML/OF codes in three non-overlapping 64-symbol lanes of one
  safe 256-entry workspace, advances canonical history in the same prepared-
  record walk, carries known totals/maxima directly into normalization, and
  constructs tables in C's LL/OF/ML order. Fast retains the previous wide
  count path: enabling the compact workspace for both strategies improved
  level 3 by `0.6294%` but regressed level 1 by `0.6248%`, so the shared form
  was rejected. With DFast-only gating, Lunx's unattended level-1/3/5/8/16
  medians are `590,553,364`, `1,016,680,870`, `2,086,499,689`,
  `2,667,968,720`, and `7,557,320,806` instructions: `-0.0841%`, `-0.6245%`,
  `+0.0150%`, `+0.0105%`, and `-0.1261%` versus the history baseline.
  Focused bytes and all 1,095 broad rows are exact. Full tests, all codegen
  suites, strict workspace Clippy, formatting, release builds, and
  `git diff --check` pass. Artifacts end in
  `c-dfast-compact-sequence-statistics-{1,2,3}.stat` and
  `c-dfast-compact-sequence-statistics.csv`. KEEP; this is the current
  low-level baseline.
- Kept August 28 Fast/DFast offset-history transaction: the isolated one-pass
  LL/ML/OF table-preparation walk now also advances canonical repeat-offset
  history, removing a separate forward sequence pass for Fast and DFast.
  Greedy/Lazy and higher strategies are unchanged. Lunx's unattended
  level-1/3/5/8/16 medians are `591,050,197`, `1,023,070,414`,
  `2,086,187,823`, `2,667,689,851`, and `7,566,863,445` instructions:
  `-0.0722%`, `-0.0087%`, `+0.0332%`, `+0.0260%`, and `-0.0008%` versus the
  sequence-table baseline. Focused bytes and all 1,095 broad rows are exact.
  Full tests, all codegen suites, strict workspace Clippy, formatting, release
  builds, and `git diff --check` pass. Artifacts end in
  `c-fast-sequence-history-transaction-{1,2,3}.stat` and
  `c-fast-sequence-history-transaction.csv`. KEEP as the retained history
  precursor.
- Kept August 28 Fast/DFast sequence-table preparation transaction: one
  isolated function now performs C's LL/ML/OF code counting in a single
  prepared-record walk and owns all three Fast table-mode decisions plus any
  required table construction. It avoids the three independently allocated C
  code arrays because that representation previously measured worse in Rust,
  but removes the retained path's three duplicate conversion/count scans.
  Greedy/Lazy and higher strategies are unchanged. Lunx's unattended
  level-1/3/5/8/16 medians are `591,477,364`, `1,023,159,220`,
  `2,085,495,652`, `2,666,996,236`, and `7,566,926,455` instructions:
  `-0.6035%`, `-0.7113%`, `+0.0039%`, `+0.0007%`, and `+0.0080%` versus the
  retained emission transaction. Focused bytes and all 1,095 broad rows are
  exact. Full tests, all codegen suites, strict workspace Clippy, formatting,
  release builds, and `git diff --check` pass. Artifacts end in
  `c-fast-sequence-table-transaction-{1,2,3}.stat` and
  `c-fast-sequence-table-transaction.csv`. KEEP as the retained table-
  preparation precursor.
- Kept August 28 Fast/DFast prepared-sequence emission transaction: the
  all-table path now ports the complete hot `ZSTD_encodeSequences_body()`
  boundary into one strategy-gated generated function. It owns code
  conversion, all three compact FSE transitions, the following extra bits,
  state flushes, and the end marker while retaining the safe `BitWriter` and
  16-byte prepared records. FSE and extra-bit batches are joined into one write
  when their combined width fits 64 bits and split exactly at that boundary
  otherwise. Greedy/Lazy and higher strategies keep the preceding generated
  path. Lunx's unattended level-1/3/5/8/16 medians are `595,068,599`,
  `1,030,489,501`, `2,085,414,955`, `2,666,978,668`, and `7,566,320,974`
  instructions: `-0.2384%`, `-0.2890%`, `+0.1409%`, `+0.1027%`, and
  `+0.0314%` versus the explicit-Huffman-unroll baseline. The small
  strategy-excluded layout costs are accepted because this advances the only
  two remaining slower-than-C levels while levels 5 and 8 remain materially
  ahead. Focused bytes are exact. All 511 normal, 292 target-2048, and 292
  prepared-CDict rows are byte-identical; 715 library tests, 7 doc tests, all
  codegen tests, strict workspace Clippy, formatting, release builds, and
  `git diff --check` pass. Artifacts end in
  `c-fast-sequence-transaction-{1,2,3}.stat` and
  `c-fast-sequence-transaction.csv`. KEEP.
- Rejected August 27 packed-Huffman experiment: the final Fast/DFast-gated,
  separately generated packed loop improved 20-run level 1/3 instructions to
  `758.02M`/`1.34819B`, but still regressed level 5/8 to
  `3.89565B`/`4.85926B`, or `+5.52%`/`+5.04%` versus the tuple baselines. The
  packed flag, threading, methods, loop, and packed-only test coverage were
  removed. Post-revert levels 1/3/5/8 returned to `777.539M`, `1.360409B`,
  `3.691756B`, and `4.625917B` instructions with unchanged bytes. The safe
  overlapping eight-byte stores and Fast bounds cleanup remain retained; do
  not retry packed owning tables without a materially different codegen shape.
- Kept August 27 C SeqStore ownership port: stateful DFast and Greedy/Lazy
  match states now retain and recycle their `StoredSequence` allocation after
  block preparation, including ext- and attached-dictionary adapters. This
  mirrors C's persistent `SeqStore_t` lifetime across source blocks. The first
  candidate also covered Fast, but repeated level-1 samples regressed by
  `0.894%`, so Fast reuse was removed. Final 20-run instruction deltas versus
  the preceding tuple baseline are level 1 `+0.0001%`, level 3 `-0.752%`,
  level 5 `-2.892%`, and level 8 `-3.209%`, with identical bytes. The broad
  73-fixture levels 1/3/5/8 matrix is byte-identical to the prior checkpoint;
  artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-api-after-seqstore-reuse.csv`.
  Validation passed 691 library tests with 5 ignored, 7 integration tests,
  strict Clippy, formatting, release builds, and whitespace checks.
- Rejected August 27 frame-owned SeqStore follow-up: a separate sequence
  workspace in `FrameBlockState` avoided perturbing `FastMatchState` and moved
  level 1 from `777.540M` to `775.916M` instructions while leaving level 3
  effectively flat. However, even though Greedy/Lazy did not consume that
  workspace, their focused level 5/8 paths regressed from `3.58499B` and
  `4.47749B` to `3.68201B` (`+2.706%`) and `4.61567B` (`+3.086%`). Bytes were
  unchanged. The frame workspace was removed; retain the strategy-local DFast
  and Greedy/Lazy sequence stores unless a new boundary isolates their
  generated code. Post-revert levels 1/3/5/8 measured `777.538M`, `1.350168B`,
  `3.585004B`, and `4.477488B`, all within `0.001%` of the retained bands.
- Kept August 27 numeric-offBase port: `StoredSequence` now stores C's numeric
  nonzero `offBase` directly and occupies 12 bytes instead of 16;
  `PreparedSequence` uses zero as the generic unset sentinel and occupies 16
  bytes instead of 20. Direct numeric repeat-history methods mirror
  `ZSTD_updateRep()`. Fast and DFast share the resulting C-SeqStore preparation
  helper, while Greedy/Lazy/optimal retain a separate local loop to preserve
  their generated code and exact literal-capacity policy. The first globally
  shared helper regressed levels 5/8 by `2.341%`/`2.879%` and was split before
  acceptance. Final 20-run instruction improvements at levels 1/3/5/8 are
  `0.740%`, `1.203%`, `0.221%`, and `0.101%`; a level-16 guard improves
  `0.376%`. All focused bytes and all 511 broad byte rows are unchanged.
  Artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-numeric-offbase.csv`.
  Validation passed 691 library tests with 5 ignored, 7 integration tests,
  strict Clippy, formatting, release builds, and whitespace checks.
- Rejected August 27 matcher-owned literal-store follow-up: copying literals
  inside Fast/DFast at the C `ZSTD_storeSeq()` boundary preserved focused
  bytes and all 511 broad fixture/level byte rows, but regressed 20-run
  instructions at levels 1/3/5/8 by `6.103%`, `2.431%`, `2.710%`, and
  `3.068%`. The measured medians were `818,887,957`, `1,366,367,235`,
  `3,673,997,531`, and `4,610,191,074`. The candidate was removed. Keep the
  post-match preparation scan unless literal storage is redesigned around a
  preallocated representation closer to C's `SeqStore_t`; repeated safe
  `Vec::extend_from_slice()` calls in the generated matcher loops cost more
  than the later contiguous scan. Rejected broad artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fast-dfast-literal-store.csv`.
  Post-revert levels 1/3/5/8 measured `773,452,650`, `1,327,069,624`,
  `3,577,060,492`, and `4,472,955,558` instructions with the expected output
  totals; levels 5/8 returned within `0.0003%` of the retained medians.
- Rejected August 27 direct C-sequence entropy follow-up: a generic
  sequence-value interface let numeric-offBase `PreparedSequence` records
  bypass allocation of the duplicate encoded `Sequence` vector and feed
  table selection/FSE emission directly. An equivalence test and the full
  511-row matrix proved identical bytes. Levels 1/3/16 improved by
  `1.241%`/`1.259%`/`0.282%`, but the shared generic shape regressed levels
  5/8 by `2.125%`/`2.642%`, to `3,653,087,399` and `4,591,122,462`
  instructions. Luna recommended REVERT and the interface was removed. A
  future direct path must be structurally isolated from Greedy/Lazy generated
  code, as with the retained split preparation helper. Rejected artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-direct-c-sequences.csv`.
  Post-revert levels 1/3/5/8 measured `773,452,532`, `1,327,069,559`,
  `3,577,071,075`, and `4,472,955,283` instructions with exact expected
  output totals; levels 5/8 returned to the retained bands.
- Rejected August 27 compilation-isolated-in-source C-sequence follow-up: a
  separate Fast/DFast block compressor owned one-pass code counting/history,
  count-based table selection, direct FSE loops, and a separate emission entry
  point. The generic compressor and Greedy/Lazy call sites stayed
  source-identical; dedicated predefined/compressed-table tests and all 511
  broad rows proved exact bytes. Nevertheless Luna measured level 5/8 at
  `3,674,261,984` (`+2.717%`) and `4,610,612,244` (`+3.078%`). Level 1/3
  improved to `765,905,125` (`-0.762%`) and `1,322,981,644` (`-0.822%`),
  while level 16 was neutral. The implementation was removed. The large added
  encoder still perturbed Greedy/Lazy at the codegen-unit or binary-layout
  boundary, so a future parallel path needs compilation-level isolation, not
  only a separate Rust function/module. Rejected artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-isolated-c-sequences.csv`.
  Post-revert levels 1/3/5/8 were `773,451,904`, `1,327,059,481`,
  `3,577,070,245`, and `4,472,944,974` instructions with exact expected
  bytes; levels 5/8 returned within `0.0003%` of retained medians.
- Kept August 27 Greedy/Lazy row-cache parity fix: after a found row-hash match
  exits lazy skipping, Rust now refills the eight cached hashes before clearing
  the state, matching C. Six broad level-5/8 rows improve and three formerly
  positive rows become exact; aggregate Rust-minus-C bytes move from `-714` to
  `-752` at level 5 and from `-203` to `-214` at level 8. Isolated Luna medians
  at levels 1/3/5/8/16 are `773,455,753`, `1,327,071,957`, `3,570,566,874`,
  `4,466,926,964`, and `7,730,421,172` instructions. The affected levels 5/8
  improve by `0.182%`/`0.135%`, level 3 improves `0.515%`, level 16 is flat,
  and level 1's `0.216%` increase is minor. Artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-row-cache-refill.csv`.
  Full tests, strict Clippy, release builds, broad decode comparison,
  formatting, and whitespace checks passed.
- Rejected companion depth specialization: making the lazy parser's depth
  0/1/2 a const generic preserved bytes but regressed level 5/8 instructions
  by `1.406%`/`2.247%`, to `3,627,367,430` and `4,573,468,867`; levels 1/3
  also regressed slightly and level 16 was flat. It was removed. The earlier
  combined candidate showed nearly the same loss, proving the regression came
  from monomorphization rather than the retained cache refill. Keep runtime
  depth dispatch unless a different generated-code boundary has new evidence.
- Rejected August 27 buffered row-candidate port: the active and attached row
  paths collected up to 64 matching indexes into safe initialized stack
  buffers, prefetched their source positions, inserted the current position,
  and then scored the buffer, matching C's two-phase pipeline. Focused and all
  511 broad bytes were unchanged, but Luna measured level-5/8 medians of
  `4,102,489,191` and `5,149,129,403` instructions, regressions of
  `14.897%`/`15.272%`; level 3 also regressed `0.729%`. The candidate was
  removed. Rejected artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-buffered-row-candidates.csv`.
  Post-revert levels 5/8 returned to `3,570,557,332` and `4,466,927,616`
  instructions with the retained row-refill bytes. Keep direct row scoring;
  C's uninitialized candidate buffer/prefetch cadence is not efficient when
  expressed as an initialized safe Rust array.
- Rejected August 27 Greedy/Lazy direct frame-output port: normal no-dict,
  streaming ext-dict, loaded-dictionary, and attached-dictionary paths passed
  the existing frame allocation through block encoding, removing per-block
  encoded `Vec` allocation and the final copy. Target mode kept independent
  candidate buffers. An allocation/prefix test, all 511 normal rows, a
  dictionary levels-3/5/8 matrix, full tests, and strict Clippy passed. Luna
  nevertheless measured level-5/8 medians of `3,659,043,219` and
  `4,596,381,301` instructions, regressions of `2.478%`/`2.898%`; level 3
  regressed `0.732%`. The candidate was removed. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-greedy-direct-output.csv`
  and
  `benchmarks/tmp/dictionary-levels-3-5-8-api-after-greedy-direct-output.csv`.
  Post-revert levels 5/8 returned to `3,570,567,157` and `4,466,936,269`.
  Retain the small block-local output plus final copy; threading the frame Vec
  through the hot Greedy encoder produces worse code than the copy it removes.
- Rejected August 28 strategy-local prepared-workspace port: DFast and
  Greedy/Lazy match states retained the derived literal and `PreparedSequence`
  vectors across normal, ext-dictionary, attached-dictionary, and target
  exits. Allocation-reuse tests, full validation, all 511 normal rows,
  prepared-CDict comparison, and target-2048 comparison passed. Luna's stable
  level-1/3/5/8/16 medians were `776,167,107`, `1,341,320,753`,
  `3,669,881,436`, `4,609,168,614`, and `7,731,471,263` instructions. The
  relevant levels 3/5/8 regressed `1.074%`/`2.781%`/`3.184%`; level 16 was
  neutral. The candidate was removed. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-prepared-workspace.csv`,
  `benchmarks/tmp/prepared-cdict-levels-3-5-8-api-after-prepared-workspace.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-prepared-workspace.csv`.
  Post-revert levels 5/8 returned to `3,570,557,746` and `4,466,928,765`.
  Keep fresh derived prepared vectors while retaining matcher-owned
  `StoredSequence` reuse.
- Rejected August 28 complete Fast virtual-index port: all normal, prefix,
  CDict, and ext-dictionary hash-table producers stored C-style `source + 2`
  indexes; zero was invalid and every consumer compared virtual bounds before
  decoding. This was the coherent full representation, not the earlier
  disproven partial acceptance of physical zero. Normal, prepared-CDict, and
  target-2048 level-1 broad bytes were stable, and full validation passed.
  Luna measured a level-1 median of `837,184,677` instructions (`+8.240%`),
  plus level-3/5/8 regressions of `0.612%`/`2.675%`/`3.060%`; level 16 was
  neutral. The conversion was removed. Rejected artifacts:
  `benchmarks/tmp/normal-level-1-api-after-fast-virtual-index.csv`,
  `benchmarks/tmp/prepared-cdict-level-1-api-after-fast-virtual-index.csv`, and
  `benchmarks/tmp/target-2048-level-1-api-after-fast-virtual-index.csv`.
  Retain physical Fast indexes and the explicit `u32::MAX` invalid value.
- Kept August 28 fixed Huffman code-table emission view: normal 256-symbol
  tables now enter a separately generated loop through a safe
  `&[(u32, u8); 256]` view, matching C's fixed `HUF_CElt[256]` lookup bound.
  Compact dictionary/direct tables retain their own checked fallback, and
  equivalence tests cover both paths. No owning table padding or copy was
  added. Luna measured level-1/3/5/8/16 medians of `730,974,166`,
  `1,307,142,527`, `3,543,052,789`, `4,438,895,995`, and `7,708,588,431`
  instructions: improvements of `5.492%`, `1.502%`, `0.771%`, `0.628%`, and
  `0.282%`. Paired level-1 80-run counters are Rust `2,922,990,467`
  instructions and `448,586,921` branches versus C `2,002,999,839` and
  `194,367,009`; Rust's remaining instruction gap is about `45.93%`. All 511
  normal, 292 prepared-CDict, and 292 target-2048 rows are unchanged.
  Validation passed 692 library tests plus 7 integration tests, strict Clippy,
  formatting, release builds, and whitespace checks. Artifacts:
  `benchmarks/tmp/perf-z000033-l1-rust-after-fixed-huffman-code-view.stat`,
  `benchmarks/tmp/perf-z000033-l1-c-api-after-fixed-huffman-code-view.stat`,
  `benchmarks/tmp/perf-instructions-z000033-l1-after-fixed-huffman-code-view.data`,
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fixed-huffman-code-view.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-fixed-huffman-code-view.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-fixed-huffman-code-view.csv`.
- Rejected August 28 paired Huffman-container follow-up: a faithful
  table-log-specialized `K=5..9`, two-container batch cadence improved levels
  1/3/16 by `4.584%`/`1.771%`/`0.299%`, but regressed level 5 by `0.824%`.
  A source-separated Fast/DFast-only API retained the accepted original loop
  for Greedy/Lazy/optimal, yet merely adding and threading those methods
  regressed levels 5/8 by `6.328%`/`4.702%`, to `3,767,250,613` and
  `4,647,624,240` instructions. Both forms were removed. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-paired-huffman-containers.csv`
  and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fast-paired-huffman-containers.csv`.
  Retain the fixed-array lookup and single-container loop; additional
  monomorphized emission methods perturb Greedy/Lazy even when not selected.
- Rejected August 28 isolated paired-Huffman boundary: fresh retained level-1
  instruction profiles attributed about `1.005B` Rust instructions to the Fast
  matcher versus about `1.128B` in C, but about `684M` Rust instructions to the
  fixed Huffman stream path versus roughly `310M` for C's complete Huffman
  compression. The candidate put the fixed 256-code stream loop behind a
  non-generic `#[inline(never)]` boundary (a standalone `0x455`-byte release
  symbol), packed two reverse batches independently, and merged them in stream
  order. Exact output held across all 1,095 broad rows and full validation
  passed. Lunx measured level-1/3/5/8/16 medians of `754,467,511`,
  `1,323,589,319`, `3,880,186,323`, `4,834,607,353`, and `7,717,300,844`
  instructions: `+3.214%`, `+1.258%`, `+9.515%`, `+8.915%`, and `+0.113%`.
  The boundary and helpers were removed; post-revert levels 1/5/8 returned to
  `730,974,257`, `3,543,043,799`, and `4,438,895,350`. Profile artifacts:
  `benchmarks/tmp/perf-instructions-z000033-l1-retained-aug28.data` and
  `benchmarks/tmp/perf-instructions-z000033-l1-c-api-retained-aug28.data`.
  Rejected broad artifacts end in `after-isolated-paired-huffman.csv`, and
  post-revert counters end in `after-isolated-paired-huffman-revert.stat`.
  A normal Rust hot-call boundary is too costly; another paired-stream attempt
  needs a separately compiled artifact with explicit ABI/code-placement control
  or an improvement to the retained inlined loop.
- Kept August 28 cross-crate paired-Huffman port: the complete fixed-table
  four-stream transaction now lives in the dedicated no-std
  `ruzstd-huff0-codegen` crate. It includes C's table-log-specific
  `kUnroll=5..9` loops, two independently packed reverse batches, bounded safe
  overlapping stores, and direct six-byte jump-table output. Compact
  dictionary/direct tables retain the checked local emitter. The release binary
  contains one separately compiled `0x1ea4`-byte `encode_four_streams` symbol,
  providing the code-layout isolation the ordinary boundary lacked. Lunx
  measured level-1/3/5/8/16 medians of `697,332,385`, `1,273,481,236`,
  `3,520,660,311`, `4,416,116,638`, and `7,688,020,879` instructions:
  improvements of `4.602%`, `2.575%`, `0.632%`, `0.513%`, and `0.267%`.
  Focused bytes remain exact. All 1,065 broad rows with unchanged source inputs
  are byte-identical; the 30 excluded rows are the two Cargo-manifest fixtures
  changed by adding this crate. A table-log 1-11 per-symbol reference test and
  full tests/lints/build checks pass. Counter artifacts:
  `benchmarks/tmp/level-{1,3,5,8,16}-cross-crate-paired-huffman.stat`; broad
  artifacts end in `after-cross-crate-paired-huffman.csv`. KEEP this boundary
  and reuse the dedicated-codegen-crate pattern for future substantial entropy
  specializations.
- Rejected August 28 exact C Huffman inner state: the retained codegen crate
  converted its tuple table once per four-stream transaction to C's packed
  high-bit `HUF_CElt`, then used a safe port of `HUF_CStream_t`'s two
  containers, fast/slow last-symbol rules, merge cadence, flushes, and close
  marker. The motivating retained profile put the isolated Huffman stream at
  `20.23%` of level-1 sampled instructions; artifact:
  `benchmarks/tmp/perf-instructions-z000033-l1-after-cross-crate-huffman.data`.
  Exact output held in the table-log 1-11 reference test and all 1,080 broad
  rows with unchanged inputs; full validation passed. Lunx measured
  level-1/3/5/8/16 medians of `699,050,097`, `1,278,700,651`,
  `3,525,494,131`, `4,421,012,144`, and `7,694,860,827` instructions:
  regressions of `0.246%`, `0.410%`, `0.137%`, `0.111%`, and `0.089%`.
  The exact C inner state was removed and level 1 returned to `697,332,163`.
  Counter artifacts end in `exact-c-huffman-stream.stat`, broad artifacts end
  in `after-exact-c-huffman-stream.csv`, and the post-revert counter is
  `benchmarks/tmp/perf-z000033-l1-after-exact-c-huffman-stream-revert.stat`.
  Keep the dedicated crate with its Rust low-bit tuple batches.
- Kept August 28 one-reservation Huffman output port: the isolated fixed-table
  four-stream compressor now initializes its complete proven output bound once,
  emits every specialized stream through one absolute cursor, fills jump-table
  sizes directly, and truncates once after the final stream. This safely ports
  C's single destination transaction without the code-layout regression of the
  earlier non-isolated outer boundary. The release `encode_four_streams` symbol
  shrank from `0x1ea4` to `0x1d23`. Lunx measured level-1/3/5/8/16 medians of
  `695,070,704`, `1,271,810,296`, `3,517,211,158`, `4,412,570,416`, and
  `7,686,836,280` instructions: improvements of `0.324%`, `0.131%`, `0.098%`,
  `0.080%`, and `0.015%`. Focused bytes and all 1,095 broad rows are exact.
  Full tests, strict Clippy, formatting, packaging, release builds, and
  whitespace checks pass. Counter artifacts end in
  `one-reservation-huffman.stat`; broad artifacts end in
  `after-one-reservation-huffman.csv`. KEEP the single-reservation cursor.
- Rejected August 28 isolated all-table sequence emitter: a dedicated no-std
  crate ported the complete hot all-table `ZSTD_encodeSequences_body()` path,
  including LL/ML/OF code conversion, three borrowed compact FSE states,
  batched state and extra-bit writes, state flushes, and the end marker. The
  existing 12-byte `Sequence` and FSE symbol-transform layouts moved without a
  staging allocation. A 280-sequence reference covered all LL/ML ranges and
  offsets through `2^20`; decoder, full validation, and unchanged broad rows
  were exact. Lunx level-1/3/5/8/16 medians were `691,984,284`,
  `1,278,011,853`, `3,608,721,942`, `4,545,639,771`, and `7,681,890,857`
  instructions: `-0.444%`, `+0.488%`, `+2.602%`, `+3.016%`, and `-0.064%`.
  The crate and all integration changes were removed; level 1 returned to
  `695,070,878`. Counter artifacts end in `isolated-sequence-codegen.stat`,
  broad artifacts end in `after-isolated-sequence-codegen.csv`, and the
  post-revert counter is
  `benchmarks/tmp/perf-z000033-l1-after-isolated-sequence-codegen-revert.stat`.
  Keep sequence emission local; Huffman's isolated-crate success does not
  generalize to the shared FSE/Sequence type graph.
- Rejected August 28 split literal-statistics boundary: aggregate C-cost
  counting and optional legacy/small-table four-stream counting were separated
  into distinct functions. The aggregate release symbol shrank to `0x341`
  bytes and its stack frame fell from roughly 14 KiB to 6 KiB. All 1,095 broad
  rows remained byte-identical and full validation passed. Lunx
  level-1/3/5/8/16 medians were `694,844,238`, `1,280,006,405`,
  `3,515,636,543`, `4,411,001,568`, and `7,677,781,242` instructions:
  `-0.033%`, `+0.644%`, `-0.045%`, `-0.036%`, and `-0.118%` versus the
  one-reservation baseline. The split was removed because the stable level-3
  regression outweighed the small gains; level 3 returned to `1,271,810,567`
  instructions after revert. Counter artifacts end in
  `split-literal-stats.stat`, broad artifacts end in
  `after-split-literal-stats.csv`, and the post-revert counter is
  `benchmarks/tmp/perf-z000033-l3-after-split-literal-stats-revert.stat`.
  Preserve the mixed function unless the underlying histogram algorithm also
  changes enough to earn a material cross-level win.
- Rejected August 28 C-width sequence histograms: `CodeCounts`, selection
  policy, sequence cost models, estimators, and table construction used C's
  32-bit count width, retaining 256 entries for statically safe `u8` indexing
  and converting once at the existing `usize` normalizer boundary. Each count
  object shrank from roughly 2,064 to 1,032 bytes. All 1,095 broad rows and the
  full validation gate were exact. Lunx level-1/3/5/8/16 medians were
  `695,492,994`, `1,284,258,764`, `3,616,935,750`, `4,554,702,137`, and
  `7,707,887,063` instructions: regressions of `0.061%`, `0.979%`, `2.835%`,
  `3.221%`, and `0.274%`. The port was removed; post-revert levels 5/8
  returned to `3,517,220,646` and `4,412,570,054` instructions. Counter
  artifacts end in `c-width-code-counts.stat`, broad artifacts end in
  `after-c-width-code-counts.csv`, and post-revert counters end in
  `after-c-width-code-counts-revert.stat`. Keep pointer-width counts unless
  the whole normalizer/table path can stay 32-bit end to end.
- Kept August 28 safe post-match sequence-literal wild copy: the retained
  `prepare_stored_sequences()` scan reserves 64 bytes of literal padding and
  uses fixed 16/32/48/64-byte initialized stores for 1-64-byte literals when
  enough source remains, truncating to the logical length before the next
  append. Larger and source-end copies retain exact slices. This ports the hot
  cadence of C's `ZSTD_storeSeq()` without repeating the rejected design that
  copied literals inside Fast/DFast matcher loops. Transition and source-end
  tests prove exact slices. Lunx level-1/3/5/8/16 medians are `690,436,740`,
  `1,271,531,320`, `3,517,160,266`, `4,412,500,046`, and `7,687,029,038`
  instructions: `-0.667%`, `-0.022%`, `-0.001%`, `-0.002%`, and `+0.003%`.
  Focused bytes and all 1,095 broad rows are exact; full validation passes.
  Counter artifacts end in `sequence-literal-wildcopy.stat` and broad
  artifacts end in `after-sequence-literal-wildcopy.csv`. KEEP this
  post-match fixed-store boundary.
- Kept August 28 fused numeric `offBase`/history transition: sequence
  preparation now resolves C's numeric `offBase` and mutates `RepeatOffsets`
  in one operation, while entropy history consumes the same pre-encoded value
  directly through `OffsetHistory`. Generic zero-valued records retain raw-
  offset encoding. `StoredSequence` remains 12 bytes and `PreparedSequence`
  remains 16 bytes because the latter also carries the resolved raw offset.
  Exhaustive equivalence tests cover the fused C transition and debug builds
  verify each production pre-encoded raw offset. Lunx level-1/3/5/8/16
  medians are `682,694,968`, `1,257,718,491`, `3,498,033,539`,
  `4,394,623,260`, and `7,668,201,525` instructions: `-1.121%`, `-1.086%`,
  `-0.544%`, `-0.405%`, and `-0.245%` versus the retained wildcopy baseline.
  Focused bytes and all 1,095 broad rows are exact; 701 library tests (696
  passed, 5 ignored), 7 integration tests, codegen tests/package, strict
  Clippy, formatting, and diff hygiene pass. Counter artifacts end in
  `fused-offbase-history.stat`; broad artifacts end in
  `after-fused-offbase-history.csv`. KEEP this as the C `ZSTD_updateRep()`
  ownership boundary and preserve numeric `offBase` through entropy history.
- Rejected August 28 compact `PreparedSequence` representation: one `u32`
  stored either raw offset or C `offBase`, with the otherwise unused high
  match-length bit tagging the kind. This reduced the record from 16 to 12
  bytes without narrowing its full offset payload. Layout/kind tests, full
  validation, and all 1,095 broad byte rows were exact. Lunx level-1/3/5/8/16
  medians were `685,430,834`, `1,262,753,980`, `3,601,951,798`,
  `4,539,149,069`, and `7,694,627,271` instructions: `+0.401%`, `+0.400%`,
  `+2.971%`, `+3.289%`, and `+0.345%`. The complete representation and test
  conversion were removed. Rejected broad artifacts end in
  `after-compact-prepared-sequence.csv`. Keep raw offset and `offBase` together
  in the 16-byte prepared record; the smaller layout materially regresses
  Greedy/Lazy generated code. Post-revert level-5/8 samples returned to
  `3,498,034,008` and `4,394,614,425` instructions; artifacts end in
  `compact-prepared-sequence-revert.stat`.
- Rejected August 28 direct final offset-history handoff: preparation returned
  its fully replayed three-slot repeat history with the prepared block, and
  normal C-strategy entropy emission mapped stored numeric `offBase` values
  directly into `Sequence` while accepting that final history. This also
  corrected the wrapper state from the matcher's intentionally stale unused
  third slot to preparation's canonical state; debug replay proved both paths
  equivalent. All 1,095 broad rows and full validation were exact. Lunx
  level-1/3/5/8/16 medians were `670,402,403`, `1,238,383,172`,
  `3,572,525,646`, `4,512,348,670`, and `7,663,654,724` instructions:
  `-1.801%`, `-1.537%`, `+2.130%`, `+2.679%`, and `-0.059%` versus the fused
  baseline. The boundary was removed because Greedy/Lazy regressed
  materially. Post-revert L5/L8 returned to `3,498,024,048` and
  `4,394,624,164`. Counter artifacts end in
  `direct-final-offset-history-{1,2,3}.stat`; broad artifacts end in
  `after-final-offset-handoff.csv`; revert artifacts end in
  `direct-final-offset-history-revert.stat`. Keep entropy's replay despite the
  duplicated history walk until the strategy families are codegen-isolated.
- Rejected August 28 dual-history preparation follow-up: matcher
  `RepeatOffsets` stayed unchanged for the next block while preparation
  advanced entropy's distinct `OffsetHistory`, initialized from the encoder
  context, in the same sequence walk. Debug checks proved both histories
  resolved every emitted match to the same raw offset. This removed the prior
  candidate's third-slot semantic change while still skipping final entropy
  replay. Full validation and all 1,095 broad rows were exact. Lunx
  level-1/3/5/8/16 medians were `676,215,997`, `1,247,178,536`,
  `3,583,244,950`, `4,522,419,655`, and `7,674,847,423` instructions:
  `-0.949%`, `-0.838%`, `+2.436%`, `+2.908%`, and `+0.087%`. The dual history
  and direct handoff were removed. Post-revert L5/L8 returned to
  `3,498,034,208` and `4,394,623,853`. Counter artifacts end in
  `dual-history-preparation-{1,2,3}.stat`; broad artifacts end in
  `after-dual-history-preparation.csv`; revert artifacts end in
  `dual-history-preparation-revert.stat`. The unchanged matcher state rules
  out the previous semantic confound: a future direct-history path needs
  compilation-level isolation from the shared Greedy/Lazy entropy code.
- Rejected August 28 isolated C histogram boundary: the entropy codegen crate
  owned the complete safe `HIST_countFast_wksp()` analogue, including the
  1,500-byte threshold, simple path, four-lane workspace, cached 32-bit preload
  loop, tail, reduction, max symbol, and largest count. Threshold/terminal
  boundary tests, full validation, and all 1,095 broad rows were exact. Lunx
  level-1/3/5/8/16 medians were `700,617,232`, `1,270,638,489`,
  `3,607,071,483`, `4,544,919,602`, and `7,708,260,599` instructions:
  regressions of `2.625%`, `1.027%`, `3.117%`, `3.420%`, and `0.522%`. The
  isolated boundary was removed. Post-revert L1/L5/L8 returned to
  `682,698,427`, `3,498,033,636`, and `4,394,623,245`. Counter artifacts end
  in `isolated-c-histogram-{1,2,3}.stat`; broad artifacts end in
  `after-isolated-c-histogram.csv`; revert artifacts end in
  `isolated-c-histogram-revert.stat`. Keep the local 16-byte striped loop;
  safe sliced cached loads plus a cross-crate call lose decisively here.
- Rejected August 28 fixed-width Huffman batch follow-up: the isolated stream
  encoder replaced dynamic subslice packing for every complete C `kUnroll`
  batch with const-generic five-through-nine-symbol array references and
  reverse index loops; only the initial remainder stayed runtime-sized. Direct
  fixed/dynamic equivalence tests, the table-log 1-11 stream oracle, full
  validation, and all 1,095 broad rows passed exactly. Lunx level-1/3/5/8/16
  medians were `683,809,753`, `1,258,468,460`, `3,498,711,292`,
  `4,395,316,247`, and `7,668,525,239` instructions: regressions of `0.163%`,
  `0.060%`, `0.019%`, `0.016%`, and `0.004%` versus the fused-offBase
  baseline. The helper and its test were removed. Counter artifacts end in
  `fixed-width-huffman-batches-{1,2,3}.stat`; broad artifacts end in
  `after-fixed-width-huffman-batches.csv`. Keep the dynamic subslice packer;
  its generated code is marginally better across every active strategy.
- Latest kept August 28 sequence-selection code-layout change: the entire
  exact combinatorial candidate-build/search transaction moved behind a
  separate non-inlined `choose_exact_sequence_table_modes()`. C strategies
  disable this search, but its body previously shared their 18,009-byte common
  selector symbol; that common symbol is now 5,515 bytes and the exact-only
  body is 12,945 bytes. Exact-search coverage, full validation, focused bytes,
  and all 1,095 broad rows passed unchanged. Lunx level-1/3/5/8/16 medians are
  `681,588,277`, `1,255,863,163`, `3,495,985,588`, `4,392,754,912`, and
  `7,666,283,736` instructions: improvements of `0.162%`, `0.148%`, `0.059%`,
  `0.043%`, and `0.025%` versus the fused-offBase baseline. Counter artifacts
  end in `exact-sequence-search-split-{1,2,3}.stat`; broad artifacts end in
  `after-exact-sequence-search-split.csv`. Keep the split: removing the dormant
  search ownership graph from the C-level selector improves every strategy
  without changing shared records or table modes.
- Latest kept August 28 policy-isolated sequence selection: non-exact selection
  now dispatches its full three-table operation to separate non-inlined C Fast,
  C cost, and legacy functions. The former 5,515-byte common symbol is gone;
  the release functions are 1,507 bytes for C Fast, 1,396 for C cost, 3,050
  for legacy, and 12,945 for the already isolated exact search. Full tests,
  strict validation, focused bytes, and all 1,095 broad rows passed unchanged.
  Lunx level-1/3/5/8/16 medians are `681,032,274`, `1,254,871,589`,
  `3,494,953,335`, `4,391,789,596`, and `7,664,903,280` instructions: further
  improvements of `0.082%`, `0.079%`, `0.030%`, `0.022%`, and `0.018%` over
  the exact-search-only split. Counter artifacts end in
  `policy-isolated-sequence-selection-{1,2,3}.stat`; broad artifacts end in
  `after-policy-isolated-sequence-selection.csv`. Keep all three boundaries;
  each active policy now avoids carrying the other two selection graphs.
- Rejected August 28 sequence-emitter variant isolation: the unchanged
  9,276-byte `encode_sequences()` union became a 376-byte dispatcher plus
  separate non-inlined 4,005-byte all-table, 4,298-byte mixed-table, and
  1,253-byte all-RLE transactions. Existing variant tests, exact sizing, full
  validation, and all 1,095 broad rows passed exactly. Lunx level-1/3/5/8/16
  medians were `681,909,078`, `1,256,536,863`, `3,593,755,427`,
  `4,531,673,040`, and `7,666,823,422` instructions: regressions of `0.129%`,
  `0.133%`, `2.827%`, `3.185%`, and `0.025%` versus the policy-isolated
  baseline. The split was removed. Counter artifacts end in
  `sequence-emitter-variant-split-{1,2,3}.stat`; broad artifacts end in
  `after-sequence-emitter-variant-split.csv`. Keep the local emitter unified;
  hot variant call boundaries again perturb Greedy/Lazy badly.
- Latest kept August 28 target-only literal code-layout change:
  `compress_literals()` now delegates the complete preferred-valid-repeat
  transaction to a non-inlined `try_preferred_repeat_literals()` helper. That
  target-only transaction owns the prior-table checks, 1,024-literal limit,
  RLE/treeless emission, expansion rollback, and raw fallback; normal focused
  compression never calls it. The helper is 447 release bytes and the common
  literal compressor shrank from 5,280 to 4,865 bytes. Targeted repeat-table
  tests, full strict validation, focused bytes, and all 1,095 broad rows passed
  exactly. Lunx level-1/3/5/8/16 medians are `681,033,417`, `1,254,869,656`,
  `3,494,951,231`, `4,391,787,357`, and `7,665,098,189` instructions:
  `+0.000168%`, `-0.000154%`, `-0.000060%`, `-0.000051%`, and `+0.002543%`
  versus the policy-isolated baseline. Counter artifacts end in
  `preferred-repeat-literal-split-{1,2,3}.stat`; broad artifacts end in
  `after-preferred-repeat-literal-split.csv`. Keep this neutral cold boundary;
  unlike splitting hot emission, it removes a dormant ownership graph without
  materially perturbing any active strategy.
- Rejected August 28 literal-policy specialization: the complete literal
  transaction was const-specialized by smallest-table search and C-cost mode,
  producing four separate non-inlined 1.8-2.8 KiB release bodies instead of
  the retained 4.8 KiB union. Focused bytes, full strict validation, and all
  1,095 broad rows passed exactly. Lunx level-1/3/5/8/16 medians were
  `680,802,084`, `1,253,381,059`, `3,590,407,174`, `4,528,393,540`, and
  `7,661,857,087` instructions: `-0.034%`, `-0.119%`, `+2.731%`, `+3.110%`,
  and `-0.042%` versus the preferred-repeat baseline. The specialization was
  removed. Counter artifacts end in
  `literal-policy-specialization-{1,2,3}.stat`; broad artifacts end in
  `after-literal-policy-specialization.csv`. Keep the shared literal
  compressor: unlike the sequence selector's cold decision graph, this split
  duplicated hot table/emission work and materially perturbed Greedy/Lazy.
- Rejected August 28 compact C `SeqDef` port: Fast, DFast, Greedy/Lazy, and
  optimal matcher outputs used C's exact eight-byte `{u32 offBase, u16
  litLength, u16 mlBase}` record plus block-level long-length type/position
  metadata instead of Rust's retained 12-byte full-length record. Direct tests
  covered overflowing literal and match lengths. Full strict validation,
  focused bytes, and all 1,095 broad rows passed exactly. Lunx level-1/3/5/8/16
  medians were `726,375,370`, `1,294,874,612`, `3,898,288,421`,
  `4,852,875,458`, and `7,687,098,506` instructions: regressions of `6.658%`,
  `3.188%`, `11.541%`, `10.499%`, and `0.287%`. The complete compact store was
  removed. Post-revert L1/L5/L8 returned to `681,032,041`, `3,494,937,948`,
  and `4,391,798,633`. Candidate artifacts end in
  `compact-seqdef-{1,2,3}.stat`; revert artifacts end in
  `compact-seqdef-revert.stat`; broad artifacts end in
  `after-compact-seqdef.csv`. Keep the 12-byte direct record: the wrapper and
  overflow reconstruction cost much more than the reduced footprint saves,
  especially in Greedy/Lazy.
- Rejected August 28 bounded native-width sequence-code histograms: the whole
  sequence-cost graph used `[usize; 64]` code counts (528 bytes including the
  total) instead of `[usize; 256]` (2,064 bytes), while retaining native-width
  counters. Direct coverage proved the actual LL/ML/OF domains are 0-35,
  0-52, and 0-31. Full strict validation, focused bytes, and all 1,095 broad
  rows passed exactly. Lunx level-1/3/5/8/16 medians were `684,261,292`,
  `1,249,638,683`, `3,587,374,595`, `4,526,079,269`, and `7,628,136,556`
  instructions: `+0.474%`, `-0.417%`, `+2.644%`, `+3.058%`, and `-0.482%`
  versus the preferred-repeat baseline. The bounded histograms were removed;
  post-revert L1/L5/L8 returned to `681,029,938`, `3,494,937,823`, and
  `4,391,785,472`. Candidate artifacts end in
  `bounded-code-counts-{1,2,3}.stat`; revert artifacts end in
  `bounded-code-counts-revert.stat`; broad artifacts end in
  `after-bounded-code-counts.csv`. Keep the 256-entry arrays: full-`u8`
  indexing lets LLVM eliminate the bounds check, while the smaller physical
  arrays materially perturb Greedy/Lazy generated code.
- Rejected August 28 direct preallocated `SeqStore` output arena: Fast/DFast's
  whole post-match preparation transaction used reserved literal and sequence
  arrays instead of repeated literal `Vec` extend/truncate operations and
  prepared-sequence pushes. A narrowly owned `MaybeUninit` writer performed
  C's retained bounded 16/32/48/64-byte wild copies directly, filled each
  sequence slot, and exposed both initialized prefixes once; Greedy/Lazy kept
  its existing separate loop. Full strict validation, focused bytes, and all
  1,095 broad rows passed exactly. Lunx level-1/3/5/8/16 medians were
  `680,222,656`, `1,253,323,119`, `3,591,958,291`, `4,529,963,261`, and
  `7,664,898,035` instructions: `-0.119%`, `-0.123%`, `+2.776%`, `+3.146%`,
  and `-0.003%` versus the preferred-repeat baseline. The arena was removed;
  post-revert L1/L5/L8 returned to `681,031,844`, `3,494,951,077`, and
  `4,391,798,609`. Candidate artifacts end in
  `direct-seqstore-arena-{1,2,3}.stat`; revert artifacts end in
  `direct-seqstore-arena-revert.stat`; broad artifacts end in
  `after-direct-seqstore-arena.csv`. Keep the safe extend/truncate and push
  path: its removal yields only a tiny Fast/DFast gain while the extra arena
  code perturbs Greedy/Lazy by about 3% even though they never call it.
- Rejected August 28 cross-crate `SeqStore` ownership port: a dedicated no-std
  crate owned C's stored/prepared sequence records, repeat-offset state and
  fused transition, plus the direct Fast/DFast preparation arena. The isolated
  preparation symbol shrank from 1,671 to 1,072 bytes, and Greedy/Lazy kept its
  existing preparation algorithm. Full validation passed, focused bytes were
  exact, and all 1,065 broad rows with unchanged source inputs were identical;
  the other 30 rows were the two changed Cargo manifests and remained decode-
  verified against C. Lunx level-1/3/5/8/16 medians were `680,620,453`,
  `1,253,936,185`, `3,816,757,998`, `4,771,908,781`, and `7,657,776,362`
  instructions: `-0.061%`, `-0.074%`, `+9.208%`, `+8.655%`, and `-0.096%`.
  Moving the shared type identity and inline repeat methods across crates
  rebuilt the three Greedy/Lazy bodies at only 2,640-3,191 bytes versus the
  retained 3,631-4,329 bytes, but made them much slower. The entire crate and
  ownership move were removed. The exact original hot-symbol addresses,
  hashes, and sizes returned; post-revert L1/L5/L8 were `681,033,138`,
  `3,494,952,646`, and `4,391,784,979`. Candidate artifacts end in
  `cross-crate-seqstore-{1,2,3}.stat`; revert artifacts end in
  `cross-crate-seqstore-revert.stat`; broad artifacts end in
  `after-cross-crate-seqstore.csv`. Keep shared hot record types local. A
  future codegen boundary must use a primitive ABI without changing the Rust
  types or inline methods instantiated inside Greedy/Lazy.
- Latest kept August 28 primitive-ABI `SeqStore` isolation: the new no-std
  `ruzstd-seqstore-codegen` crate owns only Fast/DFast's complete post-match
  preparation transaction. It accepts `[u32; 3]` stored-sequence words and
  returns `[u32; 4]` prepared-sequence words; all shared Rust record and repeat-
  state types and inline methods stay local. Compile-time size/alignment/offset
  assertions prove the narrow owning casts. The 1,087-byte isolated body owns
  C's preallocated literal/sequence arrays, bounded wild copies, numeric repeat
  transition, and initialized-prefix handoff. The three Greedy/Lazy symbols
  retain their exact 3,631/4,090/4,329-byte sizes and identical complete
  instruction-mnemonic hashes. Full strict validation passed, all 1,095 rows
  match the manifest-equivalent matrix, and focused bytes are exact. Lunx
  level-1/3/5/8/16 medians are `681,236,851`, `1,255,079,947`,
  `3,494,938,092`, `4,391,787,263`, and `7,664,899,486` instructions:
  `+0.030%`, `+0.017%`, `-0.0004%`, `-0.000002%`, and `-0.00005%` versus the
  preferred-repeat baseline. Counter artifacts end in
  `primitive-abi-seqstore-{1,2,3}.stat`; broad artifacts end in
  `after-primitive-abi-seqstore.csv`. KEEP this neutral structural boundary as
  the safe separately compiled target for larger Fast/DFast ports; do not move
  shared Rust type identity into it.
- Latest kept August 28 persistent primitive `SeqStore` workspace: normal
  Fast/DFast block encoding now stores the leaf crate's prepared literal and
  sequence allocations in matcher state across blocks, mirroring C's
  persistent `SeqStore_t`. Preparation clears and reuses the primitive buffers,
  and both compressed and raw emission exits recycle them. Target mode retains
  its existing owned conversion, shared local record types do not cross the
  crate boundary, and Greedy/Lazy keep their exact 3,631/4,090/4,329-byte
  generated bodies and instruction-mnemonic hashes. Fast and DFast lifecycle
  tests prove identical backing allocations survive a compressed block and a
  following raw fallback.

  Lunx level-1/3/5/8/16 medians are `680,583,126`, `1,252,665,609`,
  `3,494,756,270`, `4,391,599,355`, and `7,664,109,191` instructions:
  `-0.096%`, `-0.192%`, `-0.005%`, `-0.004%`, and `-0.010%` versus the
  primitive-boundary baseline. Focused bytes are exact. Full validation passed
  703 library tests (698 passed, 5 ignored), 7 integration tests, 4 sequence-
  codegen tests, the Huffman codegen test, both leaf packages, strict Clippy,
  formatting, release builds, and `git diff --check`. All 1,095 normal,
  target-2048, and prepared-CDict rows exactly match the prior matrix. Counter
  artifacts end in `persistent-primitive-seqstore-{1,2,3}.stat`; broad
  artifacts end in `after-persistent-primitive-seqstore.csv`. KEEP this as the
  new baseline.
- Latest kept August 28 DFast store-operation codegen change:
  `dfast_helpers::store_match()` is forced inline into normal and external-
  dictionary DFast matchers, matching C's inline `ZSTD_storeSeqOnly()` state
  update and removing the repeated sequence-push/IP/anchor helper boundary.
  Fast and Greedy/Lazy are untouched. Lunx level-1/3/5/8/16 medians are
  `680,583,390`, `1,242,968,242`, `3,494,756,455`, `4,391,633,868`, and
  `7,664,108,408` instructions: `+0.000039%`, `-0.774%`, `+0.000005%`,
  `+0.000786%`, and `-0.000010%` versus the persistent-workspace baseline.
  Focused bytes and all 1,095 broad rows remain exact. Full strict validation
  passed. Counter artifacts end in `dfast-inline-store-{1,2,3}.stat`; broad
  artifacts end in `after-dfast-inline-store.csv`. KEEP this as the current
  level-3 baseline.
- Rejected August 28 DFast `ZSTD_selectAddr()` candidate-load port: the normal
  DFast search selected a readable source or dummy address for both C
  candidate-validity boundaries and unconditionally loaded before checking
  validity. The unsafe boundary was centralized in `unaligned.rs`; focused
  bytes, full strict validation, and all 1,095 broad rows passed. Lunx measured
  level-1/3/5/8/16 medians of `680,583,202`, `1,317,014,505`,
  `3,591,760,210`, `4,529,796,266`, and `7,664,104,291` instructions. The
  level-3 regression was `5.957%` and level 5/8 regressed
  `2.776%`/`3.146%`; levels 1/16 were neutral. The helper and both call sites
  were removed. Artifacts end in `dfast-selectaddr-{1,2,3}.stat`; keep Rust's
  explicit validity branches.
- Latest kept August 28 DFast raw table-access port: every normal and external-
  dictionary DFast hash-table read/write now crosses the audited
  `dfast_table.rs` boundary. `DFastMatchState::ensure_tables()` establishes
  `1 << log` entries and every caller supplies a hash reduced to the same log;
  debug builds assert the slot while release uses C-equivalent unchecked
  access. Dictionary construction remains checked because it is not hot. The
  main release DFast body shrank from about `0x3504` to `0x3053` bytes. Lunx
  level-1/3/5/8/16 medians are `680,582,604`, `1,122,787,855`,
  `3,494,721,120`, `4,391,634,539`, and `7,664,108,827` instructions. Level 3
  improves `9.669%`; all guard levels are neutral and all focused bytes are
  exact. Validation passed 704 library tests (699 passed, 5 ignored), 7
  integration tests, both codegen crates, strict Clippy, formatting, release
  builds, and all 1,095 normal/target/prepared-CDict rows. Counter artifacts
  end in `dfast-unchecked-table-{1,2,3}.stat`; broad artifacts end in
  `after-dfast-unchecked-table.csv`. KEEP this as the current baseline.
- Latest kept August 28 complete FSE CTable raw-access port: `FSETable`
  emission now uses C-equivalent unchecked symbol-transform and compact-state
  loads after table selection validates symbols. Construction uses the
  normalized-distribution, masked spread-position, and cumulative state-range
  invariants for its raw writes. Debug assertions guard every boundary, and a
  test exhaustively covers every encodable symbol/state pair for all three
  predefined tables plus a mixed normalized table. The retained writer,
  emitter layout, and vector ownership are unchanged. Lunx level-1/3/5/8/16
  medians are `667,947,037`, `1,099,186,757`, `3,469,610,893`,
  `4,369,265,957`, and `7,629,779,483` instructions: improvements of `1.857%`,
  `2.102%`, `0.719%`, `0.509%`, and `0.448%`. Focused bytes and all 1,095 broad
  rows remain exact. Validation passed 705 library tests (700 passed, 5
  ignored), 7 integration tests, both codegen crates, strict Clippy,
  formatting, and release builds. Counter artifacts end in
  `fse-raw-table-{1,2,3}.stat`; broad artifacts end in
  `after-fse-raw-table.csv`. KEEP this as the current baseline. The post-change
  level-3 profile is
  `benchmarks/tmp/profile-c-port-z000033-l3-after-fse-raw-table.perf.data`.
  DFast is now slightly below C in absolute sampled matcher work; remaining
  absolute excess is led by sequence emission (about `512M` Rust versus
  `383M` C instructions per 80 runs) and Huffman stream emission (about `334M`
  versus `215M`). Continue in those retained entropy paths without retrying the
  rejected separate sequence writer/crate layouts.
- Latest kept August 28 isolated Huffman raw-cursor port: the existing
  `ruzstd-huff0-codegen` emitter now uses raw bounded input traversal, its
  complete 256-entry code table, and unaligned overlapping word stores as one
  audited C-style boundary. The outer transaction still reserves all four
  tight stream bounds plus seven initialized padding bytes before any write;
  debug assertions retain the cursor proof. Representation, unroll cadence,
  stream splitting, and caller layout are unchanged. Release `encode_stream`
  shrank from `0x5a6` to `0x29d`, and the four-stream transaction from `0x1d23`
  to `0xf7f`. Lunx level-1/3/5/8/16 medians are `658,967,804`,
  `1,092,749,602`, `3,463,615,192`, `4,363,196,208`, and `7,624,930,884`:
  improvements of `1.344%`, `0.586%`, `0.173%`, `0.139%`, and `0.064%`.
  Focused bytes and all 1,095 broad rows remain exact. Full strict validation
  passed. Counter artifacts end in `huffman-raw-cursor-{1,2,3}.stat`; broad
  artifacts end in `after-huffman-raw-cursor.csv`. KEEP this as the current
  baseline and keep the unsafe cursor inside the isolated crate. The
  post-change profile is
  `benchmarks/tmp/profile-c-port-z000033-l3-after-huffman-raw-cursor.perf.data`.
  Sequence emission remains about `524M` sampled instructions per 80 runs
  versus C's `383M`; audit the retained unified emitter next without recreating
  the rejected separate writer, variant split, or codegen-crate shapes.
- Latest kept August 28 audited raw row-table port: active and attached-CDict
  row lookups plus all row insertions now use one `row_table.rs` boundary.
  `ensure_tables()` establishes the exact `1 << hash_log` table sizes, hash
  bits select a complete row, and masked positions stay within its 16/32/64
  entries; debug assertions preserve those proofs and tests cover the first
  and last rows. `row_match.rs` still forbids unsafe code. The three Lazy
  release bodies shrink from about `0xe2f`/`0xffa`/`0x10e9` bytes to
  `0xa50`/`0xbc2`/`0xc77`. Lunx level-1/3/5/8/16 medians are `658,971,056`,
  `1,092,778,624`, `3,246,428,797`, `4,076,027,746`, and `7,625,098,399`:
  neutral changes of `+0.000493%`, `+0.002656%`, and `+0.002197%` at 1/3/16,
  with material `-6.271%` and `-6.582%` wins at 5/8. Focused bytes and all
  1,095 broad rows are exact; full strict validation passes. Counters end in
  `row-raw-table-{1,2,3}.stat`; broad artifacts end in
  `after-row-raw-table.csv`. KEEP this as the current baseline and confine the
  unchecked accesses to the audited helper.
- Latest kept August 28 Greedy/Lazy mode specialization: `DEPTH`, `EXT_DICT`,
  and `ATTACHED_DICT` are const parameters of the complete parser, matching
  C's separately generated greedy/lazy/lazy2 and
  noDict/extDict/dictMatchState bodies. The search finder receives the attached
  constant, and repeat matching/backward extension receive the extDict
  constant, so normal compression carries none of those runtime branches. The
  normal row bodies shrink from about `0xa50`/`0xbc2`/`0xc77` to
  `0x7b9`/`0x803`/`0x886`. Lunx level-1/3/5/8/16 medians are `658,992,365`,
  `1,092,842,400`, `3,165,131,463`, `4,081,241,793`, and `7,626,305,663`:
  noise-level changes at 1/3/16, a `2.504%` level-5 win, and a `0.128%`
  level-8 movement inside the `0.2%` gate. Focused bytes and all 1,095 broad
  rows are exact; full strict validation passes. Counters end in
  `greedy-mode-depth-specialization-{1,2,3}.stat`; broad artifacts end in
  `after-greedy-mode-depth-specialization.csv`. KEEP this as the current
  cross-level baseline before the following fused-row checkpoint.
- Latest kept August 28 isolated fused no-dictionary row-search port: the new
  no-std `ruzstd-row-codegen` crate emits C's nine `(minMatch,rowLog)`
  specializations behind a primitive slice/scalar ABI. Each outlined generated
  function now owns the cached row updates, candidate tag scan and source
  prefetch, current-position insertion, and match-count continuation. The
  candidate array uses `MaybeUninit` and reads exactly its written prefix,
  matching C without the 256-byte per-search clearing that caused the earlier
  initialized safe-buffer experiment's 15% regression. Audited raw accesses
  stay inside the isolated crate with debug assertions for table, row, source,
  and prefix invariants; attached-dictionary search and dictionary table loads
  retain the existing checked/local path. The nine release functions are about
  `0x56c`-`0x5de` bytes versus C's focused `0x665` function.

  Lunx level-1/3/5/8/16 medians are `658,975,106`, `1,092,771,882`,
  `3,115,660,190`, `3,956,264,774`, and `7,623,313,260` instructions. The
  target levels 5/8 improve `1.563%`/`3.062%`; levels 1/3/16 improve by
  noise-level `0.003%`/`0.007%`/`0.039%`. Focused bytes remain exact. Full
  validation passed 706 library tests (701 passed, 5 ignored), 7 integration
  tests, all three codegen crates, workspace all-target Clippy with warnings
  denied, formatting, release tool builds, and `git diff --check`. All 1,095
  broad rows decode and compare; all 1,065 unchanged-input rows are byte-exact
  against the preceding checkpoint, while the 14 normal and 8+8 target/CDict
  Cargo-manifest rows reflect the new workspace member/dependency. Counter
  artifacts end in `isolated-fused-row-search-{1,2,3}.stat`; broad artifacts
  end in `after-isolated-fused-row-search.csv`. KEEP this as the current
  fused-row baseline before the following dispatch checkpoint.
- Latest kept August 28 block-level row-function selection: the Greedy/Lazy
  parser now asks `ruzstd-row-codegen` for the exact `(minMatch,rowLog)`
  function once while constructing its per-block search context, then calls
  that pointer directly for every search. This matches C's generated
  `searchMax` selection and removes the former runtime match and wrapper from
  each hot search. The standalone `find_best_match_no_dict` symbol disappears
  from the release binary; the nine generated bodies remain stable at about
  `0x546`-`0x5de` bytes.

  Lunx level-1/3/5/8/16 medians are `658,975,226`, `1,092,771,604`,
  `2,688,938,934`, `3,463,273,935`, and `7,623,520,391` instructions. Levels
  5/8 improve `13.696%`/`12.461%` versus the fused-row checkpoint; levels
  1/3/16 move only `+0.00002%`/`-0.00003%`/`+0.0027%`. Focused bytes and all
  1,095 broad byte/reference rows are exact against the preceding checkpoint.
  Full validation passed 706 library tests (701 passed, 5 ignored), 7
  integration tests, all three codegen crates, workspace all-target Clippy with
  warnings denied, formatting, release builds, and `git diff --check`.
  Counters end in `row-block-selection-{1,2,3}.stat`; broad artifacts end in
  `after-row-block-selection.csv`. KEEP this as the current cross-level
  baseline before the following direct-repeat checkpoint.
- Latest kept August 28 direct no-dictionary repeat continuation: every
  Greedy/Lazy main-loop, lazy-depth, and immediate-repeat probe now takes a
  const-specialized inline prefix path. It uses the parser's already-proven
  active-offset and block-limit invariants, performs C's direct four-byte
  compare, and continues through the no-dictionary word counter without the
  shared external-dictionary-aware `Option` boundary. Ext-dictionary and
  attached-dictionary paths retain their existing bounds routine. Dedicated
  tests cover a full match to the block end, zero offset, and first-word
  mismatch. The standalone `rep_match_length` symbol disappears; generated
  parser bodies grow to `0x7df`-`0xf5e`, matching C's deliberate inlining.

  Lunx level-1/3/5/8/16 medians are `658,974,704`, `1,092,767,896`,
  `2,494,974,503`, `3,173,938,483`, and `7,623,313,161` instructions. Levels
  5/8 improve another `7.213%`/`8.354%`; levels 1/3/16 are neutral. Focused
  bytes and all 1,095 broad byte/reference rows remain exact. Full validation
  passed 708 library tests (703 passed, 5 ignored), 7 integration tests, all
  three codegen crates, workspace all-target Clippy with warnings denied,
  formatting, release builds, and `git diff --check`. Counters end in
  `direct-no-dict-rep-{1,2,3}.stat`; broad artifacts end in
  `after-direct-no-dict-rep.csv`. KEEP this as the current cross-level
  baseline. The remaining focused Rust/C instruction gaps are about `13.72%`
  at level 5 and `9.96%` at level 8; refresh attribution before the next port.
- Rejected August 28 cached-FSE-state follow-up: a borrowed C
  `FSE_CState_t`-style state cached the compact table and transform pointers
  once per LL/ML/OF stream without changing the retained batched writer. It
  passed strict validation and all 1,095 broad byte rows, but Lunx measured
  level-1/3/5/8/16 medians of `659,140,831`, `1,093,037,801`,
  `2,495,232,615`, `3,174,167,606`, and `7,623,589,137` instructions. Those
  are small regressions of `0.0252%`, `0.0247%`, `0.0103%`, `0.0072%`, and
  `0.0036%` versus the retained direct-repeat baseline. It was reverted. Keep
  the existing table references and numeric states; pointer caching alone does
  not remove enough work from the all-table sequence loop.
- Latest kept August 28 shared SeqStore handoff: Greedy/Lazy and the optimal
  paths consuming `GreedyBlockOutput` now reuse the complete isolated
  primitive-ABI transaction already used by Fast/DFast. This removes their
  duplicate literal-copy, repeat-resolution, prepared-record, and tail-literal
  loop. `PreparedBlock` allocations transfer to and from primitive word vectors
  without copying under compile-time layout assertions; a regression test
  verifies allocation identity and record contents. Full validation passed 709
  library tests (704 passed, 5 ignored), 7 integration tests, all codegen
  crates, workspace strict Clippy, formatting, release builds, and all 1,095
  broad rows exactly. Lunx level-1/3/5/8/16 medians are `659,199,799`,
  `1,093,179,538`, `2,476,441,432`, `3,160,186,320`, and `7,605,769,549`:
  `+0.034%`, `+0.038%`, `-0.743%`, `-0.433%`, and `-0.230%` versus the
  direct-repeat baseline. Its corrected recommendation is KEEP. Counters end
  in `greedy-seqstore-handoff-{1,2,3}.stat`; broad artifacts end in
  `greedy-seqstore-handoff.csv`.
- Rejected August 28 whole-byte BitWriter flush: the overflow path replaced
  repeated remainder-byte pushes with one bounded runtime-length slice append,
  matching C's single flush transaction without changing writer ownership.
  Strict validation passed and all unchanged-input broad rows were exact, but
  Lunx level-1/3/5/8/16 medians regressed by `0.139%`, `0.121%`, `0.058%`,
  `0.047%`, and `0.043%`. It recommended REVERT and the change was removed.
  Keep the compiler-specialized small byte-push loop. Artifacts end in
  `bitwriter-whole-byte-flush-{1,2,3}.stat` and
  `bitwriter-whole-byte-flush.csv`.
- Kept August 28 exact sequence-prefix fill: entropy-sequence materialization
  now reserves the complete prefix, initializes its spare capacity directly,
  and commits the vector length once. This keeps the existing 12-byte record
  and all table/emission code intact while removing repeated vector length
  maintenance. A regression test proves allocation identity and mixed
  generic/preencoded fields. Full strict validation and all 1,095 broad rows
  passed. Lunx level-1/3/5/8/16 medians are `656,070,543`, `1,087,655,516`,
  `2,470,462,300`, `3,154,652,394`, and `7,599,718,995`, improvements of
  `0.475%`, `0.505%`, `0.241%`, `0.175%`, and `0.080%` versus the shared
  SeqStore checkpoint. Recommendation: KEEP. Counters end in
  `sequence-prefix-fill-{1,2,3}.stat`; broad artifacts end in
  `sequence-prefix-fill.csv`.
- Kept August 28 complete no-dictionary row-parser transaction: normal
  Greedy/Lazy/Lazy2 row parsing now lives with the generated row matcher in
  `ruzstd-row-codegen`. One primitive transaction owns the block loop, row
  search, repeat probes, lazy decisions, backward extension, bounded sequence
  writes, repeat history, and row state. The 27 `(depth,minMatch,rowLog)`
  functions are selected once per block, eliminating the hot cross-module
  search ABI. Hash-chain, binary-tree, ext-dictionary, and attached-dictionary
  paths are unchanged. A direct oracle compares old and new block output and
  complete matcher state for every depth. Full strict validation and all 1,095
  broad rows passed exactly. Lunx level-1/3/5/8/16 medians are `656,068,549`,
  `1,087,655,341`, `2,117,901,080`, `2,699,936,389`, and `7,599,915,925`:
  changes of `-0.0003%`, `-0.00002%`, `-14.271%`, `-14.414%`, and `+0.0026%`.
  Recommendation: KEEP. Counters end in
  `row-parser-transaction-{1,2,3}.stat`; broad artifacts end in
  `row-parser-transaction.csv`.
- Kept August 28 complete isolated DFast block transaction: the new no-std
  `ruzstd-dfast-codegen` crate owns the four `minMatch`-specialized hash probes,
  repeat handling, long/short match choice, backward extension, complementary
  inserts, immediate repcodes, bounded sequence writes, and repeat-history
  result behind one primitive ABI. The caller reserves and commits the proven
  initialized sequence prefix once. A direct oracle compares the old and new
  implementations for `minMatch` 4–7, including both tables and all output
  state. Full strict validation passed; all 1,095 broad rows decode and all
  1,065 unchanged-input rows are byte-identical. Lunx level-1/3/5/8/16 medians
  are `656,033,681`, `1,068,201,844`, `2,117,751,776`, `2,698,973,209`, and
  `7,593,707,962`, improvements of `0.0053%`, `1.7886%`, `0.0071%`, `0.0357%`,
  and `0.0817%`. Recommendation: KEEP. Counters end in
  `dfast-block-codegen-{1,2,3}.stat`; broad artifacts end in
  `dfast-block-codegen.csv`.
- Kept August 28 complete isolated Fast block transaction: the new no-std
  `ruzstd-fast-codegen` crate owns all four `minMatch`-specialized hash probes,
  table updates, direct repeat probes, skip progression, backward extension,
  C fill-after-match, immediate repcodes, bounded sequence writes, and repeat
  result behind one primitive ABI. A direct oracle compares every output and
  hash-state field with the former local implementation. Full strict
  validation passed; all 1,095 broad rows decode and all 1,065 unchanged-input
  rows are byte-identical. Lunx level-1/3/5/8/16 medians are `635,430,077`,
  `1,066,202,536`, `2,115,728,625`, `2,697,913,377`, and `7,595,945,021`:
  changes of `-3.1406%`, `-0.1872%`, `-0.0955%`, `-0.0393%`, and `+0.0295%`.
  Recommendation: KEEP. Counters end in
  `fast-block-codegen-{1,2,3}.stat`; broad artifacts end in
  `fast-block-codegen.csv`. Fresh paired profiles show both Rust Fast and
  DFast matcher transactions are already cheaper than C; the remaining
  level-1/3 gaps of about `+26.9%`/`+26.4%` are below the matcher in sequence
  preparation, table selection, and entropy emission.
- Rejected August 28 C-style `ZSTD_seqToCodes()` arrays: normal compression
  materialized compact LL/OF/ML arrays once and reused them for C table
  selection and final FSE emission, preserving the 12-byte sequence value and
  the isolated parser crates. A direct 280-sequence bit oracle, full strict
  validation, release builds, and all 1,095 exact broad rows passed. Lunx
  level-1/3/5/8/16 medians were `643,013,886`, `1,083,545,950`,
  `2,134,196,342`, `2,715,217,975`, and `7,616,394,471`, regressions of
  `1.1935%`, `1.6267%`, `0.8729%`, `0.6414%`, and `0.2692%`. Recommendation:
  REVERT. Artifacts end in `seq-to-codes-arrays-{1,2,3}.stat`; broad artifacts
  end in `seq-to-codes-arrays.csv`. Do not stage another per-block code-array
  allocation; a future faithful sequence port must own more of the transaction
  or use persistent C-style SeqStore workspace.
- Kept August 28 direct C PreparedSequence entropy path: C-port block emission
  now consumes the existing 16-byte prepared records directly for history
  replay, code counting/table selection, and reverse FSE emission. It removes
  the duplicate per-block 12-byte generic `Sequence` allocation and copy while
  leaving the generic compressor and exact-mode search unchanged. A stateful
  two-block oracle proves exact bytes, offset history, fresh table state, and
  following-block repeat behavior. Full strict validation passed, including
  714 library tests, 7 integration tests, all codegen crates, and all 1,095
  exact broad rows. Lunx level-1/3/5/8/16 medians are `627,787,929`,
  `1,055,302,323`, `2,102,770,693`, `2,684,909,809`, and `7,579,348,047`,
  improvements of `1.2027%`, `1.0223%`, `0.6125%`, `0.4820%`, and `0.2185%`.
  Recommendation: KEEP. Counters end in
  `direct-prepared-entropy-{1,2,3}.stat`; broad artifacts end in
  `direct-prepared-entropy.csv`.
- Kept August 28 explicit isolated Huffman batch unroll: every symbol addition
  in C's complete `kUnroll` 5–9 family is now spelled out inside
  `ruzstd-huff0-codegen`. Full batches no longer use the runtime reverse-symbol
  loop; only the one variable remainder does. Paired-container ordering, the
  one-reservation raw cursor, low-bit tuple table, and safe overlapping stores
  are unchanged. The table-log reference oracle, full strict validation, and
  all 1,095 broad rows pass exactly. Lunx level-1/3/5/8/16 medians are
  `596,490,351`, `1,033,476,436`, `2,082,480,218`, `2,664,241,544`, and
  `7,563,948,957`, improvements of `4.9854%`, `2.0682%`, `0.9649%`, `0.7698%`,
  and `0.2032%`. Recommendation: KEEP. Counters end in
  `huff-explicit-unroll-{1,2,3}.stat`; broad artifacts end in
  `huff-explicit-unroll.csv`. The remaining paired whole-compressor gaps are
  now about `+19.2%` at level 1 and `+22.5%` at level 3.
- Rejected August 28 compact coded-sequence port: an encoder-private 12-byte
  record packed precomputed LL/ML codes into the high bytes of the bounded
  values and threaded that representation through normal table selection,
  estimation, superblock handling, table construction, and emission. This was
  the complete C `seqToCodes()` boundary without the earlier 16-byte record.
  Exhaustive metadata tests, 706 library tests, 7 integration tests, strict
  validation, and all 1,095 broad byte rows passed before measurement. Lunx
  medians at levels 1/3/5/8/16 were `659,025,297`, `1,092,681,427`,
  `3,560,721,634`, `4,500,743,462`, and `7,652,169,727`, changing the retained
  baseline by `+0.009%`, `-0.006%`, `+2.804%`, `+3.152%`, and `+0.357%`.
  Focused bytes were exact, but the material Greedy/Lazy regressions required
  REVERT. Artifacts end in `compact-coded-sequences-{1,2,3}.stat`. A compact
  shared record alone does not isolate generated-code layout; do not retry
  stored codes without a stronger isolation mechanism.
- Rejected August 28 C-width Huffman workspace port: the 192-entry sort rank
  table used C's 4-byte `u16` cursor pair instead of a 16-byte Rust pair, and
  one audited raw cursor covered the fixed node workspace after allocation.
  A checked implementation matched every node field across 256 generated
  histograms; the hot stack frame fell from `0xc18` to `0x328`, full strict
  validation passed, and all 1,095 rows were exact. Lunx medians at levels
  1/3/5/8/16 were `653,952,526`, `1,076,384,029`, `3,543,925,286`,
  `4,485,133,003`, and `7,566,847,705`: `-0.761%`, `-1.498%`, `+2.319%`,
  `+2.795%`, and `-0.762%`. Despite real Fast/DFast/optimal gains, the material
  Greedy/Lazy regressions required REVERT. Artifacts end in
  `compact-huffman-workspace-{1,2,3}.stat`. Do not retry this shared workspace
  shape until code placement can be isolated from Greedy/Lazy.
- Rejected August 28 Greedy/Lazy source-loop alignment: binary comparison
  proved the compact Huffman rank diagnostic merely shifted the unchanged row
  matcher and three lazy-parser bodies by `0x20`. Five x86-64 `.p2align 6`
  boundaries then kept their symbol addresses and sizes identical across that
  perturbation, but the selected alignment was itself unfavorable. Full
  validation and all 1,095 broad rows passed with exact bytes. Lunx medians at
  levels 1/3/5/8/16 were `658,973,918`, `1,092,750,013`, `3,503,365,988`,
  `4,412,500,522`, and `7,624,931,297`: neutral at 1/3/16 but `+1.148%` and
  `+1.130%` at 5/8. The helper and all alignment sites were removed. Artifacts
  end in `greedy-hot-loop-alignment-{1,2,3}.stat`; broad artifacts end in
  `after-greedy-hot-loop-alignment.csv`. Preserve the observed favorable
  layout through a real compilation/link boundary instead of blanket loop
  alignment.
- Rejected August 28 isolated full Huffman table builder: the existing Huffman
  codegen crate temporarily owned compact C-width sorting, raw node-tree
  construction, height limiting, and canonical-code extraction through opaque
  reusable scratch. A checked builder matched it across 256 generated
  histograms and table logs 4-11. The external `0xc43`-byte builder preserved
  the exact sizes and retained cache-line offsets of the row updater and all
  three lazy bodies, so this was a real test of the compilation-boundary
  hypothesis. Full validation and all 1,095 broad rows passed with exact bytes.
  Lunx level-1/3/5/8/16 medians were `658,446,778`, `1,091,811,756`,
  `3,559,887,076`, `4,500,775,430`, and `7,633,557,690`: `-0.079%`,
  `-0.086%`, `+2.780%`, `+3.153%`, and `+0.113%`. The whole module and
  integration were removed. Artifacts end in
  `isolated-huffman-table-builder-{1,2,3}.stat`; broad artifacts end in
  `after-isolated-huffman-table-builder.csv`. Stable hot-symbol alignment was
  insufficient; do not retry this wholesale cross-crate boundary without a
  causal profile for the remaining Greedy/Lazy regression.
- Rejected precursor from the same checkpoint: Fast source-sequence allocation
  reuse plus forced Fast/DFast store inlining improved level 3 by `0.777%` but
  regressed level 1 by a repeatable `0.338%`; other levels were neutral and all
  bytes were exact. The Fast matcher grew from 9,387 to 12,261 bytes. The Fast
  state/recycling/inlining portion was removed, after which the isolated DFast
  win reproduced. Rejected counters end in
  `fast-source-store-inline-{1,2,3}.stat`; broad artifacts end in
  `after-fast-source-store-inline.csv`.
- Rejected isolated Fast source-sequence allocation reuse: with forced Fast
  inlining absent, `FastMatchState` retained only its `Vec<StoredSequence>`
  across normal and external-dictionary preparations. Allocation-identity
  tests, 704 library tests, full strict validation, focused bytes, and all
  1,095 broad rows passed. Lunx level-1/3/5/8/16 medians were `687,923,574`,
  `1,242,969,376`, `3,494,719,667`, `4,391,633,788`, and `7,664,238,379`.
  Level 1 regressed tightly by `1.079%`; other levels were neutral. The state
  and recycling path were removed. Counters end in
  `fast-sequence-reuse-{1,2,3}.stat`; broad artifacts end in
  `after-fast-sequence-reuse.csv`. Keep Fast's fresh per-block source-sequence
  vector.
- Rejected August 28 persistent packed Huffman-code representation: the whole
  Huffman transaction replaced its 8-byte `(u32, u8)` tuple with one
  transparent 8-byte word used directly by construction, metrics, weight
  serialization, compact emission, and the isolated four-stream crate. This
  removed the earlier exact-C experiment's per-block conversion and added no
  staging allocation. Exact layout/reference coverage, focused bytes, full
  strict validation, and all 1,095 broad rows passed. Lunx level-1/3/5/8/16
  medians were `699,259,846`, `1,261,853,476`, `3,512,800,565`,
  `4,409,618,512`, and `7,700,086,840`: regressions of `2.744%`, `1.519%`,
  `0.516%`, `0.410%`, and `0.469%`. The representation was removed and the
  isolated emitter returned from 8,004 to 7,459 bytes. Counter artifacts end
  in `packed-huffman-code-{1,2,3}.stat`; broad artifacts end in
  `after-packed-huffman-code.csv`. Keep the tuple representation.
- Rejected August 28 four-stream Huffman output boundary: fresh paired profiles
  put Rust's Fast matcher below C in absolute instructions but Huffman emission
  near `704M` Rust versus `320M` C for 80 runs. The candidate ported
  `HUF_compress4X_usingCTable_internal()` as one aligned destination
  transaction with a directly written six-byte jump table, while preserving
  the retained single-container stream loop. A table-log 1-11 plus compact-
  table reference test proved exact output, all 1,095 broad rows were unchanged,
  and full validation passed. Lunx measured level-1/3/5/8/16 medians of
  `724,110,786`, `1,299,084,797`, `3,632,255,920`, `4,569,425,861`, and
  `7,689,874,882` instructions: `-0.939%`, `-0.616%`, `+2.518%`, `+2.941%`,
  and `-0.243%`. The boundary was removed; post-revert levels 1/5/8 returned
  to `730,975,990`, `3,543,053,399`, and `4,438,893,929` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-huffman-four-stream-boundary.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-huffman-four-stream-boundary.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-huffman-four-stream-boundary.csv`.
  Keep four independent aligned stream calls; outer-boundary grouping alone
  does not isolate Greedy/Lazy code placement.
- Rejected August 28 fused sequence-code statistics: normal table selection,
  post-split estimation, and superblock table construction shared one pass for
  LL/ML/OF counts, extra-bit totals, and terminal codes without changing the
  sequence record. All 511 normal, 292 prepared-CDict, and 292 target-2048
  rows were byte-identical, and full validation passed. Lunx measured
  level-1/3/5/8/16 medians of `730,940,662`, `1,297,499,013`,
  `3,640,048,235`, `4,577,120,977`, and `7,696,608,802` instructions:
  `-0.005%`, `-0.738%`, `+2.738%`, `+3.114%`, and `-0.155%`. The shared type
  and reuse paths were removed; post-revert levels 1/5/8 returned to
  `730,973,862`, `3,543,045,884`, and `4,438,894,367` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fused-sequence-stats.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-fused-sequence-stats.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-fused-sequence-stats.csv`.
  Retain independent normal table-selection scans and the estimator-local
  materialize-then-count pass; the shared code shape still regressed
  Greedy/Lazy materially.
- Rejected August 28 C-shaped Huffman remainder join: a retained level-1
  instruction profile placed `25.17%` of Rust work in the fixed-table stream
  closure, so the candidate replaced `rchunks()` with C's join-remainder then
  exact-`kUnroll` cadence while retaining the safe overlapping stores and all
  existing table/sequence layouts. All 511 normal, 292 prepared-CDict, and 292
  target-2048 rows were exact; 636 library tests, 7 integration tests, strict
  Clippy, formatting, and release builds passed. Lunx measured level-1/3/5/8/16
  medians of `771,231,436`, `1,336,256,618`, `3,570,200,508`,
  `4,466,460,984`, and `7,728,090,602` instructions: regressions of `5.507%`,
  `2.227%`, `0.766%`, `0.621%`, and `0.253%`. The loop was removed;
  post-revert levels 1/5/8 returned to `730,974,191`, `3,543,045,946`, and
  `4,438,894,296` with exact bytes. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-c-huffman-remainder-join.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-c-huffman-remainder-join.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-c-huffman-remainder-join.csv`.
  Keep `rchunks()`: C's loop relies on compile-time `kUnroll`, while Rust's
  runtime-width remainder/exact-loop form generates substantially worse code.
- Rejected August 28 compact sequence-code view: normal compression represented
  C's separate `llCode`/`mlCode`/`ofCode` arrays with one three-byte record,
  preserving the 12-byte entropy sequence and every parser record. The forward
  code pass fed non-exact table selection and reverse FSE emission, while safe
  256-entry LL/ML bit tables recovered C's low-bit extra values. Exhaustive
  equivalence coverage, 637 library tests, 7 integration tests, strict Clippy,
  formatting, release builds, and all 1,095 broad byte rows passed. Lunx
  measured level-1/3/5/8/16 medians of `738,957,269`, `1,323,508,161`,
  `3,656,251,400`, `4,591,748,913`, and `7,718,439,297` instructions:
  regressions of `1.092%`, `1.252%`, `3.195%`, `3.443%`, and `0.128%`. The
  complete code view was removed; post-revert levels 1/5/8 returned to
  `730,975,540`, `3,543,052,962`, and `4,438,896,398` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-compact-sequence-code-view.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-compact-sequence-code-view.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-compact-sequence-code-view.csv`.
  Do not retry separately allocated code arrays: their allocation, parallel
  traversal, and code surface outweigh repeated conversion even without a
  larger parser record.
- Rejected August 28 compact FSE symbol transform: the sequence FSE table's
  12-byte transform was narrowed in place to 8 bytes with bounded `i16`
  probability and `deltaFindState` fields, preserving the existing allocation
  and call graph. All 1,095 broad rows were exact; 637 library tests, 7
  integration tests, strict Clippy, formatting, and release builds passed.
  Lunx measured level-1/3/5/8/16 medians of `730,324,966`, `1,297,120,681`,
  `3,639,699,436`, `4,576,949,891`, and `7,707,862,341` instructions: changes
  of `-0.089%`, `-0.767%`, `+2.728%`, `+3.110%`, and `-0.009%`. The compact
  fields and footprint test were removed; post-revert levels 1/5/8 returned to
  `730,974,643`, `3,543,053,522`, and `4,438,895,870` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-compact-fse-symbol-transform.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-compact-fse-symbol-transform.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-compact-fse-symbol-transform.csv`.
  Keep the current `i32` fields. C's smaller transform comes from omitting the
  probability after NCount serialization, so any future attempt needs that
  architectural split or real codegen isolation rather than field packing.
- Rejected August 28 pre-serialized NCount/FSE-table separation: this port
  serialized normalized counts before construction, stored the exact bytes
  separately, and reduced the hot transform to C's two 32-bit words. Cost
  selection recovered normalized magnitudes from `deltaNbBits`, while the
  serialized bytes preserved the `-1`/`1` distinction. Dedicated layout and
  header tests, 638 library tests, 7 integration tests, strict Clippy,
  formatting, release builds, and all 1,095 broad rows passed. Lunx measured
  level-1/3/5/8/16 medians of `731,314,837`, `1,311,365,543`, `3,547,231,468`,
  `4,440,781,615`, and `7,731,937,466` instructions: regressions of `0.047%`,
  `0.323%`, `0.118%`, `0.042%`, and `0.303%`. The candidate was removed;
  post-revert levels 1/5/8 returned to `730,975,624`, `3,543,053,214`, and
  `4,438,903,481` with exact bytes. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-preserialized-fse-ncount.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-preserialized-fse-ncount.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-preserialized-fse-ncount.csv`.
  Do not retain serialized header storage in `FSETable`. A future faithful
  split must return transient header bytes alongside the final table and
  consume them before repeat-history storage without widening shared modes.
- Rejected August 28 transient `BuiltFSETable` lifecycle: the broader follow-up
  moved the existing normalized-count vector through selection, estimation,
  sequence/Huffman fresh-table emission, and superblock handling, then
  discarded it before installing the exact two-word C transform in repeat
  history. No count copy or additional probability allocation was introduced.
  A lifecycle test proved exact header bytes, transform size, and post-discard
  cost recovery. All 1,095 broad rows, 637 library tests, 7 integration tests,
  strict Clippy, formatting, release builds, and diff checks passed. Lunx
  measured level-1/3/5/8/16 medians of `730,501,920`, `1,307,895,897`,
  `3,640,700,873`, `4,575,984,182`, and `7,709,373,399` instructions: changes
  of `-0.065%`, `+0.058%`, `+2.756%`, `+3.088%`, and `+0.010%`. The complete
  cross-module API was removed; post-revert levels 1/5/8 returned to
  `730,975,343`, `3,543,044,405`, and `4,438,903,641` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-transient-fse-build-result.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-transient-fse-build-result.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-transient-fse-build-result.csv`.
  This rules out both obvious compact-table ownership variants. Do not widen
  the shared mode with transient counts again; require a genuine codegen or
  compilation-unit boundary before revisiting the exact C transform.
- Rejected August 28 C no-low-probability FSE spread: the dedicated
  `FSE_buildCTable_wksp()` branch built an ordered spread buffer and populated
  two table symbols per iteration from one safe allocation, while preserving
  the general `-1` low-probability path. A reference test proved exact state
  tables for balanced, sparse, and uneven counts. All 1,095 broad rows, 637
  library tests, 7 integration tests, strict Clippy, formatting, release
  builds, and diff checks passed. Lunx measured level-1/3/5/8/16 medians of
  `731,424,865`, `1,311,279,392`, `3,547,463,149`, `4,442,243,745`, and
  `7,716,142,370` instructions: regressions of `0.062%`, `0.316%`, `0.124%`,
  `0.075%`, and `0.098%`. The branch and test were removed; post-revert levels
  1/5/8 returned to `730,975,706`, `3,543,043,658`, and `4,438,895,910` with
  exact bytes. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-c-fse-no-lowprob-spread.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-c-fse-no-lowprob-spread.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-c-fse-no-lowprob-spread.csv`.
  Keep the general walk. C's benefit depends on caller-owned workspace and
  unchecked overlapping fills; safe owning scratch does not reproduce it.
- Rejected August 28 dedicated aligned sequence writer: the whole sequence
  stream used one safe pre-sized `u64` container boundary with overlapping
  word stores, mirroring C's `BIT_CStream_t`. A 1,027-sequence reference test
  proved byte equivalence, mixed/RLE coverage passed, and all 511 broad rows
  were unchanged. Luna measured level-1/3/5/8/16 medians of `737,906,778`,
  `1,320,085,804`, `3,879,937,733`, `4,832,461,996`, and `7,725,579,125`
  instructions, regressions of `0.948%`, `0.990%`, `9.508%`, `8.866%`, and
  `0.220%`. The writer and test were removed; post-revert levels 1/5/8
  returned within `0.0003%` of retained. Keep the existing batched
  `BitWriter` path. Rejected broad artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-sequence-aligned-writer.csv`.
- Rejected August 28 C high-bit LL/ML formulas: large length-code range
  matches were replaced with C's `highbit` and base subtraction, with
  exhaustive full-range tests and exact bytes across all 511 broad rows.
  Luna still measured level-1/3/5/8/16 medians of `732,697,611`,
  `1,310,245,871`, `3,868,153,932`, `4,822,182,961`, and `7,762,475,753`,
  regressions of `0.236%`, `0.237%`, `9.176%`, `8.635%`, and `0.699%`.
  The formulas were removed. Even this small shared rewrite materially moved
  Greedy/Lazy generated code, so retain the piecewise conversion until codegen
  units or linker layout can be isolated. Rejected broad artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-c-length-code-formulas.csv`.
- Rejected August 28 stored sequence-code boundary: a 16-byte entropy-ready
  record carried LL/ML/OF plus their three codes from one C-style
  `ZSTD_seqToCodes()` conversion. Table selection, estimation, superblock
  handling, repeat/RLE checks, and emission used the stored values, with safe
  256-entry LL/ML extra-bit tables. All 511 normal, 292 prepared-CDict, and
  292 target-2048 rows remained exact and decode-verified. Luna measured
  level-1/3/5/8/16 medians of `717,688,938`, `1,286,820,812`,
  `3,618,345,406`, `4,556,628,387`, and `7,702,517,562` instructions. The
  first two improved `1.817%`/`1.555%`, but levels 5/8 regressed
  `2.125%`/`2.652%`, so the entire representation was removed. Post-revert
  levels 1/5/8 returned to `730,976,109`, `3,543,053,555`, and
  `4,438,894,650`. Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-stored-sequence-codes.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-stored-sequence-codes.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-stored-sequence-codes.csv`.
  The conversion gain is real for Fast/DFast, but do not expand the shared
  Greedy/Lazy sequence record without stronger code- and data-layout isolation.
- Rejected August 28 shared Fast match continuation: the three no-dictionary
  hit sites converged on one C-style `_offset`/`_match` tail. Tests and all
  511 normal, 292 prepared-CDict, and 292 target-2048 rows remained exact, and
  the active Fast symbol shrank from 3,040 to 2,648 bytes. Luna nevertheless
  measured level-1/3/5/8/16 medians of `742,745,041`, `1,306,995,768`,
  `3,639,903,439`, `4,576,942,006`, and `7,699,065,699` instructions:
  changes of `+1.610%`, `-0.011%`, `+2.734%`, `+3.110%`, and `-0.124%`.
  The merge was removed; post-revert levels 1/5/8 returned to `730,974,336`,
  `3,543,055,559`, and `4,438,894,911` with exact expected bytes. Rejected
  artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fast-shared-match.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-fast-shared-match.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-fast-shared-match.csv`.
  Preserve the duplicated Rust hit tails; the smaller symbol did not offset
  the merged live-state and code-layout cost.
- Rejected August 28 bounded-short FSE normalization workspace: normalization,
  its slow fallback, sequence and Huffman table construction, dictionary-table
  conversion, transforms, and NCount consumers were ported from heap
  `Vec<i32>` distributions to a safe 514-byte value containing C's bounded
  256-entry `short` alphabet and active length. All 511 normal, 292 prepared-
  CDict, and 292 target-2048 rows remained exact and decode-verified, and full
  validation passed. Luna measured level-1/3/5/8/16 medians of `729,082,780`,
  `1,307,963,753`, `3,865,683,767`, `4,821,136,450`, and `7,719,473,803`
  instructions: `-0.259%`, `+0.063%`, `+9.106%`, `+8.611%`, and `+0.141%`.
  The representation was removed; post-revert levels 1/5/8 returned to
  `730,975,348`, `3,543,045,680`, and `4,438,893,887` with exact bytes.
  Rejected artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fse-short-normalization.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-fse-short-normalization.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-fse-short-normalization.csv`.
  Keep compact heap vectors. C's workspace should only be revisited as
  caller-owned storage with real code/data-layout isolation; a safe initialized
  full-alphabet return value helps Fast but costs about 9% at Greedy/Lazy.
- Rejected August 28 caller-owned sequence-FSE normalization workspace: the
  follow-up stored one initialized 256-entry C `short` workspace behind shared
  `FseTables`, used it only for sequence normalization/table construction, and
  preserved both Huffman-weight FSE and the owning `FSETable` layout. All 511
  normal, 292 prepared-CDict, and 292 target-2048 rows remained exact, and full
  validation passed. Luna measured level-1/3/5/8/16 medians of `731,115,297`,
  `1,308,199,224`, `3,641,200,083`, `4,578,993,644`, and `7,710,757,638`
  instructions: `+0.019%`, `+0.081%`, `+2.770%`, `+3.156%`, and `+0.028%`.
  The candidate was removed; post-revert levels 5/8 returned to
  `3,543,044,044` and `4,438,903,406` with exact expected bytes. Artifacts:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fse-caller-workspace.csv`,
  `benchmarks/tmp/prepared-cdict-levels-1-3-5-8-api-after-fse-caller-workspace.csv`,
  and
  `benchmarks/tmp/target-2048-levels-1-3-5-8-api-after-fse-caller-workspace.csv`.
  Retain the compact owning `Vec<i32>` path: caller ownership alone did not
  isolate the Greedy/Lazy generated-code cost.
- A fresh representative CPU sweep shows that the remaining performance gap
  is concentrated at low levels. On `corpus_z000033` for 20 runs, Rust used
  `+80.83%`, `+66.19%`, `+70.02%`, and `+61.60%` instructions versus C at
  levels 1, 3, 5, and 8. Rust used fewer instructions at levels 16, 19, and 22
  by `10.32%`, `8.96%`, and `12.71%`. Sizes remained at or slightly below C.
  Whole-compressor CPU parity is therefore not complete even though the
  optimal levels now compare favorably.
- The Fast matcher no longer repeats source bounds checks in
  `match4_found()` after the enclosing loop and hash-table writers have proved
  those bounds. It keeps the explicit `u32::MAX` invalid entry and uses safe
  four-byte reads. Focused level-1 20-run instructions fell from about
  `905.91M` to a stable `828.75M`-`828.76M`, and branches fell from
  `154.55M` to `138.44M`, with identical focused and broad bytes. The paired
  80-run result is Rust `3.314B` instructions and `553.6M` branches versus C
  `2.003B` and `194.4M`, so substantial Fast-path work remains. Artifacts:
  `benchmarks/tmp/perf-z000033-l1-rust-after-fast-match-bounds.stat`,
  `benchmarks/tmp/perf-z000033-l1-c-api-after-fast-match-bounds.stat`, and
  `benchmarks/tmp/normal-level-1-api-after-fast-match-bounds.csv`.
- Huffman streams now use C's overlapping full-word output cadence through
  safe initialized slices. The output grows once to the proven maximum stream
  length plus seven padding bytes, each batch writes a complete eight-byte
  container, and the stream advances by only its emitted bytes before final
  truncation. No pointer arithmetic or unsafe code is needed. Focused level-1
  20-run instructions improve again from about `828.76M` to a stable
  `777.54M`, cycles from roughly `302M`-`310M` to about `275M`, and branches
  from `138.44M` to `124.73M`. All 511 broad byte rows remain unchanged. The
  resulting Rust/C instruction gaps at levels 1/3/5/8/16/19/22 are
  `+55.21%`, `+61.26%`, `+68.25%`, `+60.23%`, `-10.71%`, `-8.98%`, and
  `-12.72%`. Artifacts:
  `benchmarks/tmp/perf-z000033-l1-rust-after-huffman-overlap-store.stat`,
  `benchmarks/tmp/perf-z000033-l1-c-api-after-huffman-overlap-store.stat`, and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-huffman-overlap-store.csv`.
- Rejected Huffman output variant: fixed 1-8-byte safe dispatch arms reduced
  branches but raised focused cycles to about `329M` and branch misses to
  about `3.65M`. The single bounded eight-byte overlapping store was retained.
- Rejected Fast experiment: zero-initializing the Rust hash table and accepting
  physical index zero reduced focused instructions to about `809.76M`, but
  changed 24 previously stable broad byte rows. C's zero is invalid because
  `ZSTD_WINDOW_START_INDEX` makes its first source index 2; Rust's physical
  index zero is real data. The explicit Rust sentinel was restored. Do not
  retry this representation without converting the complete Fast state to
  C-style virtual indexes.
- Current July 18 Huffman bucket-subslice checkpoint: focused `corpus_z000033` level 16
  emits Rust `36,844,400` bytes versus C `36,864,480` for 80 runs. Paired
  counters are Rust `31,036,963,180` instructions, `17,678,427,275` cycles,
  `4,805,010,253` branches, and `193,896,413` misses versus C
  `34,608,032,979`, `18,731,833,914`, `4,309,914,383`, and `173,172,484`.
  Rust is `0.0545%` smaller, uses `10.32%` fewer instructions and `5.62%` fewer
  cycles in these paired samples, while the remaining branch gap is `11.49%`.
  Stable final-code 20-run instruction samples were `7,759,785,101`,
  `7,759,451,002`, and `7,759,785,299`, with about `1.20134B` branches.
- The Huffman bucket-sort walk now iterates its exact bounded rank-position
  subslice directly instead of compiling an `iter().take().skip()` adapter
  chain into `Take::nth` state. This preserves C's bucket order and uses normal
  safe slice iteration. Focused and all 511 broad byte rows are unchanged. The
  80-run sample removes about `16.3M` instructions and `2.5M` branches from the
  rank-table checkpoint. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-bucket-subslice.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-bucket-subslice.stat`,
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-bucket-subslice.data`, and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-bucket-subslice.csv`.
  Validation passed 689 library tests with 5 ignored plus 7 integration tests,
  strict all-target Clippy, formatting, release builds, broad decode comparison,
  and whitespace checks.
- Rejected after this checkpoint: inserting zero-count symbols into the
  Huffman workspace exactly like C and explicitly restoring parent barriers
  preserved bytes but regressed 20-run instructions to `7.7769B`-`7.7773B`
  and branches to about `1.20395B`, versus the retained `7.7595B`-`7.7598B`
  and `1.20131B`-`1.20134B` bands. The compact nonzero workspace was restored.
  Keep skipping zero-count workspace entries unless a materially different
  representation removes the extra writes and barrier initialization cost.
- Rejected parser/match follow-up: replacing the no-dictionary repcode
  collector's `if ll0` split with a safe four-entry logical offset array and
  three unrolled indexed probes preserved bytes and reduced 20-run branches to
  `1,187,833,579`, versus the retained `1.20134B` band. Instructions regressed
  much more substantially to `7,941,790,200`, versus about `7.7595B`. The
  direct branch form was restored, and the post-revert sample returned to
  `7,759,786,992` instructions and `1,201,342,031` branches. Keep the current
  predictable branch and direct unrolled accesses; branch-count reduction alone
  is not a win when total work regresses and retained end-to-end CPU already
  compares favorably with C.
- The direct C-style Huffman builder now uses 256-entry count/value rank tables
  for its `u8` lengths. This makes every index statically in bounds and removes
  hot safe-Rust bounds checks without unsafe indexing. Focused and all 511
  broad byte rows are unchanged. The 80-run sample removes about `42.8M`
  instructions and `21.6M` branches from the bits-to-weight checkpoint.
  Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-rank-u8-tables.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-rank-u8-tables.stat`,
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-rank-u8-tables.data`, and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-rank-u8-tables.csv`.
  Validation passed 689 library tests with 5 ignored plus 7 integration tests,
  strict all-target Clippy, formatting, release builds, broad decode comparison,
  and whitespace checks.
- Huffman code lengths now use C's precomputed `bitsToWeight` conversion rather
  than branching per symbol. A 256-entry safe Rust table covers every possible
  `u8` index, allowing bounds-check elimination without unsafe code. Focused and
  all 511 broad rows remain byte-identical to the prior checkpoint. The 80-run
  sample removes about `39.7M` instructions and `14.0M` branches. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-bits-to-weight.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-bits-to-weight.stat`, and
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-bits-to-weight.data`,
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-bits-to-weight.csv`.
  Validation passed 689 library tests with 5 ignored plus 7 integration tests,
  strict all-target Clippy, formatting, release builds, broad decode comparison,
  and whitespace checks.
- Huffman-weight FSE encoding now batches C's two-symbol join, four-symbol main
  loop, and final two states into bounded safe `u64` writes. The main batch is
  at most 48 bits under `FSE_MAX_TABLELOG == 12`; no unsafe code was added.
  Focused and all 511 broad byte rows are unchanged, while the 80-run sample
  removes about `14.2M` instructions and `16.6M` branches from the NCount
  checkpoint. The FSE implementation is split by responsibility into a
  341-line root, 305-line normalization module, 244-line table module, and
  180-line tests module. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-weight-fse-batching.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-weight-fse-batching.stat`,
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-weight-fse-batching.data`,
  `benchmarks/tmp/callgrind-z000033-l16-rust-after-weight-fse-batching.out`, and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-weight-fse-batching.csv`.
  Validation passed 688 library tests with 5 ignored plus 7 integration tests,
  strict all-target Clippy, formatting, release builds, broad decode comparison,
  and whitespace checks.
- Rejected after this checkpoint: recycling the estimator's owning Huffman code
  vector through `HuffmanBuildScratch` preserved bytes and reduced the 80-run
  branch count by about `2.7M`, but increased hardware instructions by about
  `8.3M`. Three matched user-space 20-run samples were `7.78604B` instructions
  versus the retained `7.7839B`-`7.7841B` band. Callgrind improved by about
  `575K` Ir per frame, but the hardware counters are the acceptance gate, so
  the change was reverted. Keep fresh estimator code-vector allocation unless
  a materially different ownership shape has stronger evidence. Always use
  `instructions:u`, `cycles:u`, `branches:u`, and `branch-misses:u` for these
  A/Bs; including kernel events invalidates comparison with the retained
  artifacts.
- FSE normalized-count serialization now mirrors C's safe-buffer
  `FSE_writeNCount_generic()` path with a bounded `u32` accumulator, compact
  zero-run handling, 16-bit flushes, and direct aligned output append. Exact
  byte-equivalence tests compare it with the previous general BitWriter shape.
  FSE normalization also receives the already-known total from callers, as C
  does, instead of summing counts again. No unsafe code was added. Relative to
  the sequence-batching checkpoint these changes remove about `65.8M`
  instructions and `19.0M` branches over 80 runs while preserving every broad
  byte row. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-c-ncount-known-total.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-c-ncount.stat`, and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-c-ncount-known-total.csv`.
- Callgrind now provides a sharper Huffman target. Rust performs 569 table
  builds per focused frame (376 estimate and 193 emission), while C performs
  648 (456 estimate and 192 emission), so extra invocation count is not the
  issue. Rust costs about 52K Callgrind instructions per build versus roughly
  31-32K for C; the residual is concentrated in table-description finalization
  and owning allocation more than the compact tree walk. Artifacts:
  `benchmarks/tmp/callgrind-z000033-l16-rust-after-c-ncount-known-total.out` and
  `benchmarks/tmp/callgrind-z000033-l16-c-api-current.out`.
- Rejected after this checkpoint: reserving the Huffman weight-header size byte
  directly and assigning it after FSE encoding, instead of using the current
  BitWriter placeholder and `change_bits()`, preserved bytes but measured
  `7.78831B` instructions for 20 runs versus the retained
  `7.7874B`-`7.7877B` band. It was reverted.
- Normal C-cost Huffman construction now keeps code depths in the compact
  8-byte tree nodes, applies C's `HUF_setMaxHeight()` redistribution directly,
  and assigns canonical codes with C's `nbPerRank`/`valPerRank` passes. This
  avoids generic `usize` length/symbol materialization and the subsequent
  reconstruction pass. The C-style path also no longer runs the legacy Rust
  flat-distribution bypass, which C does not have; alternate small/optimal
  searches retain their existing builder and gate. No unsafe code was added,
  and an equivalence test checks the direct and vector builders across table
  logs 4 through 11. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-c-huffman-no-flat-gate.stat`,
  `benchmarks/tmp/perf-z000033-l16-c-api-after-direct-huffman-tree.stat`, and
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-direct-huffman-module-split.data`.
  The encoder is split into normal private Rust modules by role: construction
  (`table.rs`, 424 lines), tree building/sorting (`tree.rs`, 420), alternate
  length limiting (`lengths.rs`, 324), encoded-size queries (`metrics.rs`, 92),
  weight serialization (`weights.rs`, 128), stream emission (`stream.rs`, 81),
  tests (`tests.rs`, 489), and a 124-line root. Final validation passed 687
  tests with 5 ignored plus 7 integration tests, strict all-target Clippy,
  formatting, release tool builds, broad decode comparison, and whitespace
  checks.
- Sequence emission now mirrors C's bit-container accumulation more closely:
  the all-table path packs OF/ML/LL FSE outputs into one safe `u64` write and
  packs LL/ML/OF extra fields into a second; the RLE path also batches its
  extras. The maximum batches are 26 and 63 bits respectively. Focused bytes
  and all broad rows are unchanged, while the 80-run sample removes about
  `24.4M` instructions and `31.4M` branches from the preceding checkpoint.
  `encode_sequences` branch attribution falls from about `2.03%` to `1.41%`.
  Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-sequence-bit-batching.stat`,
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-sequence-bit-batching.data`,
  and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-sequence-bit-batching.csv`.
- The FSE encoder now ports C's compact state table and per-symbol compression
  transforms instead of building nested state vectors and direct lookup arrays.
  C encoder states stay in `[tableSize, 2*tableSize)` and only their low
  `tableLog` bits are flushed. The safe `u16` table is bounded by C's default
  `FSE_MAX_TABLELOG == 12`; the previous release-only unsafe lookup
  initialization is gone. The logical split is `fse_encoder.rs` (stream and
  NCount writing), `fse_encoder/normalize.rs`, and `fse_encoder/table.rs`, all
  below 400 lines. Fresh branch attribution reduces
  `build_table_from_probabilities()` from `2.50%` to `0.84%`. Artifact:
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-compact-fse-table.data`.
- Broad normal output is byte-identical to the retained estimator checkpoint
  across all 511 rows at levels 1, 3, 5, 8, 16, 19, and 22. Aggregate Rust-C
  gaps remain `-1671`, `-1283`, `-714`, `-203`, `-315`, `-174`, and `-64`
  bytes. Artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-sequence-bit-batching.csv`.
  Validation passed all 691 library tests (686 passed, 5 ignored), 7 integration
  tests, all-target clippy with warnings denied, formatting, release builds,
  the decode-verified broad comparison, and whitespace checks.
- Rejected sequence-estimator experiments before the FSE change: six separate
  precomputed LL/ML/OF code and extra-bit vectors regressed focused 20-run
  instructions to `8.698B`-`8.704B`; a fused direct-`PreparedSequence`
  estimator still measured `8.667B`-`8.676B`. Both preserved bytes but were
  removed. Keep the reusable canonical `Sequence` materialization followed by
  count-based estimation; after restoration it measured `8.212B`-`8.213B`
  before the compact FSE table lowered the band further.
- Rejected after the direct Huffman checkpoint: building the temporary code
  `Vec` at a fixed 256 entries and truncating it afterward preserved bytes but
  regressed focused 20-run instructions to `7.857B`-`7.858B` and slightly
  increased branches. Keep the compact runtime-length `Vec`.

- Dictionary attach-mode checkpoint: `params.rs` has a tested
  `should_attach_dict_by_default()` helper mirroring C's default
  `ZSTD_shouldAttachDict()` strategy cutoffs: fast `8 KiB`, dfast `16 KiB`,
  greedy/lazy/lazy2/btlazy2/btopt `32 KiB`, btultra/btultra2 `8 KiB`, and
  unknown source size attaches. This confirms the focused
  `repo_Cargo.lock` level-5 target-2048 dictionary case is a C
  attach-CDict/`dictMatchState` case. The encoder is not wired to this helper
  yet because the current greedy/lazy dictionary implementation is still a
  combined-buffer loaded-prefix/ext-dict approximation. Port the attached
  `dictMatchState` topology before using `CParamMode::AttachDict` in those
  frame paths.
- Attached row-dictionary match-state slice after that checkpoint:
  `MatchSearchConfig` can now carry an optional attached dictionary descriptor,
  and `row_match.rs` has the C-shaped row-DMS search order: active row
  candidates, insert current active position, then search attached dictionary
  rows with the remaining attempt budget. The attached descriptor now models
  C's virtual index referential explicitly: `ZSTD_WINDOW_START_INDEX` is `2`,
  so CDict row entries store dictionary byte 0 at virtual index `2`, and
  offset calculation translates between that CDict referential and Rust's
  zero-based active slice indexes. `greedy_ext.rs` also has a narrow
  `compress_block_greedy_attached_row_dict_with_state()` block adapter that
  runs the existing lazy loop with clean active rows plus a separate dictionary
  `GreedyMatchState`. This is intentionally a block-level primitive only; frame
  routing still needs CDict parameter setup, attach selection, and frame-state
  integration before it should affect compressed output.
- Rejected frame-routing probe for that primitive: simply routing greedy
  dictionary frames through the attached row block adapter regressed the focused
  dictionary+target grid. Artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-attached-row-frame.csv`
  moved `repo_Cargo.lock` level 5 target 2048 from the kept `+4` byte gap to
  `+155` bytes and level 5 aggregate from `-21` to `+130`. The route was
  removed; post-revert artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-attached-row-frame-revert.csv`
  restores level 5 `-21` with `repo_Cargo.lock +4` and level 16 `+1` from
  `generated_poetry.lock`. Keep the row-DMS primitive, but do not retry this
  frame route without deeper C trace evidence and more complete CDict/frame
  setup parity.
- C-indexed attached row-DMS frame follow-up after that rejection: the greedy
  dictionary frame path now routes through the attached row-DMS block adapter
  only for the C-shaped greedy row case. It derives CDict parameters with
  `CParamMode::CreateCDict`, adjusts active parameters with
  `CParamMode::AttachDict`, preserves target block size, requires C's default
  attach decision and matching active/CDict row width, and loads the CDict row
  table at virtual index `ZSTD_WINDOW_START_INDEX == 2`. Focused artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-cindexed-attached-row-frame.csv`
  closes the `repo_Cargo.lock` level-5 target-2048 positive gap: Rust moves
  from `7997` bytes (`+4` versus C `7993`) to `7992` bytes (`-1`). Aggregate
  level 5 moves from `-21` to `-26`; levels 1, 3, 8, and 22 are exact, and the
  only remaining positive row is the existing `generated_poetry.lock` level 16
  `+1`.
- Target+dictionary optimal follow-up after the C-indexed row-DMS checkpoint:
  the remaining `generated_poetry.lock` level-16 target-2048 `+1` row was in
  the final optimal target block. C encoded that block as one raw literal and a
  single sequence `LL=1, ML=567, OF=1`, where `OF=1` reuses rep1. Rust had
  chosen a zero-literal full-block match with a large explicit offset. A
  rejected target-emitter experiment that preserved the explicit offset widened
  the final block to 332 bytes, so the C parity lever is the repcode selection,
  not the literal split alone. The kept change in `opt_encode.rs` rewrites only
  verified final single full-block zero-literal optimal target matches when
  incoming rep1 reproduces `block[1..]`; the prepared block becomes one
  leading literal plus `encoded_offset_value = Some(1)`. Focused inspect
  artifact
  `benchmarks/tmp/inspect-dict-target-poetry-l16-opt-rep1-leading-lit` now has
  Rust and C both at `330` bytes and no block-summary differences. Focused
  dictionary+target artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-opt-rep1-leading-lit.csv`
  has no positive rows: levels 1, 3, 8, and 22 are exact, level 5 is `-26`
  bytes, and level 16 is `-1` byte across the five fixtures.
- Rejected normal-mode post-split probe after that checkpoint: changing
  `post_split::derive_block_splits_helper()` to recompute the original
  partition estimate at each recursive call, mirroring the literal C control
  flow instead of reusing the already-computed parent half estimate, preserved
  focused `corpus_z000033` level-16 output exactly. Artifact
  `benchmarks/tmp/inspect-z000033-l16-postsplit-recompute-estimates` still has
  Rust `461224` bytes versus C `460806`, Rust `191` compressed blocks versus C
  `192`, `50,177` sequences on both sides, and source-aligned summary
  `groups=128,total_delta=418,abs_delta=1006,rust_groups=131,c_groups=132`.
  It was reverted because it only removes estimator reuse without changing the
  size or source-boundary gap.
- Rejected level-22 post-split follow-up after the high-level size review:
  rerunning that same recursive-estimate recomputation on focused
  `corpus_z000033` level 22 also preserved the output and block layout exactly.
  Artifact `benchmarks/tmp/inspect-z000033-l22-recompute-recursive-estimates`
  still has Rust `426585` bytes versus C `426312`, Rust `252` compressed blocks
  versus C `256`, `114,233` sequences on both sides, and source-aligned summary
  `groups=34,total_delta=273,abs_delta=461,rust_groups=44,c_groups=48`. The
  dominant final source-aligned row remains `985341..1022035` at `+259` bytes:
  Rust keeps one partition while C emits eight. The recompute change was
  reverted. Do not retry recursive original-estimate recomputation for this
  final-block level-22 gap unless a new trace shows estimator scratch side
  effects rather than the reused numeric estimate are the cause.
- Follow-up diagnostic after that rejection: a temporary forced split at the
  exact final `corpus_z000033` level-22 helper subrange (`985341..1022035`,
  `36,694` source bytes, `2,840` sequences) used C's observed recursive
  boundaries `[710, 887, 1065, 1420, 1775, 1952, 2130]`. Artifact
  `benchmarks/tmp/inspect-z000033-l22-force-c-final-helper-splits` shows Rust
  improves from `426585` bytes to `426324` bytes versus C `426312`; the
  source-aligned total gap drops from `+273` to `+12`, and the final
  source-aligned `+259` row disappears. The forced Rust tail partitions match C
  except for a `-2` byte row on `991395..993350`. A separate temporary estimate
  panic for the same helper range measured Rust estimates `original=17652`,
  `first=5199`, `second=12463`, `sum=17662`, so Rust rejects the split by only
  `10` estimated bytes even though emitted partitions are much smaller. The
  forced split was reverted. Next size-parity work should compare Rust's
  post-split estimator with C for this exact range; do not add a blind split
  tolerance without explaining the estimator mismatch, because older tolerance
  artifacts shifted other rows.
- Latest kept size-parity fix after that diagnostic:
  `BlockCompressionConfig::for_c_block_split_estimate()` now preserves
  `HuffmanTableSearch::AllSections` when the strategy config already selected
  it for `BtUltra`/`BtUltra2`, while still mapping non-optimal searches such as
  `FileTypeSmall` to C's normal `Heuristic` path. The previous estimate cleanup
  was correct for level-16 `BtOpt`, but it accidentally disabled C's
  `HUF_flags_optimalDepth` behavior for level-22 post-split estimates. An
  instrumented C copy under `/tmp/zstd-est-trace` showed the decisive final
  `corpus_z000033` level-22 range had matching original and second-half
  estimates but a first-half literal-estimate difference: C `first=5183` with
  `lit=2119` and `seq=3061`, Rust before the fix `first=5199` with `lit=2135`
  and `seq=3061`. Preserving optimal-depth Huffman search in the post-split
  estimate made Rust accept the C-shaped recursive split. Focused inspect
  artifact `benchmarks/tmp/inspect-z000033-l22-after-btultra-postsplit-optdepth`
  shows Rust `426270` bytes versus C `426312` on `corpus_z000033` level 22;
  the source-aligned total gap is now `-42` and the largest positive rows are
  only `+10` bytes. Broad artifact
  `benchmarks/tmp/normal-levels-16-19-22-api-runs1-after-btultra-postsplit-optdepth-rebuilt.csv`
  shows level 16 unchanged at `+402` bytes versus C across 73 fixtures,
  level 19 improved to `-174`, and level 22 improved to `-64`; worst positive
  rows at levels 19 and 22 are now `+2` bytes on `corpus_z000044`. Focused
  20-run wall-clock smoke emitted Rust `8,525,400` bytes in `3.352s` versus C
  API `8,526,240` bytes in `3.915s`. Validation passed `cargo fmt --check`,
  focused config tests, `cargo test -p ruzstd compressed --quiet`, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, release rebuilds for `benchmark_c_port`, `profile_c_port`, and
  `profile_c_api`, the focused inspect, broad API comparison, and
  `git diff --check`.
- Diagnostic follow-up after that probe: `inspect_c_port_blocks` now persists
  full block and source-aligned comparison CSVs in the output directory:
  `rust.blocks.csv`, `c.blocks.csv`, and `source-aligned.csv`. Focused artifact
  `benchmarks/tmp/inspect-z000033-l16-current-with-source-csv` confirms the
  level-16 `corpus_z000033` source-aligned totals (`128` rows, `+418` total
  delta, `1006` absolute delta) and keeps the first source-boundary mismatch
  visible as data: Rust splits `168226..172032` into two blocks while C emits
  one. Future split/entropy investigations should use these CSVs instead of
  scraping console output.
- Latest normal-mode C-fast Huffman table-log checkpoint: release
  `profile_c_port`, `profile_c_api`, `benchmark_c_port`, and
  `inspect_c_port_blocks` were rebuilt after teaching normal C-strategy emitted
  literal sections to use C's fast/non-optimal Huffman table log when
  `HUF_flags_optimalDepth` is not active. The focused `corpus_z000033`
  level-16 one-run output improved from Rust `461,224` bytes versus C
  `460,806` bytes (`+418`) to Rust `460,572` bytes versus C `460,806` bytes
  (`-234`). Focused 80-run perf counters were Rust `36,845,760` bytes,
  `33,678,645,198` instructions, `19,878,801,734` cycles, `5,449,699,950`
  branches, and `215,219,140` branch misses; C API was `36,864,480` bytes,
  `34,690,615,149` instructions, `18,514,582,045` cycles, `4,327,110,245`
  branches, and `175,021,650` branch misses. The broad API artifact is
  `benchmarks/tmp/normal-levels-16-19-22-api-runs1-after-c-fast-huff-log.csv`;
  aggregate byte totals are level 16 `-253` bytes (`-0.018%`), level 19
  `-174` bytes (`-0.013%`), and level 22 `-64` bytes (`-0.005%`) versus C
  across 73 fixtures. `inspect_c_port_blocks` now records
  `literal_table_size` so future entropy-size investigations can separate table
  description bytes from literal stream bytes. Validation passed format, full
  `ruzstd` tests, clippy with warnings denied, focused Huffman/compressed tests,
  focused inspect, broad API comparison, and `git diff --check`.
- Latest safe branch-count checkpoint: `OptBlockState::opt` is now a boxed
  `[Optimal; ZSTD_OPT_NUM + 4]` instead of a `Vec<Optimal>`. The table has
  always had that fixed size; expressing the invariant in its type lets LLVM
  remove hot parser bounds checks while preserving heap allocation and avoiding
  unchecked indexing. Focused `corpus_z000033` level-16 output stayed at
  `36,845,760` bytes for 80 runs. Rust measured `33,636,424,860`
  instructions, `20,626,060,462` cycles, `5,343,865,430` branches, and
  `215,532,074` branch misses, versus the previous kept Rust checkpoint at
  `33,678,645,198` instructions and `5,449,699,950` branches. Same-session C
  API measured `36,864,480` bytes, `34,938,271,121` instructions, and
  `4,326,294,025` branches. The focused Rust branch count improved by about
  `1.9%`; the remaining Rust/C branch gap is about `+23.5%`. Broad artifact
  `benchmarks/tmp/normal-levels-16-19-22-api-runs1-after-fixed-opt-table.csv`
  preserved aggregate gaps of level 16 `-253` bytes, level 19 `-174`, and
  level 22 `-64` across 73 fixtures. Retained source-mapped branch profile:
  `benchmarks/tmp/perf-z000033-l16-rust-branches-after-fixed-opt-table.data`.
  Validation passed format, focused parser tests, the full `ruzstd` suite,
  clippy with warnings denied, broad API comparison, and `git diff --check`.
- Latest safe binary-tree follow-up: the two no-dictionary optimal tree walks
  now derive their ring mask from the allocated chain-table length instead of
  recomputing it from `CompressionParameters::chain_log`. The table is always
  allocated as `2^chain_log` tree slots plus one dummy cleanup slot, so the
  table shape is the stronger local invariant and helps LLVM relate masked
  indexes to the actual allocation. No unchecked indexing was added. Focused
  `corpus_z000033` level-16 output stayed at `36,845,760` bytes for 80 runs.
  Rust measured `33,438,229,034` instructions, `20,892,152,097` cycles,
  `5,316,893,231` branches, and `215,313,947` branch misses, improving on the
  boxed-table checkpoint by about `0.59%` instructions and `0.50%` branches.
  Same-session C API measured `36,864,480` bytes, `34,893,567,366`
  instructions, `19,569,922,873` cycles, `4,316,986,014` branches, and
  `176,184,405` branch misses. Broad artifact
  `benchmarks/tmp/normal-levels-16-19-22-api-runs1-after-table-derived-bt-mask.csv`
  preserved aggregate gaps of level 16 `-253` bytes, level 19 `-174`, and
  level 22 `-64` across 73 fixtures. Retained source-mapped branch profile:
  `benchmarks/tmp/perf-z000033-l16-rust-branches-after-table-derived-bt-mask.data`.
  Validation passed format, focused match tests, the full `ruzstd` suite,
  clippy with warnings denied, broad API comparison, and `git diff --check`.
- Rejected primary hash-table follow-ups after that checkpoint: deriving
  `hash_log` from the allocated hash-table length preserved focused bytes but
  raised a 20-run instruction sample from the kept `8.416B` band to `8.554B`.
  Narrowly wrapping the two primary hash reads/writes in `get_unchecked()`
  reduced branches from about `1.341B` to `1.333B`, but raised instructions to
  `8.504B`-`8.505B` and showed no reliable cycle benefit. Both experiments
  were reverted; no new unsafe hash indexing remains.
- Latest attached-dictionary extension: the row-based `dictMatchState` block
  adapter and frame route now carry the actual lazy-search depth, covering C's
  row-hash `Greedy`, `Lazy`, and `Lazy2` attach paths when default attach mode
  is selected and active/CDict row widths match. `BtLazy2` remains on the
  binary-tree/ext-dict path. Direct tests cover attached matches at depths 1
  and 2 and the frame-selection boundaries. Target+dictionary artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-attached-lazy-row.csv`
  has no positive rows across five fixtures: levels 1, 3, 8, and 22 are exact,
  level 5 totals `-26` bytes, and level 16 totals `-1`. Normal dictionary
  artifact
  `benchmarks/tmp/dictionary-normal-levels-5-10-api-after-attached-lazy-row.csv`
  shows the affected 31,858-byte `repo_Cargo.lock` exact at level 6, `-1` at
  level 7, and exact at levels 8-10. Full `ruzstd` tests, focused attached-row
  tests, clippy with warnings denied, format, dictionary decode comparison,
  and `git diff --check` passed.
- Latest attached binary-tree extension: default-attach `BtLazy2` dictionary
  frames now use a separate, fully sorted CDict tree and the C-shaped
  `ZSTD_DUBT_findBetterDictMatch()` search after the active DUBT walk. The
  implementation preserves the CDict virtual index base of 2, consumes only
  the active search's remaining comparison budget, continues matches across
  the dictionary/prefix boundary, and applies C's offset-cost gate. It lives in
  `bt_match/attached.rs`; no unsafe code was added. On the attach-eligible
  31,858-byte `repo_Cargo.lock`, normal level 11 moved from Rust `7768` versus
  C `7767` to exact `7767`. Artifact
  `benchmarks/tmp/dictionary-normal-levels-11-15-api-after-attached-bt.csv`
  leaves levels 12-15 unchanged; the remaining level-11 `+4` bytes come from a
  51,962-byte source above C's 32 KiB attach cutoff. Target artifact
  `benchmarks/tmp/dictionary-target-levels-11-15-api-after-attached-bt.csv`
  also has the attach-eligible level-11 row exact. Direct match and routing
  tests pass, and the split keeps the production files within the 300-500 line
  reviewability target.
  Post-split release smoke
  `benchmarks/tmp/dictionary-normal-level-11-api-after-attached-bt-split.csv`
  reproduces the exact attach-eligible level-11 row.
- Latest resume branch-profile pass: `perf record -e branches:u -c 100000` on
  focused `corpus_z000033` level 16 for 20 Rust runs produced
  `benchmarks/tmp/perf-z000033-l16-rust-branches-current.data`. Branch samples
  keep `forward_pass()` dominant at about `40.7%` self branch samples, with
  `compress_block_opt_with_state_and_ldm()` next at about `21.9%`. The obvious
  pass-local short literal-length increment cache was not retried because it is
  already documented as a rejected shape. Keep investigating generated-code
  shape or new profile-backed parser/tree changes for branch parity.
- Rejected follow-up after that branch-profile pass: changing
  `refresh_node_reps()` to check `state.opt[cur].litlen` before copying the
  current `Optimal`, and then loading only `mlen`/`off` on the match endpoint
  path, preserved focused bytes (`9,224,480` for 20 runs) but did not reduce
  the focused branch/instruction band. Three 20-run samples were
  `8,439,203,742`, `8,438,362,108`, and `8,439,682,077` instructions, with
  branches still about `1.372B` to `1.373B`. It was reverted; keep the current
  whole-node copy in `refresh_node_reps()` unless a new profile gives a
  stronger reason.
- Latest reviewability cleanup outside this module: C fast/cost sequence-table
  selection policies moved from
  `ruzstd/src/encoding/blocks/compressed/sequence_tables/selection.rs` into
  `ruzstd/src/encoding/blocks/compressed/sequence_tables/selection/policy.rs`.
  The main selection file is now `488` lines. This is behavior-preserving and
  was validated with format, focused compressed tests, full `ruzstd` tests,
  clippy with warnings denied, and `git diff --check`.
- Latest target-mode porting checkpoint: levels 1 and 3 now support
  no-dictionary `targetCBlockSize` by routing fast and double-fast prepared
  blocks through the existing target-block/superblock writer. The normal fast
  and double-fast paths still call their existing wrappers with
  `BlockEncodeMode::Normal`. Broad one-run C API comparisons across
  73 real-world fixtures for target sizes 2048, 4096, and 8192 produced
  aggregate gaps versus C of level 1 `-39`, `-48`, and `-50` bytes,
  respectively, and level 3 `-1829`, `-1761`, and `-1704` bytes,
  respectively. Level 1 had no positive fixture gaps; level 3 had two tiny
  positive gaps at each target size, worst `+191` bytes on `corpus_z000050` at
  target 8192. Artifacts:
  `benchmarks/tmp/target-levels-1-3-api-runs1-after-fast-dfast-target.csv`,
  `benchmarks/tmp/target-levels-1-3-api-runs1-target4096-after-fast-dfast-target.csv`,
  and
  `benchmarks/tmp/target-levels-1-3-api-runs1-target8192-after-fast-dfast-target.csv`.
- Follow-up reviewability split after the fast/double-fast target checkpoint:
  stateful `BlockEncodeMode` adapters moved into
  `fast_block/mode.rs` and `dfast_block/mode.rs`, and the target/superblock
  handoff stays in sibling `target.rs` files. This keeps `fast_block.rs` at
  `456` lines and `dfast_block.rs` at `457` lines. The target-mode invalid-size
  test name was updated now that levels 1 and 3 are supported. Validation
  passed `cargo fmt --check`, focused `strategy_frame_target_c_block_size`,
  `fast`, `dfast`, and `target_block` tests, full `cargo test -p ruzstd
  --quiet`, and `cargo clippy -p ruzstd --all-targets -- -D warnings`.
  Release benchmark tools were rebuilt, and
  `benchmarks/tmp/target-levels-1-3-api-runs1-after-fast-dfast-target-refactor.csv`
  matches the previous checkpoint row-by-row for C and Rust compressed bytes.
- Latest broad all-level C API checkpoint after that split:
  `benchmarks/tmp/normal-levels-1-22-api-runs1-current.csv` covers levels 1
  through 22 across the 73 real-world fixtures. Normal-mode aggregate size gaps
  versus C range from level 3 `-2589` bytes (`-0.163%`) to level 22 `+1347`
  bytes (`+0.103%`). The largest positive fixture row is focused
  `corpus_z000033` level 22 at `+1026` bytes (`427338` Rust versus `426312`
  C). Focused 80-run perf counters on that fixture show high levels are not the
  remaining instruction/cycle problem: at level 18 Rust used `62.147B`
  instructions and `36.274B` cycles versus C `74.142B` and `41.624B`; at level
  22 Rust used `124.270B` instructions and `55.271B` cycles versus C
  `147.908B` and `68.618B`. Rust branch counts remain higher: `10.087B` versus
  C `9.854B` at level 18 and `21.092B` versus C `17.838B` at level 22.
- Latest all-level target-mode checkpoint:
  `benchmarks/tmp/target-levels-1-22-api-runs1-target2048-current.csv` covers
  levels 1 through 22 across the same 73 fixtures with `targetCBlockSize=2048`.
  Levels 13 through 16 and 18 through 22 are byte-identical to C on every
  fixture; level 17 is `-9` bytes in aggregate with no positive rows. Lower
  levels remain tiny: level 3 is the largest aggregate negative gap at `-1829`
  bytes (`-0.115%`), and the largest positive row is `corpus_z000050` level 3
  at `+57` bytes. This supersedes older notes that target-mode fast paths were
  unported.
- Latest kept high-level compression-size change:
  `BlockCompressionConfig::for_c_strategy()` now uses
  `HuffmanTableSearch::AllSections` for strategies `BtUltra` and `BtUltra2`
  (`strategy >= 8`), matching C's `HUF_flags_optimalDepth` threshold while
  preserving the cheaper heuristic for `BtOpt` and below. This avoids
  reintroducing the rejected level-16 `BtOpt` optimal-depth dispatch. The
  broad normal artifact is
  `benchmarks/tmp/normal-levels-1-22-api-runs1-after-btultra-huff-optdepth.csv`:
  levels 1 through 12 are unchanged, level 18 improves from `+1278` to `+183`
  bytes, level 19 from `+1057` to `-77`, level 20 from `+1043` to `-75`,
  level 21 from `+1205` to `+143`, and level 22 from `+1347` to `+265`.
  Focused `corpus_z000033` level 22 improved from `+1026` bytes to `+273`
  bytes, and the literal payload gap dropped from `+1162` bytes to `+401`.
  Focused 80-run perf still favors Rust on instructions/cycles at levels 18
  and 22, but Rust branch counts remain higher. Target-mode high levels stayed
  stable in
  `benchmarks/tmp/target-levels-13-22-api-runs1-target2048-after-btultra-huff-optdepth.csv`:
  levels 13 through 16 and 18 through 22 remain byte-identical to C, and level
  17 remains `-9` bytes with no positive rows.
- Diagnostic tooling follow-up: `inspect_c_port_blocks` now prints
  `source_aligned_deltas`, grouping Rust and C blocks by decompressed source
  range before comparing compressed bytes. Use this when post-split boundary
  choices differ, because index-aligned block deltas become misleading after
  the first source-boundary mismatch.
- Follow-up diagnostic refinement: `inspect_c_port_blocks` now prints
  `source_aligned_summary` before the ranked source-aligned rows. For focused
  `corpus_z000033` level 16, the artifact
  `benchmarks/tmp/inspect-z000033-l16-current/source-aligned.txt` shows Rust
  `461224` bytes versus C `460806` (`+418`), with
  `groups=128,total_delta=418,abs_delta=1006,rust_groups=131,c_groups=132`.
  The largest same-source-range deltas are small and include Rust wins
  (`-114`, `-66`, `-30`, `-27` bytes in the top rows); the largest positive
  same-range rows are only `+24`, `+23`, and `+23` bytes. The large
  index-aligned deltas in that run are caused by the first source-boundary
  mismatch at block 33 and later post-split boundary shifts, not one obvious
  broken entropy decision. Existing runtime tuning probes for exact sequence
  modes / all-section Huffman / repeat-table thresholds are not valid for
  `for_c_strategy()` because that path does not apply file-type tuning
  overrides.
- Reviewability cleanup after that diagnostic pass: the inspection tool's
  comparison and source-aligned delta reporting moved from
  `tools/src/bin/inspect_c_port_blocks.rs` into
  `tools/src/bin/inspect_c_port_blocks/comparison.rs`. The main binary is now
  `434` lines and the comparison module is `339` lines. Validation passed
  format, release rebuild, clippy for the `inspect_c_port_blocks` binary, and a
  focused level-22 smoke confirmed the `source_aligned_summary` line is still
  emitted.
- Target-mode surface cleanup after that split: the public
  `compress_slice_c_level_with_target_c_block_size()` doc comment and the
  `profile_c_port`, `benchmark_c_port`, and `inspect_c_port_blocks` error
  messages no longer say target mode may be unported for the resolved level.
  All no-dictionary levels 1 through 22 now route to an implementation; the
  helper returns `None` only when `targetCBlockSize` is outside C's accepted
  range.
- All-target-size validation after the target-mode surface cleanup: all-level
  C API target-mode comparisons now cover target sizes 2048, 4096, and 8192
  across all no-dictionary levels 1 through 22 and the 73 real-world fixtures.
  New artifacts:
  `benchmarks/tmp/target-levels-1-22-api-runs1-target4096-current.csv` and
  `benchmarks/tmp/target-levels-1-22-api-runs1-target8192-current.csv`,
  alongside the existing target-2048 artifact. At target sizes 4096 and 8192,
  levels 13 through 22 are byte-identical to C on every fixture. At target
  2048, levels 13 through 16 and 18 through 22 are byte-identical, while level
  17 is `-9` bytes in aggregate with no positive rows. Lower levels remain tiny
  and usually smaller than C in aggregate; the worst positive rows across all
  three target sizes are `corpus_z000050` level 3 at `+191` bytes for target
  8192, `+133` bytes for target 4096, and `+57` bytes for target 2048.
- Latest kept parser price follow-up after the short literal-length increment
  fast path: `OptPriceState::dynamic_lit_length_increment_price_unchecked()`
  now handles literal lengths `1..=15` as direct adjacent-code deltas before
  falling back to the sparse transition helper. For those dense codes,
  `LL_BITS` and the sum-base terms cancel, so the delta is just
  `weight(previous_freq) - weight(current_freq)`. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). The focused 80-run Rust sample was
  `34,724,664,562` instructions, `23,541,986,061` cycles, `5,689,710,362`
  branches, and `241,570,850` branch misses. Same-session C API was
  `36,864,480` bytes, `34,710,851,353` instructions, `22,595,867,999` cycles,
  `4,330,895,918` branches, and `176,546,761` branch misses. This leaves the
  focused instruction gap at about `+0.04%`; the remaining measurable CPU gap
  is cycles, wall time, and branch/control-flow shape. Validation passed
  `cargo fmt --check`, focused opt-price and opt-parser tests, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, release rebuilds for `profile_c_port` and `profile_c_api`, and
  `git diff --check`.
- Rejected after the direct dense LL delta checkpoint: adding an
  equal-frequency fast path for literal lengths `1..=15`, returning zero before
  the two `weight()` calls when adjacent LL frequencies matched, preserved
  focused bytes but regressed focused 20-run instruction samples to
  `8,905,119,942` and `8,906,188,453` versus the kept `8.873B` to `8.880B`
  band. It also raised branch count to about `1.472B` for 20 runs. It was
  reverted; do not retry without a fresh profile signal.
- Latest kept tiny parser cleanup after the direct dense LL delta checkpoint:
  `refresh_node_reps()` now copies the current and previous parser nodes into
  locals before writing the updated repcode history, reducing repeated
  `state.opt` indexing in the hot forward pass without changing the algorithm.
  Focused bytes stayed unchanged (`9,224,480` for 20 runs and `36,897,920` for
  80 runs). Three focused 20-run instruction samples were `8,881,873,333`,
  `8,876,467,976`, and `8,872,578,047`. The focused 80-run sample was
  `34,724,188,577` instructions, `23,785,786,398` cycles, `5,689,785,477`
  branches, and `242,995,852` branch misses. Same-session C API was
  `36,864,480` bytes, `34,711,691,664` instructions, `22,545,950,850` cycles,
  `4,331,112,642` branches, and `177,105,696` branch misses. Treat this as a
  neutral-to-tiny instruction cleanup; the remaining work is still branch and
  cycle parity.
- Latest kept parser borrow cleanup after that checkpoint:
  `raw_literal_cost()` now takes `&OptPriceState` and `&mut LiteralPriceCache`
  directly instead of borrowing the whole `OptBlockState`. This keeps the
  literal-price cache shape unchanged while narrowing aliases in the hot
  literal-price path. Focused bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). Three focused 20-run instruction samples were
  `8,877,574,614`, `8,874,501,779`, and `8,874,302,490`. The focused 80-run
  sample was `34,721,273,930` instructions, `21,058,286,081` cycles,
  `5,688,820,453` branches, and `240,112,501` branch misses. Same-session C
  API was `36,864,480` bytes, `34,708,905,974` instructions,
  `23,368,568,796` cycles, `4,330,775,908` branches, and `177,607,455` branch
  misses. The focused instruction gap remains about `+0.04%`; branch count
  remains about `+31%`.
- Rejected follow-up after the parser borrow cleanup: applying the same
  field-borrow split to `update_match_prices()` preserved focused bytes and
  reduced branch count slightly, but regressed the focused 20-run instruction
  band to `8,884,808,156`, `8,889,492,645`, and `8,888,961,923` versus the
  kept `8.874B` to `8.878B` band. It was reverted; keep `update_match_prices()`
  taking `&mut OptBlockState` unless a stronger profile signal appears.
- Rejected follow-up after the same checkpoint: snapshotting `state.opt[cur]`
  once after `refresh_node_reps()` and reusing that local for the skip check,
  rep history, zero-literal flag, and large-match stretch preserved focused
  bytes but regressed focused 20-run instruction samples to `8,918,738,569`,
  `8,913,761,120`, and `8,915,618,718`, with branch count also higher at about
  `1.471B` per 20 runs. It was reverted; keep the current direct `state.opt`
  access shape unless a new profile gives a much stronger reason.
- Rejected follow-up after the parser borrow cleanup: replacing the final safe
  tail-byte comparisons in `match_count.rs` with a checked-by-caller `read8()`
  helper preserved focused bytes (`9,224,480` for 20 runs) but regressed three
  focused 20-run instruction samples to `8,983,615,053`, `8,980,016,221`, and
  `8,980,931,144` versus the kept `8.874B` to `8.878B` band. It was reverted;
  keep the safe single-byte tail comparisons unless a new profile gives
  stronger evidence.
- Rejected follow-up after that: replacing the decisive safe byte reads in the
  two no-dictionary binary-tree loops with an unsafe `get_unchecked()` helper,
  guarded by the existing match-end checks, preserved focused bytes
  (`9,224,480` for 20 runs) but regressed focused 20-run instruction samples to
  `8,908,106,906`, `8,906,476,774`, and `8,907,544,796`. Branch counts were
  around `1.431B`, not enough to justify the extra unsafe code or the
  instruction regression. It was reverted; keep the current safe byte reads
  unless a fresh profile gives a stronger signal.
- Latest kept CPU change after the parser borrow cleanup: `OptBlockState` now
  owns a reusable `EstimateScratch`, and the no-dictionary/ext-dictionary
  optimal post-split paths pass it into `encode_split_block()` instead of
  constructing a fresh estimator scratch for every split decision tree. This
  mirrors C's reusable temporary workspace while staying in safe Rust. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). Three focused 20-run instruction samples were
  `8,871,865,495`, `8,876,424,982`, and `8,870,265,527`. The focused 80-run
  Rust sample was `34,082,041,893` instructions, `24,302,296,491` cycles,
  `5,556,270,386` branches, and `227,186,708` branch misses. Same-session C
  API was `36,864,480` bytes, `34,717,968,848` instructions,
  `25,176,232,975` cycles, `4,332,657,101` branches, and `177,931,971` branch
  misses. This puts the focused instruction sample about `-1.83%` below C,
  while branch count remains about `+28%`.
- Latest kept allocation cleanup after the reusable `EstimateScratch`
  checkpoint: the no-dictionary and ext-dictionary optimal block encoders now
  pre-size their per-block output `Vec` to the block length plus the zstd block
  header. This keeps the normal `Vec` ownership shape but avoids starting the
  hot compressed/raw/RLE block output buffer at zero capacity. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). Three focused 20-run instruction samples were
  `8,846,630,893`, `8,856,719,799`, and `8,840,146,276`; after reverting only
  this capacity edit, two focused 20-run samples returned to `8,878,157,552`
  and `8,869,565,224`. The kept focused 80-run Rust sample was
  `35,428,271,607` instructions, `21,729,012,515` cycles, `5,837,893,066`
  branches, and `256,256,242` branch misses. Same-session C API was
  `36,864,480` bytes, `34,701,628,748` instructions, `18,763,185,396` cycles,
  `4,329,151,407` branches, and `175,469,965` branch misses. Treat this as a
  small safe allocation cleanup; it preserves bytes and improves the current
  short-run instruction band, but the residual branch gap remains the main
  CPU-parity issue.
- Latest kept allocation follow-up after the output-buffer capacity checkpoint:
  `prepare_from_greedy_output()` now reserves the prepared literal stream to the
  exact emitted literal count (`sum(sequence.lit_len) + last_literals`) instead
  of reserving the full source block length. This keeps the same prepared-block
  representation while avoiding a large over-reservation on match-heavy optimal
  blocks. Focused `corpus_z000033` level-16 bytes stayed unchanged
  (`9,224,480` for 20 runs and `36,897,920` for 80 runs). Three focused
  20-run instruction samples were `8,654,044,809`, `8,653,339,868`, and
  `8,649,562,896`, with branch counts around `1.417B` to `1.418B`. The
  focused 80-run Rust sample was `33,817,861,181` instructions,
  `23,277,005,768` cycles, `5,505,592,571` branches, and `223,231,659` branch
  misses. Same-session C API was `36,864,480` bytes, `34,701,670,143`
  instructions, `19,833,235,458` cycles, `4,329,162,276` branches, and
  `175,810,737` branch misses. This puts the focused instruction sample about
  `-2.55%` below C, while branch count remains about `+27.2%`. The broad
  one-run API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-exact-literal-capacity.csv`;
  byte totals stayed unchanged at level 8 `+277` bytes (`+0.018%`), level 16
  `+666` bytes (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C
  across 73 fixtures. Validation passed `cargo fmt --check`, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, `git diff --check`, release rebuilds for `profile_c_port`,
  `profile_c_api`, and `benchmark_c_port`, focused perf samples, and the broad
  byte comparison.
- Latest kept allocation follow-up after exact prepared-literal capacity:
  `OptBlockState` now owns a reusable prepared block, and the optimal
  no-dictionary/ext-dictionary encoders fill it with
  `prepare_from_greedy_output_in()` before recycling its literal and sequence
  vectors after target, post-split, or normal block emission. This keeps the
  same prepared-block representation and encoding decisions, while matching C's
  reused workspace shape more closely. Focused `corpus_z000033` level-16 bytes
  stayed unchanged (`9,224,480` for 20 runs and `36,897,920` for 80 runs).
  Three focused 20-run instruction samples were `8,456,200,976`,
  `8,456,918,219`, and `8,460,032,939`, with branch counts around `1.375B`.
  The focused 80-run Rust sample was `33,606,694,214` instructions,
  `20,249,941,451` cycles, `5,455,670,125` branches, and `214,930,848` branch
  misses. Same-session C API was `36,864,480` bytes, `34,701,780,406`
  instructions, `19,750,435,821` cycles, `4,329,179,049` branches, and
  `175,751,493` branch misses. This puts the focused instruction sample about
  `-3.15%` below C, while branch count remains about `+26.0%`. The broad
  one-run API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-prepared-block-reuse.csv`;
  byte totals stayed unchanged at level 8 `+277` bytes (`+0.018%`), level 16
  `+666` bytes (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C
  across 73 fixtures. Validation passed `cargo fmt --check`, focused
  opt-parser tests, full `cargo test -p ruzstd --quiet`, `cargo clippy -p
  ruzstd --all-targets -- -D warnings`, `git diff --check`, release rebuilds
  for `profile_c_port`, `profile_c_api`, and `benchmark_c_port`, focused perf
  samples, and the broad byte comparison.
- Latest kept allocation follow-up after prepared-block reuse:
  `OptBlockState` now owns a reusable per-block encoded byte buffer. The
  optimal block encoders take that buffer before emission, and the optimal frame
  loops recycle it after appending the encoded block bytes to the final frame.
  This keeps the public `GreedyEncodedBlock` shape unchanged while avoiding a
  fresh encoded-block byte allocation for each optimal block. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). Three focused 20-run instruction samples were
  `8,434,046,931`, `8,432,965,569`, and `8,469,170,768`, with the first two
  branch samples around `1.370B`. The focused 80-run Rust sample was
  `33,581,418,415` instructions, `19,435,891,152` cycles, `5,448,721,492`
  branches, and `213,123,635` branch misses. Same-session C API was
  `36,864,480` bytes, `34,700,873,186` instructions, `19,996,199,813` cycles,
  `4,328,936,017` branches, and `175,446,578` branch misses. This puts the
  focused instruction sample about `-3.23%` below C, while branch count remains
  about `+25.9%`. The broad one-run API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-block-byte-reuse.csv`;
  byte totals stayed unchanged at level 8 `+277` bytes (`+0.018%`), level 16
  `+666` bytes (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C
  across 73 fixtures. Validation passed `cargo fmt --check`, focused
  opt-parser tests, full `cargo test -p ruzstd --quiet`, `cargo clippy -p
  ruzstd --all-targets -- -D warnings`, `git diff --check`, release rebuilds
  for `profile_c_port`, `profile_c_api`, and `benchmark_c_port`, focused perf
  samples, and the broad byte comparison.
- Rejected follow-up after the block-byte-reuse checkpoint: threading a reusable
  `CompressedBlockScratch` through optimal normal and post-split emission, so
  `compress_prepared_block_with_stats()` could reuse its encoded sequence
  vector, preserved focused bytes (`9,224,480` for 20 runs) but did not improve
  the focused CPU band. Three focused 20-run samples were `8,459,276,213`,
  `8,459,668,496`, and `8,460,628,884` instructions with branch counts around
  `1.379B`, versus the kept `8.432B` to `8.469B` band and a restored-code
  sample of `8,436,762,404` instructions and `1,370,643,323` branches. It was
  reverted; do not retry compressed-block sequence scratch threading without a
  new profile signal.
- Rejected follow-up after the same checkpoint: adding
  `OptMatchTable::last_value()` and using it at the longest-match call sites
  preserved focused bytes (`9,224,480` for 20 runs) but did not improve the
  focused CPU band enough to justify keeping the helper. Focused 20-run
  samples were `8,435,314,851`, `8,435,069,033`, and `8,432,093,200`
  instructions with branch counts around `1.370B`, versus the restored-code
  sample of `8,436,762,404` instructions and `1,370,643,323` branches. It was
  reverted; keep the current `matches.get(match_count - 1)` shape unless a new
  profile points back there.
- Fresh post-revert focused baseline from the same checkpoint:
  `profile_c_port` emitted `36,897,920` bytes for 80 runs on
  `corpus_z000033` level 16 with `33,586,353,604` instructions,
  `19,495,744,652` cycles, `5,449,536,019` branches, and `213,324,059` branch
  misses. Same-session `profile_c_api` emitted `36,864,480` bytes with
  `34,698,155,418` instructions, `19,011,420,718` cycles, `4,328,407,208`
  branches, and `175,420,015` branch misses. The focused size gap remains
  `+0.091%`, Rust is about `-3.20%` instructions versus C, and the branch-count
  gap remains about `+25.9%`. Continue treating branch/control-flow shape and
  cycles as the residual focused CPU work.
- Latest kept correctness cleanup after that baseline: literal-section size
  fallbacks in `ruzstd/src/encoding/blocks/compressed/literals.rs` and the
  C-port superblock literal writer now use explicit invariant `panic!` messages
  instead of `unimplemented!("too many literals")`. Legal zstd blocks are
  bounded to 128 KiB, below the raw/RLE and compressed-literals size-field
  limits, so these arms are invariant violations rather than missing port work.
  Added tests for max-block raw/RLE literal headers and the max-block compressed
  literal size format. Validation passed the three focused new tests, the
  compressed-block test module, `cargo fmt --check`, `git diff --check`, full
  `cargo test -p ruzstd --quiet`, and clippy for all `ruzstd` targets with
  warnings denied.
- Latest kept unsafe-reduction cleanup after that baseline:
  `ruzstd/src/encoding/levels/c_port/row_match.rs` now uses safe slice/index
  access for row-table slice creation, candidate lookup, and row insertion/update
  writes instead of `from_raw_parts()` and `get_unchecked()` for those table
  accesses. Remaining unsafe in that file is limited to x86 prefetch/SSE2 row
  mask loads and unaligned scalar row-mask reads. The affected-path broad smoke
  `benchmarks/tmp/normal-level5-api-runs1-after-row-safe-indexing.csv` passed
  decode validation across 73 fixtures at level 5; Rust total output was
  `1,555,507` bytes versus C API `1,557,737` bytes (`-2,230`, `-0.143%`), with
  largest positive fixture gap `+27` bytes on `corpus_z000057`. Validation
  passed row-match tests, greedy tests, strategy-frame tests,
  `cargo fmt --check`, clippy for all `ruzstd` targets with warnings denied,
  full `cargo test -p ruzstd --quiet`, `git diff --check`, and release rebuilds
  for `benchmark_c_port` and `profile_c_port`.
- Latest kept unsafe-centralization cleanup after the row safe-indexing
  checkpoint: duplicated unaligned little-endian read helpers in fast,
  double-fast, hash-chain, match-count, and row-match code now route through
  `ruzstd/src/encoding/levels/c_port/unaligned.rs`. This keeps the intentional
  unsafe `ptr::read_unaligned` operations in one documented module while
  preserving the existing helper exports used by sibling modules. Remaining
  unsafe occurrences under `c_port` at that checkpoint were the three shared
  unaligned reads plus the row matcher's x86 prefetch/SSE2 row-mask blocks.
  Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs
  and `36,897,920` for 80 runs). Three focused 20-run samples were
  `8,435,434,022`, `8,435,255,036`, and `8,429,928,377` instructions with
  branches around `1.370B`. The focused 80-run Rust sample was
  `33,579,613,394` instructions, `19,120,891,902` cycles, `5,449,333,509`
  branches, and `213,485,253` branch misses. The level-5 affected-path broad
  artifact is
  `benchmarks/tmp/normal-level5-api-runs1-after-unaligned-read-centralization.csv`;
  it is byte-identical to the previous row-safe-indexing artifact and totals
  Rust `1,555,507` bytes versus C API `1,557,737` bytes (`-2,230`, `-0.143%`)
  across 73 fixtures. Validation passed focused fast, double-fast, row-match,
  opt-match, and match-count tests, full `cargo test -p ruzstd --quiet`, clippy
  for all `ruzstd` targets with warnings denied, `cargo fmt --check`, `git diff
  --check`, release rebuilds for `profile_c_port` and `benchmark_c_port`, the
  focused perf samples, and the level-5 broad smoke.
- Latest kept unsafe-boundary cleanup after the unaligned-read centralization:
  the row matcher's x86 prefetch and SSE2 row-mask intrinsics now live in
  `ruzstd/src/encoding/levels/c_port/x86.rs`, so `row_match.rs` has no remaining
  unsafe blocks. The remaining unsafe occurrences under `c_port` are centralized
  in `unaligned.rs` and `x86.rs`. The level-5 affected-path broad artifact
  `benchmarks/tmp/normal-level5-api-runs1-after-x86-helper-isolation.csv` is
  byte-identical to the prior unaligned-read centralization artifact and totals
  Rust `1,555,507` bytes versus C API `1,557,737` bytes (`-2,230`, `-0.143%`)
  across 73 fixtures. Focused `corpus_z000033` level-16 bytes stayed unchanged
  (`9,224,480` for 20 runs); the focused 20-run sample was `8,435,296,554`
  instructions, `5,040,718,936` cycles, `1,370,685,814` branches, and
  `54,651,059` branch misses. Validation passed row-match tests, full
  `cargo test -p ruzstd --quiet`, clippy for all `ruzstd` targets with warnings
  denied, `cargo fmt --check`, `git diff --check`, release rebuilds for
  `profile_c_port` and `benchmark_c_port`, the level-5 broad smoke, and the
  focused perf smoke.
- Latest kept safety-coverage cleanup after the x86 helper isolation:
  `ruzstd/src/encoding/levels/c_port/unaligned.rs` now has direct unit tests for
  little-endian interpretation and unaligned offsets for `read16`, `read32`, and
  `read64`. This covers the centralized unsafe read boundary itself rather than
  only its match-finder callers. Validation passed the focused unaligned-read
  tests, match-count tests, row-match tests, fast tests, full `cargo test -p
  ruzstd --quiet` (`647` passed, `5` ignored), clippy for all `ruzstd` targets
  with warnings denied, `cargo fmt --check`, and `git diff --check`.
- Latest kept x86 safety-coverage cleanup after that: `x86.rs` now has direct
  unit tests for the raw SSE2 row tag mask helper across 16-, 32-, and 64-byte
  row widths, plus a no-match case. This covers the intrinsic wrapper itself
  instead of only the rotated row matcher path. Validation passed the focused
  `row_tag_match_mask`, `row_match_mask`, and `row_match` tests, full `cargo
  test -p ruzstd --quiet` (`649` passed, `5` ignored), clippy for all `ruzstd`
  targets with warnings denied, `cargo fmt --check`, and `git diff --check`.
- Latest kept CPU change after the July 17 resume: non-row
  `GreedyMatchState` chain tables now allocate one extra dummy entry, and the
  no-dictionary optimal binary-tree cleanup writes in `opt_match/tree.rs` use
  that real dummy slot instead of carrying a `TREE_SLOT_NONE` sentinel plus
  conditional final writes. This mirrors C's `dummy32` cleanup sink without
  unsafe pointers. Focused bytes stayed unchanged (`9,224,480` for 20 runs and
  `36,897,920` for 80 runs). Three focused 20-run instruction samples were
  `8,955,119,650`, `8,948,194,542`, and `8,948,996,934`. The focused 80-run
  sample was `35,012,763,886` instructions, `22,943,765,039` cycles,
  `5,715,690,317` branches, and `240,853,109` branch misses. Same-session C
  API was `36,864,480` bytes, `34,718,897,835` instructions, `24,417,110,177`
  cycles, `4,332,511,818` branches, and `178,307,441` branch misses. This
  leaves the focused instruction gap at about `+0.85%`. Final-code profile
  artifact: `benchmarks/tmp/perf-z000033-l16-rust-after-chain-dummy-slot.data`.
  The broad one-run API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-chain-dummy-slot.csv`;
  byte totals are unchanged from the previous resume artifact: level 8 is
  `+277` bytes (`+0.018%`), level 16 is `+666` bytes (`+0.048%`), and level 19
  is `+1057` bytes (`+0.081%`) versus C across 73 fixtures. Validation passed
  `cargo fmt --check`, focused greedy-state/opt-match/opt-parser tests, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, release rebuilds for `profile_c_port`, `profile_c_api`, and
  `benchmark_c_port`, broad byte comparison, and `git diff --check`.
- Rejected after the dummy-slot checkpoint: hoisting `ip + match_length` into a
  local `ip_match` in the two no-dictionary binary-tree loops preserved focused
  bytes (`9,224,480` for 20 runs) but immediately regressed the first focused
  20-run instruction sample to `9,073,101,669` versus the kept `8.95B` band. It
  was reverted; keep the current repeated `ip + match_length` expression shape
  unless a new profile gives stronger evidence.
- Fresh no-edit focused baseline from the July 17 resume: release profiler
  binaries were current, `profile_c_port` emitted `9,224,480` bytes for
  20 runs and `36,897,920` bytes for 80 runs on `corpus_z000033` level 16,
  while `profile_c_api` emitted `9,216,120` bytes for 20 runs and
  `36,864,480` bytes for 80 runs. The 80-run Rust counters were
  `34,995,745,307` instructions, `21,004,415,236` cycles, `5,712,223,476`
  branches, and `238,741,615` branch misses. Same-session C API counters were
  `34,698,563,324` instructions, `19,177,252,159` cycles, `4,328,684,308`
  branches, and `175,513,556` branch misses. The focused instruction gap is
  about `+0.86%`; branch count and cycles remain the more visible residual gap.
- Latest kept CPU change after the fresh no-edit baseline:
  `OptPriceState::update_stats()` now skips the compressed-literal frequency
  loop entirely when `lit_length == 0`, while still updating literal-length,
  offset, and match-length statistics. This preserves the C price model and
  avoids building an empty literal slice for the many zero-literal
  optimal-parser sequences. Focused bytes stayed unchanged (`9,224,480` for
  20 runs and `36,897,920` for 80 runs). Three focused 20-run instruction
  samples were `8,943,075,115`, `8,942,462,061`, and `8,942,590,605`. The
  focused 80-run sample was `34,982,048,359` instructions, `20,555,284,514`
  cycles, `5,708,600,270` branches, and `238,547,710` branch misses.
  Same-session C API was `36,864,480` bytes, `34,697,125,531` instructions,
  `18,794,718,268` cycles, `4,328,280,285` branches, and `175,395,203` branch
  misses. Validation passed `cargo fmt --check`, focused opt-price and
  opt-parser tests, full `cargo test -p ruzstd --quiet`,
  `cargo clippy -p ruzstd --all-targets -- -D warnings`, and a release
  `profile_c_port` rebuild.
- Rejected after the zero-literal `update_stats()` checkpoint: splitting
  `forward_pass()` into a match-collection loop bounded by `ip + cur <= ilimit`
  plus a tail literal-collapse loop preserved focused bytes (`9,224,480` for
  20 runs) but regressed the first focused 20-run instruction sample to
  `9,080,890,905` versus the kept `8.942B` to `8.943B` band. It was reverted;
  keep the current in-loop `if inr > ilimit { cur += 1; continue; }` shape
  unless a fresh profile gives a stronger reason.
- Latest kept CPU change after the zero-literal `update_stats()` checkpoint:
  the two no-dictionary binary-tree branch decisions in `opt_match/tree.rs` now
  load `src[ip + match_length]` into a local `current_byte` before comparing it
  with the candidate match byte. This is narrower than the rejected `ip_match`
  index hoist and keeps the proven match-count and end-check expression shapes.
  Focused bytes stayed unchanged (`9,224,480` for 20 runs and `36,897,920` for
  80 runs). Three focused 20-run instruction samples were `8,918,956,029`,
  `8,917,256,880`, and `8,917,844,516`. The focused 80-run sample was
  `34,907,574,328` instructions, `20,866,673,858` cycles, `5,698,471,717`
  branches, and `239,276,665` branch misses. Same-session C API was
  `36,864,480` bytes, `34,699,413,590` instructions, `18,737,483,139` cycles,
  `4,328,755,062` branches, and `175,530,872` branch misses. Validation passed
  `cargo fmt --check`, focused opt-match and opt-parser tests, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, and a release `profile_c_port` rebuild.
- Latest kept CPU change after the branch-byte `current_byte` checkpoint:
  `Fingerprint::record_hash2()` in `pre_split.rs` now routes the
  `SAMPLING_RATE == 1` case through a safe rolling two-byte value. This
  preserves the exact hash stream for level-16 pre-split fingerprinting while
  avoiding a second adjacent byte reload on every sample. Focused bytes stayed
  unchanged (`9,224,480` for 20 runs and `36,897,920` for 80 runs). Three
  focused 20-run instruction samples were `8,889,753,086`, `8,894,047,177`,
  and `8,890,261,731`. The focused 80-run sample was `34,791,411,780`
  instructions, `21,108,074,835` cycles, `5,639,777,323` branches, and
  `238,545,892` branch misses. Same-session C API was `36,864,480` bytes,
  `34,693,551,015` instructions, `19,048,337,920` cycles, `4,327,672,182`
  branches, and `175,343,827` branch misses. The broad normal API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-after-presplit-rolling.csv`; byte
  totals stayed at level 8 `+277` bytes (`+0.018%`), level 16 `+666` bytes
  (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C across
  73 fixtures. Validation passed `cargo fmt --check`, focused pre-split and
  strategy-frame tests, full `cargo test -p ruzstd --quiet`, `cargo clippy -p
  ruzstd --all-targets -- -D warnings`, release rebuilds for `profile_c_port`,
  `profile_c_api`, and `benchmark_c_port`, and the broad byte comparison.
- Latest kept CPU follow-up after the pre-split rolling hash checkpoint:
  `Fingerprint::merge()` now iterates over `other.events.iter().copied()`
  instead of zipping with `other.events` by value. That avoids copying the
  whole 1024-entry fixed array out of the borrowed fingerprint before the merge
  loop, matching C's read-from-the-other-buffer shape in safe Rust. Focused
  bytes stayed unchanged (`9,224,480` for 20 runs and `36,897,920` for
  80 runs). Three focused 20-run instruction samples were `8,881,179,201`,
  `8,888,140,513`, and `8,886,629,934`. The focused 80-run sample was
  `34,757,167,095` instructions, `21,543,566,925` cycles, `5,638,915,934`
  branches, and `238,294,624` branch misses. Same-session C API was
  `36,864,480` bytes, `34,692,450,623` instructions, `18,757,952,155` cycles,
  `4,327,475,208` branches, and `175,328,491` branch misses. The broad normal
  API artifact is
  `benchmarks/tmp/normal-levels-8-16-19-api-after-presplit-merge-ref.csv`;
  byte totals stayed at level 8 `+277` bytes (`+0.018%`), level 16 `+666`
  bytes (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C across
  73 fixtures. Validation passed `cargo fmt --check`, focused pre-split tests,
  full `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets
  -- -D warnings`, release rebuilds for `profile_c_port`, `profile_c_api`,
  and `benchmark_c_port`, and the broad byte comparison.
- Rejected normal-mode Huffman-depth experiment after the pre-split merge-copy
  checkpoint: enabling the existing C-style optimal-depth Huffman table builder
  for btopt+ normal literal sections found a real compression lever but was too
  expensive in its current Rust form. The all-sections variant improved focused
  `corpus_z000033` level-16 size to `460,015` bytes, beating the C API's
  `460,806` bytes, and broad one-run API totals moved to level 16 `-880`
  bytes and level 19 `-17` bytes versus C. However, focused 20-run
  instructions regressed to `9,580,409,250` versus the kept `8.88B` band and
  same-session C at `8,714,318,613`. A sparse-alphabet gate still produced
  `460,940` bytes and `9,249,696,103` instructions. Both variants were
  reverted. Do not re-enable normal-mode optimal Huffman depth without first
  making `build_c_optimal_depth_from_counts()` much cheaper or proving a
  narrower gate.
- Follow-up kept after that rejection:
  `HuffmanTable::build_c_optimal_depth_from_counts()` now builds the base
  Huffman tree once and reuses it while probing candidate depths, instead of
  rebuilding the sorted tree for every table log. A regression test compares the
  optimized path with the old rebuild-each-depth behavior. This preserves bytes
  and helps target/superblock code that already uses optimal depth. Re-testing
  the full normal-mode optimal-depth dispatch after this change still produced
  `460,015` bytes on focused `corpus_z000033` level 16, but focused 20-run
  instructions were still `9,417,604,355` versus same-session C at
  `8,795,793,446` and the kept Rust baseline near the `8.88B` band. The normal
  dispatch experiment was reverted again; keep only the builder reuse
  optimization.
- Rejected follow-ups after the pre-split merge-copy checkpoint: changing
  `record_hash2_rate1()` to process the final sample outside the loop removed
  the inner `if pos < limit` branch and preserved focused bytes, but did not
  beat the kept 80-run checkpoint. Three focused 20-run samples were
  `8,885,351,925`, `8,887,788,972`, and `8,881,763,152`; the focused 80-run
  sample was `34,761,274,300` instructions, `21,895,797,871` cycles,
  `5,638,591,834` branches, and `239,446,970` branch misses versus the kept
  `34,757,167,095` instruction checkpoint. It was reverted. Changing
  `fingerprint_distance()` from its range iterator/map/sum shape to an
  explicit `while` loop also preserved bytes but immediately regressed focused
  20-run instruction samples to `8,899,106,681` and `8,904,636,616`, with
  higher branch misses. It was reverted; keep the current iterator distance
  shape unless a new profile gives stronger evidence.
- Rejected literal-statistics follow-up after the pre-split merge-copy
  checkpoint: changing `LiteralStats::from_literals_with_stream_counts()` from
  `literals.chunks(split_size).enumerate()` to an explicit four-stream
  start/end loop preserved focused bytes but regressed the focused 20-run
  instruction/branch band. Samples were `8,896,159,806` and `8,892,897,905`
  instructions, with branches around `1.456B`, versus the kept `8.881B` to
  `8.888B` instruction band and about `1.449B` to `1.451B` branches. It was
  reverted; keep the chunk iterator shape unless a new profile gives stronger
  evidence.
- Rejected Huffman stream follow-up after the same checkpoint: changing
  `HuffmanEncoder::encode_stream()` from `data.iter().rev()` to a counted
  reverse index loop preserved focused bytes but did not improve the focused
  band. Samples were `8,886,757,164`, `8,888,734,628`, and `8,884,567,000`
  instructions, with branch misses around `64.1M` to `64.4M`. It was reverted;
  keep the reversed slice iterator unless a new profile gives stronger
  evidence.
- Rejected after the branch-byte `current_byte` checkpoint: also loading
  `src[current_match_index + match_length]` into a local `match_byte` preserved
  focused bytes but did not improve the current focused instruction band. Three
  focused 20-run samples were `8,917,151,713`, `8,917,523,902`, and
  `8,922,681,533` versus the kept `8.917B` to `8.919B` band. It was reverted;
  keep only the `current_byte` local unless a new profile gives stronger
  evidence.
- Rejected after the same checkpoint: adding an
  `OptPriceState::update_stats()` repeat-code fast path that mapped offBase
  values 1-3 directly to offset codes preserved focused bytes but regressed the
  focused 20-run instruction band. Samples were `8,918,791,897`,
  `8,922,373,720`, and `8,921,882,692` versus the kept `8.917B` to `8.919B`
  band. It was reverted; keep the direct `highbit32(off_base)` update-stats
  shape unless a fresh profile points back there.
- Rejected after the same checkpoint: adding direct BtOpt/BtUltra
  const-specialized dynamic price helpers and routing `forward_pass()` through
  them preserved focused bytes (`9,224,480` for 20 runs) but regressed the
  focused 20-run instruction band. Samples were `8,919,964,991`,
  `8,921,688,733`, and `8,921,417,615` versus the kept `8.917B` to `8.919B`
  band. It was reverted; keep the current `OptLevel`-threaded dynamic helper
  shape unless a fresh profile gives stronger evidence.
- Rejected after the same checkpoint: changing
  `HuffmanEncoder::write_table()` to emit the already-cached table description
  with `BitWriter::append_bytes()` instead of byte-by-byte `write_bits()`
  preserved focused bytes (`9,224,480` for 20 runs) but did not produce a
  defensible CPU win. The append-bytes samples were `8,922,544,583`,
  `8,923,295,990`, and `8,920,131,310`; post-revert samples were
  `8,925,040,596`, `8,922,236,879`, and `8,917,646,679`. Keep the cached
  table-description data, but emit it through `write_bits()` unless a fresh
  profile points back to this path.
- Rejected C-parity probe after the same checkpoint: clamping the
  no-dictionary binary-tree `window_low`/`match_low` values to at least `1`, to
  mimic C's raw hash-table zero sentinel, is not valid with Rust's `index + 1`
  stored-value layout. It breaks the existing `opt_match` source-index-zero
  and repcode boundary tests, so keep the current `window_low` values unless
  the table storage representation is deliberately redesigned.
- Latest tiny CPU cleanup after the loaded-bound specialization: the no-LDM
  optimal parser call into `forward_pass()` now passes `None` directly, and
  only the LDM specialization reborrows `ldm_cursor.as_deref_mut()`. Focused
  bytes stayed unchanged (`9,224,480` for 20 runs and `36,897,920` for
  80 runs). Three focused 20-run samples were `9.073B`, `9.070B`, and
  `9.069B` instructions. The focused 80-run sample was `35,509,335,228`
  instructions, `21,775,128,903` cycles, `5,825,740,531` branches, and
  `239,986,812` branch misses. Treat this as neutral-to-tiny cleanup rather
  than a material gap closure. Final-code profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-forward-ldm-cursor-boundary.data`.
  Validation passed focused opt-parser/opt-match tests, full `cargo test -p
  ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`, and
  a release `profile_c_port` rebuild.
- Rejected after the forward LDM-cursor boundary checkpoint: adding separate
  code-only LL/ML lookup tables for `literal_length_code()` and
  `match_length_code()` in `sequence_codes.rs` preserved focused bytes
  (`9,224,480` for 20 runs) but regressed focused 20-run instruction samples to
  `9,145,930,272` and `9,148,757,349` versus the kept `9.069B` to `9.073B`
  band. It was reverted; keep the current tuple-table helper shape unless a new
  profile gives stronger evidence.
- Rejected after the same checkpoint: adding an `OptMatchTable::push_parts()`
  helper and routing the no-dictionary binary-tree match recording through
  direct field writes preserved focused bytes but did not improve the current
  CPU band. Three focused 20-run samples were `9,070,970,688`,
  `9,066,929,693`, and `9,080,035,484`; the focused 80-run sample was
  `35,515,458,633` instructions versus the kept `35,509,335,228`. It was
  reverted; keep the current `matches.push(OptMatch { .. })` shape unless a new
  profile says otherwise.
- Rejected after that: changing normal `select_path()` reconstruction to write
  the selected forward path in-place into `state.opt`, while keeping the
  large-match scratch path unchanged, preserved focused bytes (`9,224,480` for
  20 runs and `36,897,920` for 80 runs) but regressed two focused 20-run
  instruction samples to `9,087,576,512` and `9,080,310,676` versus the kept
  `9.069B` to `9.073B` band. It was reverted; keep the separate `path` plus
  `path.reverse()` shape unless a new profile gives stronger evidence.
- Fresh resume profile and broad benchmark after those rejections: the focused
  `corpus_z000033` level-16 80-run smoke still emits `36,897,920` bytes. Fresh
  Rust counters were `35,509,474,210` instructions, `22,296,520,000` cycles,
  `5,825,157,267` branches, and `240,860,614` branch misses. Fresh profile
  artifact: `benchmarks/tmp/perf-z000033-l16-rust-resume-20260717.data`. The
  delegated broad one-run API benchmark artifact is
  `benchmarks/tmp/agent-validation-resume-20260717.csv`: level 8 is `+277`
  bytes (`+0.018%`), level 16 is `+666` bytes (`+0.048%`), and level 19 is
  `+1057` bytes (`+0.081%`) versus C across 73 real-world fixtures. Worst
  positive gaps remain `corpus_z000033`: `+319`, `+418`, and `+724` bytes at
  levels 8, 16, and 19 respectively. Treat the rounded broad CPU seconds as
  noisy; use focused perf counters for CPU conclusions.
- Latest normal-mode CPU checkpoint after the no-dict repcode specialization:
  the optimal parser now also carries a compile-time `LOADED_DICT`
  specialization. Normal no-dictionary/no-loaded frames use the plain
  window-low calculation in the binary-tree update and match-collection loops,
  while dictionary and ext-dict paths still use the loaded-dictionary bound
  logic. Focused `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480`
  for 20 runs and `36,897,920` for 80 runs). Three focused 20-run instruction
  samples were `9.074B`, `9.072B`, and `9.070B`. The focused 80-run sample was
  `35,509,415,821` instructions, `21,825,122,189` cycles, `5,825,392,946`
  branches, and `240,285,767` branch misses. Same-session C API was
  `36,864,480` bytes, `34,714,791,138` instructions, `24,165,225,995` cycles,
  `4,331,728,208` branches, and `177,645,328` branch misses. Final-code
  profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-loaded-bound-specialized.data`.
  Validation passed focused opt-match, opt-parser, dictionary, and opt-frame
  tests, full `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd
  --all-targets -- -D warnings`, and a release `profile_c_port` rebuild.
- Rejected after the loaded-bound specialization: making repcode collection
  return the best-possible stop decision directly, instead of calling
  `should_stop_after_best_match()` after collection, preserved focused bytes but
  regressed focused 20-run instruction samples to about `9.134B` to `9.137B`
  versus the kept `9.070B` to `9.074B` band. It was reverted; keep the current
  helper shape unless a new profile gives a stronger reason.
- Latest normal-mode CPU checkpoint from July 17, 2026: the optimal parser now
  threads a compile-time no-dict/ext-dict split into match collection, so the
  no-dictionary binary-tree collector calls `collect_repcode_matches_no_dict()`
  directly while ext-dict compression still uses the generic
  `collect_repcode_matches()` path. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9,224,480` for 20 runs and
  `36,897,920` for 80 runs). Final focused 20-run instruction samples were
  `9.113B` and `9.109B`, versus a fresh pre-change `9.408B` sample. The
  focused 80-run sample was `35,672,501,984` instructions, `26,222,223,890`
  cycles, `5,792,228,377` branches, and `243,530,382` branch
  misses. Same-session C API was `36,864,480` bytes, `34,714,927,527`
  instructions, `23,128,216,291` cycles, `4,331,862,481` branches, and
  `177,059,972` branch misses. Final-code profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-nodict-repcode-specialized.data`.
  Validation passed `cargo fmt --check`, `cargo test -p ruzstd opt_match
  --quiet`, `cargo test -p ruzstd opt_parser --quiet`, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, `git diff --check`, and a release `profile_c_port` rebuild.
- Latest targetCBlockSize checkpoint from July 17, 2026: broad one-run API
  comparisons across the 73-fixture real-world corpus are byte-identical for
  levels 13, 16, and 19 at target sizes 2048, 4096, and 8192. Artifacts:
  `benchmarks/tmp/target-cblock-2048-levels-13-16-19-after-c-huf-sort.csv`,
  `benchmarks/tmp/target-cblock-4096-levels-13-16-19-after-c-huf-sort.csv`,
  and
  `benchmarks/tmp/target-cblock-8192-levels-13-16-19-after-c-huf-sort.csv`.
  All nine target/level combinations report `rows=73`, `differing=0`,
  `positive=0`, and total byte gap `+0` against the C API. These one-run CPU
  numbers are noisy and should not be used as the CPU parity conclusion.
- The latest target-block ordering fixes are in `target_block.rs`. Target mode
  now tries repeat/treeless Huffman literals with the selected sequence modes,
  which closes the prior positive `corpus_z000044` level-19 target gap. The
  pure repeat-literal/all-repeat sequence fallback must run after the fresh
  compressed-literal/all-compressed sequence candidate; running it earlier made
  Rust smaller than C on `corpus_z000033` level 19 at target sizes 4096 and
  8192 by using repeat FSE tables where C writes fresh FSE tables.
- The final `corpus_z000022` level-19 target 8192 mismatch was a Huffman
  literal payload parity issue. It was closed by making Rust's Huffman tree
  builder use C's `HUF_sort()` bucket ordering: C keeps symbol order for small
  exact-count buckets, but larger log buckets use an unstable quicksort by
  count only. Rust had been sorting all equal counts by symbol, which produced
  the same weight histogram but swapped four symbol weights in the superblock
  Huffman table. The regression test is
  `huffman_sort_matches_c_log_bucket_tie_order`. Inspector artifact
  `benchmarks/tmp/inspect-target8192-l19-corpus-z000022-after-c-huf-sort.txt`
  reports `content_delta=0`, `source_delta=0`, `type_diffs=0`, and
  `first_diff=none`.
- Latest normal-mode CPU checkpoint from July 17, 2026: after the C
  `HUF_sort()` parity fix, `c_sorted_huffman_nodes()` now allocates and fills
  only the nonzero node prefix while keeping C-compatible bucket positions.
  Focused `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), target-mode parity stayed exact, and
  `perf stat` on the focused 80-run normal-mode smoke moved from
  `49,548,126,529` instructions before the cleanup to `48,728,786,295` after
  it. The profile artifact is
  `benchmarks/tmp/perf-z000033-l16-rust-after-nonzero-huf-sort.data`.
- Latest CPU follow-up: `opt_match/tree.rs` now keeps the binary-tree link
  walks in the C table representation (`0` means empty, otherwise `index + 1`)
  and subtracts one only after confirming a non-empty slot. This avoids
  constructing `Option<usize>` in the two hot tree loops while preserving the
  source-index-zero parity fix. Focused `corpus_z000033` level-16 bytes stayed
  at `9198420` for 20 runs and `36793680` for 80 runs. Same-session 20-run
  instruction samples moved from `12.376B` and `12.378B` before the change to
  `12.325B`, `12.325B`, and `12.325B` after it. Target parity smoke
  `corpus_z000022` level 19 target 8192 still matches the C API at `78067`
  bytes. Validation passed `cargo fmt --check`, `cargo test -p ruzstd
  opt_match --quiet`, `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd
  --all-targets -- -D warnings`, and `git diff --check`. A same-session A/B of
  adding `#[inline(always)]` to `hash_chain_match::highbit32()` was neutral
  (`12.377B`, `12.381B`, `12.376B` with the attribute versus `12.376B`,
  `12.378B` without), so the attribute was not kept.
- Fresh follow-up profile and broad benchmark after the tree-link change:
  `benchmarks/tmp/perf-z000033-l16-rust-after-tree-sentinel.data` is the current
  focused Rust profile. It still shows `forward_pass` and
  `compress_block_opt_with_state_and_ldm` as the dominant CPU costs. The broad
  normal API benchmark artifacts are
  `benchmarks/tmp/normal-levels-8-16-19-api-runs3-after-tree-sentinel.csv` and
  `.md`. Across 73 real-world fixtures, size totals versus C API remain at
  practical parity: level 8 Rust `-560` bytes (`-0.036%`), level 16 Rust
  `-967` bytes (`-0.070%`), and level 19 Rust `-155` bytes (`-0.012%`). Worst
  positive byte gaps remain tiny: `+6` at level 8, `+4` at level 16, and `+5`
  at level 19. The three-run broad CPU seconds are rounded and noisy, so use
  focused perf counters for CPU conclusions. Current focused 80-run
  `corpus_z000033` level-16 counters: Rust `48,529,866,636` instructions,
  `27,729,170,739` cycles, `8,365,254,048` branches, and `279,943,935` branch
  misses; C API `34,677,320,506` instructions, `20,406,964,727` cycles,
  `4,332,735,580` branches, and `175,791,305` branch misses. This is progress
  from the previous Rust `48,728,786,295` instruction checkpoint, but the
  focused CPU gap is still real and the next work should stay on parser /
  match-finder instruction count rather than target-mode byte parity.
- Latest kept CPU change after the tree-link checkpoint: sequence C-cost table
  selection now reuses the already-computed max symbol when calculating entropy,
  predefined, and repeat costs. This avoids repeatedly scanning/filtering all
  256 code slots inside `choose_c_cost_table()` without reintroducing the
  rejected `CodeCounts::max_symbol` field in the per-code counting path.
  Focused `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for
  20 runs, `36793680` for 80 runs). Three focused 20-run instruction samples
  were `12.196B`, `12.201B`, and `12.200B`; two focused 80-run samples were
  `48,030,727,202` and `48,086,528,143` instructions. Target parity smoke
  `corpus_z000022` level 19 target 8192 still matches the C API at `78067`
  bytes. The broad normal API artifacts are
  `benchmarks/tmp/normal-levels-8-16-19-api-runs3-after-seq-cost-max-symbol.csv`
  and `.md`; their C/Rust byte fields match the previous tree-sentinel artifact
  exactly, with the same aggregate gaps (`-560`, `-967`, `-155` bytes for
  levels 8, 16, and 19). Validation passed `cargo fmt --check`, focused
  sequence and compressed tests, `cargo test -p ruzstd --quiet`, `cargo clippy
  -p ruzstd --all-targets -- -D warnings`, and `git diff --check`.
- Follow-up FSE/Huffman cleanup after the sequence-cost max-symbol change:
  `build_huffman_weight_table_from_data()` now uses `data.len()` for the total
  weight count instead of summing the already-counted nonzero prefix. This
  preserves behavior while removing a small redundant scan in the Huffman
  weight FSE path. Focused `corpus_z000033` level-16 bytes stayed unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs). Three focused 20-run
  instruction samples were `12.195B`, `12.193B`, and `12.191B`; the focused
  80-run sample was `48,036,223,779` instructions. Target parity smoke
  `corpus_z000022` level 19 target 8192 still matches the C API at `78067`
  bytes. Validation passed `cargo fmt --check`, `cargo test -p ruzstd fse
  --quiet`, `cargo test -p ruzstd huff --quiet`, `cargo test -p ruzstd
  --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`, and
  `git diff --check`.
- Follow-up Huffman stream-count reuse: `HuffmanTable` now has a
  `build_smallest_from_counts_with_stream_counts()` entry point so the
  compressed literal encoder and estimator can reuse the four-stream literal
  counts they already computed, instead of recounting the same literal streams
  inside `build_smallest_from_counts()`. The old data-based wrapper is now
  test-only. Focused `corpus_z000033` level-16 bytes stayed unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs). Three focused 20-run
  instruction samples were `12.139B`, `12.141B`, and `12.141B`; the focused
  80-run sample was `47,817,650,572` instructions. Current profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-stream-count-reuse.data`. The
  one-run broad API artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-stream-count-reuse.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs3-after-seq-cost-max-symbol.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target parity smoke `corpus_z000022` level 19 target
  8192 still matches the C API at `78067` bytes. Validation passed `cargo
  fmt --check`, focused Huffman and compressed tests, `cargo test -p ruzstd
  --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`, and
  `git diff --check`.
- Rejected follow-up parser cache A/B: forcing `#[inline(always)]` on
  `LiteralPriceCache::begin_pass()`, `lookup()`, and `insert()` preserved
  focused bytes but did not improve the current instruction band (`12.193B`,
  `12.196B`, `12.192B` for 20-run samples versus the preceding `12.195B`,
  `12.193B`, `12.191B` band). It was reverted; do not retry without a
  different profile signal.
- Follow-up rank-limited Huffman redistribution cleanup: the sorted
  `rank_limited_nonzero_weights()` buffer is now reduced by locating the first
  entry in the highest eligible weight bucket with `partition_point()`, instead
  of rescanning all weights plus a parallel unit table for every reduction
  step. The tie-break is the same first-entry-in-bucket choice as the old full
  scan, and the sorted order remains intact after each decrement. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs). Three focused 20-run instruction samples were
  `11.695B`, `11.692B`, and `11.689B`; the focused 80-run sample was
  `46,036,337,077` instructions. Current profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-rank-limited-bucket-reduce.data`;
  `rank_limited_nonzero_weights()` dropped to about `0.81%` self time in that
  profile. The one-run broad API artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-rank-limited-bucket-reduce.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-stream-count-reuse.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target parity smoke `corpus_z000022` level 19 target
  8192 still matches the C API at `78067` bytes. Validation passed `cargo
  fmt --check`, focused Huffman and compressed tests, `cargo test -p ruzstd
  --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`, and
  `git diff --check`.
- Follow-up FSE direct-lookup cast refresh: the nested lookup fill in
  `build_table_from_probabilities()` now uses a debug-asserted
  `state_idx as u16` instead of a release `u16::try_from()` branch, matching
  the documented kept shape for this hot bounded index. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs). Three focused 20-run instruction samples were
  `11.685B`, `11.687B`, and `11.687B`; the focused 80-run sample was
  `46,005,243,303` instructions. Current profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-fse-lookup-cast-refresh.data`.
  Target parity smoke `corpus_z000022` level 19 target 8192 still matches the C
  API at `78067` bytes. Validation passed `cargo fmt --check`, focused FSE
  tests, `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets
  -- -D warnings`, and `git diff --check`.
- Follow-up Huffman duplicate-candidate skip: when the base Huffman lengths
  already fit `MAX_HUFFMAN_BITS`,
  `build_smallest_from_counts_with_stream_counts()` now builds that base
  candidate directly and only searches candidate `max_bits` below the base
  tree's largest bit length. `base_code_lengths()` carries that largest bit
  length out with the lengths and sorted symbols so the caller does not need a
  second symbol scan. Larger candidate limits reproduce the same base table, so
  this avoids duplicate length-vector/symbol-vector cloning and duplicate
  Huffman table construction without changing the serialized table-description
  path. Focused `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for
  20 runs, `36793680` for 80 runs). Three final focused 20-run instruction
  samples were `11.337B`, `11.339B`, and `11.341B`; the focused 80-run sample
  was `44,626,089,276` instructions, `26,884,991,446` cycles,
  `7,373,300,245` branches, and `278,152,360` branch misses. Current profile
  artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-huffman-base-largest-bits.data`.
  The broad one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-huffman-base-largest-bits.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-forward-loop-bound.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target smoke `corpus_z000022` level 19 target 8192
  still matches the C API at `78067` bytes. Validation passed `cargo fmt
  --check`, focused Huffman/parser tests, full `cargo test -p ruzstd --quiet`,
  a release `profile_c_port` rebuild, broad byte comparison, and the target
  smoke.
- Follow-up match-bound threading cleanup:
  `insert_bt_and_get_all_matches_no_dict()` already computes the C window low
  bound before repcode collection, so `collect_repcode_matches()` now accepts
  that `window_low` instead of recomputing the same loaded-dictionary/window
  bound for the same `ip`. This is a small C-shape cleanup in the hot
  no-dictionary optimal match path. Focused `corpus_z000033` level-16 bytes
  stayed unchanged (`9198420` for 20 runs, `36793680` for 80 runs). Three
  focused 20-run samples were `11.341B`, `11.337B`, and `11.338B`; the focused
  80-run sample was `44,619,350,625` instructions, `26,000,036,427` cycles,
  `7,371,035,905` branches, and `276,500,385` branch misses. Current profile
  artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-match-window-low-threading.data`.
  The broad one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-match-window-low-threading.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-huffman-base-largest-bits.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target smoke `corpus_z000022` level 19 target 8192
  still matches the C API at `78067` bytes. Validation passed `cargo fmt
  --check`, focused opt-match/parser tests, full `cargo test -p ruzstd
  --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`, a release
  `profile_c_port` rebuild, broad byte comparison, `git diff --check`, and the
  target smoke.
- Follow-up post-split entropy-estimate cleanup: post-split size estimation now
  calls `BlockCompressionConfig::for_c_block_split_estimate()` before estimating
  a candidate partition. This only changes the split-estimation path; emitted
  partitions keep the original block config. The reason is C-specific: at level
  16 (`btopt`), `ZSTD_buildBlockEntropyStats_literals()` runs without
  `HUF_flags_optimalDepth`, so the block splitter builds a normal Huffman table
  per estimate instead of doing Rust's file-type small-Huffman-table search for
  each small partition. Focused `corpus_z000033` level-16 bytes moved closer to
  C (`36,812,080` Rust bytes for 80 runs versus `36,864,480` C API bytes).
  Focused 80-run counters improved to `39,191,745,940` instructions,
  `23,451,212,641` cycles, `6,336,575,376` branches, and `249,372,902` branch
  misses; same-session C API counters were `34,710,580,784` instructions,
  `21,593,230,051` cycles, `4,331,183,411` branches, and `177,317,409` branch
  misses. The broad one-run API artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-c-split-estimate-huffman-rebuilt.csv`
  reports aggregate byte gaps of level 8 `-560`, level 16 `-737`, and level 19
  `-58` bytes versus C, with worst positive gap `+7` bytes on
  `corpus_z000050` level 19. Validation passed `cargo fmt --check`, full
  `cargo test -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D
  warnings`, `git diff --check`, and release rebuilds of `profile_c_port`,
  `profile_c_api`, and `benchmark_c_port`.
- Follow-up C-strategy Huffman-search cleanup: `BlockCompressionConfig::for_c_strategy()`
  now uses `HuffmanTableSearch::Heuristic` instead of `FileTypeSmall`. This is
  closer to C's strategy behavior: C only enables `HUF_flags_optimalDepth` for
  `btultra` and above, so level 16 (`btopt`) should not run the Rust
  file-type small-Huffman-table search for emitted blocks. Focused
  `corpus_z000033` level-16 output is now slightly larger than C instead of
  smaller: Rust `36,897,920` bytes for 80 runs versus C API `36,864,480`.
  Focused 80-run counters improved to `36,834,241,992` instructions,
  `22,029,444,650` cycles, `5,894,194,022` branches, and `239,865,041` branch
  misses, leaving Rust about 6.1% over the same-session C API
  `34,710,580,784` instruction sample. The broad normal one-run API artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-c-strategy-heuristic-huffman.csv`
  reports level 8 `+277` bytes (`+0.018%`), level 16 `+666` bytes
  (`+0.048%`), and level 19 `+1057` bytes (`+0.081%`) versus C. Target mode
  stayed byte-identical across target sizes 2048, 4096, and 8192 at levels 13,
  16, and 19; each target CSV reports `rows=73`, `differing=0`, `positive=0`,
  and `total_gap=+0`. Fresh profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-c-strategy-heuristic-huffman.data`.
  Top self costs are again the optimal parser: `forward_pass` about 36.0% and
  `compress_block_opt_with_state_and_ldm` about 29.8%, followed by much smaller
  costs in `base_code_lengths`, pre-split fingerprinting, literal stats, FSE
  table building, `OptPriceState::update_stats`, and sequence encoding.
  Validation passed `cargo fmt --check`, full `cargo test
  -p ruzstd --quiet`, `cargo clippy -p ruzstd --all-targets -- -D warnings`,
  `git diff --check`, focused compressed/price/match tests, and release
  rebuilds of `profile_c_port` and `benchmark_c_port`.
- Rejected follow-up after the C-strategy Huffman-search cleanup: changing
  `hash_chain_match::highbit32()` from the current
  `u32::BITS - 1 - value.leading_zeros()` form to `value.ilog2()` preserved
  the new focused bytes (`9,224,480` for 20 runs) but regressed focused
  instruction samples to `10.255B` and `10.257B` for 20 runs, versus the
  current `9.400B` band after the C-strategy Huffman-search change. It was
  reverted; do not retry the `ilog2()` highbit shape without a new profile
  signal.
- Rejected follow-up after the match-bound cleanup: threading the already
  computed `sufficient_len` through `collect_matches_mls()` and
  `BtMatchRequest` into `insert_bt_and_get_all_matches_no_dict()` preserved
  focused bytes but regressed the current focused instruction band. Three
  20-run samples were `11.347B`, `11.351B`, and `11.341B`; the 80-run sample
  was `44,637,165,017` instructions versus the current kept
  `44,619,350,625`. It was reverted; the larger request payload costs more
  than recomputing the capped target length in the tree collector.
- Rejected follow-up after the match-bound cleanup: changing
  `build_table_from_probabilities()` to use an explicit loop for the
  negative-probability distribution pass preserved focused bytes but did not
  beat the current focused instruction band. Three 20-run samples were
  `11.340B`, `11.339B`, and `11.341B`. It was reverted; do not retry this FSE
  table-builder loop shape without a new profile signal.
- Rejected follow-up after the match-bound cleanup: changing
  `LiteralPriceCache::lookup()` from `then_some(self.prices[idx])` to an
  explicit branch preserved focused bytes and had neutral 20-run samples
  (`11.342B`, `11.337B`, `11.339B`), but the 80-run sample regressed to
  `44,631,142,272` instructions versus the kept `44,619,350,625`. It was
  reverted; keep the current cache lookup shape unless a new profile points
  back there.
- Rejected follow-up after the select-path direct-last checkpoint: adding
  indexed `LiteralPriceCache` accessors and computing the literal table index
  once in `raw_literal_cost()` preserved focused bytes but did not beat the
  current focused CPU band. Three 20-run samples were `11.334B`, `11.337B`,
  and `11.336B`; the focused 80-run sample regressed to `44,611,877,335`
  instructions versus the kept `44,599,131,257`. It was reverted; keep the
  current literal cache `lookup(literal)` / `insert(literal, price)` shape
  unless a new profile gives a stronger reason.
- Rejected follow-up after the match-bound cleanup: replacing the FSE
  table-builder single-state checks from `state.states.len() == 1` to the
  already-normalized `prob == 1` preserved focused bytes but worsened
  sequential 20-run instruction samples to `11.352B` and `11.346B` versus the
  kept `11.337B` to `11.341B` band. It was reverted; keep the current
  `Vec::len()`-based checks unless a new profile gives a stronger reason.
- Rejected follow-up after the select-path direct-last checkpoint: rewriting
  `pop_smallest_huffman_node()` from the current `Option`/`match` shape to
  direct availability branches preserved focused bytes and had neutral 20-run
  samples (`11.334B`, `11.339B`, `11.334B`, `11.332B`), but two focused 80-run
  samples regressed to `44,607,199,252` and `44,610,423,941` instructions
  versus the kept `44,599,131,257`. It was reverted; keep the current Huffman
  pop helper unless a new profile points clearly at it.
- Rejected follow-up after the same checkpoint: changing the hot
  `update_match_prices()` endpoint write from the current struct update with
  `..state.opt[pos]` to direct `price`/`off`/`mlen`/`litlen` field mutation
  preserved focused bytes but worsened 20-run instruction samples to
  `11.341B`, `11.336B`, and `11.338B` versus the kept `11.331B` to `11.339B`
  band. It was reverted; keep the current struct update shape unless a new
  profile points back to this exact write.
- Rejected follow-up after the select-path direct-last checkpoint: changing
  the `update_match_prices()` downward match-length scan from the current
  `while match_len >= start_len` plus bottom equality break to a `loop` guarded
  only by the bottom equality check preserved focused bytes but clearly
  regressed the 20-run instruction band. Samples were `11.394B`, `11.387B`,
  and `11.390B` versus the kept `11.331B` to `11.339B` band. It was reverted;
  keep the current scan loop unless a new profile gives a stronger reason.
- Rejected follow-up after the select-path direct-last checkpoint: projecting
  `state.price_state`, `state.matches`, and `state.opt` into local borrows
  inside `update_match_prices()` preserved focused bytes but did not improve the
  focused CPU band. Three 20-run samples were `11.340B`, `11.339B`, and
  `11.333B`; the focused 80-run sample regressed to `44,622,960,223`
  instructions versus the kept `44,599,131,257`. It was reverted; keep the
  current direct `state.*` access shape unless a new profile gives a stronger
  reason.
- Rejected follow-up after the same checkpoint: changing
  `pre_split::Fingerprint::merge()` from `zip(other.events)` to
  `zip(other.events.iter().copied())` preserved focused bytes but did not
  improve the focused CPU band. Three 20-run samples were `11.335B`,
  `11.336B`, and `11.338B`; the 80-run sample regressed to
  `44,608,752,112` instructions versus the kept `44,599,131,257`. It was
  reverted; keep the current merge shape unless a new profile points back
  there.
- Latest kept parser follow-up: the no-LDM optimal parser path now calls
  `collect_matches_no_ldm_mls()` and only the LDM path calls
  `collect_matches_with_ldm_mls()`, avoiding the hot-path
  `Option<&mut LdmOptCursor>` reborrow/plumbing in normal compression while
  preserving the existing LDM behavior. Focused `corpus_z000033` level-16 bytes
  stayed unchanged (`9198420` for 20 runs, `36793680` for 80 runs). Three
  focused 20-run instruction samples were `11.333B`, `11.335B`, and `11.338B`;
  the focused 80-run sample was `44,604,623,552` instructions,
  `25,155,228,543` cycles, `7,367,765,426` branches, and `274,272,622` branch
  misses. Current profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-no-ldm-match-specialization.data`.
  The broad one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-no-ldm-match-specialization.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-match-window-low-threading.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target smoke `corpus_z000022` level 19 target 8192
  still matches the C API at `78067` bytes. Validation passed `cargo fmt
  --check`, focused parser/LDM/FSE tests, full `cargo test -p ruzstd --quiet`,
  `cargo clippy -p ruzstd --all-targets -- -D warnings`, release profiler and
  benchmark rebuilds, broad byte comparison, `git diff --check`, and the target
  smoke.
- Latest kept path reconstruction follow-up: `select_path()` now updates the
  last path entry by direct index after the initial stretch has been pushed,
  instead of checking `path.last_mut()` on each reverse traversal step. This is
  a small safe step toward C's in-place `_shortestPath` reconstruction while
  keeping the existing validated output order and final `path.reverse()` shape.
  Focused `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for
  20 runs, `36793680` for 80 runs). Three focused 20-run instruction samples
  were `11.339B`, `11.331B`, and `11.335B`; the focused 80-run sample was
  `44,599,131,257` instructions, `25,202,115,212` cycles, `7,367,593,524`
  branches, and `279,163,848` branch misses. Current profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-select-path-direct-last.data`.
  The broad one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-select-path-direct-last.csv`
  has byte fields identical to
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-no-ldm-match-specialization.csv`;
  aggregate byte gaps remain level 8 `-560`, level 16 `-967`, and level 19
  `-155` bytes against C. Target smoke `corpus_z000022` level 19 target 8192
  still matches the C API at `78067` bytes. Validation passed `cargo fmt
  --check`, focused parser tests, full `cargo test -p ruzstd --quiet`,
  `cargo clippy -p ruzstd --all-targets -- -D warnings`, release profiler and
  benchmark rebuilds, broad byte comparison, `git diff --check`, and the target
  smoke.
- Rejected follow-up from the July 17 resume: changing the initial
  `seed_match_prices()` endpoint writes from the current struct update to
  direct field mutation preserved focused bytes but did not improve the current
  focused instruction band. Three 20-run samples were `11.689B`, `11.687B`,
  and `11.683B`; the 80-run sample was `46,006,881,364` instructions versus
  the current `46,005,243,303` checkpoint. It was reverted; do not retry
  without a new profile signal.
- Rejected follow-up after the forward-loop-bound checkpoint: replacing
  `ForwardResult.last_stretch: Option<Optimal>` with a C-like `lastStretch`
  plus valid flag preserved focused bytes but did not improve the current
  focused CPU band. Three 20-run samples were `11.650B`, `11.649B`, and
  `11.654B`; the 80-run sample was `45,869,289,435` instructions versus the
  current kept `45,865,640,102` checkpoint. It was reverted; do not retry
  without a new profile signal.
- Rejected follow-up after the same checkpoint: adding direct BtOpt
  dynamic-price helpers in `OptPriceState` and routing the hot `forward.rs`
  price calls through const-generic BtOpt/BtUltra dispatch preserved focused
  bytes but regressed the current focused instruction band. Three 20-run
  samples were `11.728B`, `11.729B`, and `11.727B` instructions versus the
  current `11.65B` band. It was reverted; do not retry this specialization
  without a new profile signal.
- Fresh July 17 profile comparison after that rejection:
  `benchmarks/tmp/perf-z000033-l16-rust-current.data` and
  `benchmarks/tmp/perf-z000033-l16-c-api-current.data` were recorded with the
  same focused `corpus_z000033` level-16 80-run command. Current counters:
  Rust `36,793,680` bytes, `46.0B` instructions, `25.5B` cycles, `7.84B`
  branches, and `276.8M` branch misses; C API `36,864,480` bytes, `34.7B`
  instructions, `19.3B` cycles, `4.33B` branches, and `175.7M` branch misses.
  The C profile shows `ZSTD_compressBlock_opt0`, and Rust level dispatch tests
  confirm level 16 maps to `BtOpt`, so the focused CPU gap is not a wrong
  BtOpt/BtUltra strategy selection. Continue with binary-tree match collection
  / parser branch-count investigation; `count_match_no_dict()` loop variants
  are already documented as tried.
- Rejected follow-up after the fresh Rust/C profile: changing
  `opt_match/tree.rs` relinks from `stored_value(current_match_index)` to the
  already-loaded `match_index as u32` preserved focused bytes but regressed the
  focused CPU band. Three 20-run samples were `11.695B`, `11.690B`, and
  `11.692B`; the 80-run sample was `46,025,113,943` instructions. It was
  reverted; keep the current `stored_value(current_match_index)` form unless a
  new profile says otherwise.
- Rejected follow-up after the July 17 resume: keeping
  `opt_match/tree.rs` tree cursor values as `u32` in the two no-dictionary
  binary-tree loops, only widening to `usize` at table/slice indexing points,
  preserved focused bytes (`9,198,420` for 20 runs) but clearly regressed the
  focused instruction band. Two 20-run samples were `11.493B` and `11.493B`
  instructions versus the kept `11.33B` to `11.34B` band. It was reverted;
  keep the current immediate `usize` widening shape unless a new profile gives
  a stronger reason.
- Fresh work-count evidence from the July 17 continuation: one-run Callgrind
  on the focused fixture at level 16 produced `434,843,293` Ir for C API and
  `564,897,202` Ir for Rust. Both paths processed 86 optimal blocks. C
  `ZSTD_compressBlock_opt0` called `ZSTD_btGetAllMatches_noDict_5` `672,736`
  times and `ZSTD_btGetAllMatches_noDict_5` called `ZSTD_insertBt1` `186,992`
  times. Temporary Rust counters, removed after measurement, showed the same
  `672,736` match-collector wrapper calls and the same `186,992`
  `insert_bt1_no_dict()` calls; `9,242` Rust collector calls returned at the
  skipped-area guard before entering the lower tree helper, matching C's
  `ip < nextToUpdate` behavior. The remaining focused CPU gap is therefore not
  from extra top-level optimal blocks, extra match-collector calls, or extra
  tree-update insert calls; continue looking at per-call generated-code shape,
  parser table work, entropy/post-split work, or deeper loop iteration costs.
- Latest kept match-finder CPU change: the two no-dictionary optimal
  binary-tree walks in `opt_match/tree.rs` now put the `nb_compares` and
  match-low checks in the `while` condition, matching C's `for (; nbCompares
  && matchIndex >= matchLow; --nbCompares)` shape after accounting for Rust's
  `index + 1` tree sentinel encoding. Focused `corpus_z000033` level-16 bytes
  stayed unchanged (`9198420` for 20 runs, `36793680` for 80 runs). Three
  focused 20-run samples were `11.664B`, `11.662B`, and `11.661B`; the focused
  80-run sample was `45,908,122,464` instructions, `25,730,746,419` cycles,
  `7,662,146,783` branches, and `278,838,202` branch misses. The broad
  one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-tree-loop-condition.csv`
  has byte fields identical to the previous
  `normal-levels-8-16-19-api-runs1-after-rank-limited-bucket-reduce.csv`
  artifact. Target smoke `corpus_z000022` level 19 target 8192 still matches
  the C API at `78067` bytes. Validation passed `cargo fmt --check`, focused
  opt-match/parser tests, full `cargo test -p ruzstd --quiet`, `cargo clippy
  -p ruzstd --all-targets -- -D warnings`, and release profiler rebuild.
- Latest kept parser CPU follow-up: `forward_pass()` now carries the
  `ZSTD_OPT_NUM` guard in the main loop condition instead of entering the loop
  and immediately branching out. This preserves the C parser's bounded-table
  invariant while removing a hot branch from the Rust loop shape. Focused
  `corpus_z000033` level-16 bytes stayed unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs). Three focused 20-run samples were `11.655B`,
  `11.652B`, and `11.652B`; the focused 80-run sample was `45,865,640,102`
  instructions, `26,469,839,330` cycles, `7,652,782,851` branches, and
  `277,699,116` branch misses. The broad one-run normal artifact
  `benchmarks/tmp/normal-levels-8-16-19-api-runs1-after-forward-loop-bound.csv`
  has byte fields identical to the previous
  `normal-levels-8-16-19-api-runs1-after-tree-loop-condition.csv` artifact.
  Target smoke `corpus_z000022` level 19 target 8192 still matches the C API at
  `78067` bytes. Validation passed `cargo fmt --check`, focused
  opt-match/parser tests, full `cargo test -p ruzstd --quiet`, `cargo clippy
  -p ruzstd --all-targets -- -D warnings`, and release profiler rebuild.
- Current normal-mode broad benchmark artifact:
  `benchmarks/tmp/normal-levels-8-16-19-api-runs3-after-nonzero-huf-sort.csv`.
  Against the C API across 73 fixtures, level 8 totals are Rust `-560` bytes
  (`-0.036%`), level 16 totals are Rust `-967` bytes (`-0.070%`), and level 19
  totals are Rust `-155` bytes (`-0.012%`). Rounded aggregate CPU in that
  three-run sweep is level 8 Rust `0.04s` vs C `0.02s`, level 16 Rust `0.25s`
  vs C `0.23s`, and level 19 Rust `0.51s` vs C `0.50s`. The remaining positive
  byte gaps are tiny; continue CPU parity work, especially the focused
  `corpus_z000033` level-16 normal path.
- Latest parser-divergence checkpoint from July 17, 2026: the `corpus_z000021`
  level-16 optimal-parser gap is closed. Rust now matches the C API at
  `26329` bytes with `targetCBlockSize=2048` and at `24877` bytes in normal
  non-target mode. Inspector artifact
  `benchmarks/tmp/inspect-target2048-z000021-after-opt-index-bias.txt` reports
  `content_delta=0`, `source_delta=0`, `type_diffs=0`, `first_diff=none`, and
  `first_source_diff=none`.
- The fix was in `opt_match/tree.rs`: the optimal binary-tree match finder now
  stores table indexes as `source_index + 1`, keeping `0` as the empty sentinel
  while allowing real source index `0` to be matched. This mirrors C's
  effective window indexing, where `matchLow == 1` still permits the first
  source byte because the C window base is biased. The regression test is
  `opt_match_collector_can_match_source_index_zero`.
- Latest targetCBlockSize checkpoint from July 17, 2026: the 73-row level-16
  real-world target smoke at `targetCBlockSize=2048` is byte-identical with the
  C API. Artifact:
  `benchmarks/tmp/target-cblock-2048-after-c-optimal-huffman.csv`. Summary:
  `rows=73`, `differing=0`, C bytes `1412464`, Rust bytes `1412464`, total gap
  `+0`. The remaining one-run CPU numbers in that smoke are noisy
  (`0.24s` C vs `0.29s` Rust), so do not treat them as a performance
  conclusion.
- The fixes after the `z000021` index-bias checkpoint were:
  target Huffman literal table construction now uses C's strategy gate, so
  `BtUltra` and `BtUltra2` use a C-style `HUF_optimalTableLog()`
  shallow-to-deep probe instead of Rust's broader smallest-table search; the
  C-style optimal-depth builder keeps the first shallower table when estimated
  compressed size plus table description ties and does not try Rust's
  rank-limited alternate table; target mode now tries compressed Huffman
  literal-only blocks for zero-sequence non-RLE literal blocks before raw
  fallback; and target mode now tries all-basic/predefined selected sequence
  modes before repeat/RLE sequence candidates.
- These closed `corpus_z000002` (`580 -> 577`),
  `repo_Cargo.toml` (`73 -> 67`), `corpus_z000016` (`19 -> 18`),
  `corpus_z000064` (`1273 -> 1272`), and the negative `repo_io_std.rs` delta
  (`131 -> 133`) against the C API. Fresh block-identical inspector artifacts:
  `benchmarks/tmp/inspect-target2048-z000002-after-optimal-depth-literals.txt`,
  `benchmarks/tmp/inspect-target2048-repo-Cargo-toml-after-literal-only-huffman.txt`,
  `benchmarks/tmp/inspect-target2048-corpus-z000016-after-basic-single-sequence.txt`,
  and
  `benchmarks/tmp/inspect-target2048-corpus-z000064-after-huffman-tie-break.txt`.
- Historical target smoke after the earlier opt-index fix:
  `benchmarks/tmp/target-cblock-2048-after-opt-index-bias.csv` and `.md`.
  For 73 level-16 rows, C totals `1412464` bytes and Rust totals `1412542`
  bytes, a remaining `+78` bytes (`+0.006%`). `corpus_z000021` is exact;
  `corpus_z000002` improved to `580` Rust bytes vs `577` C bytes (`+3`). The
  largest remaining positive fixture gap in that historical smoke was
  `repo_Cargo.toml` at `+6` bytes. Those gaps are now closed by the latest
  targetCBlockSize checkpoint above. CPU in the one-run smoke was noisy
  (`0.29s` C vs `0.26s` Rust), so do not treat it as a performance conclusion.
- Validation for this checkpoint passed `cargo fmt --check`, `cargo test -p
  ruzstd opt_match --quiet`, `cargo test -p ruzstd opt_parser --quiet`, `cargo
  clippy -p ruzstd --all-targets -- -D warnings`, `git diff --check`, release
  builds for `profile_c_port`, `profile_c_api`, `inspect_c_port_blocks`, and
  `benchmark_c_port`, focused `z000033` target smokes (`477631`, `477121`,
  `476853`), and the normal `z000033` 20-run smoke (`9198420`). The follow-up
  full-suite failure in
  `greedy_tests::btlazy2_loaded_dictionary_keeps_full_dictionary_valid_like_c`
  was fixed by making `greedy_dict::load_binary_tree_prefix()` use btlazy2's
  DUBT prefix loader from `bt_match.rs` instead of the optimal-parser
  sorted-tree loader. After that fix, `cargo test -p ruzstd --quiet`, `cargo
  clippy -p ruzstd --all-targets -- -D warnings`, `cargo fmt --check`, and
  `git diff --check` all passed.
- Historical `z000021` target block 0 evidence: decoded sequence dumps from
  `benchmarks/tmp/c-port-block-inspect/corpus_z000021.l16.target2048.rust.zst`
  and `.c.zst` show the first local divergence as C choosing
  `ll=1 ml=10 of=8` then `ll=0 ml=50 of=30`, while Rust chooses
  `ll=1 ml=11 of=8` then `ll=0 ml=49 of=30`. Both paths rejoin at the same
  source position. A larger grouping divergence starts around sequence 91,
  where C combines what Rust emits as a short `ml=4` step plus the following
  match. The root cause was not path-backtracking or a price comparison; Rust's
  binary-tree tables could not represent source index `0` as a valid match
  because `0` was also the empty sentinel.
- Be explicit about which C entry point a benchmark uses. The `zstd` CLI can
  produce different block boundaries from `ZSTD_compress2()` one-shot
  compression, even with `--single-thread`. For example, on
  `decodecorpus_files/z000044` at level 19, a local debug harness calling
  `ZSTD_compress2()` produced 231,857 bytes, matching Rust's 231,859 bytes,
  while the CLI comparison produced 231,665 bytes by taking a different
  streaming/chunking path. Treat CLI-only block-shape deltas as streaming-mode
  evidence until they reproduce through the one-shot C API.
- `targetCBlockSize` / superblock compression is selected by C only when
  `ZSTD_c_targetCBlockSize` is explicitly non-zero. Normal level-based
  compression, including the local level-16 benchmarks, uses the regular
  block path. Treat this as a full-port gap, not as an explanation for the
  current C comparison gap.
- Restart checkpoint from July 16, 2026: target compressed block size dispatch
  is threaded through the CCtx, frame state, optimal frame path, and hash-chain
  frame path. The active target-mode encoder lives in `target_block.rs`; it
  currently tries a literal-only RLE superblock, then non-empty sequence
  sub-blocks with Huffman compressed or treeless literal metadata, then
  basic-literal sequence sub-blocks with all-RLE, all-repeat, or
  all-compressed sequence metadata, and then falls back to a raw block. The
  resolved `targetCBlockSize` value is consumed by the multi-sub-block path,
  which can write a full-superblock Huffman literal table once, use treeless
  literal sections for later sub-blocks, write full-superblock FSE sequence
  tables once on the Huffman-literal and basic-literal paths, use table-backed
  mixed LL/ML/OF modes when they can be repeated by later sub-blocks, use
  repeat sequence metadata for later sub-blocks, and fall back to a
  single-subblock path that can select mixed LL/ML/OF sequence entropy modes.
  Target sequence entropy selection is strategy-aware and reuses the shared
  C-style sequence table selector from the regular compressed-block path, so
  fast strategies use the C fast heuristic while optimal strategies use the C
  cost model. Target multi-sub-block encoding primes temporary repeat tables
  after the first committed sequence entropy write, which allows later
  sub-blocks to use repeat mode after C-style basic/RLE/mixed metadata without
  making basic/RLE tables repeat-valid for later normal blocks. Target
  multi-sub-block planning uses C-style rough estimates for Huffman literal
  sections and sequence sections, including the hard-coded 3-byte
  sequence-header estimate and FSE table-definition costs, instead of
  estimating by fully emitting a candidate block. The final target block
  wrapper applies C's whole-superblock minimum-gain acceptance gate and
  restores FSE/offset state before emitting a single raw block when the target
  candidate is rejected. Target mode also mirrors C's pre-superblock RLE
  shortcut for non-first frame blocks: when the prepared sequence store matches
  C's `ZSTD_maybeRLE()` heuristic (`nbSeqs < 4 && nbLits < 10`) and the source
  is a single repeated byte, the encoder emits an RLE block directly instead of
  trying the superblock path.
- Ported superblock pieces from `zstd_compress_superblock.c` now include the
  planning helpers, target acceptance gate, literal header sizing, basic and
  RLE literal emission, Huffman compressed and treeless literal emission,
  zero-sequence emission, literal-only compressed block assembly, non-empty
  single-sub-block assembly, and a predefined/basic all-RLE, all-repeat, and
  all-compressed sequence-section writer. Mixed per-stream LL/ML/OF sequence
  entropy mode emission and the conservative single-subblock selector are split
  into `superblock_sequences.rs`.
- Ported target multi-sub-block paths now include raw-tail fallback when only
  part of the target superblock can be compressed. After a compressed prefix
  has been committed, the final sub-block attempt snapshots FSE and
  repeat-offset state, restores to the committed prefix if the final sub-block
  is not worth committing, and appends a raw block for the remaining source
  bytes.
- Target Huffman literal sub-block emission now mirrors C's
  `ZSTD_compressSubBlock_literal()` raw-literal fallback when a repeat or
  compressed Huffman literal section expands or would require a larger literal
  header than was reserved. It also mirrors C's `litSize == 0` branch by
  emitting a raw zero-literal section instead of rejecting the sub-block. The
  target multi-sub-block path only returns a new Huffman table when a committed
  sub-block actually wrote that table. Target tests now cover a split Huffman
  superblock where later compressed sub-blocks have non-empty sequences and zero
  literals, forcing the raw zero-literal literal section after the first block.
- Target multi-sub-block wrappers now preserve the sequence helper's
  `entropy_written` flag instead of forcing it to true when
  `write_sequence_entropy` is requested, matching C's zero-sequence path where
  `ZSTD_compressSubBlock_sequences()` leaves `seqEntropyWritten` false.
- TargetCBlockSize validation is now available through
  `profile_c_port INPUT LEVEL RUNS TARGET_C_BLOCK_SIZE` and the matching
  `profile_c_api` argument. The Rust helper is deliberately scoped to valid
  C-port target-size validation; no-dictionary fast, double-fast, hash-chain,
  and optimal strategies are wired through target mode rather than silently
  benchmarking normal compression. The first level-16
  smoke on `benchmarks/archive/tmp/realworld-100/corpus_z000033` with target
  size 2048 exposed a repeat FSE table panic. Sequence-section emitters now
  validate that repeat, predefined, and mixed FSE table modes can encode the
  candidate LL/ML/OF symbols before writing the bitstream, and roll back
  offset/FSE state on rejection. After the fix, Rust completes the smoke at
  `488728` bytes and the C API completes at `477631` bytes for one run, so the
  current target-mode gap is compression parity rather than a crash.
- Single-subblock target candidates now try compressed Huffman literals with the
  selected C sequence entropy modes before falling back to all-compressed
  sequence modes. This matches C cases such as compressed literals with repeat
  or mixed sequence modes. The latest planner ordering also tries
  basic-literal multi-sub-blocks only after Huffman literal multi-sub-block and
  Huffman single-subblock candidates, matching C cases where one compressed
  Huffman block should win before the basic-literal splitter. The target
  Huffman table builder now also mirrors C's
  `ZSTD_buildBlockEntropyStats_literals()` no-gain gate: when a new Huffman
  table's estimated payload plus table description is not smaller than raw
  literals, target mode selects the basic/raw literal path instead. It also
  uses C's fast Huffman table-log calculation for non-`BtUltra` strategies:
  `FSE_optimalTableLog_internal(max=11, srcSize, maxSymbolValue, minus=1)`.
  The final planner-estimate fix made the target-superblock literal estimate
  match C's `ZSTD_estimateSubBlockSize_literal()` rough formula by using
  `HUF_estimateCompressedSize + hufDesSize + 3` and excluding the emitted
  four-stream jump table. Focused target-mode bytes now stand at target 2048
  `488728 -> 477631`, target 4096 `479088 -> 477121`, and target 8192
  `477751 -> 476853`, matching the C API comparison bytes `477631`, `477121`,
  and `476853`. Latest inspector artifacts are
  `benchmarks/tmp/inspect-target2048-after-superblock-estimate-parity.txt`,
  `benchmarks/tmp/inspect-target4096-after-superblock-estimate-parity.txt`, and
  `benchmarks/tmp/inspect-target8192-after-superblock-estimate-parity.txt`; all
  three report `content_delta=0`, `source_delta=0`, `type_diffs=0`,
  `first_diff=none`, and `first_source_diff=none`.
- `benchmark_c_port` now supports `--target-c-block-size N` for API-backed C
  comparisons. It is intentionally rejected with the CLI backend. A first
  10-fixture smoke at level 16, target 2048 produced
  `benchmarks/tmp/target-cblock-2048-smoke.csv` and
  `benchmarks/tmp/target-cblock-2048-smoke.md`: 10 rows, C `602540` bytes, Rust
  `602628` bytes, total gap `+88` bytes across 6 differing rows. The largest
  smoke gap was `corpus_z000021` at `+78` bytes; inspector artifacts
  `benchmarks/tmp/inspect-target2048-z000021-smoke-gap.txt`,
  `benchmarks/tmp/inspect-target2048-z000002-smoke-gap.txt`, and
  `benchmarks/tmp/inspect-target2048-z000023-smoke-gap.txt` show the next
  target gaps are caused by different sequence/literal stores or earlier
  parser/block choices, not just the already-fixed target sub-block sizing
  loop.
- `benchmark_c_port` now also supports `--dictionary PATH` for dictionary
  comparisons through the C API and CLI backends. Dictionary mode is kept
  separate from `--target-c-block-size`, and the tool rejects combining them.
  Dictionary outputs are decoded with `zstd -D` before comparing them with the
  original input. A focused API run on
  `benchmarks/archive/tmp/dict-secondnewest-focused-fixtures` with levels
  `1,3,8,16,22` produced
  `benchmarks/tmp/dictionary-focused-levels-1-3-8-16-22-api-current.csv`:
  level 1 aggregate gap `+0` bytes, level 3 `+22` bytes (`+0.235%`), level 8
  `+0`, level 16 `+1` byte (`+0.012%`), and level 22 `+0` across five
  fixtures. Treat the CPU columns in that artifact as smoke only because the
  focused dictionary files are too small for reliable timing. Validation
  passed format, clippy for `benchmark_c_port`, whitespace checks, release
  rebuild of `benchmark_c_port`, and API and CLI dictionary smokes. Combined
  dictionary+target support was ported later.
- Reviewability follow-up: `benchmark_c_port` is now split by responsibility.
  The root binary owns CLI parsing, fixture walking, and benchmark
  orchestration (`469` lines), `benchmark_c_port/reference.rs` owns C/Rust
  reference compression, dictionary decoding, target-size handling, and CPU
  timing helpers (`332` lines), and `benchmark_c_port/report.rs` owns
  CSV/Markdown rendering (`164` lines). This keeps the benchmark tool under
  the project review target without changing benchmark behavior. Validation
  passed format, bin tests, clippy for `benchmark_c_port`, release rebuild,
  API dictionary, CLI dictionary, and target-mode smokes, and whitespace
  checks.
- Dictionary profiling tooling checkpoint: `profile_c_port` and `profile_c_api`
  now accept dictionary profiling in the same positional command shape as
  target profiling. The fourth argument remains `TARGET_C_BLOCK_SIZE` when it
  parses as a number; otherwise it is treated as `DICTIONARY_PATH`. Dictionary
  profiling was intentionally rejected when combined with targetCBlockSize in
  this older checkpoint.
  Normal and target profiling smokes still match Rust/C bytes. Dictionary smoke
  on
  `benchmarks/archive/tmp/dict-secondnewest-focused-fixtures/repo_Cargo.lock`
  level 16 for 20 runs emitted identical Rust and C API totals (`148,960`
  bytes). An 80-run `perf stat` dictionary sample on the same fixture emitted
  identical totals (`595,840` bytes); Rust counters were `6,012,722,969`
  instructions, `2,760,080,384` cycles, `986,207,423` branches, and
  `28,132,169` branch misses versus C API `5,901,855,672` instructions,
  `2,478,522,349` cycles, `750,664,636` branches, and `22,494,473` branch
  misses. Validation passed format, release rebuild of both profilers, clippy
  for both profiler binaries with warnings denied, normal/target/dictionary
  smokes, and whitespace checks. Combined dictionary+target support was ported
  later.
- Target+dictionary porting checkpoint: C `zstd` supports combining `-D` with
  `--target-compressed-block-size`; a CLI smoke on
  `benchmarks/archive/tmp/dict-secondnewest-focused-fixtures/repo_Cargo.lock`
  at level 16 decoded correctly and changed C bytes from `7448` normal
  dictionary bytes to `7509` target+dictionary bytes. Rust has a validation
  hook,
  `compress_slice_c_level_with_dictionary_and_target_c_block_size()`, and the
  internal `encode_frame_with_dictionary_and_target_c_block_size()` path
  supports optimal dictionary strategies (`BtOpt`, `BtUltra`, and `BtUltra2`)
  by threading a target-sized `CctxParameters` into the existing dictionary
  optimal frame path. Follow-up support now also wires dictionary hash-chain
  strategies (`Greedy`, `Lazy`, `Lazy2`, and `BtLazy2`) through a cctx-aware
  dictionary frame entry point and the existing target-aware hash-chain block
  path. Fast and double-fast dictionary target modes are now wired too, via
  cctx-aware dictionary frame entry points and target-aware Fast/DFast ext-dict
  block adapters. Combined target+dictionary is now strategy-wired; remaining
  work is byte and CPU parity rather than unsupported strategy dispatch.
- The benchmark and profiler tools now allow target+dictionary instead of
  rejecting it globally. Unsupported target sizes fail explicitly when the
  target size is outside C's accepted range. Focused level-16 `repo_Cargo.lock`
  target 2048 dictionary profiling matched C bytes exactly: 20-run totals were
  Rust/C `150,180` bytes, and 80-run `perf stat` totals were Rust/C `600,720`
  bytes. The 80-run Rust counters were `5,552,023,258` instructions,
  `2,340,172,068` cycles, `909,915,589` branches, and `24,567,705` branch
  misses versus C API `5,851,987,642` instructions, `2,508,155,831` cycles,
  `743,936,771` branches, and `22,454,540` branch misses. A two-fixture
  benchmark smoke is `benchmarks/tmp/dictionary-target-api-smoke-current.csv`:
  aggregate C `296` bytes, Rust `304` bytes, gap `+8` bytes (`+2.703%`), with
  the positive gap on `generated_go.sum`.
- Latest hash-chain target+dictionary smoke after that follow-up:
  `benchmarks/tmp/dictionary-target-hashchain-api-smoke-current.csv` covers
  level 5 with target 2048 and dictionary across the 5 focused dictionary
  fixtures. Aggregate C API bytes were `9074`, Rust bytes were `9063`, gap
  `-11` bytes (`-0.121%`), with `4` differing rows and `2` positive Rust rows.
  Latest Fast/DFast target+dictionary smoke:
  `benchmarks/tmp/dictionary-target-fast-dfast-api-smoke-current.csv` covers
  levels 1 and 3 with target 2048 and dictionary across the same 5 focused
  dictionary fixtures. Aggregate C API bytes were `19019`, Rust bytes were
  `19076`, gap `+57` bytes (`+0.300%`), with `8` differing rows and `7`
  positive Rust rows. A broader representative combined-mode artifact is
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-current.csv`.
  Across the same 5 fixtures at levels `1,3,5,8,16,22`, aggregate level gaps
  are level 1 `+16` bytes (`+0.167%`), level 3 `+41` bytes (`+0.435%`), level
  5 `-11` bytes (`-0.121%`), level 8 `+9` bytes (`+0.101%`), level 16 `+9`
  bytes (`+0.106%`), and level 22 `+8` bytes (`+0.094%`). The rounded CPU
  fields are smoke-only.
- Latest DFast target+dictionary parity checkpoint: the earlier assumption that
  `ZSTD_CCtx_loadDictionary()` used the DFast `dictMatchState` block compressor
  for the focused C API target+dictionary benchmark was wrong. A fresh `perf`
  profile on `generated_yarn.lock` level 3 target 2048 with the focused
  dictionary shows C spends block time in
  `ZSTD_compressBlock_doubleFast_extDict_generic`, while dictionary setup
  builds a CDict (`ZSTD_createCDict_advanced2`) and copies CDict tables into
  the active CCtx. The kept Rust fix mirrors that path for DFast dictionary
  frames: `ParsedDictionary` carries the raw dictionary size, `dfast_frame`
  derives copied-CDict compression parameters with `CParamMode::CreateCDict`
  while preserving the active window log and target size, and `dfast_dict` has
  `load_cdict_copy_prefix()` which fills the CDict-style tagged double-fast
  tables and strips tags into the active match state like
  `ZSTD_copyCDictTableIntoCCtx()`.
  This closed the previous worst level-3 target+dictionary row:
  `generated_yarn.lock` is now C/Rust `379` bytes. Focused levels 1 and 3
  across the 5 dictionary fixtures are
  `benchmarks/tmp/dictionary-target-fast-dfast-api-after-dfast-cdict-params.csv`:
  level 1 stayed `+16` bytes (`+0.167%`), while level 3 improved from `+41`
  bytes (`+0.435%`) to `+10` bytes (`+0.106%`). The broader focused artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-dfast-cdict-params.csv`
  shows only level 3 changed: level 1 `+16`, level 3 `+10`, level 5 `-11`,
  level 8 `+9`, level 16 `+9`, and level 22 `+8` bytes versus C. Remaining
  positive rows are dominated by `generated_go.sum` at all levels; continue
  there next. Rejected/removed in this checkpoint: the untracked
  `dfast_dict_match.rs` route and the artificial `>4` dictionary short-match
  filter. Validation passed format, focused DFast and dictionary-target tests,
  full `ruzstd` tests, all-target `ruzstd` clippy with warnings denied,
  release rebuild of `benchmark_c_port`, focused dictionary+target smokes, and
  whitespace checks.
- Follow-up target+dictionary Huffman-repeat checkpoint: target single-block
  candidate ordering now mirrors C's superblock literal entropy choice when a
  previous Huffman table is available. Rust computes the C-style repeat
  criterion from `ZSTD_buildBlockEntropyStats_literals()`: the previous table
  must encode the literal counts, its estimated payload must be below raw, and
  it must beat/tie the fresh table including the table description, or the new
  description must be too expensive (`hSize + 12 >= srcSize`). If that
  criterion selects repeat, target mode tries the treeless-literal candidate
  before the fresh compressed-literal candidate. This closed
  `generated_go.sum` target+dictionary gaps at levels 3 and 16:
  `154/164 -> 154/154` and `138/146 -> 138/138` for C/Rust. Artifact:
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-target-repeat-estimate.csv`.
  Focused aggregate gaps across the 5 dictionary fixtures are now level 1
  `+7`, level 3 `+0`, level 5 `-21`, level 8 `+0`, level 16 `+1`, and level
  22 `+0` bytes versus C. Remaining positives are tiny: `generated_yarn.lock`
  level 1 `+2`, `repo_Cargo.lock` level 1 `+5`, `repo_Cargo.lock` level 5
  `+4`, and `generated_poetry.lock` level 16 `+1`. Rejected follow-up: a broad
  fastish `literals <= 1024` repeat-first rule closed `generated_go.sum` but
  regressed `generated_yarn.lock` level 3 from exact parity to `+6` bytes, so
  keep the C-estimate criterion. Validation passed format, focused
  target/dictionary tests, full `ruzstd` tests, all-target `ruzstd` clippy
  with warnings denied, release rebuild of `benchmark_c_port`, the focused
  dictionary+target benchmark, and whitespace checks.
- Follow-up Fast target+dictionary CDict-copy checkpoint:
  `FastMatchState::load_cdict_copy_prefix()` now fills the dictionary hash
  table with C's copied-CDict tagged-hash shape (`hashLog + 8`, every third
  position plus empty adjacent slots) and strips tags into the active table.
  `fast_frame` derives copied-CDict table parameters with
  `CParamMode::CreateCDict` while preserving the active window log and target
  size. Artifact:
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-fast-cdict-copy.csv`.
  This closed the focused level-1 positives: `generated_yarn.lock` `+2 -> +0`
  and `repo_Cargo.lock` `+5 -> +0`. Focused aggregate gaps are now level 1
  `+0`, level 3 `+0`, level 5 `-21`, level 8 `+0`, level 16 `+1`, and level
  22 `+0` bytes versus C. Remaining positives are `repo_Cargo.lock` level 5
  `+4` and `generated_poetry.lock` level 16 `+1`. Rejected follow-up:
  changing greedy/hash-chain dictionary frames to use only copied-CDict
  parameter derivation did not close `repo_Cargo.lock` level 5 and regressed
  useful negative level-5 gaps on `generated_poetry.lock` and
  `generated_yarn.lock` back to exact parity, so it was reverted. Validation
  passed format, focused Fast and target/dictionary tests, full `ruzstd` tests,
  all-target `ruzstd` clippy with warnings denied, release rebuilds of
  `benchmark_c_port` and `inspect_c_port_blocks`, the focused
  dictionary+target benchmark, and whitespace checks.
- Follow-up profile after the Fast CDict-copy checkpoint:
  `benchmarks/tmp/perf-c-api-repo-cargo-l5-target2048-dict.data` shows the C
  API `repo_Cargo.lock` level-5 target+dictionary row spends match-finder time
  in `ZSTD_RowFindBestMatch_dictMatchState_4_4`,
  `ZSTD_row_update`, and `ZSTD_compressBlock_greedy_dictMatchState_row`.
  This means the remaining `repo_Cargo.lock` level-5 `+4` gap is a greedy
  row-match dictionary-match-state gap, not a hash-chain/ext-dict gap. A probe
  changing only the hash-chain ext-dict dictionary-candidate first-four-byte
  test to mirror C was byte-neutral across the focused grid
  (`benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-hc-extdict-dict-probe.csv`)
  and was reverted. Continue this row by porting or narrowly experimenting
  with row `dictMatchState` behavior, not by retrying hash-chain/ext-dict
  tweaks. Validation for the investigation pass included format, focused
  greedy/row/target tests, release rebuilds of `profile_c_api`,
  `benchmark_c_port`, and `inspect_c_port_blocks`, the focused benchmark grid,
  and whitespace checks.
- Rejected follow-up after the row-DMS investigation: a narrow separate row
  dictionary match-state path for greedy dictionary frames, with the active
  row table starting after the dictionary instead of being loaded with
  dictionary rows, did not close the focused gap. It preserved all other
  focused rows but moved `repo_Cargo.lock` level 5 from `+4` bytes to `+5`
  bytes, with level-5 aggregate `-21 -> -20` bytes versus C. Artifact:
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-row-dms.csv`.
  The experiment was reverted; post-revert artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-row-dms-revert.csv`
  matches the Fast CDict-copy checkpoint exactly: level 5 total `-21` with
  `repo_Cargo.lock` still `+4`, and level 16 total `+1` from
  `generated_poetry.lock`. Do not retry that exact separate-row-state shape
  without first proving a deeper C row iteration/order difference.
- Follow-up row evidence after the rejected row-DMS shape: a C-like active-row
  buffering experiment changed `row_match.rs` to collect row candidates, insert
  the current position, and then score the buffered candidates. Focused artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-row-active-buffer.csv`
  was byte-neutral versus the post-revert checkpoint: level 5 total stayed
  `-21` with `repo_Cargo.lock` still `+4`, and level 16 stayed `+1`. The
  experiment was removed because it added hot-path work without a byte win.
  Temporary probes showed Rust dispatches the focused row through greedy
  ext-dict with loaded dictionary length `51812`; C's divergent sequence at
  source block position `3466` corresponds to combined `ip=55278`, C offBase
  `25977`, and dictionary match index `29304`. A dictionary-only row-table
  simulation contains that candidate, but the actual Rust active row table at
  `ip=55278` does not. A later additive row-DMS experiment kept the current
  active-table loading behavior and searched a copied dictionary row table as a
  secondary candidate source. It also failed to close the gap: artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-additive-row-dms.csv`
  has level 5 total `-20` and `repo_Cargo.lock` level 5 `+5`, regressing from
  the kept `+4`. That implementation was removed; post-revert artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-additive-row-dms-revert.csv`
  restored level 5 total `-21` with `repo_Cargo.lock` level 5 `+4`, and level
  16 total `+1` from `generated_poetry.lock`. Do not retry either row-DMS shape
  without deeper C row candidate-order or lazy-parser evidence.
- Additional row-DMS probe after reading C `zstd_lazy.c`: C's
  `ZSTD_RowFindBestMatch(..., ZSTD_dictMatchState)` computes the
  dictionary-match-state row with `ZSTD_hashPtr()` on `dms->rowHashLog`, while
  active rows use the salted active hash. A corrected additive probe built a
  separate unsalted dictionary row table and searched it after the active row
  candidates. It still regressed the focused grid exactly like the earlier
  additive probe: artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-unsalted-row-dms.csv`
  has level 5 total `-20` and `repo_Cargo.lock` level 5 `+5`. The
  implementation was removed; post-revert artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-unsalted-row-dms-revert.csv`
  restored level 5 total `-21` with `repo_Cargo.lock` level 5 `+4`, and level
  16 total `+1` from `generated_poetry.lock`. The remaining issue is therefore
  not just the absence of C's dictionary candidate at `ip=55278`; adding that
  candidate without the rest of C's row/lazy interaction worsens the selected
  parse.
- Rejected follow-up after the unsalted row-DMS probe: matching C's row
  ordering more exactly by collecting active candidates, inserting the current
  position, then collecting unsalted DMS candidates before scoring the combined
  candidate buffer also regressed the focused grid. Artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-exact-order-row-dms.csv`
  has level 5 total `-20` and `repo_Cargo.lock` level 5 `+5`, with level 16
  still `+1`. That implementation was removed; post-revert artifact
  `benchmarks/tmp/dictionary-target-levels-1-3-5-8-16-22-api-after-exact-order-row-dms-revert.csv`
  restored level 5 total `-21` with `repo_Cargo.lock` level 5 `+4`, and level
  16 total `+1` from `generated_poetry.lock`. Do not retry active-buffered
  row-DMS ordering without new C trace evidence that explains why C accepts the
  early `ip=55278` dictionary match without causing the later parse regression.
- Fresh current-source sequence trace after the row-DMS rejections: rebuilt
  `inspect_c_port_blocks` and generated fresh focused outputs in
  `benchmarks/tmp/inspect-dict-target-repo-cargo-l5-current/`, then used the
  ignored `ruzstd` `inspect_archive_from_env` diagnostic with
  `RUZSTD_INSPECT_SEQUENCE_DUMP_BLOCK=0` to decode block-0 sequences.
  Artifacts:
  `benchmarks/tmp/dictionary-target-repo-cargo-l5-current-c-block0-sequences.txt`,
  `benchmarks/tmp/dictionary-target-repo-cargo-l5-current-rust-block0-sequences.txt`,
  the corresponding `.csv` files, and summary
  `benchmarks/tmp/dictionary-target-repo-cargo-l5-current-block0-diff-summary.txt`.
  Current block 0 has `266` C sequences versus `264` Rust sequences. The first
  content split is still C seq 118 `start=3465 ll=1 match_start=3466 ml=4
  of=25977 end=3470` plus C seq 119 `start=3470 ll=5 match_start=3475 ml=4
  of=6808 end=3479`, while Rust seq 118 is `start=3465 ll=10
  match_start=3475 ml=4 of=6808 end=3479`; both streams resynchronize by
  `end=3483`. The next independent split after resync is at decoded position
  `5654`: C emits `ll=0 ml=5 of=38947`, while Rust emits `ll=1 ml=4 of=6725`,
  and both end at `5659`. C also has an additional final block-0 sequence at
  `start=6878 ll=4 match_start=6882 ml=6 of=44 end=6888`, while Rust block 0
  ends at `6878`. Mechanical comparison by decoded start/end positions found
  only these two content differences in shared block-0 positions, plus C's
  final extra sequence. This suggests the next investigation should look for a
  repeated pattern in zero-literal dictionary matches selected by C but
  skipped/delayed by Rust, rather than focusing only on the first `ip=55278`
  row-DMS candidate.
- Historical note: the next inspection paragraph records the initial
  target+dictionary analysis before the DFast CDict-copy fix above. Its
  conclusion that C used the DFast `dictMatchState` block compressor for the
  focused C API target+dictionary benchmark was later disproved by perf; keep
  it only as historical context.
- `inspect_c_port_blocks` now accepts `--dictionary` and can inspect/decode
  dictionary and dictionary+target frames. First structural inspection
  artifact: `benchmarks/tmp/dictionary-target-inspect-yarn-l3.txt` for
  `generated_yarn.lock` level 3 target 2048 dictionary, the worst positive
  row. Rust and C both emitted two compressed blocks with identical source
  ranges; the 18-byte gap is entirely in block 0. Rust block 0 is `370` bytes
  with `59` sequences, regenerated literals `202`, and literal payload `173`;
  C block 0 is `352` bytes with `41` sequences, regenerated literals `262`,
  and literal payload `217`. This points to match-choice/sequence-cost parity,
  not frame or block-split dispatch. Fresh C-source check:
  `ZSTD_CCtx_loadDictionary()` uses Fast/DFast `dictMatchState` block
  compressors for this mode, while the current Rust dictionary frame path still
  loads the dictionary into the active match state and compresses through the
  ext-dict/combined-buffer approximation. In particular, C fast
  `dictMatchState` only uses a dictionary match when the normal prefix match
  index is invalid. Treat porting Fast/DFast `dictMatchState` behavior as the
  next parity target for these remaining combined-mode byte gaps. Validation
  passed focused
  strategy/public tests, full `ruzstd` tests, clippy for all `ruzstd` targets
  with warnings denied, profiler/benchmark release rebuilds, profiler and
  benchmark smokes, tool clippy for `profile_c_port`, `profile_c_api`, and
  `benchmark_c_port`, tool clippy for `inspect_c_port_blocks`, full
  `zstd-rs-tools` tests, format, and whitespace checks.
- Rejected DFast dictionary-match-state checkpoint: `dfast_frame` temporarily
  kept loaded dictionary tables separate from active prefix tables for DFast
  dictionary frames and routed loaded-dictionary blocks through an untracked
  `dfast_dict_match.rs`, a safe Rust port attempt of C's
  `ZSTD_compressBlock_doubleFast_dictMatchState_*` search order. This was the
  wrong model for the focused C API target+dictionary benchmark and was removed
  after perf proved C uses copied-CDict tables with the extDict block
  compressor. The normal dictionary 5-fixture API smoke
  `benchmarks/tmp/dictionary-fast-dfast-api-after-dfast-dictmatch.csv` reports
  level 1 `+0` bytes and level 3 `+19` bytes (`+0.203%`) versus C. The
  target+dictionary smoke
  `benchmarks/tmp/dictionary-target-fast-dfast-api-after-dfast-dictmatch.csv`
  reports level 1 `+16` bytes (`+0.167%`) and level 3 `+39` bytes
  (`+0.414%`), only a 2-byte improvement from the previous `+41` level-3
  target gap. The worst `generated_yarn.lock` level-3 target row remains Rust
  `397` bytes versus C `379`; the fresh inspection artifact
  `benchmarks/tmp/dictionary-target-inspect-yarn-l3-after-dfast-dictmatch.txt`
  still shows Rust block 0 with `59` sequences versus C `41`. Continue below
  the frame/dispatch layer, likely in DFast candidate choice, dictionary table
  representation, or sequence-cost parity. Validation passed focused
  dictionary-target and DFast tests, full `cargo test -p ruzstd --quiet`,
  `cargo clippy -p ruzstd --all-targets -- -D warnings`, `cargo fmt --check`,
  and `git diff --check`.
- Fresh focused profile after the benchmark-tool split: release
  `profile_c_port` and `profile_c_api` were rebuilt and run on
  `benchmarks/archive/tmp/realworld-100/corpus_z000033` at level 16 for
  80 runs. Rust emitted `36,897,920` bytes and C API emitted `36,864,480`
  bytes. `perf stat` counters were Rust `33,579,117,341` instructions,
  `20,605,748,481` cycles, `5,451,552,337` branches, and `213,027,639`
  branch misses; same-session C API was `34,696,939,913` instructions,
  `19,146,844,452` cycles, `4,328,350,102` branches, and `175,228,833`
  branch misses. Rust remains about `-3.22%` instructions versus C but about
  `+25.9%` branches. Fresh branch profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-branches-after-benchmark-split.data`.
  Top branch self costs were `forward_pass()` at `40.13%` and
  `compress_block_opt_with_state_and_ldm()` at `22.43%`.
- Rejected follow-up from that profile: replacing
  `literal_length_code_transition()`'s full match table with a small <=64 match
  plus `is_power_of_two()`/`trailing_zeros()` arithmetic for large literal
  lengths preserved focused bytes (`9,224,480` for 20 runs) but regressed the
  focused 20-run sample to `8,434,968,214` instructions and `1,371,012,978`
  branches versus the fresh kept baseline implied by the 80-run sample near
  `8.395B` instructions and `1.363B` branches. It was reverted; keep the
  current literal-length transition match unless a new profile gives stronger
  evidence.
- Latest target-mode broad validation on July 17, 2026:
  `benchmark_c_port --fixtures benchmarks/archive/tmp/realworld-100 --levels
  8,16,19 --runs 1 --c-backend api --target-c-block-size N --no-sync` was run
  for target sizes 2048, 4096, and 8192. Artifacts are
  `benchmarks/tmp/target-cblock-2048-levels-8-16-19-resume.{csv,md}`,
  `benchmarks/tmp/target-cblock-4096-levels-8-16-19-resume.{csv,md}`, and
  `benchmarks/tmp/target-cblock-8192-levels-8-16-19-resume.{csv,md}`. At
  target 2048, level 8/16/19 aggregate byte gaps were `+0`, `+0`, and `+0`
  across 73 fixtures. At target 4096 they were `-1`, `+0`, and `+0`. At target
  8192 they were `+2`, `+0`, and `+0`. Only level 8 had differing rows in
  these sweeps: four fixtures per target size, with worst positive fixture
  `corpus_z000048 +4` bytes and best negative fixture `corpus_z000085 -9`
  bytes.
- Next implementation step: return to normal-mode CPU parity work on focused
  `corpus_z000033` level 16. Do not spend more time on target-mode
  `corpus_z000021`, `corpus_z000002`, `repo_Cargo.toml`, `corpus_z000016`,
  `corpus_z000064`, or `repo_io_std.rs` unless a broader fixture reopens one
  of them.
- Validation after the latest target-superblock estimate parity fix passed for
  `cargo fmt --check`, `cargo test -p ruzstd --quiet`,
  `cargo clippy -p ruzstd --all-targets -- -D warnings`, `git diff --check`,
  the release build of `profile_c_port`, `profile_c_api`, and
  `inspect_c_port_blocks`, the focused target smokes above, and the normal
  focused 20-run smoke (`9198420` bytes).
- `zstd_opt.c` seeds optimal-parser literal/LL/ML/offset frequencies from
  full-dictionary entropy tables when dictionary repeat tables are valid. The
  Rust full-dictionary path now derives and consumes the same shape of
  price-model seeds alongside the block entropy tables.

Current profiling notes:

- On July 17, 2026, `update_literal_price()` changed the previous-match capture
  for the ULTRA-only match-plus-one-literal replacement branch from
  `then_some(state.opt[cur])` to an explicit `if`. This preserves the earlier
  parser copy gate while making the old-match copy unambiguously lazy in Rust
  source, matching C's branch shape more closely. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.460B`, `12.462B`, and `12.462B`
  (`12460483361`, `12461702264`, `12461660527`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-explicit-lazy-prev-match-copy.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser/price tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `rank_limited_nonzero_weights()` changed to thread the
  generated power-of-two weight sum log from `distribute_weights_with_sum_log()`
  into `redistribute_weights_with_sum_log()`, avoiding the old scan over
  generated weights before redistribution. The release path uses the sum-log
  helper directly; compatibility wrappers are test-only. Focused bytes were
  unchanged (`9198420` for 20 runs, `36793680` for 80 runs), and three focused
  20-run instruction samples were `12.470B`, `12.461B`, and `12.466B`
  (`12470025456`, `12461412754`, `12466474150`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-rank-limited-sum-log-threading.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman/weights tests,
  full `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  a release `profile_c_port` rebuild.
- Rejected follow-up after the explicit lazy parser-copy checkpoint: adding an
  early return in `redistribute_weights_with_sum_log()` when the lower-weight
  raise pass produces `added_weights == 0` preserved focused bytes but did not
  improve instruction samples (`12.465B`, `12.461B`, `12.466B`) versus the
  latest `12.460B`, `12.462B`, `12.462B` checkpoint, so it was reverted. Do not
  retry that branch-only shape without new evidence.
- Rejected pre-split table-size experiment after the explicit lazy parser-copy
  checkpoint: changing `Fingerprint` to const-generic table sizes so hashLog 8
  and 9 zero only 256/512 entries, closer to C's
  `recordFingerprint_generic()`, preserved focused bytes but regressed the first
  two focused 20-run instruction samples to `12.476B` and `12.477B`. It was
  reverted. Do not retry that safe const-generic shape without new evidence; the
  code-size/monomorphization cost outweighed the smaller zeroing on the focused
  fixture.
- On July 17, 2026, `update_literal_price()` changed to delay copying the old
  `Optimal` node until C's ULTRA-only match-plus-one-literal replacement branch
  is eligible. The price/path decision is unchanged; the Rust port just avoids
  copying the full previous match node for literal replacements that cannot use
  it. Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.470B`,
  `12.467B`, and `12.465B` (`12469947147`, `12466918100`, `12465320113`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-literal-prev-match-copy-gate.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser/price tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `HuffmanTable::build_from_code_lengths()` was added for
  length-limited Huffman candidates. It builds candidate tables directly from
  code lengths instead of allocating a temporary code-length-to-weight `Vec`,
  while keeping the existing serialized table-description path because earlier
  direct table-description experiments regressed. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.471B`, `12.475B`, and `12.464B`
  (`12470903544`, `12475333302`, `12463845666`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-code-length-builder.perf.data`.
  Validation passed for `cargo fmt --check`, `cargo test -p ruzstd huff
  --quiet`, full `ruzstd` tests, clippy, an 80-run focused smoke
  (`36793680` bytes), `git diff --check`, and a release `profile_c_port`
  rebuild.
- On July 17, 2026,
  `OptPriceState::dynamic_lit_length_increment_price()` changed to handle the
  dense literal-length code transitions `1..=16` directly before falling back
  to the larger sparse transition match. This is a no-state version of the
  short literal-length increment optimization; unlike the rejected pass-local
  table, it does not add per-pass stack state. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.481B`, `12.483B`, and `12.483B`
  (`12481190235`, `12482917789`, `12483042243`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-short-ll-increment-fast-path.perf.data`.
  Validation passed for `cargo fmt --check`, focused price/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected after the short literal-length increment fast path: changing
  `price_delta()` from an `i64` widening subtraction to bounded `i32`
  subtraction preserved focused bytes but worsened focused 20-run instruction
  samples to about `12.488B` and `12.487B` (`12487885688`, `12486717750`), so
  it was reverted.
- Rejected after the short literal-length increment fast path:
  caching `acc_log` once for the two final state writes in
  `FSEEncoder::encode_interleaved()` preserved focused bytes but worsened
  focused 20-run instruction samples to about `12.485B` and `12.486B`
  (`12485299857`, `12486216310`), so it was reverted.
- Rejected after the short literal-length increment fast path: forcing
  `sequence_bitstream`'s all-table and all-RLE sequence encoders to remain
  separate with `#[inline(never)]` preserved focused bytes but worsened three
  focused 20-run instruction samples to about `12.485B`, `12.489B`, and
  `12.485B` (`12484608895`, `12489007295`, `12485197671`), so it was reverted.
- Rejected after the short literal-length increment fast path: changing
  `dynamic_lit_length_code_delta()` to compute the transition delta directly
  from bit-count and frequency-weight differences preserved focused bytes but
  worsened three focused 20-run instruction samples to about `12.488B`,
  `12.490B`, and `12.491B` (`12488111147`, `12489540061`, `12491306078`), so it
  was reverted.
- Rejected after the short literal-length increment fast path: replacing hot
  `saturating_sub()` uses in `opt_match/tree.rs` with explicit C-style `if`
  branches for `btLow` and repetitive-pattern skip positions preserved focused
  bytes but worsened three focused 20-run instruction samples to about
  `12.560B`, `12.557B`, and `12.556B` (`12560230366`, `12556558855`,
  `12555889182`), so it was reverted.
- Rejected after the short literal-length increment fast path: adding a
  4096-entry stateless lookup table for BtOpt `bit_weight()` preserved focused
  bytes but worsened three focused 20-run instruction samples to about
  `12.604B`, `12.605B`, and `12.603B` (`12603629925`, `12605207402`,
  `12603013097`), so it was reverted.
- On July 17, 2026, `HuffmanTable::build_smallest_from_counts()` changed to
  reuse the nonzero symbol count already available from `base_code_lengths()`
  when choosing the minimum candidate bit width. The fallback path still scans
  counts when the base tree is unavailable, so table selection and output are
  unchanged. Focused bytes were unchanged (`9198420` for 20 runs, `36793680`
  for 80 runs), and three focused 20-run instruction samples were `12.498B`,
  `12.497B`, and `12.495B` (`12498055159`, `12497391247`, `12494754101`).
  The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-nonzero-count-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman/parser tests,
  full `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected after the Huffman nonzero count reuse: precomputing a pass-local
  `[i32; 17]` table for short literal-length increment prices preserved focused
  bytes but the first focused 20-run instruction sample regressed to about
  `12.681B` (`12681380226`) versus the `12.495B` to `12.498B` checkpoint, so it
  was reverted.
- On July 17, 2026, `forward_pass()` changed to use
  `OptPriceState::dynamic_lit_length_price()` for parser-root and zero-literal
  match prices. Optimal parser blocks return before the predefined-price case
  can be used (`block_len <= HASH_READ_SIZE`), so this preserves the focused
  BtOpt price model while avoiding dead predefined-price branches in the hot
  setup and match-price paths. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `12.506B`, `12.505B`, and `12.506B` (`12506188790`,
  `12505167428`, `12506493283`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-dynamic-literal-length-price-helper.perf.data`.
  Validation passed for `cargo fmt --check`, focused price/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected after the dynamic literal-length price helper: changing
  `OptPriceState::update_stats()` to take an exact literal slice instead of
  C-shaped `(lit_length, literals)` arguments preserved focused bytes but
  worsened three focused 20-run instruction samples to about `12.517B`
  (`12517484677`, `12517664869`, `12516688879`), so it was reverted.
- On July 17, 2026, `forward_pass()` changed to call
  `OptPriceState::dynamic_raw_literal_cost()` when filling the parser's lazy
  literal-price cache. Optimal parser blocks return before the predefined-price
  case can be used (`block_len <= HASH_READ_SIZE`), and this Rust port keeps
  compressed literal pricing enabled on this path, so this preserves the
  focused BtOpt price model while avoiding dead literal-price branches in the
  hot raw-literal price path. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `12.529B`, `12.529B`, and `12.535B` (`12529881732`,
  `12528528169`, `12534818387`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-dynamic-raw-literal-price-helper.perf.data`.
  Validation passed for `cargo fmt --check`, focused price/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `forward_pass()` changed to call
  `OptPriceState::dynamic_match_offset_price()` and
  `dynamic_match_length_price()` when seeding and updating match prices.
  Optimal parser blocks return before the predefined-price case can be used, so
  this preserves the focused BtOpt price model while avoiding dead predefined
  branches in the hot match-price path. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.552B`, `12.549B`, and `12.550B`
  (`12551897402`, `12549331712`, `12549532267`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-dynamic-match-price-helpers.perf.data`.
  Validation passed for `cargo fmt --check`, focused price/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `forward_pass()` changed to call
  `OptPriceState::dynamic_lit_length_increment_price()` through its local
  `ll_increment_price()` helper. Optimal parser blocks return before the
  predefined-price case can be used (`block_len <= HASH_READ_SIZE`), so this
  preserves the focused BtOpt price model while avoiding the dead predefined
  branch in the hot literal update path. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.579B`, `12.583B`, and `12.578B`
  (`12579256659`, `12583461014`, `12577903159`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-dynamic-ll-increment-helper.perf.data`.
  Validation passed for `cargo fmt --check`, focused price/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild. Rejected in the
  same pass: replacing match-node struct-update writes with direct field
  assignments preserved bytes but worsened focused 20-run instruction samples
  to about `12.631B`, `12.633B`, and `12.628B`, so it was reverted.
- On July 17, 2026, `seed_match_prices()` changed to return the computed
  `LL_PRICE(0)` value and `forward_pass()` reuses it when calling
  `update_match_prices()`. The optimal-parser price state is unchanged between
  seeding and the forward pass, so this preserves the C price model while
  avoiding a duplicate literal-length price calculation for every parser
  segment. Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.627B`,
  `12.625B`, and `12.628B` (`12626825870`, `12624992297`, `12627862842`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-seeded-zero-ll-price-threading.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `forward_pass()` changed to compute `LL_PRICE(0)` once per
  pass and pass it into `update_match_prices()`. The optimal-parser price state
  is unchanged during the pass, so this preserves the C price model while
  avoiding repeated literal-length price work before match-price updates.
  Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.669B`,
  `12.665B`, and `12.668B` (`12669108032`, `12664597852`, `12667522755`).
  The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-forward-zero-ll-price-hoist.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild. Rejected in the
  same pass: caching dynamic literal-length symbol prices and transition
  increments inside `OptPriceState` preserved bytes but regressed the first
  focused 20-run instruction sample to about `13.520B` (`13519765282`), likely
  due to extra state/layout or refresh cost; it was reverted.
- On July 17, 2026, `forward_pass()` changed to compute the ULTRA
  `LL_INCPRICE(1)` value once per pass and pass it into
  `update_literal_price()`. The optimal-parser price state is unchanged during
  the pass, so this preserves the C price model while avoiding repeated
  literal-length increment work in the hot literal-replacement branch. Focused
  bytes were unchanged (`9198420` for 20 runs, `36793680` for 80 runs). Six
  focused 20-run instruction samples were `12.689B`, `12.684B`, `12.689B`,
  `12.685B`, `12.684B`, and `12.684B` (`12688839304`, `12684087332`,
  `12689369275`, `12685012766`, `12684243667`, `12684362756`). The profile
  artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-forward-one-literal-increment-hoist.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `refresh_node_reps()` in the optimal parser changed to
  match C's offset-history refresh condition, `if (opt[cur].litlen == 0)`, by
  removing Rust's extra `mlen == 0` defensive guard. Parser initialization
  should guarantee that, for `cur >= 1`, a zero literal length means the node
  ends in a real match. Focused bytes were unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs), and three focused 20-run instruction samples were
  `12.687B`, `12.688B`, and `12.690B` (`12687496008`, `12687623422`,
  `12689835773`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-refresh-node-reps-c-guard.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `HuffmanTable::build_from_weights()` changed to compute
  the minimum nonzero Huffman weight during its first weight scan, set
  `table.max_num_bits` once from that minimum after `max_num_bits` is known,
  and avoid the per-symbol `table.max_num_bits.max(current_num_bits)` update.
  This preserves the canonical codes because `current_num_bits` is
  `max_num_bits - weight + 1`, so the largest bit width occurs at the smallest
  nonzero weight. Focused bytes were unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs), and three focused 20-run instruction samples were
  `12.697B`, `12.701B`, and `12.698B` (`12697186055`, `12701063410`,
  `12698498767`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-max-bits-once.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, Huffman weight-table encoding changed to compute
  `c_huff_weight_fse_is_unusable()` once in `encoded_weight_table_bytes()` and
  pass that result into `encode_weight_table_fse_bytes()`. This preserves the C
  FSE/raw selection logic while avoiding a duplicate weight count/max scan.
  Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.713B`,
  `12.712B`, and `12.712B` (`12713240712`, `12712209042`, `12712357645`).
  The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-weight-usability-scan-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild. Rejected in the
  same pass: generating the Huffman table description directly from canonical
  codes preserved bytes but regressed focused instruction samples to about
  `12.94B`, so it was reverted. Also rejected: replacing the release FSE
  direct-lookup unsafe initialization with safe sentinel or zero-filled vectors
  preserved bytes but regressed focused samples to about `12.79B` to `12.80B`;
  forcing `FSETable::c_start_state_index()` inline preserved bytes but was
  slightly worse at about `12.718B` to `12.720B`. Rejected parser experiments
  from the same checkpoint: copying `state.opt[cur]` into a local `Optimal`
  after rep refresh preserved bytes but regressed focused samples to about
  `12.80B`; replacing the half-bit skip threshold helper call with a local
  constant preserved bytes but only sampled around `12.716B` to `12.718B`, so
  it was reverted as noise/worse than the checkpoint.
- On July 17, 2026, Huffman `limit_code_lengths()` changed its temporary
  `rank_last` structure from an allocated `Vec<Option<usize>>` to a fixed
  sentinel array. This mirrors C's stack `rankLast` table in `HUF_setMaxHeight`
  while staying in safe Rust. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `12.715B`, `12.714B`, and `12.714B` (`12714646163`,
  `12713568259`, `12713852193`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-rank-last-array.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `LiteralStats::from_literals_with_stream_counts()` changed
  to count literal symbols first and compute `largest` plus `max_symbol` in a
  fixed 256-counter pass afterward. This preserves four-stream counts while
  removing two max updates from every literal in the hot counting loop. Focused
  bytes were unchanged (`9198420` for 20 runs, `36793680` for 80 runs), and
  three focused 20-run instruction samples were `12.751B`, `12.750B`, and
  `12.751B` (`12751122656`, `12750146084`, `12751293243`). The profile
  artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-literal-stats-post-count-max.perf.data`.
  Validation passed for `cargo fmt --check`, focused literal/compressed tests,
  full `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `distribute_weights()` changed from repeatedly pushing
  identical rank-limited weights to filling each run with `Vec::resize()`.
  Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.947B`,
  `12.944B`, and `12.943B` (`12947222283`, `12943655778`, `12942817724`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-rank-limited-resize-fill.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected on July 17, 2026 after the unrolled no-dict repcodes: removing the
  hot `BtMatchRequest` wrapper preserved focused bytes but regressed focused
  20-run instruction samples to about `12.98B` (`12976512552`, `12979895227`,
  `12981386844`); making `try_repcode_match_no_dict()` const-generic over the
  repcode was noise-level to slightly worse after six samples; replacing
  literal stream counting's `chunks(split_size).enumerate()` loop with a fixed
  four-slice loop preserved focused bytes but regressed samples to about
  `12.95B` (`12954098953`, `12952131889`, `12953769793`). All three were
  reverted.
- On July 17, 2026, `collect_repcode_matches_no_dict()` changed from iterating
  over a computed `first_rep..last_rep` range to unrolled fixed no-dictionary
  repcode probes, with the common probe body in `try_repcode_match_no_dict()`.
  Ext-dict repcode handling is unchanged. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.950B`, `12.949B`, and `12.950B`
  (`12950491653`, `12948870263`, `12949894502`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-unrolled-nodict-repcodes.perf.data`.
  Validation passed for `cargo fmt --check`, focused match/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `should_stop_after_best_match()` changed from
  `matches.last().is_some_and(...)` to an explicit empty check plus direct
  `get(len - 1)` of the latest match. This preserves C's sufficient-match/block
  end stop condition while avoiding the option/closure path in hot match
  collection. Focused bytes were unchanged (`9198420` for 20 runs, `36793680`
  for 80 runs), and three focused 20-run instruction samples were `13.270B`,
  `13.269B`, and `13.268B` (`13270212203`, `13268583477`, `13268343682`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-direct-best-match-stop.perf.data`.
  Validation passed for `cargo fmt --check`, focused match/parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected on July 17, 2026 after the Huffman weight-loop cleanup: forcing
  `forward_pass()` inline preserved focused bytes but regressed focused 20-run
  instruction samples to about `13.45B` (`13455426768`, `13452489912`,
  `13454993841`); forcing inline on `OptMatchBounds::lowest_match_index()` and
  `lowest_prefix_index_with_loaded_dict()` preserved focused bytes but was
  neutral to slightly worse at about `13.30B` (`13299036958`, `13297773146`,
  `13295556789`). Both were reverted.
- On July 17, 2026, `HuffmanTable::build_from_weights()` changed its hot
  weight scan from `weights.iter().copied().filter(...)` to an explicit loop
  that skips zero weights. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `13.297B`, `13.295B`, and `13.298B` (`13297100154`,
  `13295365817`, `13297570728`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-weight-loop-cleanup.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected on July 17, 2026 after the sequence-cost scan removal: using input
  Huffman weights directly for table descriptions broke Huffman/compressed-block
  tests with decode corruption; shrinking the Huffman-weight FSE count array to
  16 entries preserved focused bytes but regressed focused 20-run instruction
  samples to about `13.42B` (`13342441267`, `13342413364`, `13341600468`);
  replacing sequence-cost `iter_present()` chains with direct loops preserved
  focused bytes but regressed samples to about `13.300B` (`13300498198`,
  `13300799788`, `13300154161`). All three were reverted.
- On July 17, 2026, C-style optimal sequence table selection stopped doing a
  full `default_allowed()` scan before computing basic-table cross entropy, and
  `repeat_table_cost()` stopped scanning once to prove support before scanning
  again to compute bit cost. Unsupported symbols still return `None` from the
  cost helpers, preserving the selected table mode while removing redundant
  symbol-count scans. Focused bytes were unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs), and three focused 20-run instruction samples were
  `13.300B`, `13.299B`, and `13.294B` (`13299515088`, `13298522967`,
  `13294162063`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-sequence-cost-scan-removal.perf.data`.
  Validation passed for `cargo fmt --check`, focused sequence tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected on July 17, 2026: forcing `seed_parser_root()` inline preserved
  focused bytes but worsened focused 20-run instruction samples to about
  `13.44B` (`13440615080`, `13438463217`, `13436664765`) versus the `13.35B`
  no-LDM-specialization baseline, so it was reverted.
- On July 17, 2026, the optimal parser gained a const `WITH_LDM`
  specialization through the MLS/strategy dispatch, `forward_pass()`, and
  `collect_matches_mls()`. Normal no-LDM calls now compile out the LDM
  candidate processing branch, while LDM-enabled calls keep the C-style cursor
  processing path. Focused bytes were unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs), and three focused 20-run instruction samples were
  `13.358B`, `13.353B`, and `13.351B` (`13357588368`, `13353432010`,
  `13350670590`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-no-ldm-specialization.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `seed_match_prices()` was marked `#[inline(always)]`,
  matching C's inlined initial match-price setup inside
  `ZSTD_compressBlock_opt_generic()` instead of keeping it as a separate hot
  Rust symbol. Focused bytes were unchanged (`9198420` for 20 runs,
  `36793680` for 80 runs), and three focused 20-run instruction samples were
  `13.390B`, `13.392B`, and `13.387B` (`13389556631`, `13392412402`,
  `13387000775`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-inline-seed-match-prices.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `SymbolStates::get()` began keeping the rare FSE
  state-search fallback in a cold, non-inlined helper. The common small-table
  path still uses the direct lookup vector, matching C's transform-table style
  more closely in the encoder hot loop, while large-table fallback behavior is
  unchanged. A fresh rebuild before this change measured the focused level-16
  fixture at about `13.407B` instructions (`13405533398`, `13410399268`,
  `13406722006`), so the older `12.902B` checkpoint number should be considered
  stale unless reproduced. The FSE cold-helper change preserved focused bytes
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `13.396B`, `13.396B`, and `13.394B`
  (`13396119373`, `13395971850`, `13394379395`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-cold-search-helper.perf.data`.
  Validation passed for `cargo fmt --check`, `cargo test -p ruzstd fse --quiet`,
  full `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- Rejected after the FSE cold search helper: direct indexing in
  `next_rank_limited_weight()`, individual field writes for hot parser match
  stretches, and combining FSE normalized-probability total/max-symbol scans
  all preserved focused bytes but regressed focused 20-run instruction samples
  to roughly `13.40B` to `13.41B`, so they were reverted.
- On July 17, 2026, `build_smallest_from_counts()` began reusing a cached
  rank-limited weight vector when the base Huffman candidate is unavailable.
  This avoids building and evaluating the same rank-limited fallback table
  twice, while preserving candidate selection and canonical table output.
  Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `12.901B`,
  `12.902B`, and `12.902B` (`12901378143`, `12901668749`, `12901669146`).
  The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-rank-limited-fallback-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `BtMatchRequest` began carrying the parser's raw `[u32; 3]`
  repcode array, and `collect_repcode_matches()` began consuming that array
  directly. This matches `zstd_opt.c`'s direct `rep` array flow and removes the
  hot-path `RepeatOffsets::from_offsets(...).as_offsets()` wrapper churn from
  every optimal match collection. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `12.906B`, `12.905B`, and `12.905B` (`12905636712`,
  `12905345365`, `12905345539`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-raw-rep-match-request.perf.data`.
  Validation passed for `cargo fmt --check`, focused optimal match/parser
  tests, full `ruzstd` tests, clippy, an 80-run focused smoke
  (`36793680` bytes), `git diff --check`, and a release `profile_c_port`
  rebuild.
- On July 17, 2026, `pre_split::Fingerprint::record()` began dispatching the
  four C-generated fingerprint recorder modes `(43,8)`, `(11,9)`, `(5,10)`,
  and `(1,10)` to const-specialized safe Rust helpers. This mirrors
  `zstd_preSplit.c`'s compile-time record functions while preserving C's
  `limit / samplingRate` `nbEvents` accounting. Focused bytes were unchanged
  (`9198420` for 20 runs, `36793680` for 80 runs), and three focused 20-run
  instruction samples were `12.915B`, `12.915B`, and `12.915B`
  (`12914867945`, `12914576966`, `12914576818`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-presplit-specialized-fingerprint.perf.data`.
  Validation passed for `cargo fmt --check`, focused pre-split tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, `FSETable` began storing `SymbolStates` only up to the
  actual normalized-probability slice length instead of always constructing a
  256-symbol array. Missing symbols still report probability zero, while valid
  encoded symbols use the same state lookup path. This matches the C encoder's
  max-symbol bounded FSE table construction more closely and avoids the hot
  fixed 256-entry initialization cost without adding unsafe code. Focused bytes
  were unchanged (`9198420` for 20 runs, `36793680` for 80 runs), and three
  focused 20-run instruction samples were `13.045B`, `13.045B`, and `13.045B`
  (`13045049097`, `13045339529`, `13045048963`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-symbol-vector.perf.data`.
  Validation passed for `cargo fmt --check`, focused FSE tests, full `ruzstd`
  tests, clippy, an 80-run focused smoke (`36793680` bytes),
  `git diff --check`, and a release `profile_c_port` rebuild.
- On July 17, 2026, the normal optimal encode path began recycling its
  temporary `StoredSequence` vector through `OptBlockState` after
  `prepare_from_greedy_output()` copies the sequence metadata into the prepared
  block. This matches C's reused seqStore shape more closely without unsafe
  code. Focused bytes were unchanged (`9198420` for 20 runs, `36793680` for
  80 runs), and three focused 20-run instruction samples were `13.820B`,
  `13.820B`, and `13.820B` (`13820460179`, `13820386739`, `13820461433`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-opt-sequence-scratch.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and a
  release `profile_c_port` rebuild.
- On July 16, 2026, the broad API benchmark over the available
  `realworld-100` fixture links produced slightly smaller Rust output than C
  but still showed a CPU gap in the optimal levels:
  level 8 `-559` bytes / wall-clock ratio `1.74x`, level 16 `-880` bytes /
  wall-clock ratio `1.48x`, and level 19 `-211` bytes / wall-clock ratio
  `1.33x`. The level-16 and level-19 time deltas are still dominated by
  `corpus_z000033`.
- On July 6, 2026, the focused level-16 `corpus_z000033` comparison still shows
  the remaining CPU gap in the optimal-parser family rather than in an unrelated
  subsystem. `profile_c_port` compressed 80 runs in about 11.2s, while the
  matching `profile_c_api` helper compressed 80 runs through `ZSTD_compress2()`
  in about 4.5s.
- On July 16, 2026, a literal-length increment-price fast path in `opt_price.rs`
  preserved the level-16 `corpus_z000033` output (`9198420` bytes for 20 runs)
  and reduced the focused 20-run sample from roughly `2.61s` to `2.53s`.
  `perf stat` still showed about `21.21B` Rust instructions versus C's `8.71B`,
  so this is only a small improvement and the main CPU gap remains in the
  optimal-parser/match-finder path.
- A follow-up C-style match-finder branch-condition change in `opt_match`
  preserved output and produced the current focused 80-run sample:
  Rust `36793680` bytes in about `9.91s` versus C API `36864480` bytes in about
  `4.41s`. The corresponding `perf stat` sample still shows about `21.20B`
  Rust instructions, so the remaining gap is still much larger than this
  branch-shape improvement.
- Reusing already-computed LL/ML/OF code counts in the non-exact post-split
  estimator preserved the same focused bytes and reduced the 20-run instruction
  count to about `20.92B`. The longer 80-run wall-clock samples remained around
  `10.0s`, so this is an instruction-count cleanup rather than a large elapsed
  time win.
- A C-shaped no-dictionary repcode window guard in `opt_match` preserved the
  focused level-16 bytes and shaved a tiny number of instructions from the hot
  sample by matching C's unsigned `repOffset - 1 < curr - windowLow` test.
- A direct `OptMatchTable::get()` accessor was kept because it preserved the
  focused level-16 bytes and reduced the hot sample to about `20.88B`
  instructions by avoiding the sliced `Index` path in parser match loops.
- The no-dictionary repcode guard now precomputes C's `curr - windowLow`
  distance and uses wrapping `rep[0] - 1`, preserving focused bytes and reducing
  the hot sample to about `20.83B` instructions.
- The initial optimal-parser match-seed sentinel now matches C by resetting only
  the sentinel price instead of overwriting the full `Optimal`. This preserved
  focused bytes and reduced the hot sample to about `20.82B` instructions.
- Huffman table construction now caches the serialized table-description bytes
  and `HuffmanEncoder::write_table()` reuses them instead of reconstructing the
  same header during emission. After fixing the small-table nibble order to
  match the old bit-level writer, full `ruzstd` tests pass and the focused
  level-16 sample is about `20.69B` instructions. The latest 80-run comparison
  is Rust `36793680` bytes in about `9.4s` to `10.2s` versus C API `36864480`
  bytes in about `4.9s`.
- Literal C-cost-model decisions and block-size estimates now use the cached
  Huffman table-description length instead of subtracting two encoded-length
  estimates. This preserved focused bytes and reduced the hot level-16 sample
  to about `20.66B` instructions.
- Post-split recursion now reuses parent-computed half-size estimates when a
  split is accepted, avoiding duplicate estimator calls for child intervals.
  This preserved focused bytes and reduced the hot level-16 sample to about
  `19.18B` instructions.
- Estimate-only offBase conversion now avoids replaying local offset-history
  updates when every C-port sequence already has a preencoded offset value.
  This preserved focused bytes and reduced the hot level-16 20-run sample to
  about `19.11B` instructions. Full `ruzstd` tests and clippy pass after this
  change.
- The hot parser `price_i32()` helper now mirrors C's bounded `int` conversion
  with a debug-asserted cast instead of a release-mode saturating
  `TryFrom<u32>`. This preserved focused bytes and reduced the hot level-16
  20-run sample to about `19.10B` instructions.
- `OptPriceState` now caches the scalar `lit_sum_base_price -
  BITCOST_MULTIPLIER` as `lit_price_max` when base prices are refreshed. This
  preserved focused bytes and stayed around `19.10B` instructions across three
  focused 20-run samples (`19.096B`, `19.104B`, `19.098B`), so treat it as
  neutral-to-tiny cleanup rather than a material CPU win.
- The post-split encoder now preallocates the split list, the concatenated
  partition output buffer, and each partition encoder's raw-size buffer.
  `PreparedChunk` / `prepared_chunk()` were checked and are test-only. This
  preserved the focused level-16 bytes (`9198420` for 20 runs) and produced
  three focused `perf stat` samples around `19.09B` instructions (`19.092B`,
  `19.089B`, `19.090B`). Treat it as a tiny allocation cleanup, not a material
  closure of the C CPU gap.
- Fresh profile artifact:
  `benchmarks/tmp/profile-c-port-z000033-l16-after-post-split-prealloc.perf.data`.
  The profile still shows `forward_pass` and
  `compress_block_opt_with_state_and_ldm` as the dominant Rust hot spots, with
  `post_split::estimate_partition_size_with_sequences` and `__memmove` still
  visible at about `4.8%` each. A suspected `nextToUpdate` parity issue was
  checked; the public `opt_match.rs` wrapper already returns zero matches when
  `ip < state.next_to_update`, matching C's `ZSTD_btGetAllMatches_internal()`,
  so do not duplicate that guard in `opt_match/tree.rs`.
- `OptPriceState::match_price()` is now split into exact offset and
  match-length components, allowing `forward_pass` to hoist the offset-price
  component once per `OptMatch` while scanning candidate match lengths. This
  preserves the C price formula and focused level-16 bytes (`9198420` for
  20 runs), while three focused `perf stat` samples dropped to about `19.05B`
  to `19.06B` instructions (`19.057B`, `19.058B`, `19.050B`). Treat it as
  another small parser cleanup, not a material closure of the C CPU gap.
- `FSEEncoder::encode_symbol()` now has `#[inline(always)]`. This preserved the
  focused level-16 bytes (`9198420` for 20 runs) and dropped focused
  instruction samples to about `18.73B` (`18.729B`, `18.732B`, `18.728B`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-encode-symbol-inline.perf.data`.
  The interrupted validation was later completed: clippy and `git diff --check`
  passed, and a short report showed the parser and match-finder still dominant.
- Rejected July 17 experiments: Huffman weight-table analysis reuse in
  `huff0_encoder.rs` and parser `state.opt[cur].rep` hoisting both preserved
  bytes but worsened focused instruction counts, so they were reverted.
- `count_match_no_dict()` now uses the same loop-limit shape as C's
  `ZSTD_count()`: precompute `match_limit - 7`, perform the first word compare,
  then loop while `pos < loop_limit`. This preserved focused bytes (`9198420`
  for 20 runs, `36793680` for 80 runs) and reduced three 20-run instruction
  samples to about `18.56B` (`18.567B`, `18.558B`, `18.565B`). The profile
  artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-count-match-loop-limit.perf.data`.
  Validation passed for focused match-count, opt-match, and opt-parser tests,
  clippy, formatting, and `git diff --check`.
- Rejected July 17 literal-price cache experiment: caching all 256 literal
  symbol prices in `OptPriceState` preserved focused bytes but worsened the
  20-run sample to about `22.70B` instructions because refresh work dominated,
  so it was reverted.
- `HuffmanNode` now uses sentinel `usize` values for missing parent/symbol
  instead of `Option<usize>`, matching the compact C workspace shape more
  closely, and `length_limited_code_lengths()` reserves exact parent-node
  capacity before building the tree. This preserved focused bytes (`9198420`
  for 20 runs, `36793680` for 80 runs). The sentinel-only samples were about
  `18.49B` instructions (`18.495B`, `18.487B`, `18.486B`); with the reserve they
  were about `18.48B` (`18.485B`, `18.479B`, `18.477B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-node-workspace.perf.data`.
  Focused Huffman tests, clippy, formatting, and `git diff --check` passed.
- `build_table_from_data()` and `build_huffman_weight_table_from_data()` now
  update `max_symbol` while counting input symbols, removing the follow-up
  256-entry count-table scan before FSE normalization. This preserved focused
  bytes (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three
  20-run instruction samples to about `18.43B` (`18.434B`, `18.430B`,
  `18.429B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-max-symbol-inline.perf.data`.
  Focused FSE tests, clippy, formatting, and `git diff --check` passed.
- `update_match_prices()` no longer writes the current rep history into every
  candidate match endpoint. This matches the C parser update path, which writes
  `mlen`, `off`, `litlen`, and `price` there and refreshes rep history only
  when a priced endpoint becomes the current node. This preserved focused bytes
  (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three 20-run
  instruction samples to about `18.40B` (`18.405B`, `18.403B`, `18.398B`).
  Full `ruzstd` tests, focused parser/match/FSE tests, clippy, formatting, and
  `git diff --check` passed.
- `seed_match_prices()` also leaves rep history untouched for initial match
  endpoints, matching C's initial match seeding path. Rep history is still set
  on literal states and refreshed when a match endpoint becomes current. This
  preserved focused bytes (`9198420` for 20 runs, `36793680` for 80 runs) and
  reduced three 20-run instruction samples to about `18.39B` (`18.398B`,
  `18.390B`, `18.388B`). Full `ruzstd` tests, focused parser/match tests,
  clippy, formatting, and `git diff --check` passed.
- The initial `seed_match_prices()` placeholder loop now mutates only `price`,
  `mlen`, and `litlen`, matching C's partial initialization before the literal
  update pass. This avoids rewriting `off` and rep history for placeholders
  that normal literal propagation will fix. It preserved focused bytes
  (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three 20-run
  instruction samples to about `18.38B` (`18.385B`, `18.380B`, `18.376B`).
  Full `ruzstd` tests, focused parser/match tests, clippy, formatting, and
  `git diff --check` passed.
- The `update_match_prices()` frontier extension loop now mutates only `price`
  and nonzero `litlen` for empty positions, matching C's
  `opt[last_pos].price = ZSTD_MAX_PRICE` and `opt[last_pos].litlen = !0`
  writes. It preserved focused bytes (`9198420` for 20 runs, `36793680` for
  80 runs) and reduced three 20-run instruction samples to about `18.37B`
  (`18.373B`, `18.372B`, `18.370B`). Full `ruzstd` tests, focused parser/match
  tests, clippy, formatting, and `git diff --check` passed.
- `update_literal_price()` now reads only the previous node's `price` and
  `litlen` before comparing the literal candidate price, and copies the full
  previous `Optimal` only after the literal path wins. This mirrors C's
  `opt[cur] = opt[cur-1]` placement after the price comparison. It preserved
  focused bytes (`9198420` for 20 runs, `36793680` for 80 runs) and reduced
  three 20-run instruction samples to about `18.36B` (`18.366B`, `18.359B`,
  `18.360B`). Full `ruzstd` tests, focused parser/match tests, clippy,
  formatting, and `git diff --check` passed.
- `FSETable` now caches the `c_start_state_index()` result for each symbol
  during table construction, matching C's precomputed `FSE_initCState2()`
  transform path instead of reconstructing the symbol rank during stream
  initialization. This preserved focused bytes (`9198420` for 20 runs,
  `36793680` for 80 runs) and reduced three focused 20-run instruction samples
  to about `14.74B` (`14.735B`, `14.735B`, `14.736B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-start-state-cache.perf.data`.
  Validation passed for `cargo fmt --check`, focused FSE tests, full `ruzstd`
  tests, clippy, and `git diff --check`.
- `LiteralStats::from_literals_with_stream_counts()` now uses chunk iteration
  for four-stream literal counts instead of computing `pos / split_size` for
  every literal while estimating split partitions. This preserved focused bytes
  (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three focused
  20-run instruction samples to about `14.44B` (`14.436B`, `14.443B`,
  `14.436B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-literal-stats-chunked-stream-counts.perf.data`.
  Validation passed for `cargo fmt --check`, focused literal/compressed/Huffman
  tests, full `ruzstd` tests, clippy, an 80-run focused smoke
  (`36793680` bytes), and `git diff --check`.
- The optimal parser now reserves sequence output at `block_len / min_match`,
  matching C's `ZSTD_maxNbSeq(blockSize, minMatch, ...)` capacity formula for
  the no external sequence-producer path. The previous Rust reserve was
  effectively `block_len / (min_match * 4)`, which caused extra
  reallocation/memmove work in the hot level-16 fixture. It preserved focused
  bytes (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three
  focused 20-run instruction samples to about `13.93B` (`13.928B`, `13.928B`,
  `13.928B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-c-sequence-capacity.perf.data`.
  Validation passed for `cargo fmt --check`, focused `opt_parser` tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  `git diff --check`.
- `redistribute_weights()` now caches each weight's half-rank unit while
  repaying overflow instead of recomputing `1 << (weight - 1)` in the repeated
  scan. This preserves the old scan order and candidate selection while
  removing hot-loop shift work. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were about `13.904B` (`13.904B`, `13.904B`, `13.904B`). The profile
  artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-redistribute-unit-cache.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  `git diff --check`.
- Direct lookup construction in the FSE table builder now uses a debug-asserted
  `u16` cast for bounded per-symbol state indexes instead of a release-time
  `u16::try_from()` branch in the nested lookup fill loop. Focused bytes were
  unchanged (`9198420` for 20 runs, `36793680` for 80 runs), and three focused
  20-run instruction samples were about `13.900B` (`13.900B`, `13.900B`,
  `13.900B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-fse-lookup-index-cast.perf.data`.
  Validation passed for `cargo fmt --check`, focused FSE tests, full `ruzstd`
  tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  `git diff --check`.
- The full 256-entry lazy literal-price cache now lives in `OptBlockState` and
  uses per-pass generation stamps, preserving the lazy per-symbol lookup
  behavior while avoiding the per-forward-pass `[u32; 256]` sentinel memset.
  A follow-up narrowed those stamps from `u32` to `u16`, reducing cache
  footprint while keeping rare wraparound clearing. This is distinct from the
  rejected eager `OptPriceState` literal table, one-entry cache, and 64-entry
  direct-mapped cache shapes. Focused bytes were unchanged (`9198420` for
  20 runs, `36793680` for 80 runs), and three focused 20-run instruction
  samples were `13.868B`, `13.868B`, and `13.868B` (`13868231751`,
  `13868206912`, `13868208454`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-literal-cache-u16-generation.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser and price tests,
  full `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  `git diff --check`.
- The rank-limited non-zero weight generator now stores temporary weights as
  `u8` and casts back to `usize` only when distributing them into the public
  table-builder input. The generated weight values are tiny
  (`<= MAX_HUFFMAN_BITS`), so this preserves output while reducing temporary
  memory traffic in `rank_limited_nonzero_weights()`. Focused bytes were
  unchanged (`9198420` for 20 runs, `36793680` for 80 runs), and three focused
  20-run instruction samples were `13.867B`, `13.867B`, and `13.867B`
  (`13866999168`, `13867018924`, `13867018692`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-rank-limited-u8-weights.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, an 80-run focused smoke (`36793680` bytes), and
  `git diff --check`.
- Rejected July 17 parser/match experiment after the Huffman redistribution
  unit-cache change: forcing `OptMatchBounds::rep_match_length()` inline
  preserved focused bytes but worsened the first focused 20-run instruction
  sample to about `13.952B` versus the `13.904B` baseline, so it was reverted.
  The focused fixture is no-dictionary and the extra code size outweighed
  removing the cold ext-dict helper call.
- Rejected July 17 parser match-length price experiment after the FSE
  start-state cache: a tiny per-match ML-code price cache in `forward.rs`
  preserved focused bytes (`9198420` for 20 runs) but worsened the first
  20-run instruction sample to about `14.868B` versus the `14.735B` to
  `14.736B` baseline, so it was reverted.
- Rejected July 17 parser path-order experiment: leaving `select_path()` output
  in backward order and consuming it with `path.iter().rev()` removed the
  explicit `path.reverse()` and preserved focused bytes, but worsened two
  20-run instruction samples to about `14.757B` and `14.755B` versus the
  current `14.735B` to `14.737B` baseline. It was reverted.
- Rejected July 17 FSE transform-table experiment: adding C-style
  `deltaNbBits`/`deltaFindState` transforms and a C-ordered state table for
  `FSEEncoder::encode_symbol()` preserved focused bytes (`9198420` for
  20 runs) after fixing Rust's explicit bit masking and the single-symbol
  wrapping subtraction, but the first focused instruction sample worsened to
  about `14.990B` versus the `14.733B` to `14.737B` baseline. It was reverted.
- Rejected July 17 parser literal-cost cache experiment: a one-entry
  `LiteralCostCache` in `opt_parser/forward.rs` preserved focused bytes
  (`9198420` for 20 runs) but worsened three 20-run instruction samples to
  about `18.39B` (`18.396B`, `18.390B`, `18.393B`) versus the prior `18.366B`,
  `18.359B`, `18.360B` baseline, so it was reverted. After the revert,
  `cargo fmt --check`, focused `opt_parser` and `opt_match` tests, and the
  release `profile_c_port` build passed; the focused smoke still produced
  `9198420` bytes and one instruction sample returned to about `18.356B`.
- Huffman length limiting now reuses the already sorted leaf order from
  `length_limited_code_lengths()` when entering `limit_code_lengths()`, removing
  a duplicate non-zero-symbol collection and sort. This preserved focused bytes
  (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three focused
  20-run instruction samples to about `17.07B` (`17.069B`, `17.070B`,
  `17.064B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-limit-sort-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman and parser tests,
  full `ruzstd` tests, clippy, and `git diff --check`.
- `build_smallest_from_counts()` now computes the base Huffman code lengths once
  and reuses cloned base lengths for each candidate max-bit limit, instead of
  rebuilding the same Huffman tree for every candidate. This preserved focused
  bytes (`9198420` for 20 runs, `36793680` for 80 runs) and reduced three
  focused 20-run instruction samples to about `15.97B` (`15.972B`, `15.974B`,
  `15.971B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-base-length-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman tests, full
  `ruzstd` tests, clippy, and `git diff --check`.
- `build_smallest_from_counts()` now also derives the initial best table from
  the same cached base Huffman lengths, removing the remaining duplicate
  `build_from_counts()` tree build before the candidate max-bit search. This
  preserved focused bytes (`9198420` for 20 runs, `36793680` for 80 runs) and
  reduced three focused 20-run instruction samples to about `15.50B`
  (`15.498B`, `15.503B`, `15.500B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-best-base-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman and parser tests,
  full `ruzstd` tests, clippy, and `git diff --check`.
- Rank-limited Huffman table construction inside `build_smallest_from_counts()`
  now reuses the base Huffman pass's existing symbols sorted by count,
  preserving tie order while avoiding a second non-zero-symbol sort. This
  preserved focused bytes (`9198420` for 20 runs, `36793680` for 80 runs) and
  reduced three focused 20-run instruction samples to about `15.34B`
  (`15.340B`, `15.340B`, `15.338B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-rank-limited-order-reuse.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman/rank-limited/parser
  tests, full `ruzstd` tests, clippy, and `git diff --check`.
- `build_from_weights()` now precomputes per-weight symbol counts and starting
  codes, then assigns canonical codes in one symbol pass instead of scanning
  all symbols once per weight rank. This preserved focused bytes (`9198420` for
  20 runs, `36793680` for 80 runs) and reduced three focused 20-run instruction
  samples to about `14.83B` (`14.823B`, `14.827B`, `14.827B`). The profile
  artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-huffman-code-start-table.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman/parser tests, full
  `ruzstd` tests, clippy, and `git diff --check`.
- Rank-limited weight generation now consumes the generated non-zero weights in
  forward order instead of reversing the vector and popping it back to the
  original order; the distributed weight vector also reserves its exact
  capacity. This preserved focused bytes (`9198420` for 20 runs, `36793680` for
  80 runs) and reduced three focused 20-run instruction samples to about
  `14.80B` (`14.796B`, `14.800B`, `14.803B`). The profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-rank-limited-forward-consume.perf.data`.
  Validation passed for `cargo fmt --check`, focused Huffman/parser tests, full
  `ruzstd` tests, clippy, and `git diff --check`.
- `forward_pass()` now keeps a per-pass lazy literal-price cache for
  `raw_literal_cost()` lookups. This avoids repeating the hot literal
  `highbit32()` work without reintroducing the rejected eager 256-entry
  `OptPriceState` cache refresh cost. This preserved focused bytes (`9198420`
  for 20 runs, `36793680` for 80 runs) and reduced three focused 20-run
  instruction samples to about `14.70B` (`14.770B`, `14.771B`, `14.769B`). The
  profile artifact is
  `benchmarks/tmp/profile-c-port-z000033-l16-after-forward-literal-price-cache.perf.data`.
  Validation passed for `cargo fmt --check`, focused parser/price tests, full
  `ruzstd` tests, clippy, and `git diff --check`.
- Rejected July 17 parser cache experiments after the kept literal-price cache:
  a direct-mapped per-pass `match_length_price()` cache preserved focused bytes
  but worsened the 20-run instruction sample to about `14.864B`; shrinking the
  literal-price cache to a 64-entry direct-mapped cache also preserved bytes but
  worsened the sample to about `14.801B`. Both were reverted. Keep the full
  256-entry lazy literal-price cache unless a new profile supports a different
  shape.
- Rejected July 17 Huffman table-description experiment: deriving serialized
  table-description weights directly from the original weight vector preserved
  focused bytes but worsened focused instruction samples to about `15.447B`
  (`15.447B`, `15.447B`, `15.447B`) versus the `15.34B` baseline, so it was
  reverted. Keep `weights_from_codes()` unless a new profile shows a different
  reason to change that path.
- Latest profiling note: the debug-line release profile lives at
  `benchmarks/tmp/profile-c-port-z000033-l16-current-debug-lines.perf.data`.
  Source-line sorting collapsed to codegen units, so use `perf annotate
  --stdio` on `opt_parser::forward::forward_pass` and related symbols before
  making the next parser or match-finder change.
- The Rust profile for that fixture is led by
  `opt_match::tree::bt_get_all_matches_no_dict_mls`, with secondary cost in
  post-split block-size estimation, FSE table building, Huffman table building,
  and optimal-parser price updates.
- The C API profile for the same fixture is led by
  `ZSTD_btGetAllMatches_noDict_5`, `ZSTD_compressBlock_opt0`, and
  `ZSTD_insertBt1`, so the next high-value work should stay focused on the
  `zstd_opt.c` match finder/parser implementation before broad entropy
  refactors.
- Rejected micro-optimizations from the same session: unchecked byte reads for
  the binary-tree branch byte compare, iterating over a match slice instead of
  indexing `state.matches`, moving the FSE spread-symbol scratch buffer to the
  stack, lowering the FSE direct-lookup threshold, and enabling the optional
  `ZSTD_C_PREDICT` binary-tree insertion shortcut. The prediction shortcut
  changed compressed output and made the broad 100-file byte comparison worse
  overall, so keep it out unless revisited with stronger evidence. The other
  rejected experiments preserved compressed output bytes but did not improve the
  focused 80-run timing sample. Later rejected experiments include converting
  the optimal parser table from `Vec` to a boxed fixed array, caching
  per-literal prices in `OptPriceState`, earlier failed match-count loop
  variants, a small LL/ML/OF symbol-price cache in `OptPriceState`,
  cached dynamic literal-length transition prices in `OptPriceState`, a tracked
  `CodeCounts::max_symbol` field, mutating only individual fields when storing
  match candidates, forcing `refresh_node_reps()` inline, and a one-entry
  parser `LiteralCostCache`. These preserved compressed output bytes but made
  the focused timing or instruction count worse.
- Prepared-CDict dictionary retention was corrected after tracing focused
  `autovt@.service` level 16 against `ZSTD_compress_usingCDict()`. C keeps the
  CDict match state sized and loaded with its `CreateCDict` parameters while
  shrinking only the active source tables. Rust had passed the active params
  into `loaded_dictionary_content()` and retained only 32 KiB of a 51,812-byte
  parsed dictionary. `DictionaryFrameContext` now accepts separate dictionary
  parameters, and attached Greedy and optimal frames pass the CDict values.
  The focused frame is byte-identical at 314 bytes; both outputs hash to
  `22562be06fbc3266e9fabb342d48fe15aae3ea212e201f16b66b49496053d651`.
- Normal literal compression now carries the full-dictionary Huffman repeat
  validity that C establishes in `ZSTD_loadCEntropy()`: the table is valid only
  when all 256 symbols have nonzero weights. For Fast, DFast, and Greedy,
  valid tables on literal sections up to 1024 bytes take C's repeat-first path
  before histogram rejection. A fresh emitted table disables this state,
  matching C's transition to `HUF_repeat_check`. This closed the focused
  `apt-show-versions.timer` level-5 difference: Rust and C both emit 110-byte
  frames with the same 17 sequences and 34-byte treeless literal payload.
- Broad prepared-CDict validation used the first 100 sorted files under
  `/usr/lib/systemd/system`, levels 1, 3, 5, 8, 11, 15, 16, and 19, one run,
  the C API backend, and the 51,962-byte full dictionary at
  `benchmarks/archive/tmp/dict-focused/dict_dictionary.bin`. Artifact:
  `benchmarks/tmp/prepared-dictionary-systemd-100-levels-1-3-5-8-11-15-16-19-api-after-valid-repeat.csv`.
  Aggregate Rust-minus-C bytes are `-219`, `-67`, `+43`, `+0`, `-1`, `-7`,
  `-1`, and `+0` respectively. The level-5 gap is now `+0.222%`; levels 8-19
  are within seven aggregate bytes. Full `ruzstd` tests, all-target clippy with
  warnings denied, formatting, `git diff --check`, focused inspectors, release
  tool builds, decoding verification, and the broad matrix passed.
- A fresh same-session normal-mode CPU check after the prepared-CDict work
  confirms unchanged focused bytes but reopens the current CPU gap relative to
  an older handoff sample. For 80 runs of `corpus_z000033` level 16, Rust emits
  `36,845,760` bytes and uses `35,910,450,316` instructions,
  `20,316,691,439` cycles, `5,582,566,990` branches, and `214,287,004` branch
  misses. C emits `36,864,480` bytes and uses `34,692,747,639` instructions,
  `18,785,003,535` cycles, `4,327,516,035` branches, and `175,269,944` branch
  misses. Rust is therefore about `+3.51%` in instructions and `+29.0%` in
  branches. Adjacent Rust instruction samples were stable near `35.91B`.
  Profile artifact:
  `benchmarks/tmp/perf-z000033-l16-rust-after-prepared-cdict-fixes.data`; the
  top self costs remain `forward_pass` and
  `compress_block_opt_with_state_and_ldm_mls`, so continue with generated
  optimal-parser/tree control flow rather than dictionary entropy work.
- Optimal parser and match collection now carry a compile-time
  `ATTACHED_DICT` mode in addition to the existing ext-dictionary and loaded-
  dictionary modes. This mirrors C's separately generated no-dictionary and
  `dictMatchState` functions. The normal specialization no longer carries the
  attached-tree `Option` branches, generic repcode dispatch, terminal-match
  budget adjustment, or attached-tree call in its hot generated code. No
  unsafe indexing was added. Focused 20-run output remained `9,211,440` bytes;
  three instruction samples improved from `9.053B`-`9.055B` before the change
  to `8.429B`-`8.432B`, while branches fell from about `1.412B` to `1.352B`.
  For 80 runs Rust used `33,432,141,284` instructions,
  `20,506,680,646` cycles, and `5,350,319,451` branches; same-session C used
  `34,698,126,628` instructions, `20,418,327,807` cycles, and
  `4,328,517,348` branches. Thus Rust is about `3.65%` lower in instructions,
  within `0.43%` in cycles, and still `23.6%` higher in branches. Broad normal
  byte totals remained `-253`, `-174`, and `-64` at levels 16, 19, and 22.
  The 800-row prepared-CDict matrix had no byte changes. Artifacts:
  `benchmarks/tmp/perf-z000033-l16-rust-after-attached-const-specialization.data`,
  `benchmarks/tmp/normal-levels-16-19-22-api-after-attached-const-specialization.csv`,
  and
  `benchmarks/tmp/prepared-dictionary-systemd-100-levels-1-3-5-8-11-15-16-19-api-after-attached-const-specialization.csv`.
  Formatting, focused and full tests, all-target clippy with warnings denied,
  release builds, decode-verified broad comparisons, and `git diff --check`
  passed.
- Sequence FSE repeat state now mirrors C's three-state semantics instead of
  treating every previous table as valid for Fast/DFast's dictionary repeat
  shortcut. Fresh compressed tables become check-only; full dictionary LL/ML
  tables become valid only with full alphabet support; dictionary OF validity
  uses C's reachable-code bound
  `highbit(dictionary_content_size + 128 KiB)` and is downgraded after the
  first source block through `FrameBlockState::record_encoded_block()`. The
  transition therefore also occurs when that block is raw or RLE and after a
  target/split source-block operation. This is all safe Rust.
  Focused normal `corpus_z000050` level 3 is now exactly `440380` bytes in
  Rust and C, with identical 12 blocks, 5,506 sequences, literal sections, and
  fresh `fse/fse/fse` modes. Final normal levels 1-5 across 73 real-world
  fixtures are `-1690`, `-1812`, `-1287`, `-699`, and `-718` aggregate bytes
  versus C; levels 1-3 have no positive rows. Target-2048 level 1 is exact,
  while level 3 improves from three differing rows (`-1829` total, two
  positive) to two (`-82` total, one `+50`). The existing 800-row one-block
  prepared-CDict matrix is unchanged, and a multi-block dictionary inspect
  confirms both encoders rebuild OF after an initial raw source block while
  continuing to repeat LL/ML. Final focused level-16 80-run perf counters are
  Rust `33,431,688,254` instructions, `19,085,354,127` cycles,
  `5,349,880,293` branches, and `215,270,558` misses versus C
  `34,691,854,757`, `18,475,435,152`, `4,327,309,647`, and `175,192,529`.
  Rust is `3.63%` lower in instructions, `23.6%` higher in branches, and the
  latest noisy cycle sample is `3.3%` higher. Artifacts:
  `benchmarks/tmp/inspect-z000050-l3-after-fse-repeat-validity`,
  `benchmarks/tmp/normal-levels-1-22-api-after-fse-repeat-validity.csv`,
  `benchmarks/tmp/normal-levels-1-5-api-final-fse-repeat-state.csv`,
  `benchmarks/tmp/target-2048-levels-1-3-api-final-fse-repeat-state.csv`, and
  `benchmarks/tmp/prepared-cdict-realworld-levels-1-3-after-fse-repeat-state.csv`.
  Validation passed all 690 library tests plus 7 integration tests, all-target
  clippy with warnings denied, formatting, release builds, broad decode
  comparisons, and `git diff --check`.
- C-cost literal histogram work now avoids four per-stream 256-bin counters
  unless the small-table search actually consumes them; C's normal emitted and
  post-split cost models use the combined histogram plus the fixed four-stream
  jump-table charge. Combined scans of at least 1,500 literals also use a safe
  four-lane, 16-byte-striped counter mirroring `HIST_countFast_wksp()`. No
  unchecked indexing or other unsafe code was added. The histogram code is
  isolated in the logical `compressed/literals/stats.rs` submodule, leaving
  `literals.rs` at 413 lines. Focused
  `corpus_z000033` level-16 bytes remain Rust `36,845,760` versus C
  `36,864,480` for 80 runs. The paired hardware sample is Rust
  `33,032,905,127` instructions, `18,776,475,191` cycles, `5,295,708,237`
  branches, and `215,069,662` misses versus C `34,692,650,316`,
  `18,599,964,210`, `4,327,466,594`, and `175,005,010`. Rust is about 4.78%
  lower in instructions, 0.95% higher in this cycle sample, and 22.4% higher
  in hardware branches. The 511-row normal comparison at levels 1, 3, 5, 8,
  16, 19, and 22 is byte-identical before and after the histogram changes.
  Callgrind strongly confirms the per-stream-count removal but reports a small
  regression for the striped counter despite repeated hardware improvements;
  both measurements and artifacts are recorded in `AGENTS.md`. Validation
  passed focused and full tests, all-target clippy with warnings denied,
  formatting, release builds, broad byte comparison, and `git diff --check`.
- Huffman stream emission now mirrors C's table-log-sized batching instead of
  entering the general `BitWriter` state machine for every literal. A safe
  helper packs batches into a local `u64`, appends complete little-endian
  bytes, and writes the exact same end marker and padding. It is isolated in
  `huff0_encoder/stream.rs`; `BitWriter::append_aligned_with()` is the narrow
  byte-aligned output boundary. No unchecked indexing or unsafe code was
  added. A regression test compares the full output against the former
  per-symbol writer for every table log from 1 through 11.
  Focused `corpus_z000033` level-16 output remains Rust `36,845,760` bytes
  versus C `36,864,480` for 80 runs. Paired Rust counters are
  `33,050,604,598` instructions, `18,824,697,257` cycles, `5,270,036,897`
  branches, and `212,467,075` misses versus C `34,692,854,283`,
  `18,703,207,658`, `4,327,500,704`, and `175,228,327`. Compared with the
  histogram checkpoint this removes about `25.7M` hardware branches and
  `2.6M` branch misses while leaving instructions effectively flat. Direct
  branch-event profiling reduces Huffman stream attribution from about `121M`
  to `84M` branches and shows that most of the remaining Rust/C branch gap is
  entropy work rather than parser/tree work. Broad aggregate byte gaps are
  unchanged across the sampled 511 rows. Validation passed 691 library tests
  plus 7 integration tests, all-target clippy with warnings denied,
  formatting, release builds, focused and broad comparisons, and
  `git diff --check`.
- C-cost `HuffmanTableSearch::Heuristic` now selects one C-fast table-log build
  through `BlockCompressionConfig::search_smallest_huffman_table()` rather
  than entering the Rust file-type small-table search. `AllSections` retains
  the BtUltra+ optimal-depth search, and legacy/file-type modes retain their
  existing behavior. This is a semantic C-parity cleanup: focused CPU and
  bytes were neutral, while broad low-level byte totals moved only slightly
  closer to C. Artifact:
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-c-heuristic-single-table.csv`.
- Huffman base-tree construction now mirrors C's fixed-workspace cursor
  barriers without unsafe code. `base_code_lengths()` stores a real
  maximum-count leaf barrier at index zero, preallocates dummy future
  parents with the same count, and uses a direct leaf/parent count comparison
  in the hot pop operation. Real leaves occupy `1..=N` and parent slots occupy
  `N+1..=2N-1`, so cursor bounds remain represented by valid safe indexes.
  Focused `corpus_z000033` level-16 bytes remain Rust `36,845,760` versus C
  `36,864,480` for 80 runs. Three Rust 20-run samples were
  `8,326,722,936`, `8,325,783,927`, and `8,327,392,518` instructions with
  `1.3233B` to `1.3236B` branches, improving on the preceding `8.332B` and
  `1.3321B` bands. The paired 80-run sample is Rust `33,088,480,594`
  instructions, `19,457,737,459` cycles, `5,248,073,560` branches, and
  `211,979,420` misses versus C `34,694,900,337`, `18,597,536,586`,
  `4,328,000,819`, and `175,291,131`. Direct branch attribution for
  `base_code_lengths` fell from about `3.68%` of `5.262B` events to `3.51%`
  of `5.223B`. The broad 511-row normal matrix is byte-identical to the prior
  checkpoint. Artifacts:
  `benchmarks/tmp/perf-branches-z000033-l16-rust-after-fixed-huffman-workspace.data`
  and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-fixed-huffman-workspace.csv`.
  Validation passed 691 library tests plus 7 integration tests, focused
  Huffman/compressed-block tests, all-target clippy with warnings denied,
  formatting, release builds, broad comparison, and `git diff --check`.
- The Huffman tree workspace now also matches C's node layout. Rust's prior
  `HuffmanNode` used four `usize` fields and occupied 32 bytes; it now uses the
  same 8-byte shape as C's `nodeElt`: `u32` count, `u16` parent, `u8` symbol,
  and `u8` depth. Counts and indexes cross from public `usize` values through
  checked conversions, parent sums are checked, and a regression assertion
  protects the 8-byte layout. No unsafe code was added. Focused bytes remain
  Rust `36,845,760` versus C `36,864,480` for 80 runs. Three 20-run Rust
  samples improved from the preceding `8.326B` instruction band to
  `8,267,309,488`, `8,266,117,906`, and `8,264,037,566`; branch counts were
  essentially flat. The paired 80-run sample is Rust `32,910,489,407`
  instructions, `19,375,269,759` cycles, `5,262,099,782` branches, and
  `211,409,545` misses versus C `34,692,090,912`, `18,599,843,832`,
  `4,327,400,755`, and `175,038,263`. Rust is about `5.14%` lower in
  instructions while the noisy cycle sample remains `4.17%` higher. All 511
  broad size columns are byte-identical to the prior checkpoint. Artifacts:
  `benchmarks/tmp/perf-instructions-z000033-l16-rust-after-compact-huffman-nodes.data`
  and
  `benchmarks/tmp/normal-levels-1-3-5-8-16-19-22-api-after-compact-huffman-nodes.csv`.
  Validation passed 691 library tests plus 7 integration tests, focused tests,
  all-target clippy with warnings denied, formatting, release builds, broad
  comparison, and `git diff --check`.
- Rejected immediately before the compact-node checkpoint: storing the owning
  256-entry Huffman code table inline instead of in its current compact `Vec`
  preserved bytes but regressed focused 20-run instructions to
  `8.588B`-`8.596B` and branches to about `1.378B`. Moving/copying the 2 KiB
  value outweighed the saved allocation. It was reverted. Reuse external
  construction workspace at frame scope instead of embedding C's output table
  directly in the Rust owning value.
- The post-split C-cost literal estimator now applies C's fast Huffman table-log
  bound and reuses a compact Huffman node workspace inside `EstimateScratch`.
  On focused `corpus_z000033` level 16 this reduces source-aligned differing
  groups from 17 to 8 and absolute byte difference from 322 to 255. Rust emits
  `460,555` bytes per frame versus C's `460,806`. Across the 511-row levels
  1/3/5/8/16/19/22 matrix only two level-16 rows change, by `-17` and `-45`
  Rust bytes. The final paired 80-run counters are Rust `32,933,311,930`
  instructions, `18,796,301,006` cycles, and `5,273,622,165` branches versus
  C `34,696,476,982`, `18,716,889,041`, and `4,328,116,303`. Validation passed
  691 library tests, 7 integration tests, all-target clippy with warnings
  denied, formatting, broad decode comparison, and `git diff --check`.
- Rejected after the estimator checkpoint: reusing Huffman construction scratch
  for emitted blocks through either an extra frame/parser-context pointer or a
  field beside `FseTables`. Both preserved bytes but regressed the focused
  20-run instruction band from `8.276B`-`8.281B` to roughly
  `8.317B`-`8.326B`. Both shapes were reverted; keep estimator-local scratch
  only unless a different generated-code design has new profile support.
