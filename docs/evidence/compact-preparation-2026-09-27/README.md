# Compact preparation representation

Date: 2026-09-27. Based on `dd606b0a`. This test-only codec/selector is part of
the proposed shared-control experiment. It does not activate a public writer or
grant allocation/cleanup authority.

## Representation

A `RV4PST01` preparation stub occupies 4,096 bytes at offset 8,192 in each of the
two 64 KiB control regions. It binds archive identity and mode; encrypted archives
use the existing journal-derived key with a distinct authenticated magic/version
prefix. Private padding remains inside AEAD. Plaintext journal checksums do not
grant owner authority; selected-publication ownership remains mandatory before
any destructive action.

Up to 155 reservations fit inline. Larger records retain the existing maximum of
2,048 reservations using the full existing 65,536-byte journal encoding in two
aligned, separate external failure regions. The stub commits their locations,
length and stored-byte digest. Sequence, previous-stub commitment and publication
base are checked again after reading overflow. No reservation limit was reduced.

Selection checks both bounded stubs before fetching only the newest selected
record. Missing or damaged current overflow does not permit selecting an older
reservation set. Ordinary I/O errors propagate. A verified surviving overflow copy
can replace a damaged peer for reading. Malformed counts are rejected against the
remaining body length before allocating the reservation vector.

The existing journal's body serializer/parser is shared with this representation;
its original wire encoding and independent fixture remain unchanged. Its tested
write-ahead, rollback and cleanup paths still use their original placement.

## Validation

From `rust/`, before formatting:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib file_format
cargo clippy -p revault_lockbox_api --all-targets --features external-source,native-block-layout -- -D warnings
```

[164 format tests pass](format-tests.log), five explicit probes remain ignored;
[strict Clippy passes](clippy.log). Six new tests cover:

- Inline/overflow boundaries and the full reservation limit in four protection
  modes, with separate loss of either control or overflow region.
- Every byte prefix of a torn stub replacement in unsigned encrypted and signed
  plaintext modes: 8,194 prefix cases retain the old or new complete record.
- Both current overflow copies missing while an older stub survives.
- Wrong keys/context, cross-encoding rejection, invalid external placement,
  substituted overflow publication base, malformed counts and nonzero padding.
- Read-only selection and propagation of injected I/O errors.

Tests use internal fixtures because no public CLI writes this experimental layout.
The torn-prefix cases are byte-level representation tests, not process-death or
power-loss qualification of a new writer. Existing journal fault, process-death
and independent-vector tests also pass after extracting shared serialization.

After the hook, [journal tests](postformat-tests.log) pass (18 passed, one ignored)
and [strict Clippy](postformat-clippy.log) passes.

## Required integration

Before writing overflow, establish durable append ownership. Synchronize both
overflow copies before exposing the new stub; synchronize and repair old stub
copies before replacing one. Reservation writes may begin only after the new
journal state is durable as required by the existing protocol. Retire overflow
only when no selectable journal/publication state still depends on it. Recover
interrupted transitions without dropping reservation or cleanup progress.

Connect these requirements to the shared-control allocator and full reachable
metadata graph, including role-bound inline ranges. Preserve the rule that only
selected authenticated ownership authorizes erasure. New creation/mutation,
overflow rotation and cleanup need real byte-level fault, process-death and
region-damage tests. This representation by itself closes no archive acceptance
gate and supplies no throughput or peak-space result.
