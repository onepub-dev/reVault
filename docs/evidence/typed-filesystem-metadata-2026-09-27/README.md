# Typed filesystem metadata in the shared-control experiment

Date: 2026-09-27. Based on `70cf9636`. This connects persisted directory/symlink
metadata and real permission bits to the bounded catalogue and metadata COW
protocol. It remains test-only; the public Lockbox/CLI writer is not switched.

## Representation and semantics

The distinct private-catalogue magic `RV4DENS3` extends the existing file vector
with a node table before the physical-pack table. This is an internal catalogue
revision in an unreleased v4 experiment, not archive format 3 or a release-line
number. Existing `RV4COST1` and `RV4DENS2` layouts remain recognized.

The table has a canonical unsigned-varint count. Each node carries the existing
node-kind tag (2 symlink, 3 directory), length-prefixed canonical UTF-8 path,
little-endian 32-bit permissions, and length-prefixed target. Directories have no
target; symlinks have a nonempty canonical target. Their public length is zero.
Unknown kinds, malformed counts, duplicate names and trailing data are rejected.
All typed file/node paths use the production stored-path validator, including NFC;
parent paths must name directories. Root is implicit. Dangling canonical symlink
targets are allowed, matching the public API. Private node byte buffers are wiped.

Permission validation now uses the public `validate_permissions`: only `0000` to
`0777` are supported. The earlier prototype's acceptance of special bits through
`07777` was not equivalent to public semantics. File identities, lengths, fragment
bindings and payload allocations remain unchanged by a metadata snapshot.

An internal complete-snapshot operation accepts public `LockboxEntry` metadata
plus explicit symlink targets. It requires every existing regular file exactly
once with its correct length. Missing parents/files, path collisions, inconsistent
kinds/targets and unsupported permissions fail before a new transaction writes.
It uses the existing ownership/preparation/publication/retirement protocol; an
unchanged snapshot preserves archive bytes exactly. Return-inline compaction
preserves typed metadata and restores the original physical size in the fixtures.

## Evidence

The logical source is created through public APIs: add a file with permissions,
create an empty directory, set directory/link permissions, add a symlink, commit,
list, read file bytes and retrieve the link target. The candidate then persists
that actual public metadata rather than synthesizing `0644` for every entry.

[Focused tests](focused-tests.log) cover all 16 protection/compression/padding
modes, metadata replacement/removal, dangling targets, no-change repeats and
630 simulated power losses. The [full format suite](format-tests.log) passes
196 tests with five explicit probes ignored. It adds 32 separate complete control
bank losses, malformed node/count checks and refusal of noncanonical, duplicate,
incomplete or incorrectly sized metadata snapshots. [Strict Clippy](clippy.log)
passes. Internal storage manipulation is needed because no public CLI constructs
this experimental encoding; these are not claimed as CLI E2E tests.

## Remaining integration

This is a metadata snapshot operation, not complete public filesystem semantics:
recursive directory rename/delete, symlink-following reads/extraction, file payload
mutation, mirror ownership/adoption rules and the public adapter remain pending.
The old file-only salvage sink explicitly refuses node-bearing images before
emitting output; a node-aware recovery sink must preserve metadata and file-result
status together. Variables, forms, access mutation/overflow and the large-catalogue
hierarchy remain separate missing record classes.

The experiment admits at most 1,024 combined files/nodes and a 64 KiB decoded
catalogue fitting the existing private envelope. These are explicit experiment
limits, not reduced product capacities. The earlier read comparison used its
frozen file-only revision and is not silently relabelled as typed/public-format
performance evidence. No format is selected and no full A1/A3/A5/A6 gate passes.
