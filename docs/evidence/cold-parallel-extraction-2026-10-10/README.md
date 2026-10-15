# Cold disk extraction and bounded parallel reads — 2026-10-10

This experiment follows the user's correction that warm-cache timings do not
establish first-use disk extraction performance. The earlier
[warm-cache comparison](../scale-extraction-2026-10-10/README.md) remains diagnostic
evidence. Its patterned many-file corpus, buffered output and cached archive
must not be described as physical disk throughput.

## Assessment

The balanced cold-input, buffered-output batch completed 96 measured and 32
excluded preparatory extractions, plus 112 recorded identity-admission skips.
The paired geometric-mean elapsed ratios show serial tree extraction taking
36.3% longer than ZIP for the 1 GiB file and 51.9% longer for 10,000 files.
For the 1 GiB file, two workers take 12.2% more elapsed time and 33.7% more CPU
than one worker. Four workers do not demonstrate an elapsed improvement—the
95% interval includes a ratio of one—and consume 40.2% more CPU.
[Balanced metrics](raw/balanced-buffered-batch/metrics-summary.json)

Keep one worker as the default. Parallel verification remains an explicitly
selected test experiment, not an accepted production optimization. The
10,000-file workload uses the small-read serial fallback, so its configured
worker variants do not demonstrate parallel verification performance.

Device counters mostly corroborate the intended mirror balance within reader
parity strata, with some unrelated host traffic visible. They do not support a
universal causal claim about mirror selection or backend performance. All
results here cover synthetic raw, unencrypted payloads on this host. Integrity
checks are unchanged, and correctness covers all sixteen mode combinations;
those modes have not all been performance-qualified. The next qualification
step is a realistic compressed/encrypted large stream before selecting further
pipeline optimization.

The secondary durable comparison remains exploratory because it predates the
identity/device observations. Its immediate per-file fsync protocol makes the
10,000-file case roughly 43 seconds, illustrating a different completion contract from
normal buffered write/close. Its timings must not replace the primary buffered
comparison or be combined with the separately measured post-verification drain.
A write-all-then-sync protocol was not timed here; 43 seconds is not a universal
cost of durable extraction.
[Durable metrics](raw/cold-batch/metrics-summary.json)

## Measurement protocol

The two disk-backed fixtures are one random 1 GiB file and 10,000 distinct random
64 KiB files (625 MiB). The latter uses the same deterministic SplitMix64 byte
formula as the native benchmark verifier, replacing the earlier patterned case
for this experiment. Contents are synthetic, raw and unencrypted. The machine's
normal Btrfs compression and mirrored storage configuration remain in use.

Before every fresh extraction process, the runner syncs the selected archive,
requests per-file cache eviction with `fadvise --advice dontneed`, and requires
`fincore` to report zero resident archive bytes. It additionally requires
process-accounted storage reads during extraction to reach at least 90% of the
logical payload. Every output is independently reopened and compared byte for
byte outside the timed section. These checks establish cold archive page cache
and substantial storage traffic; they do not establish cold filesystem metadata,
controller caches or a globally cold machine.

The primary comparison uses normal buffered completion: archive open,
destination creation, and writing, flushing and closing each output. It does not
promise output persistence at command exit. After the process independently
verifies output bytes, the runner drains its own output files in batches of at
most 500 paths with `sync --`, then syncs the output directory and its parent.
This scoped drain finishes before successful outputs are deleted and the next
sample begins, so deletion cannot cancel the pending output writes. Drain
failures preserve outputs and stop the batch. Its elapsed time is reported
separately: verification occurs between extraction and drain, so adding these
durations would not establish durable completion time.

A separate durable comparison includes archive open, destination creation, file writes and
`sync_all` on every output file, the output directory and its parent. This is a
stricter durability workload than the usual buffered command-exit time. Both ZIP
and Lockbox use the same sink and 64 KiB maximum output-write size. ZIP reaches
EOF and verifies CRC; Lockbox retains its selected metadata and chunk integrity
checks. The test endpoints exclude public CLI startup and permission restoration.

