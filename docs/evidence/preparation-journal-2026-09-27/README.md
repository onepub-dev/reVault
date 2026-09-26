# Preparation journal: durability and bounded rollback cost

Experimental candidate-C component, 2026-09-27 local date. Source base
`ebecc2c7`, with the journal addition; no production archive encoding changes.
Supports decision 003 experiments 1 and 3, and journal-only aging. It does not
complete A1, A4, A5 or archive lifecycle qualification.

## Protocol implemented

Two fixed 64 KiB journal slots follow the 16 KiB publication region. The data
region starts at byte 147456. An active marker binds the preparation to the
selected publication commitment and owns the entire unpublished append tail.
Before modifying reused storage, exact reservations must be validated against
FREE or PENDING entries in that publication's authenticated allocation index,
checked for overlap and zero contents, then mirrored and synchronized. The
journal is not authority to erase arbitrary data.

Rollback validates those reservations again, wipes them, truncates the append
tail and durably clears the journal. If the next publication won, cleanup uses
only its authenticated PENDING map; old reservations may now hold live data.
Both publication copies must be synchronized before reclaiming old state. An
error after publication starts has an uncertain commit outcome: reopen and
select the durable generation. Do not assume rollback.

Cleanup records progress after each 8 MiB of zero writes, synchronizing the
zeros before publishing the checkpoint. Progress is bound to the publication
commitment. An interruption can repeat work since the previous checkpoint; with
non-aligned ranges this interval can exceed 8 MiB by less than one 64 KiB chunk.
An empty completed journal requires no scan of the old pending map. The caller must hold the exclusive archive writer lock across recovery and the
transaction; independent begin calls are not serialized by this component. All clones
of the prepared writer become unusable after abort, commit or a mutation failure.

The `RV4PRE01` encoding has a 48-byte header, fixed-size body and 32-byte checksum.
The body records sequence, previous journal hash, base publication hash, active
flag, cleanup publication hash, cleaned byte count and up to 2,048 reservations.
Encrypted modes encrypt the body with an archive/domain-derived key. Slot
selection rejects conflicting equal sequences and non-adjacent valid records.
This candidate encoding is test-only and is not the released format-4 wire spec.

## Measured workload

Each fresh process creates a sparse synthetic inventory with 1, 1,000 or 100,000
free 4 KiB ranges separated by 4 KiB protected gaps. A real file stores its packed
allocation index, logical descriptor and publication. Setup is excluded from the
operation timer. The measured operation begins preparation, reserves one middle
range, writes 4 KiB and aborts. A separately opened file then verifies the selected
generation, original content bytes, the wiped range, adjacent gap and exact length.

There are 3 warmup and 30 measured processes for each of 12 cases, 396 successful
processes total. The runner rotates case order per round. CPU affinity is CPU 2;
no owned builds or tests ran concurrently. The host's existing cache and filesystem
were used, without cold-cache manipulation. CPU is process user plus system time;
RSS is the process lifetime high-water mark, including setup, rather than an
incremental allocation estimate. All I/O counts were identical within each case.

| Protection | Free ranges | Median CPU, ms | Median elapsed, ms | Maximum RSS, KiB |
|---|---:|---:|---:|---:|
| Plain | 1 | 2.041 | 20.549 | 5408 |
| Plain | 1,000 | 2.484 | 21.090 | 5620 |
| Plain | 100,000 | 2.570 | 21.087 | 5484 |
| Encrypted | 1 | 2.767 | 23.580 | 5516 |
| Encrypted | 1,000 | 3.296 | 23.670 | 5640 |
| Encrypted | 100,000 | 3.475 | 24.440 | 5520 |
| Signed | 1 | 6.183 | 25.880 | 5476 |
| Signed | 1,000 | 6.758 | 25.982 | 5576 |
| Signed | 100,000 | 6.605 | 25.892 | 5516 |
| Encrypted + signed | 1 | 6.756 | 27.276 | 5532 |
| Encrypted + signed | 1,000 | 7.257 | 27.908 | 5636 |
| Encrypted + signed | 100,000 | 7.198 | 27.992 | 5508 |

