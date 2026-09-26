# CPU and memory baseline for the v4 architecture evaluation

This evidence is an initial comparison, not a completed architecture selection or
release qualification. Candidate A is the current default layout; candidate B is
`native-block-layout`. Candidate C is not implemented yet. Both binaries include
the same evaluation runner and use one archive worker, CPU 2 affinity, the default
Interactive cache profile and default size padding.

## Method and reproduction

Build `archive_evaluation` in release mode in both configurations, retain the
executables separately, finish all builds/tests, then run the executables. The
runner's source, Cargo.lock and executable hashes are recorded in JSONL. Build
source begins at `232aae44` plus the accompanying runner change; no production
archive code changed for these measurements. The runner-source hash is embedded
at compile time, so rebuilding an old executable does not silently change its
provenance. `source_revision_at_run` alone is not a build revision assertion.
The exact pre-formatting [measured source](measured-runner.rs.txt) is retained so
later maintenance/formatting does not make the embedded source hash unresolvable.
Lockbox uses Zstd level 3; ZIP uses the locked ZIP crate's default Deflate level.

```sh
cargo bench -p revault_lockbox_api --bench archive_evaluation --no-run
cargo bench -p revault_lockbox_api --features native-block-layout --bench archive_evaluation --no-run
# Copy each executable before building another configuration; substitute its path below.
taskset -c 2 /path/to/default run /new/evidence/path 1 8388608 pattern compressed plain 30 stream 1 default /path/to/native
/path/to/default summarize /new/evidence/path/samples.jsonl
```

The complete `run` arguments are: new directory, file count, bytes per file,
`pattern|random|mixed`, `raw|compressed`, protection mode
(`plain|encrypted|signed|encrypted-signed`), measured pair count, `stream|range`,
read passes, `default|none` padding, optional other executable.
It refuses to replace an existing evidence directory. `sample` and `create` are
internal worker commands; normal use is through `run`.

Each timed observation has a fresh process and archive handle. Warm-up uses three
complete pairs; other measurements are preserved without filtering. ZIP/default/
native order alternates and rotates. OS caches are warm from fixture preparation
and independent byte verification. This is not a cold-disk result. Other host work
is uncontrolled and the recorded environment includes CPU affinity and load.

`stream` means sequential `open_file` reads through a 64 KiB buffer using the
default profile. It does not mean the specialized `stream_content` bulk traversal.
`range` seeks to each file's midpoint and reads 4 KiB. A raw ZIP range seeks; a
compressed ZIP range decodes its prefix and does not establish whole-entry CRC
inside the timed range. Whole ZIP reads consume EOF and therefore exercise CRC.
The first-byte result times an actual one-byte read, not a 64 KiB chunk.

Every worker snapshots elapsed time, user/system CPU, major page faults and Linux
`ru_maxrss` before verification. It then separately reopens the archive and
compares every content byte with the deterministic corpus. Range workers also
repeat the exact seek/read with byte comparison outside timing. Source generation
uses bounded buffers, including the GB case. Corpus inventories contain paths,
lengths and SHA-256; the JSONL pins the inventory hash.

Peak RSS is the worker's high-water mark before post-measurement verification,
including startup and archive opening. `baseline_peak_rss_kib` is the pre-open
high-water mark, not a separately established empty-archive baseline. These values
are not per-allocation peaks and should not be subtracted as exact heap sizes.
Fixture-create measurements include source-file input, finalization and durability;
key/source generation and subsequent verification are excluded. There is only one
fixture-create observation per candidate, so it cannot establish the write gate.

Summaries use 10,000 fixed-seed paired bootstrap resamples of log ratios and retain
all raw samples. `qualifying_sample_count` only checks the 30-pair count; it does
not certify the complete performance/security gate or an entire workload matrix.

## Initial result: 8 MiB compressible unsigned plaintext

