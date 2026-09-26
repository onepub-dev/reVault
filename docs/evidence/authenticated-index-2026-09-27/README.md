# Authenticated persistent index — 2026-09-27

Source base: `cbaf4c43`, with the implementation committed alongside this evidence.
This is the next candidate-format layer, compiled only for tests. It connects the
mirrored publication root to independently verifiable object descriptors. It does
not change normal archive output or resolve the two production native recovery
failures. Decision 001 and the complete archive acceptance gates remain open.

## Implementation and trust

`authenticated_index.rs` implements a mirrored copy-on-write Patricia index over
SHA-256 hashes of `(domain, archive ID, namespace, key length, key bytes)`. A leaf
holds the original namespace/key and a canonical descriptor supplied by its caller.
A branch holds a canonical routing prefix, its discriminating bit, private subtree
counts and two child references. Each reference binds both physical copies, length
and the digest of complete stored bytes. The publication signature/MAC authenticates
the root reference; a content-key holder cannot replace a signed leaf by merely
producing fresh valid AEAD. All callers must obtain the root from the selected
publication, never from a scanned prepared node.

Insertion, replacement and deletion rewrite only their search path. An exact
repeat or missing deletion writes nothing. Removing the last entry writes a bounded
empty root. Namespace separation also permits non-file descriptor families, but
that does not implement those families' real canonical codecs or archive APIs.
Traversal streams records in hashed identity order, not pathname order. Ordered
listing/prefix queries need an explicit design before replacing the existing TOC.
The full index is not cloned or materialized during lookup, mutation or traversal.
Copies currently append adjacently; that tests independent copy damage, not survival
of an arbitrary aligned device failure region. Mirror placement needs explicit
failure-domain separation in the allocator and corresponding damage tests.

Node references are bounded to 64 KiB and to the selected sealed extent. Child
counts must match the accessed node, child routes must match their parent, and
branch bit positions strictly increase. A path therefore has at most 256 branch
nodes. Counts are bounded to one million, keys to 4,096 bytes and inline descriptors
to 49,152 bytes. Larger descriptors need separately authenticated pages in the
complete layout. Invalid lengths, overlap, contradictory counts, route substitution,
wrong archive/mode/key and noncanonical trailing bytes fail before unbounded reads
or allocations. Storage I/O errors propagate; checksum damage can use the mirror.
Losing both selected index copies does not roll publication back to old membership.

Encryption derives a separate index key using HKDF-SHA-256 with archive-ID salt and
an index-specific domain. ChaCha20-Poly1305 binds the node's magic/version/mode/archive
header; the authenticated parent additionally binds every stored byte. Generation
is bound by publication, allowing unchanged subtree reuse across generations.
Private keys, values and serialization buffers use zeroizing allocations; encoding
reserves capacity before copying secrets to avoid unwiped reallocation remnants.
No public plaintext digest of an encrypted descriptor is stored.

Default padding stores every node in a fixed 64 KiB extent, including its header
and AEAD tag. Padding is inside encryption and is checked after decoding. Unpadded
mode stores the exact encoded length. Headers, physical locations and total archive
size remain visible. These are candidate layout choices, not compatible changes to
format 3, and the padded form's space cost must be evaluated before selection.

## Candidate node encoding

All integers are little-endian. The 44-byte envelope is:

| Offset | Bytes | Field |
| --- | --- | --- |
| 0 | 8 | `RV4IDX01` |
| 8 | 2 | Codec version 1 |
| 10 | 2 | Established archive format mode |
| 12 | 16 | Archive ID |
| 28 | 12 | AEAD nonce, or zero for plaintext |
| 40 | 4 | Stored body length |
| 44 | variable | Body, or encrypted padded body and 16-byte tag |

The body begins with kind (one byte) and subtree count (eight bytes). Kind 0 is
empty and has count zero. Kind 1 has count one, namespace (one byte), key length
(two bytes), value length (four bytes), key and value. Kind 2 has branching bit
(two bytes), routing prefix (32 bytes), then two child links of 64 bytes each.
Each child link is the publication `RootRef` (56 bytes) and private count (8 bytes).
Default padding fills unused body bytes with zeros before encryption. Unpadded
nodes must have no trailing bytes. The independent JSON fixture uses synthetic
plaintext data and unpadded mode and checks both leaf and branch encodings.

## Executed checks

Nine focused tests cover file-backed publication/reopen in all four protection
modes; additions, replacement, rename, deletion and no-change repeats; deterministic
mixed updates checked against an independent map; maximum-sized records; independent
byte vectors; padding; malformed records; content-key-holder substitution; and every
mutating operation failure during insertion plus publication. Intact stored content
is recovered after destroying both neighbouring leaf copies and one root copy.
Deleted and unpublished records remain absent from the selected membership.

