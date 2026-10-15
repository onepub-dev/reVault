# Current public reVault and ZIP read/write results

The fixed comparison completed all 32 operations in 342.15 seconds: 8 excluded
preparation operations and 24 measured operations. Each table cell has three
measured repetitions. Independent persisted byte, exact tree/type/mode, cold
input, identity and owned-agent cleanup checks passed. These are descriptive
medians and full ranges, not statistical parity or release qualification.

| Source / operation | reVault seconds, median [range] | ZIP seconds, median [range] | reVault / ZIP maximum RSS, MB |
| --- | --- | --- | --- |
| Compressible / write | 2.18 [2.14–2.27] | 23.09 [23.03–23.23] | 104.62 / 2.86 |
| Compressible / read | 3.70 [3.70–3.89] | 4.89 [4.88–4.94] | 77.28 / 3.05 |
| Random / write | 6.56 [6.56–6.57] | 24.31 [24.20–24.57] | 92.30 / 2.86 |
| Random / read | 2.92 [2.91–2.92] | 5.54 [5.52–5.60] | 75.27 / 2.99 |

Both sources contain 1 GiB (1,073,741,824 bytes). RSS uses decimal MB and is the
maximum measured subprocess peak; the separately running helper agent is
excluded. Native write includes the sum of fresh archive creation and add CLI
times; its RSS is the maximum of those processes, not their sum. Native uses
substantially more memory than Info-ZIP. Compressed writes still exceed the
100 MB target in this screen.

After adding the separately measured common output drain, median completion
times (reVault / ZIP) are 2.18 / 23.19 seconds for compressible writes,
4.44 / 5.61 for compressible reads, 6.56 / 24.91 for random writes, and
3.47 / 6.11 for random reads. Drain time is reported separately in the raw
summary; output-drained completion does not establish equivalent crash durability.
GNU time's hundredth-second resolution reports the native write drains as 0.00.

| Source | reVault archive bytes | ZIP archive bytes |
| --- | --- | --- |
| Compressible | 183,206,272 | 182,405,087 |
| Random | 1,074,206,080 | 1,073,915,836 |

This uses the current default-layout native CLI `0.5.0-dev.1`, built from
`765e4ac2` (binary `08bf819f`), against public Info-ZIP zip 3.0 and unzip 6.00.
Native archives are plaintext, unsigned, default-padded, configured Zstd level 3
with Auto write workers. ZIP uses `-6` and actually reports **Deflate for both
corpora**, including random data. Native extraction uses ExtractMany; this
one-file workload cannot activate the >=256-file parallel extraction dispatcher.
Candidate adaptive-worker results are not enabled or attributed here.

Native integrity/recovery and ZIP CRC differ, as do codecs, worker models and
metadata policies. Current native ordinary import stores mode `0600`; extraction
now restores that mode by default. ZIP preserves source mode `0664`. Both actual
modes were verified under umask 0022; no permission equivalence is claimed.
This fresh public-command screen differs from the historical Rust ZIP component
screen (including that screen's Stored/raw variant). Do not attribute the
change in relative results solely to recent optimisations or combine their ratios.
No encrypted ZIP+PGP, small/many-file, native-block-layout or cross-version
acceptance is established.

Preparation discovered the historical sources' mode `0664`, then differing
default restoration and native stored `0600` behavior. All stopped attempts and
original scripts/protocols are preserved. The user-requested CLI default change
was separately tested and committed before a new current release build and the
successful final smoke/freeze. No timed samples preceded that final preparation.
The old binary/build provenance remains separate from the measured current one.

See [audited summary](raw/metrics-summary.json), [all samples](raw/samples.jsonl),
[completion](raw/complete.json), [frozen identities](raw/baseline-identities.json),
[current build provenance](raw/build-current/) and [original frozen protocol](raw/frozen-protocol.md).
All 921 command evidence files, including stopped preparation attempts, are
losslessly packaged in [command evidence](raw/command-evidence.tar.gz), with
every extracted byte verified against [the manifest](raw/command-evidence.sha256).
Original command files remain in ignored local storage. Archive/corpus payloads
are not committed. Script and build source identities preserve the measured
epoch; the report is separate from the original frozen protocol.

The following protocol records the pre-result design; its original bytes are
preserved separately from this completed report.


This is a new descriptive comparison of current default-layout reVault and
Info-ZIP public commands. It does not reuse historical component timings or
multiply earlier optimisation ratios. Build the exact current HEAD in release
mode and preserve its source/build manifest and binary hash before timing.

Use the two existing pinned 1 GiB sources (source-zstd and random-raw), with no
new corpus copies. Each pinned source is a regular 0664 file; preserve its original metadata.
Native extraction now restores stored permissions by default, as requested by
the user in checkpoint `765e4ac2`. Ordinary add stores file mode `0600`, whereas
ZIP preserves the source mode `0664`; verify these actual output modes under
controlled umask 0022. This is an explicit metadata-policy difference, not exact
mode equivalence or source-mode preservation by native import. A tiny owned
0664-input smoke verifies both policies before freeze. Source mode is checked
before and after the batch. Store exactly its basename
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
`extract --to DIRECTORY` and therefore uses ExtractMany and the new stored-mode
restoration default. No redundant permission flag is supplied. Write jobs remain Auto; the public reader retains its actual default settings.
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