Thirty measured pairs, one sequential pass, one file, default privacy padding.
Times below include archive open and read. KiB values are reported as MiB / 1024.

| Candidate | Median elapsed | Median CPU | Median peak RSS | Paired elapsed ratio to ZIP (95% interval) |
| --- | --- | --- | --- | --- |
| ZIP/Deflate | 1.565 ms | 1.571 ms | 3.36 MiB | 1.00 |
| A: default/Zstd | 7.462 ms | 7.456 ms | 10.50 MiB | 4.757 (4.713–4.816) |
| B: native/Zstd | 8.052 ms | 8.059 ms | 14.19 MiB | 5.119 (5.080–5.158) |

For this case B is about 7.6% slower than A and uses about 35% more peak RSS.
Neither meets ZIP read parity. CPU closely follows elapsed time, so an I/O-only
explanation does not fit this result. The experiment does not identify which
codec/layout stages account for that CPU cost. It does not establish results for
other corpora, signed/encrypted modes, small-file packing or cold storage.

[Raw samples](8m-pattern-compressed-plain.jsonl) and
[paired summaries](8m-pattern-compressed-plain-summary.jsonl) are retained here.
The separate 1 GiB raw streaming probe is for memory scaling, with one measured
pair after warm-up; it is not a statistical latency gate.

## 1 GiB raw streaming probe

Unsigned plaintext, one file, default padding/profile, one worker. One measured
read observation after three warm-up pairs; creation has one observation per
backend. These values must not be presented as a statistical speed guarantee.

| Candidate | Create peak RSS | Read peak RSS | Read elapsed | Read CPU |
| --- | --- | --- | --- | --- |
| ZIP/stored | 4.44 MiB | 3.41 MiB | 0.275 s | 0.275 s |
| A: default/raw | 1,066.91 MiB | 142.54 MiB | 1.494 s | 1.493 s |
| B: native/raw | 14.64 MiB | 6.64 MiB | 0.785 s | 0.785 s |

A's creation memory exceeds the proposed 256 MiB incremental streaming budget by
a wide margin. This is archive-engine memory, not a benchmark allocating a GB
source vector: the input was generated on disk before timing and supplied through
`File` to `add_file_from_reader`, and RSS was captured before verification.

Source inspection explains a likely cause: `FilePageWriter::flush` stages ordinary
decoded pages under the Interactive profile; only BulkImport writes/discards them
immediately. `stage_decoded_page_with_policy` marks those pages dirty, which keeps
them resident until commit. This is a hypothesis supported by the implementation
and peak scaling, not an allocation-profile proof. It requires a bounded staging
policy or a layout that avoids retaining a full import, rather than documenting
BulkImport as the only way ordinary large-file imports can fit in memory.

B's large raw read/creation memory improves substantially while its 8 MiB
compressed result regresses. Both findings must remain in the architecture
scorecard. [Raw GB samples](1g-pattern-raw-plain.jsonl) and
[summaries](1g-pattern-raw-plain-summary.jsonl) are retained. The degenerate
single-observation bootstrap intervals in that summary convey no uncertainty
estimate; `qualifying_sample_count` is false.

## Runner validation

The deterministic corpus/chunking and paired-statistics tests pass. Benchmark and
test-target Clippy passes with warnings denied. Eight small seek/read smoke cases
cover four protection modes × raw/compressed, with no padding, under both layouts
and ZIP; all independently verify their full contents and requested ranges. These
are worker-correctness checks, not an A1/A2 security or performance qualification.

## Outstanding measurement work

Complete protection/codec/padding/access and workload matrices, a separate
empty-open memory baseline, specialized bulk traversal, cache profiles, timed
creation/update/mirror/recovery/compaction samples, 100/1,000-cycle accounting,
actual I/O/decoded/wiped byte instrumentation, cold I/O and remote-range counts.
Add candidate C under the same trust and transaction contract before choosing a
layout. Do not claim the matrix passes from this initial result.
