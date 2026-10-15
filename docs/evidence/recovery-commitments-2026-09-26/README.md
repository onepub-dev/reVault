# Independent recovery commitments: bounded experiment

This is evidence for [decision 001](../../decisions/001-v4-signing-and-recovery.md).
It evaluates a commitment primitive, not a complete archive or an activated wire
format. The normal recovery path and its two known native failures are unchanged.

## What was implemented and tested

A canonical object commits to namespace, identity, descriptor bytes, logical length
and content commitment. A root binds archive identity, generation, format, object
count and the commitment tree. Leaf/node/empty/flat/root/owner-message domains are
separate. Table construction rejects unordered or duplicate identities. The binary
tree pads to a power of two with a distinguished empty hash and binds the real count.

The proof decoder checks magic, reserved bytes, count, index, exact depth and exact
length before allocating siblings. Limits: 1,000,000 objects; 4,096 identity bytes;
65,536 descriptor bytes per object; 256 MiB total identity/descriptor input. Same-key
replacement preserves the aggregate bound. These validate supplied objects; a
future persisted object decoder must enforce limits before allocating their fields.

Seven primitive tests cover every index in tree sizes 1 through 129, empty trees,
independent recovery after unrelated object loss, descriptor/context/position
substitution, deleted/unpublished membership, malformed proofs, canonical ordering,
replacement versus full rebuild, and an independent little-endian byte vector.
The vector was generated with Python hashlib/struct, independently of Rust's
encoder: [fixture](../../../rust/revault_lockbox_api/tests/fixtures/recovery_commitment_v1.json).
A deterministic bootstrap test also passes.

A separate test uses real archive pages, canonical TOC descriptors and the existing
Ed25519 plus ML-DSA-65 signatures. It hashes stored extents, including ciphertext
for encrypted mode. It corrupts a neighbour's physical allocation, recovers the
survivor through the page reader, compares exact bytes and verifies its proof.
The files occupy separate allocations: shared compression/authentication-unit
damage still has unit-level recovery limits. This passes for raw/Zstd and
plaintext/encrypted signed archives, in both default and native layouts. It rejects a valid signature from another owner, altered data,
and even a valid owner signature on a different unpublished root.

That test supplies a trusted selected-root digest independently. It does **not**
implement the on-disk mechanism that establishes publication or the owner's trusted
identity. The test cannot establish crash safety, tail-proof redundancy or complete
normal-open behaviour. The prototype is compiled only for tests and this benchmark.
Targeted Clippy passes with warnings denied. No production format selector changed.

## Measurement method

The [runner](measured-runner.rs.txt) invokes a fresh child process per sample, pins
CPU 2, alternates flat/tree order and takes 30 pairs after three warm-up pairs.
Each input object has a 128-byte synthetic descriptor, a generated identity and a
precomputed synthetic digest. Inputs are resident; no payload or metadata I/O,
owner signatures, decompression, encryption or publication is timed.

Build hashes the whole object table for both candidates. Flat replacement recomputes
the whole table; tree replacement updates one existing identity and its ancestors.
Flat verification hashes the full table; tree verification hashes one descriptor
and its sibling path against an already trusted root. Tree replace/verify uses
1,000 repetitions per child to exceed timer resolution; report time/CPU per
operation. Tree setup, proof lookup/decoding and post-sample checks are untimed.
The repetitions reuse the same middle object and resident tree/proof, so these
are warm in-memory primitive costs, not random-access archive latency.

Peak RSS includes input and setup allocations before post-sample validation. It
is process lifetime RSS, not incremental heap or a standalone proof verifier's
minimum memory. The tree remains resident during measurement. Explicit retained
hash storage and proof lengths are also recorded. Summaries use 10,000 fixed-seed
paired-log bootstrap resamples and retain all samples without filtering.

The executable was built from `f5ee0307` plus the benchmark manifest declaration
and the two retained measured sources.
The independent vector test was added after measurements; it changes no measured
algorithm. Recorded source/executable hashes and deterministic summaries were
verified. [Toolchain](toolchain.txt): Rust 1.88.0, x86_64 Linux. No owned builds or
tests overlapped timing; unrelated host load is uncontrolled and recorded.

```sh
cargo test -p revault_lockbox_api --test recovery_commitment_checks
cargo test -p revault_lockbox_api --lib independent_owner_proof
cargo test -p revault_lockbox_api --features native-block-layout --lib independent_owner_proof
cargo bench -p revault_lockbox_api --bench recovery_commitment --no-run
# Invoke the built executable after all builds/tests finish.
taskset -c 2 /path/to/recovery_commitment 512 30 > 512-objects.jsonl
taskset -c 2 /path/to/recovery_commitment 100000 30 > 100000-objects.jsonl
/path/to/recovery_commitment summarize 100000-objects.jsonl
```

## Results and implications

[512-object samples](512-objects.jsonl) / [summary](512-objects-summary.jsonl);
[100,000-object samples](100000-objects.jsonl) / [summary](100000-objects-summary.jsonl).

| 100,000-object operation | Flat elapsed median | Tree elapsed median | Tree/flat ratio, 95% interval |
| --- | --- | --- | --- |
| Initial build | 20.992 ms | 37.488 ms | 1.778 (1.757–1.794) |
| Same-identity replacement | 21.174 ms | 2.006 µs | 0.0000936 (0.0000917–0.0000951) |
| Membership verification | 20.920 ms | 1.859 µs | 0.0000888 (0.0000875–0.0000899) |

CPU closely follows elapsed time (100k build medians: flat 21.008 ms, tree
37.502 ms). Tree build median RSS is 36.51 MiB versus 28.53 MiB flat; tree hashes
retain 8,388,576 bytes. A 100k proof is 568 bytes; a 512-object proof is 312 bytes.
At 512 objects, initial tree construction costs 1.576× flat (1.552–1.602).

The component trade-off favours a tree for independent membership verification
and same-identity changes while charging extra initial work and hash storage.
This fixed-position tree is a model, not the proposed persistent index: insertion,
deletion and renaming can shift ordinals and require rebuilding. A keyed index
with authenticated child links still needs implementation and mutation/aging tests.
Whole signed-open verification remains eager; independent proofs do not authorize
skipping payload checks. End-to-end recovery/performance gates remain open.