The reservation reads one index page for 1 and 1,000 ranges, two for 100,000,
plus the reserved 4 KiB to verify zero contents. Rollback adds one index read
at 100,000 ranges. It does not enumerate the whole free-space inventory.

Fixed costs remain substantial: begin writes 128 KiB with 5 syncs; reservation
writes 128 KiB with 3 syncs; payload writes 4 KiB; abort writes 132 KiB with 6 syncs
and one truncate. Total: 392 KiB written, 14 syncs, 8 writes. Journal space is
128 KiB, and rollback retains zero additional bytes. These are candidate-only
measurements, not a paired comparison against the current allocator or a claim
that A4 passes. Batch integration and complete archive measurements must assess
this overhead before selecting the format.

## Validation and limits

Tests exercise all four protection modes, failure at every storage mutation,
1,456 simulated power-loss cases across preparation and interrupted recovery,
66 actual child-process terminations using real files, 1,000 repeated abandoned
operations, a 12 MiB rollback checkpoint, and 48 additional power-loss cases
around that checkpoint. A separately constructed checksum vector and malformed
record cases check the encoding. Forged journal reservations cannot authorize
wiping live or direct control-root ranges. See `validation.txt` for executed checks.
The process-death tests are distinct from power-loss simulation; neither replaces
platform-specific durable filesystem qualification.

Remaining work is explicit:

- The allocator must derive and verify complete physical ownership, including all
  reachable child pages, payload extents, allocation-map pages, prepared partial
  writes and retired control records. The current journal checks its reservations
  against a caller-built authenticated map and excludes direct roots; it does not
  prove that the complete map is consistent with every reachable live descendant.
- The caller must remove reused PENDING ranges from the next pending map before
  publication. Whole-archive construction of that map is still pending.
- The 2,048-reservation cap has no spill implementation. Large committed cleanup
  currently gathers the pending range list before erasure; streaming and memory
  bounds for the complete allocator remain to be qualified.
- Mirror copies do not yet have independently qualified failure-region placement.
- The 1,000-cycle check covers journal aborts only. It does not replace realistic
  mirror/add/replace/remove lifecycle aging or exhaustive archive space accounting.
- Data extents, batching, production recovery, CLI/binding integration, migrations,
  platform tests and the complete CPU/RSS/read/write comparison remain unfinished.

## Reproduction and provenance

`run_series.rs` is the hook-formatted standalone Rust fresh-process runner;
`measured-run-series.rs.txt` preserves the exact runner source used for measurement. `resources.jsonl`
retains all observations including warmups, and `summary.json` includes medians
and observed minima/maxima without trimming outliers. `SHA256SUMS` covers evidence
and measured source extracts. `binary.sha256` identifies the preserved executable.

`measured-journal.rs.txt` is the exact unformatted journal implementation measured;
`measured-probe.rs.txt` is the exact resource test function. Subsequent additions
were codec and process-death/checkpoint tests, including a test-helper signature
change. They do not change the journal behavior or resource workload. The exclusive-writer
precondition was also documented after measurement. Formatting occurs
through the required commit hook after checks, so the final source differs in
formatting from these measurement snapshots.

```console
cd rust
cargo test -p revault_lockbox_api --release --lib preparation_journal -- --nocapture
# Preserve the release test executable printed above as /tmp/revault-journal-resource.
rustc --edition 2021 -O ../docs/evidence/preparation-journal-2026-09-27/run_series.rs -o /tmp/revault-journal-series
taskset -c 2 /tmp/revault-journal-series /tmp/revault-journal-resource
```

Only after the complete allocator and public paths exist should these component
results feed the A/B/C selection. The conservative production allocation protocol
remains the correctness control.
