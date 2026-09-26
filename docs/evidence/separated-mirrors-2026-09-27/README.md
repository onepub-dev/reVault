# Separated metadata mirrors and repair

Candidate C only, based on `e9695331`. No production archive reader or writer
activates these encodings. This is evidence for A1/A2/A5, not layout selection or
completion of the public API, performance or release gates.

## Damage model and implementation

The protected failure unit is one aligned 64 KiB region. Each metadata node fits
inside one region and its identical authenticated mirror occupies another region.
Publication slots occupy regions 0 and 1; the preparation journal occupies regions
2 and 3. Reuse-aware allocation enforces the same rule for index and key nodes.
Allocation maps pack nodes into two separate region-aligned banks, whose unused
capacity is explicitly owned and zero. A pair's reused ranges share one durable
reservation update; adjoining leases with the same original allocation coalesce.
Disjoint journal reservations still have a fixed 2,048-entry limit.

This does not promise survival of two failed regions, an arbitrary unaligned
64 KiB span, device loss or truncation. Payload has no additional replica here;
a region containing payload can lose that content while its metadata survives.
Unsigned plaintext supplies corruption detection, not owner authentication against
an attacker who can rewrite the archive.

Explicit metadata repair first authenticates the selected publication and audits
the complete reachable ownership graph. It reconstructs a failed metadata copy
only from a peer matching its authenticated digest. After synchronizing repaired
metadata and both publication copies, it restores declared unused space to zero
and resumes journal recovery. It never rewrites payload, selects older membership,
or treats metadata repair as proof that payload is healthy. Ordinary transaction
recovery continues to reject unexpected nonzero allocation-arena slack.

Experimental publication and journal encodings advance to version 2, with new
magic and authentication/key-derivation domains. Silently accepting version 1
would risk overlooking its adjacent second publication slot. Independent version-2
publication and relocated index vectors accompany the tests; version-1 fixtures
remain historical evidence. These experimental versions are not release-line or
production archive-format numbers.

## Correctness evidence

Both default and native-layout format suites pass 104 tests, with three manual
resource probes ignored. All-targets native/external-source Clippy passes.

- 148 whole-region damage cases cover all encryption/signing and index-padding
  combinations, a multi-level index, key records, publication, journal, allocation
  maps, unused space and payload. Every case recovers all 1,500 metadata records;
  unaffected payload is decoded and compared. Repair preserves damaged payload
  bytes and a second repair is byte-identical.
- Losing both publication copies or both root copies fails closed without writes.
  Losing both copies of a descendant refuses full repair; non-mutating salvage
  still reports unrelated records and an authenticated unavailable count.
- Repair survives 223 returned mutation failures and 1,784 power-loss model cases,
  including torn writes and uncertain sync completion. Each durable state is
  independently reopened, repaired, fully accounted and checked against all
  record metadata and the original damaged payload bytes.
- Existing allocator matrices pass 265 returned failures and 1,584 power-loss
  cases. Existing journal/publication suites retain real process-death coverage;
  the new repair-specific matrix models power loss, not actual process death.
- A false pending range over live descendant metadata is rejected before either
  ordinary recovery or explicit repair writes anything. Old publication geometry,
  crossing nodes and same-region pairs are rejected. Reservation tests cover
  4,096 adjoining requests and refuse overlapping extensions.

The candidate is internal and test-only, so the public CLI cannot create these
archives or inject the physical faults. Public CLI integration remains required.

## Resource protocol

`run_series.rs` starts fresh processes for the same allocator aging probe used in
[the previous ownership experiment](../allocation-accounting-2026-09-27/README.md).
It alternates adjacent/separated order across ten cases: 100 operations for each
of eight encryption/signing/padding combinations, plus 1,000 operations for padded
plaintext and padded encrypted-signed archives. Both versions run all cases:
5,600 independently reopened and verified operations in total.

Each eight-operation sequence adds 4,096 bytes, repeats unchanged, replaces with
12,289 then 31 bytes, creates a shared alias and key record, removes both names,
and repeats an absent removal. Mutation CPU and elapsed time include transaction
begin, publication and cleanup; independent reopen, content verification and full
physical/zero accounting are outside those timers. Peak RSS covers the whole
process, including verification. Payload uses the existing synthetic encoded-byte
helper, not a complete candidate file codec. Index padding is the varied setting.

Measurements run pinned to CPU 2, without owned builds/tests concurrently. These
are single aging runs per case, not 30 paired observations or an A4 confidence
interval. File sync timings are particularly variable. Retain the complete raw
series and avoid interpreting individual elapsed ratios as established speedups.
The old binary and its source are retained in the previous evidence; current
implementation snapshots and binary hashes are retained here. The measured probe
is unchanged; a later audit assertion outside that probe was added before native
validation. Final hook formatting also follows measurement.

Run from `rust/`:

```sh
rustc --edition=2021 /tmp/revault-region-series.rs -o /tmp/revault-region-series
taskset -c 2 /tmp/revault-region-series /tmp/revault-allocation-arena-resource /tmp/revault-regions-resource
```

The two production native recovery failures remain open. Candidate payload codecs,
small-file packing, public access/record semantics, batched transactions, compaction,
full A/B/C performance, migration and platform qualification remain outstanding.

## Observed results

All 5,600 operations passed independent reopen/content verification and exact
physical accounting. Each 1,000-operation run includes 250 no-change operations,
which neither advance the generation nor grow the archive. Pending ranges in the
final map are already zero after journal cleanup; their persisted classification
is changed by a later map publication.

| Padded 1,000-operation case | Adjacent copies | Separated copies |
| --- | ---: | ---: |
| Plaintext mutation CPU | 10.264 s | 8.933 s |
| Encrypted-signed mutation CPU | 22.550 s | 20.924 s |
| Plaintext elapsed | 59.144 s | 45.599 s |
| Encrypted-signed elapsed | 97.214 s | 72.758 s |
| Plaintext final size | 1,134,592 bytes | 1,114,112 bytes |
| Encrypted-signed final size | 1,134,620 bytes | 1,114,112 bytes |
| Last growth, operation number | 29 | 5 |
| Plaintext process peak RSS | 5,612 KiB | 5,708 KiB |
| Encrypted-signed process peak RSS | 5,660 KiB | 5,840 KiB |

CPU decreased approximately 13.0% and 7.2% in these two observations. This combined
change includes pair reservation batching, placement and map-bank geometry; the
measurement does not isolate their individual effects. Maximum candidate RSS over
all ten processes was 5,984 KiB. No full-archive or GB-streaming memory claim follows.

Unpadded tiny archives have a substantial fixed-space penalty: 655,360 bytes in
all four candidate runs, versus 177,383/177,643 bytes before separation. Candidate
unpadded runs stopped growing after the first operation; padded runs after the
fifth. The fixed front region alone rises from 147,456 to 262,144 bytes. This
trade-off must remain visible in the full small-file/compaction size comparison.
