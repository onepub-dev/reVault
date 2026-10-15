# Current public CLI read/write comparison with ZIP

This is a new descriptive comparison of current default-layout reVault and
Info-ZIP public commands. It does not reuse historical component timings or
multiply earlier optimisation ratios. Build the exact current HEAD in release
mode and preserve its source/build manifest and binary hash before timing.

Use the two existing pinned 1 GiB sources (source-zstd and random-raw), with no
new corpus copies. Each pinned source is a regular 0664 file; preserve its original metadata.
Both extractors must produce 0644 under controlled umask 0022, confirmed by an
untimed owned 0664-input smoke before freeze. Source mode is checked before
and after the batch. Store exactly its basename
at archive root in both formats. Three measured repetitions per source/variant
are descriptive only: publish medians and full ranges, without confidence,
parity, noninferiority or release qualification claims.

## Fixed schedule

Two sources × two tools × four rounds × write/read = 32 accepted operations.
Round zero is preparation: 8 excluded operations; 24 measured operations.
For each case and round, alternate tool order by round+case parity. Each tool
first creates a fresh archive, then extracts it through a separate public
invocation to a fresh directory. The extracted bytes validate both operations;
admit their rows only after exact tree/type/size/mode and independent cmp checks.
Delete only those owned archive/output paths after verification and drain.
Do not retain multi-GiB fixture archives between rounds.

Native write consists of explicit `create --encryption none --signing none
--compression zstd --compression-level 3` followed by single-file `add SOURCE
--to /BASENAME`. This includes fresh archive creation while measuring the
existing-archive single-file Interactive add policy (8 MiB cache target), not
implicit encrypted creation or BulkImport. Native extraction is pathless
`extract --to DIRECTORY` and therefore uses ExtractMany. Write jobs remain Auto; the public reader retains its actual default settings.
The one-file fixture cannot activate the >=256-file parallel extraction pool;
candidate adaptive reader gains are not enabled or attributed here.
pin CPU affinity 0–4 and record observed available processors.

ZIP write uses `/usr/bin/zip -6 -j -q ARCHIVE SOURCE`; extraction uses
`/usr/bin/unzip -q ARCHIVE -d DIRECTORY`. Capture exact executable SHA256 and
version output (Info-ZIP zip 3.0/unzip 6.00 on this host). Capture ZIP listing and
actual stored/Deflate method outside timing rather than assume compressibility.
Native doctor/list checks establish plaintext, unsigned, Zstd level 3, default
padding, and exactly one expected file. ZIP and native archive sizes are saved
for every run.

## Timing and resources

Wrap each public subprocess in GNU time with elapsed/user/system/max-RSS,
taskset, and bounded timeout. Native write has two CLI subprocesses; elapsed
and CPU totals are their sums, and process peak RSS is their maximum (never a
sum). No harness gaps, metadata inspection or correctness checks enter CLI
elapsed. Report this explicitly as summed public-command elapsed rather than
one continuous process duration. ZIP write and each extraction have one public
subprocess. RSS excludes the separately started native helper agent and is not
a simultaneous process-tree memory measurement.

Also measure an identical `sync -d` of resulting files and parent directory
after each operation. Report CLI elapsed and drain elapsed separately, plus
their sum as a common output-drained completion measure, not equivalent crash
durability. Native commits already
perform their required durability work; ZIP close/recovery semantics differ.
Do not subtract native durability or integrity work to manufacture parity.
Source eviction/fincore-zero admission precedes each write; archive eviction
precedes each extraction. These preparation costs are outside reported elapsed.

Use isolated HOME/XDG/Vault/agent directories, no inherited configuration or
credentials (ZIP/ZIPOPT/UNZIP/UNZIPOPT absent), umask 0022, and actual public agent roundtrip before timing. Keep
the agent path short, record cleanup success and require socket absence after
owned stop. Hash sources, binaries, scripts, protocol and build provenance
before and after the batch. Verify source files unchanged.

## Bounds and interpretation

Reserve archive overhead plus output before each operation, enforce the existing
24 GiB combined owned-artifact budget and 2 GiB free-space margin. Work deadline
is 30 minutes with 60 seconds reserved for bounded owned cleanup; individual
commands are capped at 600 seconds or remaining budget, with kill-after 10
seconds. Cleanup grace can extend the total wall limit. Stop on any failure;
preserve all attempts and do not add samples based on outcomes.

No builds, tests or other timed workloads overlap the batch. Runtime stays
unchanged. Report ratios of descriptive medians, elapsed ranges, peak RSS,
archive bytes and correctness status for each source and operation. This is
Zstd3 versus Deflate6/actual ZIP fallback, native integrity/metadata/recovery
versus ZIP CRC, not equivalent codecs or security guarantees. It says nothing
about encrypted ZIP+PGP, small files, many-file workloads, candidate formats,
cross-version compatibility or release readiness.