The balanced buffered protocol excludes four preparatory rounds and uses twelve
measured rounds per case. Reader TID parity alternates each round, giving two
preparatory and six measured observations per parity for every variant. All four
variants in a round request the same parity. A process with the wrong parity
returns a distinct admission-skip record before archive opening or output
creation; the runner retains the skip and retries at most sixteen times. Every
attempt still passes the archive eviction/residency gate. Admission uses identity,
never observed execution speed. The original durable protocol has three excluded
preparatory rounds and twelve measured rounds, without parity admission.

Every round rotates the order of ZIP and Lockbox with one, two
and four configured CPU workers. All variants have the same affinity mask,
CPUs 0–4, which represent five physical cores. ZIP and one-worker Lockbox execute
serially; two/four-worker Lockbox have those worker threads plus the caller.
Every round still evicts its input, including the excluded preparatory rounds.
Source inventories, archives, executables and Rust source identities are checked
before and after. Failed checks stop the run and preserve artifacts; only
successfully verified runner-owned outputs are removed.

Linux documents `POSIX_FADV_DONTNEED` as an attempt to discard cached pages, which
is why the runner checks residency separately rather than trusting the request.
The [kernel I/O counters](https://docs.kernel.org/filesystems/proc.html) report
process-accounted storage traffic, not logical read sizes or a complete account
of background filesystem work.

Buffered process write counters are sampled before the postprocess drain and
therefore need not account for eventual output persistence. The same cold input
residency and storage-read gates apply to both protocols. The primary buffered
comparison models ordinary write/close completion; the durable comparison is a
separate, stricter workload rather than a replacement for that measurement.

## Mirrored storage and separate source epochs

The host uses different NVMe models for its Btrfs mirrors (Lexar NM790 and
KINGSTON SFYRDK2000G), with the `pid` read policy. Upstream Linux 7.0 chooses a
preferred mirror using the issuing task ID modulo the mirror count; this is
the reader thread ID, which can differ from `std::process::id()`.
[Kernel mirror selection](https://github.com/torvalds/linux/blob/v7.0/fs/btrfs/volumes.c#L5885-L5945)
establishes a potential source of bias, not proof that it caused any observed
timing difference. Rotating backend order alone does not guarantee balanced
mirror choice. The original durable batch did not record reader identities or
per-device counters and remains exploratory for comparative performance.

The later buffered epoch adds safe, test-only reader identity and device-counter
observations. The measurement host was checked to have systemd as PID 1, a single
`NSpid` value and namespace `pid:[4026531836]`. Each admitted child must report
that namespace, one namespace TID, and the requested parity. IDs remain labeled
as visible through its procfs mount; these checks support the specific host setup
and do not generalize to arbitrary PID namespaces.

Per-device read/write sector counters bracket extraction before byte verification
and use 512-byte sector units. They are global counters, include unrelated host
I/O, and provide corroboration rather than process attribution. Even matched TID
parity does not prove the same physical mirror served different archive extents.
The report therefore retains per-parity distributions and actual device-read
fractions. Aggregate buffered confidence intervals resample six adjacent
two-round blocks, preserving both parities; within-parity intervals resample six
paired rounds. These are small-sample intervals without multiplicity correction.

The two epochs have separate executable hashes and source identities. The
original durable source identity, patch and changed/new files are preserved, and the later
buffered patch includes the observation module. The observation reads occur
outside the elapsed extraction timer; existing caller-level CPU accounting also
covers that helper overhead. Neither epoch changes the filesystem read policy.

To reconstruct the original durable epoch, start from `source_revision` in
`raw/durable-source/identity.json`, then apply `raw/durable-source/source.patch`.
Restore the two originally untracked modules from the retained snapshot paths
below, removing the final `.txt` suffix and placing them under the checkout's
`rust/` directory:

- `raw/durable-source/rust/revault_lockbox_api/src/file_format/candidate_files/tree_image/reader/parallel.rs.txt`
- `raw/durable-source/rust/revault_lockbox_api/src/file_format/candidate_files/tests/tree_tests/selective_reads/parallel.rs.txt`

Original changed tracked files, including `benchmark_extraction.rs`, are also
retained under `raw/durable-source/rust/` for inspection. The complete immutable
source snapshot remains in the local experiment directory and is hash-validated
before retention; unchanged source files are reproduced from the recorded Git
revision rather than duplicated in this evidence directory. Verify reconstructed
source files against the original identity's `source_hashes` map.

For the later balanced buffered epoch, start from the revision recorded in
`raw/balanced-buffered-batch/identity-after.json`, apply `raw/source.patch`, and
restore these separately retained new modules:

- `raw/source/reader-parallel.rs.txt` → `rust/revault_lockbox_api/src/file_format/candidate_files/tree_image/reader/parallel.rs`
- `raw/source/tests-parallel.rs.txt` → `rust/revault_lockbox_api/src/file_format/candidate_files/tests/tree_tests/selective_reads/parallel.rs`
- `raw/source/benchmark-observation.rs.txt` → `rust/revault_lockbox_api/src/benchmark_observation.rs`

Verify that epoch against its own `source_hashes` map. The retention manifest
maps original evidence paths to retained text filenames and records their hashes.
Executables and payload fixtures are not committed; build flags, logs, fixture
generation scripts and measured executable hashes are retained.

Bulky raw text above 128 KiB is retained losslessly as `.gz`, except the linked
metric summaries and named source/scripts. The manifest records original and
compressed hashes, sizes and encoding. Retention verifies the SHA-256 of the
decompressed stream against the original before writing the evidence directory.
Original raw files remain in the local experiment directory. Use `gzip -dc` to
recover a compressed record; no samples or repeated drain metadata are dropped.

## Implementation and limits

The experimental reader keeps index lookups, storage reads and output callbacks
on the caller thread. A per-reader pool and decoder workspaces are reused. Each
batch contains at most eight chunks per worker, capped at 32. Workers use the
same stored-byte checksum, decryption, authenticated padding and bounded
decompression checks as serial reads. Output remains in logical order; no
unverified chunk reaches the callback. All submitted jobs finish before callbacks
or return. Queued buffers and unused results wipe on drop; retained codec
workspaces and keys follow the existing session lifetime and wipe on codec drop.

Requests below 128 KiB or selecting one chunk stay serial. Therefore the
10,000-file fixture cannot gain parallel payload verification from this change:
differences there measure overhead and storage variability. The pool is still
created when explicitly configured. The one-worker default preserves the
existing optimized serial buffer reuse.

The conservative four-worker queued input/output model is 18 MiB, plus up to
1.25 MiB of transient decrypted buffers, bounded codec workspaces, metadata and
thread stacks. Process RSS is measured separately. Gathering, verification and
output occur in successive bounded stages, without overlapping archive I/O and
CPU work. Prefetch can increase first-byte latency and perform bounded extra work
before discovering a callback failure.

This remains test-only v4 work. It changes no archive encoding, dependency or
cryptographic algorithm and adds no unsafe code. Public CLI activation, encrypted
performance, million-file/TB scale and the total CLI memory acceptance gate
remain outstanding.

## Validation and retained failures

Strict Clippy and focused release checks cover all sixteen protection,
compression and padding combinations, ordered ranges, non-Send storage,
corruption rejection, callback error/unwind and subsequent pool reuse, batch
limits and the small-read fallback. Shared extraction checks cover durability
configuration, I/O accounting, output verification and overwrite refusal.

The first fixture setup failed because it read an inventory before creating it.
Its repair exposed a custom xorshift generator that disagreed with the native
verifier. Those source files and failure logs remain retained. The corrected
fixture uses a new directory, checks four windows against the existing native
random source and passes the native ZIP verifier for every generated byte.
The setup script also now excludes not-yet-created tree files from its initial
identity comparison. An initial zero-match test filter is retained and excluded
from validation; the corrected filter ran nonzero passing tests.

The first retention attempt stopped at its 16 MiB evidence-file guard before
writing any documentation. The largest reviewed sample file is 26,419,362 bytes;
the repeated `time` command text for per-file drain arguments accounts for much
of the 306 MiB buffered evidence. Retention now uses lossless compression with a
reviewed 32 MiB per-original-file bound. This was a retention failure, not a
benchmark failure; the original script and evidence remain preserved.

## Results

Both protocols use twelve measured rounds per case. Balanced buffered results
exclude four preparatory rounds; the original durable results exclude three.
RSS is the maximum process-lifetime peak observed before verification,
including startup. Whole-process RSS and storage-I/O ranges are in the linked JSON.

### Cold input, buffered completion with balanced reader parity

[Complete metrics and paired intervals](raw/balanced-buffered-batch/metrics-summary.json).

| Case | Variant | Median elapsed (s) | Median CPU (s) | Maximum peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1 GiB file | ZIP | 1.346 | 1.345 | 4.22 |
| 1 GiB file | Tree 1 | 1.851 | 1.835 | 10.30 |
| 1 GiB file | Tree 2 | 2.074 | 2.465 | 11.43 |
| 1 GiB file | Tree 4 | 1.883 | 2.610 | 12.89 |
| 10,000 × 64 KiB | ZIP | 1.123 | 1.121 | 10.32 |
| 10,000 × 64 KiB | Tree 1 | 1.703 | 1.691 | 10.56 |
| 10,000 × 64 KiB | Tree 2 | 1.699 | 1.688 | 11.19 |
| 10,000 × 64 KiB | Tree 4 | 1.704 | 1.696 | 10.82 |

Reader-parity strata each contain six measured samples. The device fraction is
the range of nvme0n1 read sectors divided by both devices' read sectors.
These counters include unrelated host activity; equal reader parity does
not prove equal physical mirror selection across different archive extents.

| Case | Variant | TID parity | Samples | Median elapsed (s) | Median CPU (s) | nvme0 read fraction range |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1 GiB file | ZIP | 0 | 6 | 1.375 | 1.374 | 0.935–1.000 |
| 1 GiB file | Tree 1 | 0 | 6 | 1.861 | 1.844 | 1.000–1.000 |
| 1 GiB file | Tree 2 | 0 | 6 | 2.085 | 2.429 | 0.988–1.000 |
| 1 GiB file | Tree 4 | 0 | 6 | 1.883 | 2.580 | 1.000–1.000 |
| 1 GiB file | ZIP | 1 | 6 | 1.337 | 1.336 | 0.000–0.001 |
| 1 GiB file | Tree 1 | 1 | 6 | 1.829 | 1.814 | 0.000–0.154 |
| 1 GiB file | Tree 2 | 1 | 6 | 2.074 | 2.478 | 0.000–0.006 |
| 1 GiB file | Tree 4 | 1 | 6 | 1.884 | 2.617 | 0.000–0.004 |
| 10,000 × 64 KiB | ZIP | 0 | 6 | 1.126 | 1.123 | 1.000–1.000 |
| 10,000 × 64 KiB | Tree 1 | 0 | 6 | 1.704 | 1.695 | 1.000–1.000 |
| 10,000 × 64 KiB | Tree 2 | 0 | 6 | 1.700 | 1.689 | 1.000–1.000 |
| 10,000 × 64 KiB | Tree 4 | 0 | 6 | 1.704 | 1.696 | 1.000–1.000 |
| 10,000 × 64 KiB | ZIP | 1 | 6 | 1.120 | 1.119 | 0.000–0.001 |
| 10,000 × 64 KiB | Tree 1 | 1 | 6 | 1.702 | 1.690 | 0.000–0.002 |
| 10,000 × 64 KiB | Tree 2 | 1 | 6 | 1.697 | 1.685 | 0.000–0.001 |
| 10,000 × 64 KiB | Tree 4 | 1 | 6 | 1.706 | 1.695 | 0.000–0.001 |

The following drain occurs after process verification and before cleanup. It is
reported separately, never added to the extraction time above; the intervening
verification gap means their sum is not a durable completion measurement.

| Case | Variant | Median postprocess drain (s) | Drain range (s) |
| --- | --- | ---: | ---: |
| 1 GiB file | ZIP | 0.699 | 0.669–0.849 |
| 1 GiB file | Tree 1 | 0.700 | 0.649–0.872 |
| 1 GiB file | Tree 2 | 0.706 | 0.670–0.848 |
| 1 GiB file | Tree 4 | 0.684 | 0.659–0.828 |
| 10,000 × 64 KiB | ZIP | 2.376 | 2.313–2.702 |
| 10,000 × 64 KiB | Tree 1 | 2.399 | 2.291–2.544 |
| 10,000 × 64 KiB | Tree 2 | 2.377 | 2.333–3.282 |
| 10,000 × 64 KiB | Tree 4 | 2.423 | 2.347–3.878 |

Paired ratios are candidate/baseline geometric means with seeded 10,000-replicate
95% paired percentile bootstrap intervals. Ratios below one mean less elapsed
time or CPU time for the candidate. Intervals have no multiplicity adjustment.

Aggregate intervals resample six adjacent two-round blocks, preserving both parity strata. Within-parity intervals resample six paired rounds.

| Case | Candidate / baseline | Elapsed ratio [95% CI] | CPU ratio [95% CI] |
| --- | --- | ---: | ---: |
| 1 GiB file | Tree 1 / ZIP | 1.363 [1.347, 1.377] | 1.352 [1.337, 1.366] |
| 1 GiB file, parity 0 | Tree 1 / ZIP | 1.343 [1.315, 1.372] | 1.333 [1.306, 1.360] |
| 1 GiB file, parity 1 | Tree 1 / ZIP | 1.382 [1.352, 1.414] | 1.372 [1.342, 1.404] |
| 1 GiB file | Tree 2 / ZIP | 1.529 [1.514, 1.547] | 1.808 [1.774, 1.840] |
| 1 GiB file, parity 0 | Tree 2 / ZIP | 1.514 [1.489, 1.545] | 1.770 [1.716, 1.825] |
| 1 GiB file, parity 1 | Tree 2 / ZIP | 1.544 [1.530, 1.558] | 1.847 [1.823, 1.877] |
| 1 GiB file | Tree 4 / ZIP | 1.381 [1.370, 1.394] | 1.896 [1.869, 1.927] |
| 1 GiB file, parity 0 | Tree 4 / ZIP | 1.363 [1.340, 1.391] | 1.861 [1.837, 1.893] |
| 1 GiB file, parity 1 | Tree 4 / ZIP | 1.399 [1.380, 1.417] | 1.932 [1.884, 1.977] |
| 1 GiB file | Tree 2 / Tree 1 | 1.122 [1.106, 1.139] | 1.337 [1.305, 1.367] |
| 1 GiB file, parity 0 | Tree 2 / Tree 1 | 1.127 [1.109, 1.145] | 1.328 [1.302, 1.357] |
| 1 GiB file, parity 1 | Tree 2 / Tree 1 | 1.117 [1.085, 1.145] | 1.347 [1.299, 1.391] |
| 1 GiB file | Tree 4 / Tree 1 | 1.014 [0.996, 1.031] | 1.402 [1.383, 1.428] |
| 1 GiB file, parity 0 | Tree 4 / Tree 1 | 1.015 [0.996, 1.035] | 1.396 [1.361, 1.431] |
| 1 GiB file, parity 1 | Tree 4 / Tree 1 | 1.012 [0.992, 1.031] | 1.408 [1.373, 1.446] |
| 10,000 × 64 KiB | Tree 1 / ZIP | 1.519 [1.514, 1.525] | 1.510 [1.505, 1.516] |
| 10,000 × 64 KiB, parity 0 | Tree 1 / ZIP | 1.524 [1.512, 1.537] | 1.517 [1.507, 1.529] |
| 10,000 × 64 KiB, parity 1 | Tree 1 / ZIP | 1.515 [1.510, 1.520] | 1.504 [1.499, 1.509] |
| 10,000 × 64 KiB | Tree 2 / ZIP | 1.513 [1.506, 1.523] | 1.504 [1.498, 1.511] |
| 10,000 × 64 KiB, parity 0 | Tree 2 / ZIP | 1.514 [1.505, 1.524] | 1.508 [1.499, 1.517] |
| 10,000 × 64 KiB, parity 1 | Tree 2 / ZIP | 1.512 [1.498, 1.525] | 1.500 [1.488, 1.511] |
| 10,000 × 64 KiB | Tree 4 / ZIP | 1.518 [1.514, 1.523] | 1.511 [1.507, 1.515] |
| 10,000 × 64 KiB, parity 0 | Tree 4 / ZIP | 1.517 [1.510, 1.526] | 1.512 [1.505, 1.520] |
| 10,000 × 64 KiB, parity 1 | Tree 4 / ZIP | 1.519 [1.510, 1.529] | 1.510 [1.501, 1.519] |
| 10,000 × 64 KiB | Tree 2 / Tree 1 | 0.996 [0.991, 1.001] | 0.996 [0.991, 1.001] |
| 10,000 × 64 KiB, parity 0 | Tree 2 / Tree 1 | 0.994 [0.981, 1.003] | 0.994 [0.982, 1.003] |
| 10,000 × 64 KiB, parity 1 | Tree 2 / Tree 1 | 0.998 [0.990, 1.005] | 0.998 [0.992, 1.002] |
| 10,000 × 64 KiB | Tree 4 / Tree 1 | 0.999 [0.995, 1.004] | 1.001 [0.996, 1.005] |
| 10,000 × 64 KiB, parity 0 | Tree 4 / Tree 1 | 0.996 [0.987, 1.001] | 0.997 [0.989, 1.002] |
| 10,000 × 64 KiB, parity 1 | Tree 4 / Tree 1 | 1.003 [0.997, 1.010] | 1.004 [0.998, 1.011] |

### Exploratory cold input, explicit durable completion

[Complete metrics and paired intervals](raw/cold-batch/metrics-summary.json).

| Case | Variant | Median elapsed (s) | Median CPU (s) | Maximum peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1 GiB file | ZIP | 2.008 | 1.976 | 4.11 |
| 1 GiB file | Tree 1 | 2.486 | 2.446 | 9.81 |
| 1 GiB file | Tree 2 | 2.707 | 3.043 | 11.41 |
| 1 GiB file | Tree 4 | 2.553 | 3.178 | 12.52 |
| 10,000 × 64 KiB | ZIP | 42.543 | 4.581 | 10.15 |
| 10,000 × 64 KiB | Tree 1 | 42.763 | 5.170 | 10.25 |
| 10,000 × 64 KiB | Tree 2 | 42.734 | 5.176 | 10.65 |
| 10,000 × 64 KiB | Tree 4 | 43.487 | 5.246 | 10.73 |

Paired ratios are candidate/baseline geometric means with seeded 10,000-replicate
95% paired percentile bootstrap intervals. Ratios below one mean less elapsed
time or CPU time for the candidate. Intervals have no multiplicity adjustment.

Durable intervals resample twelve paired rounds; reader parity and physical mirror selection were not recorded, so these intervals do not resolve that confounding.

| Case | Candidate / baseline | Elapsed ratio [95% CI] | CPU ratio [95% CI] |
| --- | --- | ---: | ---: |
| 1 GiB file | Tree 1 / ZIP | 1.253 [1.200, 1.321] | 1.227 [1.189, 1.263] |
| 1 GiB file | Tree 2 / ZIP | 1.347 [1.309, 1.382] | 1.535 [1.484, 1.579] |
| 1 GiB file | Tree 4 / ZIP | 1.313 [1.241, 1.428] | 1.599 [1.536, 1.663] |
| 1 GiB file | Tree 2 / Tree 1 | 1.074 [1.026, 1.118] | 1.251 [1.222, 1.280] |
| 1 GiB file | Tree 4 / Tree 1 | 1.047 [0.981, 1.127] | 1.303 [1.268, 1.340] |
| 10,000 × 64 KiB | Tree 1 / ZIP | 0.967 [0.835, 1.076] | 1.109 [1.042, 1.171] |
| 10,000 × 64 KiB | Tree 2 / ZIP | 0.827 [0.663, 1.007] | 1.030 [0.929, 1.124] |
| 10,000 × 64 KiB | Tree 4 / ZIP | 0.967 [0.818, 1.102] | 1.096 [1.000, 1.180] |
| 10,000 × 64 KiB | Tree 2 / Tree 1 | 0.855 [0.688, 1.002] | 0.929 [0.838, 1.002] |
| 10,000 × 64 KiB | Tree 4 / Tree 1 | 1.000 [0.775, 1.296] | 0.989 [0.876, 1.079] |


[Raw protocol, flags, samples and test logs](raw/retention-manifest.json) and
[tracked Rust changes](raw/source.patch) retain the measured implementation.
The [original durable epoch patch](raw/durable-source/source.patch) and
[original source identity](raw/durable-source/identity.json) accompany the
preserved original changed/new source files; unchanged files come from the
recorded base revision. The buffered epoch adds the
[observation module](raw/source/benchmark-observation.rs.txt). Scripts and Rust
snapshots are retained as text, with original-to-retained paths in the manifest.
The new [parallel reader](raw/source/reader-parallel.rs.txt) and
[parallel regression tests](raw/source/tests-parallel.rs.txt) are retained separately.
