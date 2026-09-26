# Packed authenticated index — 2026-09-27

Source base: `e3274563`, plus the source snapshots retained here. This replaces the
rejected one-record-per-leaf Patricia implementation inside the **test-only candidate**.
Normal archives and the public API do not activate this encoding. Production recovery,
whole-archive performance, allocator integration and release qualification remain open.

## What changed and why

The [previous observation](../authenticated-index-2026-09-27/README.md) found that
path-local updates alone did not make a practical index: incremental construction
wrote 698–751 MB for 100,000 small descriptors, and padded live nodes would need
24.41 GiB. The new ordered tree packs multiple descriptors or child links into each
page and constructs a new tree from sorted input in one streaming pass. It shares
unchanged subtrees during later mutations and does not add a second catalogue.

Leaves contain strictly increasing `(namespace, key)` identities and opaque canonical
descriptors. Branches contain height, total record count and ordered child links.
A link carries its child's minimum identity, record count and mirrored `RootRef`.
Reading a child verifies its stored-byte digest, expected height/count/minimum and
upper bound from the next sibling or ancestor. Point lookups, ordered traversal and
half-open range queries therefore share the same owner-authenticated membership.
The caller must obtain the root from the selected publication, not from a scan.

The codec rejects oversized nodes, keys, values, item counts and heights; unordered
or duplicate identities; overlapping sibling references; contradictory counts;
wrong archive/mode/key; and noncanonical padding/trailing bytes. Paths are bounded
by 32 levels; pages by 64 KiB; items per page by 1,024; total records by one million;
keys by 4,096 bytes; and inline descriptors by 49,152 bytes. Larger descriptors and
per-file extent indexes remain integration work. These are candidate limits, not a
statement that every public archive workload has been qualified against them.

Insertions and replacements split by encoded byte size and item count. Deletion
merges or redistributes an underfilled child with an adjacent sibling, and collapses
unary roots. Large individual records can prevent half-full pages, so hard byte
bounds take precedence over an assumed fixed record size. Promotion validates the
child before returning a new root. Exact repeats and absent removals append nothing.
Created/retired pairs are reported; permission to erase still requires synchronized
mirrored publication and durable allocator ownership.

Bulk construction accepts a strictly ordered stream, rejects duplicates or source
errors, and buffers one leaf plus incomplete branches by level and allocation
receipts. It writes every resulting page pair once, with no discarded intermediate
tree nodes. A failure can still leave an unpublished partial tail; the allocator
must account for and reclaim that entire preparation range. Building a replacement
index does not automatically retire the old index's ownership graph.

Private nodes retain archive-specific HKDF key derivation and ChaCha20-Poly1305.
The new encoding has its own key-derivation domain. The selected publication binds
the complete stored root, whose child hashes bind encrypted bytes all the way to a
descriptor. Knowing the content key does not grant owner signing authority. Default
padding remains a constant 64 KiB per node, with padding inside AEAD; unpadded mode
stores exact lengths. Serialization reserves before copying private bytes, and
private allocations are zeroized. Public locations and total archive size remain
visible; this is not a traffic-analysis defense.

The salvage traversal verifies and streams intact subtrees while reporting damaged
subtree references and their authenticated unavailable-record counts. It never
falls back to an earlier membership tree. I/O, wrong-key and visitor errors remain
fatal. Losing both copies of a shared leaf loses authority for all descriptors in
that leaf; an unrelated intact leaf remains recoverable. Copies currently append
adjacently, so physical failure-region separation still belongs to the allocator.

## Candidate wire encoding

Magic is now `RV4IDX02`; the envelope remains 44 bytes: magic (8), codec version 1
(2), established mode (2), archive ID (16), nonce (12; zero for plaintext), and
stored-body length (4). All integers are little-endian. AEAD binds the first 28
header bytes and uses the nonce; the parent digest binds every stored byte.

The body begins with height (1), total record count (8) and item count (2).
Height zero means a leaf: each record has namespace (1), key length (2), value length
(4), key and value. An empty root is an empty leaf. Positive height means a branch:
each child has namespace (1), minimum-key length (2), minimum key, `RootRef` (56)
and subtree count (8). Height must decrease exactly by one across every child edge.
Unary internal nodes are permitted; successful mutation collapses unary roots.
The common packing capacity reserves the AEAD tag even in plaintext mode, so
protection choice does not alter page grouping. Remaining padded body bytes are
zero; unpadded bodies permit no trailing bytes.

