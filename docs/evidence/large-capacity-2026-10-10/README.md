# Disk-backed capacity checks — 2026-10-10

The earlier 4 KiB measurements concern requested ranges, not 4 KiB archives.
However, 8/64 MiB archives and 512-file fixtures on RAM-backed `/tmp` are only
microcost evidence. They do not qualify realistic disk throughput, filesystem
extraction, large-file capacity or many-file scaling. The user explicitly asked
that realistic workloads take priority over improvements to these small cases.

## Fixed capacity cases

Both cases use the frozen journal-buffer candidate based on `1db0cb59`, Rust
1.88.0, raw/plain mode with default padding and 64 KiB logical units. Candidate
executable SHA-256 is `6e52a248f63699ae842af76c4cf38ed94edfd93dd78829e5c53fd8cb063bec18`.
Data lives under `/home/bsutton/.cache/revault-performance-20261010-large` on
Btrfs `/dev/dm-1`, not tmpfs. Mount options include `compress=zstd:1`; disk-backed
does not imply cold cache. No cold-cache or physical-device throughput claim is
made. The two cases run sequentially with a 900-second timeout each and a 20 GiB
total artifact ceiling. These are functional capacity checks, not comparative
timings. The same candidate adapter is supplied for both runner roles.

| Case | Logical data | Outcome |
| --- | --- | --- |
| One random file | 1 GiB | Fresh tree export refuses more than 4,096 fragments |
| 10,000 mixed files, each 64 KiB | 625 MiB | Fresh tree export refuses more than 1,024 files |

The first case exits 101 when the test probe unwraps
`SecurityLimitExceeded("cost model: at most 4096 fragments")`. It generates the
source, ZIP and packed staging archive before failing during fresh tree export.
Its overall command takes 15.90 seconds with a reported maximum RSS of 20,196 KiB.
These include setup and failed creation; they are neither extraction performance
nor evidence that the successful CLI would meet the 100 MB RSS goal. The source,
ZIP, staging archive and empty destination remain available. No smaller case was
substituted to obtain a passing result.

The second case also exits 101, with
`SecurityLimitExceeded("cost model: at most 1024 files")`. All 10,000 sources,
the ZIP and packed staging archive were produced; the final destination is empty.
The overall failed command takes 30.35 seconds (user 6.41, system 8.90) and reports
19,440 KiB maximum RSS. This is another construction capacity failure, not a
successful read or extraction result. Its exact executed Dart script is preserved
separately from the corrected copy that adds the required dcli package import;
the correction did not trigger a rerun.

[Exact commands, logs and result records](raw/results.json) are retained together
with the [second-case command](raw/raw-mixed-10000x64k.command.txt),
[second-case result](raw/raw-mixed-10000x64k.result.json) and
[second-case log](raw/raw-mixed-10000x64k.log). Large binary fixtures remain on the
host rather than being committed to Git.

## Implication for the implementation

The limits belong to the experimental construction/audit path, not evidence of
an inherent archive-format size ceiling or a test of the released format-3 CLI.
Source review finds several coupled bounds: fresh repacking accepts at most
1,024 files/4,096 fragments, the catalogue retains a 4,096-fragment limit, and
ownership auditing has separate page, allocation-record and claim bounds.
Increasing one constant would simply encounter later constraints. Construction
also retains metadata collections proportional to file/fragment counts.

The next scale work must address bounded-memory construction and audit paths,
preserve integrity checks and validate independent reopen/content verification.
Do not silently raise all caps, skip full verification or weaken the 100 MB total
CLI RSS objective to produce a timing result. Parallel payload verification can
be evaluated after meaningful workloads can be created and verified.

## Primary performance evidence going forward

Use the two workloads above as the first capacity gate, then measure full create,
full read and filesystem extraction, plus selected small reads within those
large archives. Report wall seconds, MiB/s, CPU time and peak RSS, alongside
absolute latency differences and ratios. Match ZIP's codec, checking and output
durability policy, and verify output bytes outside the timing window. Keep raw
and compressed, plain and protected, one-worker and bounded-parallel, warm-cache
and controlled-cold results distinct. A verified streaming sink is not extraction
to a filesystem. Use a pinned real source-tree corpus alongside generated data.

GiB and 10,000-file cases are only an initial gate. G3's TB-scale/million-file
objectives and G11's public CLI performance/memory requirements remain unqualified.
Retain small benchmarks as diagnostics; do not use them as the release scorecard.
