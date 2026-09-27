# Shared-control publication ordering model

Date: 2026-09-27. Based on `c40984e3`. This is an executable abstract ordering
model, not a persisted archive, allocator, journal implementation or power-loss
harness. It tests the next layout hypothesis before committing to its wire format.

## Finding and proposed protocol

Two 64 KiB control regions can hold publication, preparation and public bootstrap
alongside mirrored private metadata, but the private portions cannot simply be
overwritten in place. Current roots still reference them. The proposed path for a
catalogue that fits completely in the inline private slots is:

| Step | Required durable state before proceeding |
| --- | --- |
| Own the append tail | Existing preparation protocol grants ownership before any append |
| Stage new external metadata copies | Both copies complete, authenticated and in separate failure regions |
| Publish the external roots | Both publication copies durable; an intermediate newer selected copy alone does not authorize retirement |
| Retire old inline metadata | Erase only the designated private subranges; preserve anchors, preparation and public bootstrap |
| Stage replacement inline metadata | Both copies complete; selected roots still refer to the external copies |
| Publish inline roots | A second publication generation, durable in both copies |
| Retire external metadata | Erase unreferenced copies, then truncate only an owned, otherwise unused tail |

The second publication is relocation after the logical update. A writer could
leave metadata external until compaction instead; it must account for that space.
The model does not select either policy. Normal update and compaction benchmarks
must measure the chosen policy, including signatures, syncs and cleanup.

This exposes a limit of the earlier size projection: 327,680 bytes is a *final
compacted image*. Two temporary external metadata regions could add 131,072 bytes
for this inline-sized case, before payload replacements, key-directory changes or
journal overflow. It is not a complete peak-space bound. An inline empty/create
path can write both new metadata copies before its first publication because no
older live catalogue exists.

## Checks and limits

The [Rust model](../../../rust/revault_lockbox_api/src/file_format/shared_control_model.rs)
checks a 14-action schedule. At each action it tests no persisted change, a torn
record and a complete record: 42 interruption states. After each successful
barrier it separately removes each of the four failure regions: 56 damage states.
All retain an old or new readable catalogue. Negative checks reject overwriting
live inline metadata, appending without ownership, retiring after only one new
publication and truncating live external metadata.

[Two model tests](tests.log) and [strict Clippy](clippy.log) pass. Run from `rust/`:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib shared_control_model
cargo clippy -p revault_lockbox_api --all-targets --features external-source,native-block-layout -- -D warnings
```

Authentication, correct placement and successful durability barriers are model
assumptions. Torn records are represented as invalid; actual byte encodings,
partial-sector writes, unsynced reordering and recovery execution are not tested.
Interruption and region loss are separate experiments, not a claim to tolerate two
simultaneous independent faults. Erasure models unreadability, not zeroed bytes.
The ownership flag assumes the existing preparation guarantee; it does not prove a
4 KiB preparation stub or overflow protocol.

The inline catalogue is assumed to contain all its metadata, with no descendants
left in a retired slot. A real allocator must audit the entire reachable graph,
including overflow, key records and shared descendants, before erasure. Physical
ownership changes alter committed metadata even when logical contents are the
same. The model's content token intentionally abstracts those encoding details.

Post-format [model tests](postformat-tests.log) and [Clippy](postformat-clippy.log)
also pass. The [image-component follow-up](../shared-control-image-2026-09-27/README.md)
adds actual placement/authority/envelope bytes and identifies the sealed-length
and allocation-accounting work omitted by this model’s final truncation step.

Implement the shared-control profile, bounded catalogue encoding, overflow journal
and graph-derived retirement together, then repeat actual byte-level fault,
process-death, region-damage and lifecycle tests. No A1–A5 gate is passed here.