The old `RV4IDX01` test encoding is intentionally not accepted. Neither codec was
activated in production. This does not establish a released format compatibility
line or remove the need for the final normative specification and migration tests.
The independent fixture `packed_index_v1.json` covers leaves and ordered child links.

## Validation

Fourteen focused index checks pass, including all protection modes, default/absent
padding, source and storage faults, malformed nodes, maximum records, signed recovery,
read-key-holder substitution, deleted/unpublished records, independent byte vectors,
bulk/range traversal and variable-size split/merge/root shrink against an independent
map. Tests erase reported retired pairs and then verify every remaining record.
Bulk tests check zero discarded construction pages, strict ordering, bounded range
reads, and failure at every mutating operation through publication.

The final format suite passes 78 tests and the native-layout focused suite passes
14. Both default and native real-archive proof tests
pass for signed plaintext/encrypted and raw/Zstd modes. Targeted Clippy passes with
warnings denied. These are internal protocol checks: the CLI cannot create the
unactivated candidate. Public CLI, process-death across the complete new archive,
and the two original production native recovery failures remain separate gates.

## CPU, memory and space observations

The standalone Rust runner executes one fresh optimized test process per observation,
rotating four mode orders for three warmup rounds and 30 measured rounds. Every child
builds a real file with 100,000 four-byte keys and 96-byte descriptors, synchronizes
it, runs 30 warm replacements, synchronizes again, independently reopens and verifies
all records. All 132 child executions passed. CPU affinity is CPU 2; there were no
concurrent agent builds or tests. The iterator generates the same identities and
values as the earlier baseline in sorted order; **sorting arbitrary input is not
measured**. Filesystem sync variability is retained, with no outlier removal.

| Mode | Median build wall | Median build CPU | Median build peak RSS | File bytes after build | Retired construction bytes | Median warm replacement |
| --- | --- | --- | --- | --- | --- | --- |
| Plaintext, unpadded | 51.842 ms | 47.949 ms | 4,138 KiB | 21,457,822 | 0 | 0.457 ms |
| Encrypted, unpadded | 80.084 ms | 61.191 ms | 4,242 KiB | 21,463,102 | 0 | 0.600 ms |
| Plaintext, padded | 52.823 ms | 47.919 ms | 4,108 KiB | 21,643,264 | 0 | 0.655 ms |
| Encrypted, padded | 95.476 ms | 61.448 ms | 4,252 KiB | 21,643,264 | 0 | 0.903 ms |

CPU is user plus system time per process. The replacement column is the median of
30 process medians, not 900 independent observations. Build wall ranges span
51–754 ms, 78–397 ms, 52–371 ms and 82–594 ms respectively; single fsync observations
are not reliable comparative latency estimates. Largest observed process peak RSS
across build and replacement samples was 4,584 KiB. This is component working memory,
not an incremental RSS bound for an entire archive workflow.

Every construction writes 165 mirrored pages with zero node reads. Padded live
nodes occupy 21,626,880 bytes (20.625 MiB), plus the reserved 16 KiB publication
region. This directly removes the prior padded layout's 24.41 GiB live-node cost
for this descriptor set. The earlier baseline has only one construction observation
per mode and used incremental insertion; this series is **not a paired before/after
latency gate**, and it does not establish ZIP parity or A4.

Packing has an explicit update cost. The 100k-case median replacement reads two
pages: 77,131/77,163 bytes unpadded or 131,072 bytes padded. It appends two mirrored
pages: 154,262/154,326 bytes unpadded or 262,144 bytes padded, versus roughly 7.6/8.2 KB
for the rejected unpadded Patricia layout. Thus construction is far cheaper, but
single-record write amplification is larger. Batch changes should coalesce updates
to a page in one transaction, and allocator reuse/reclamation must bound aging.
Keep this trade-off visible in the full archive workload comparison.

`*.jsonl` retain every measured and warmup row with process round identifiers;
`summary.json` contains ordinary medians and extrema. `series.log.gz` retains raw
stdout/status, and `initial-probes.log` keeps the preliminary observations. Exact
pre-format source snapshots, hashes, toolchain details, validation and commands are
included. This is a candidate-component result, not a final architecture selection.

## Next integration

Persist preparation ownership and pending cleanup for complete node pairs and
partial writes. Enforce the publication durability token before reclaiming old
state, place mirrors in separate failure regions, and exercise reuse, abort,
process death and repeated aging. Integrate batched archive mutations with this
index, then independently compressed/authenticated data extents and their bounded
range descriptors. Preserve eager normal signed-open verification and independent
salvage. Complete archive comparisons, migration, bindings, normative docs and
headless/platform qualification remain required before release.