The existing real-archive proof test now persists canonical TOC descriptors and
stored-content commitments through this index, selected by the owner's publication.
It passes with default and native archive layouts, raw/Zstd codecs, and signed
plaintext/encrypted modes. Its candidate index/publication store is still separate
from the current production archive store. No CLI can create this encoding yet;
these storage-protocol tests are not substitutes for the required public CLI E2E
qualification. Targeted Clippy passes with warnings denied. The combined format
module suite passes 73 tests, including publication fault/process-death checks;
the native index suite passes nine. The resource observation is explicitly run
in both modes despite being excluded from normal correctness-test invocations.

The 2,048-entry unpadded deterministic replacement creates 11 node pairs, reads
2,224 bytes in 11 reads, and appends 4,450 bytes. Erasing the reported retired pairs
leaves the new tree fully readable; the test also verifies the previous snapshot
before erasure. This establishes local sharing/accounting behaviour, not permission
to erase those pairs before durable mirrored publication in a real archive.

## Resource observation

The explicitly ignored resource test is a manually run observation, not an excluded
correctness failure. It creates 100,000 entries in a real file, runs 30 replacements,
then independently reopens and verifies all records. The payload is a 4-byte key and
96-byte descriptor per entry. Each protection mode runs in a separate optimized test
process pinned to CPU 2, without concurrent agent builds/tests. It records wall,
user/system CPU, process peak RSS, read I/O, created nodes and appended bytes.

These are single-build, warm replacement observations without ZIP or candidate-A
controls, publication/signatures, content codecs, allocator reuse or reclamation.
They cannot establish A3/A4 parity or a whole-archive memory bound. Exact source,
binary hash, raw observations and commands accompany this evidence.

Observed results (one build per mode; 30 warm replacements within each process):

| Unpadded mode | Build wall | User CPU | System CPU | Peak RSS at build end | File bytes | Live node bytes | Retired node bytes | Median replacement wall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plaintext | 38.094 s | 3.848 s | 33.294 s | 3,628 KiB | 698,411,140 | 74,999,570 | 623,395,186 | 0.445 ms |
| Encrypted | 42.565 s | 9.983 s | 32.558 s | 3,812 KiB | 751,203,332 | 81,399,538 | 669,787,410 | 0.499 ms |

Each build made 1,549,756 node reads. The median replacement wrote 18 node pairs:
7,630 bytes plaintext or 8,206 encrypted. Peak RSS is the process high-water mark
observed after build, not a claimed incremental memory bound for a complete archive.
The raw per-update CPU values have the platform's accounting granularity; these
sub-millisecond samples should not be interpreted as precise CPU comparisons.
No confidence interval or relative-performance gate is established by these runs.

**Do not select this one-record-per-leaf layout as the final index.** The current
append-only build retains more than eight times the live-node bytes as superseded
preparation nodes. System CPU dominates despite small resident memory. Further,
100,000 entries imply 199,999 live nodes; two 64 KiB padded copies per node would
occupy 26,214,268,928 bytes (24.41 GiB), excluding all payloads. That is calculated
from the verified node layout, not a measured padded large-case result. It is
unacceptable for a roughly 10 MB descriptor set. Avoiding default padding merely
to obtain favourable results would weaken the evaluation contract.

The next architecture comparison must use packed index pages and bulk construction,
preferably a key-ordered tree that also supplies directory/prefix listing, with the
same authenticated child links and mirrored publication boundary. Compare its
update amplification against this baseline. Add durable preparation/reclamation
accounting before any production activation. Retain this implementation as a
bounded authority/protocol experiment until it is replaced or selected evidence
justifies reuse of particular pieces; do not layer a duplicate production index
alongside the current TOC.

## Remaining integration

This layer reports created/retired node pairs, but currently appends every new pair.
A failed append can leave a partial preparation pair. The allocator must durably
track the entire preparation range, including failed writes and superseded nodes,
and reclaim it only with the publication protocol's synchronized completion token.
Do not activate this implementation without that ownership/cleanup integration.
Bulk construction or packed index pages must address intermediate-node write growth
and default-padding overhead; path-local updates alone do not qualify the design.

Root publication currently validates its direct dependencies, not every descendant.
The complete writer must validate newly prepared paths/dependencies and retain the
selected generation's full ownership graph. Key rotation, ordered directory listing,
all record-family codecs, external descriptor pages, eager signed-open verification,
allocation reuse, 100/1,000-cycle aging, full process-death coverage and CLI/binding
migration remain gates. The fixed-ordinal proof experiment remains comparative
evidence, not a second production catalogue.
