# Authenticated metadata-tail retirement experiment

Date: 2026-09-27. Based on `cafebb2b`. This is a bounded, test-only continuation
of the shared-control file image, not a selected format or public API.
The [paged coexistence experiment](../paged-catalogue-cost-2026-09-27/README.md)
failed the primary size budget at every declared page granularity.

## Protocol and scope

Keep the full 48 KiB private catalogue slots. An ordinary metadata edit first
publishes its new file metadata in two external slots using the existing tested
preparation/ownership protocol. A separate `return_inline` step then:

1. Recovers the selected state and verifies its file contents.
2. Plans placement of the same selected catalogue in the two vacant inline slots.
   Its new graph excludes the retired suffix and seals only through the last
   payload pack (or the fixed prefix for an empty archive).
3. Proves that all payload and public-key claims are unchanged, both old private
   roots are in the removed suffix, both new roots occupy the reserved inline
   slots, and the removed suffix contains only private metadata or vacant space.
4. Encodes/signs the shorter anchor before any transaction write, durably prepares
   the inline writes, then writes and checks both replacement private copies.
5. Publishes and syncs both shorter anchors before erasing any retired tail byte.
6. Wipes the entire now-unsealed suffix, syncs, truncates, syncs again, verifies
   reclaimed space and finishes the preparation record against the selected commit.

Generic ownership transitions still refuse shrinking. A distinct narrow graph
proof and publication entry point admit this metadata-only case. Recovery selects
and mirrors authenticated publications before erasure; it needs no erased old
catalogue to finish a committed truncation. Before the shorter publication wins,
recovery preserves the external metadata and aborts only reserved inline writes.

The two publications expose the same logical file state. Interruption can leave
the complete external placement or the complete inline placement. Returning to
inline after an aborted compaction needs owner signing authority in signed mode;
finishing cleanup after an already committed shorter anchor does not. This is not
a claim that a failed operation always leaves the compact size.

## Validation

The format suite passes 192 tests, with five explicit probes ignored. The tail
checks cover all 16 protection/compression/padding combinations, repeated return
to original size, and independent reopened file bytes/metadata. They include:

- 117 returned storage failures and 702 torn-write/volatile-state/failed-sync cases.
- 192 interruptions of recovery itself, at abort, committed-tail and already
  truncated checkpoints; six separate losses of an entire control bank.
- Refusal to substitute or drop payload claims, shrink through the generic path,
  or use a stale predecessor commitment.

These are internal storage fixtures because no public CLI writes this experimental
profile. They do not establish real filesystem power-loss or platform guarantees.
Password bootstrap also succeeds after shrinking; missing/wrong owner signers
are refused without changing archive bytes. [All 20 dense tests](focused-tests.log)
and [strict Clippy](clippy.log) pass after the final guard changes; the preceding
[format-suite log](format-tests.log) retains the complete 192-test run.
Retained-corpus sizes are measured only after freezing.

## Limits and next measurements

`return_inline` is connected only to external private roots and a bounded file
catalogue. It is not a general compactor for arbitrary inline/free-tail states.
It does not relocate payloads, mutate key directories, support catalogue overflow,
rotate overflow journals or implement the full public record model. The existing
1,024-file / 4,096-fragment / 64 KiB decoded catalogue limits remain experiment
limits, not accepted product limits.

A successful edit plus return-inline requires two generations, extra full
catalogue writes, erasure and durability barriers. Temporary external allocations
remain necessary. The executor still verifies unrelated payloads; no fast-update
claim follows. A `dense-tail-lifecycle` probe will compare completed size against
its initial size, report peak temporary allocation, close/reopen and compare every
source byte after every edit, and hash every unchanged repeat. It is a correctness
and size probe, not an A3/A4 timing or full A5 qualification.

## Retained persisted lifecycle results

Frozen commit `4b04e382`, executable SHA-256
`1cbac289ec35db484e84a1b3486ab55f2c26a7632ff9e1f03acda62bf711bb95`.
[Batch and corpus paths](sizes/batch.json) and individual JSON/log files retain
100 permission edits plus 100 byte-identical unchanged repeats per corpus. Each
changed edit includes an explicit return-inline step; each is closed and reopened
through a separate reader, with every source file compared byte-for-byte. Persisted
permission bits and both generations per cycle are checked. Source corpora and
older archive controls remain unchanged.

| Corpus | Initial bytes | Temporary bytes after edit | Completed bytes after each return-inline |
| --- | ---: | ---: | ---: |
| 512 × 4 KiB plain | 327,680 | 458,752 | 327,680 |
| 512 × 4 KiB encrypted/signed | 327,680 | 458,752 | 327,680 |
| 8 MiB raw plain | 8,519,680 | 8,650,752 | 8,519,680 |
| 8 MiB compressed plain | 196,608 | 327,680 | 196,608 |
| 8 MiB compressed encrypted/signed | 196,608 | 327,680 | 196,608 |

Both small completed archives remain 1.382× the retained 237,078-byte ZIP, below
the proposed 1.50× completed-size bound. Temporary space is 128 KiB above the
initial archive for these aligned cases. This is a passing bounded **metadata-only
size subcase**, not mixed payload aging, a public mirror lifecycle, general
compaction, CPU/RSS qualification or a full A5 pass. It does not overturn the
original C growth failure at mixed-operation cycle 421.

Post-hook [dense tests](postformat-tests.log), [tail graph proof](postformat-graph.log)
and [strict Clippy](postformat-clippy.log) all pass before the executable was frozen.
