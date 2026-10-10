# Compressed and encrypted cold-disk extraction — 2026-10-11

## Assessment

Four workers reduce elapsed extraction time by 8.6–22.5% in the five tested
compressed or encrypted GiB cases, while increasing CPU time by 45.0–70.9%.
They regress the raw/plain source case by 9.5%. These are paired geometric-mean
changes within each workload, not ratios of the median timings below.

The user explicitly accepts increased CPU use as a reasonable trade-off for
reduced extraction time. CPU growth alone therefore does not reject these gains.
The next implementation experiment should evaluate selection of four workers for
large streams with sufficient decryption/decompression work, while retaining the
serial path for cheaper reads. Establish the size crossover and the incompressible
plaintext compression-fallback boundary before choosing an automatic policy.
Two workers do not establish an elapsed improvement in any of these six cases.

This milestone adds measurements and guidance, with no native reader changes.
The existing default remains one worker; the explicit parallel path remains an
experimental v4 capability. No archive format, integrity check or wipe lifetime
changes. The measured gains do not establish ZIP parity, PGP parity, public CLI
performance, or the 100 MB total CLI memory goal.

[Combined results](raw/combined-metrics-summary.json) retain each case's epoch.
The [first three cases](raw/balanced-buffered-batch/metrics-summary.json),
[combined protection case](raw/recovery-buffered-batch/metrics-summary.json), and
[random-data cases](raw/final-buffered-batch/metrics-summary.json) include paired
intervals, parity strata, CPU, RSS, storage counters and separate output-drain times.

## Workloads and completion contract

The source corpus contains 920 allowlisted committed code files from
`6735f6dbf949915ed7b94f0b6b9bb5fb7895e231`, excluding generated, fixture and testdata
directories, totaling 14,284,673 code bytes. Path/length framing produces
14,341,790 bytes. The primary source
stream cycles that bundle 74 times and appends a partial repetition to reach
exactly 1,073,741,824 bytes. It represents repeated real source content, not a
GiB of unique source files or a many-file extraction workload. The unexpanded
bundle is used only for preflight diagnostics. The other primary input is the
previously verified deterministic random GiB stream.

The [corpus inventory](raw/corpus-plan.json.gz) and [stream identities](raw/corpus-ready.json)
pin the selection and construction. The bundle SHA-256 is
`2b61fa08f0ddcafbc0aed21a149ea62c9134aa7a947a859d8dc2dd73e9a1f59e`;
the expanded stream is
`a90cac77289634f3369065f9037ce2d2f48490e967efaf59d78c3c51508ee77d`.

Every primary case extracts one GiB file to disk. Lockbox uses 64 KiB units,
default size padding, no owner signature, and either no encryption or
ChaCha20-Poly1305. Compression is disabled or Zstd level 3, including its raw
fallback. ZIP inputs are created with the recorded Info-ZIP executable using
stored mode (`-0`) or Deflate level 6 (`-6`), and decoded by the same Rust ZIP
benchmark endpoint as the preceding experiment. ZIP is always unencrypted and
verifies CRC through EOF. The encrypted comparisons are unequal-protection
baselines; the compressed comparison also uses different codecs and layouts.

Timing includes archive open, output creation, writes, flush and close. It uses
the shared extraction sink with output writes bounded to 64 KiB. It excludes
public CLI startup, permission restoration and output durability. Every output
is independently reopened and compared byte for byte after timing. The runner
then syncs its own output and directory chain before deleting successful output
and starting the next sample. That drain is reported separately: verification
occurs between extraction and drain, so their sum is not a measured durable
completion time.

## Cold input and measurement protocol

Fixtures, archives and output reside on the mirrored `/home` Btrfs filesystem.
Before each process, `sync -d`, per-file `fadvise DONTNEED`, and `fincore` must
establish exactly zero resident archive bytes. Neither global cache dropping
nor filesystem policy changes are used. This establishes cold archive page
cache, not cold controller caches or cold filesystem metadata.

Btrfs transparently compresses the raw/plain source archives. Its `stat` block
count did not supply a reliable physical-I/O denominator. Before primary timing,
each of the twenty archives instead received two independent cold sequential
64 KiB reads, bracketed by the reader's own `/proc/self/io` counters. Both scans
must have positive physical traffic and agree within 10%. Extraction must then
report at least 50% of the smaller calibrated count, in addition to zero initial
residency and complete output verification. This is a conservative substantial-I/O
guard, not proof of identical physical requests or complete whole-archive reads.
Selected reads may legitimately omit other archive regions.

The [calibration](raw/physical-read-calibration.json) records all scans. The
initial allocation-based preflight refusal remains preserved. The last two cases
permit up to three eviction/residency checks, 100 ms apart, before process launch;
every attempt is recorded and the final successful check must still report zero.
The [final admission policy](raw/final-admission-policy.json) was frozen before
those cases. It changes no timed operation or storage-read threshold.
The [final admission audit](raw/final-admission-audit.json) records 232 process
admission gates, including parity skips, and 233 eviction checks. Exactly one
gate needed a second eviction check; none needed a third.

Each case contains four excluded preparatory rounds and twelve measured rounds,
each with serial ZIP and Lockbox configured for one, two and four workers. Order
rotates within rounds. Reader TID parity alternates, balancing the host's mirrored
read policy with six measured samples per parity per variant. All variants use
CPU affinity 0–4. Parallel Lockbox uses its configured worker threads plus the
caller; ZIP and one-worker Lockbox remain serial. Global device counters provide
corroboration and include unrelated host activity. No builds or other tests from
this experiment overlap timed samples; other host activity is uncontrolled.

The accepted dataset has 384 extractions: 288 measured and 96 preparatory.
Forty separate preflights verify the ten fixture configurations and four variants.
Two interrupted cases contribute 24 excluded successful extractions each; neither
partial case contributes to the reported comparisons. All failures remain retained.

Ratios use 10,000 seeded paired percentile bootstrap replicates, resampling six
adjacent two-round blocks to preserve both parity strata. Within-parity results
resample six paired rounds. Intervals have no multiplicity correction and are
limited to this host and these workloads. Cross-mode medians are descriptive:
the cases run sequentially and are not paired estimates of isolated crypto or
codec costs. Encryption, padding, filesystem compression and disk traffic all
affect those comparisons.

## Results

Median elapsed seconds for cold input and buffered output:

| One GiB workload | ZIP | Lockbox 1 worker | 2 workers | 4 workers |
| --- | ---: | ---: | ---: | ---: |
| Repeated source, raw/plain | 1.460 | 1.932 | 2.256 | 2.104 |
| Repeated source, raw/encrypted | 1.430 | 2.562 | 2.677 | 2.249 |
| Repeated source, compressed/plain | 2.244 | 4.226 | 4.607 | 3.595 |
| Repeated source, compressed/encrypted | 2.412 | 4.765 | 4.609 | 3.458 |
| Random, raw/encrypted | 1.334 | 2.567 | 2.684 | 2.250 |
| Random, compression enabled/encrypted | 2.249 | 2.744 | 2.881 | 2.514 |

Median CPU seconds, summed across process threads:

| One GiB workload | ZIP | Lockbox 1 worker | 2 workers | 4 workers |
| --- | ---: | ---: | ---: | ---: |
| Repeated source, raw/plain | 1.458 | 1.930 | 2.558 | 2.606 |
| Repeated source, raw/encrypted | 1.430 | 2.549 | 3.517 | 3.894 |
| Repeated source, compressed/plain | 2.243 | 4.216 | 6.947 | 7.875 |
| Repeated source, compressed/encrypted | 2.381 | 4.737 | 6.833 | 6.984 |
| Random, raw/encrypted | 1.334 | 2.549 | 3.539 | 3.763 |
| Random, compression enabled/encrypted | 2.248 | 2.662 | 3.610 | 3.849 |

Four-worker / one-worker paired ratios; below one favors four workers:

| Workload | Elapsed ratio [95% CI] | CPU ratio [95% CI] |
| --- | ---: | ---: |
| Source, raw/plain | 1.095 [1.088, 1.101] | 1.350 [1.337, 1.363] |
| Source, raw/encrypted | 0.877 [0.869, 0.886] | 1.518 [1.490, 1.539] |
| Source, compressed/plain | 0.775 [0.674, 0.863] | 1.709 [1.429, 1.946] |
| Source, compressed/encrypted | 0.786 [0.704, 0.912] | 1.483 [1.400, 1.583] |
| Random, raw/encrypted | 0.879 [0.855, 0.904] | 1.479 [1.453, 1.514] |
| Random, compression enabled/encrypted | 0.914 [0.899, 0.929] | 1.450 [1.426, 1.480] |

The maximum extraction-stage process RSS across all accepted variants is
16.65 MiB. It includes process startup and is sampled before output verification;
the linked results also retain whole-process RSS. This small test endpoint does
not establish the total public CLI memory gate.

Calibrated full-archive physical read bytes, distinct from logical output size:

| Workload | ZIP | Lockbox |
| --- | ---: | ---: |
| Source, raw/plain | 239,755,264 | 241,721,344 |
| Source, raw/encrypted | 239,751,168 | 1,155,981,312 |
| Source, compressed/plain | 182,407,168 | 231,096,320 |
| Source, compressed/encrypted | 182,407,168 | 231,923,712 |
| Random, raw/encrypted | 1,073,745,920 | 1,155,981,312 |
| Random, compression enabled/encrypted | 1,073,917,952 | 1,438,375,936 |

## Interrupted runs, source epochs and reproduction

The first three complete cases use the exact binaries retained from the
[preceding cold-disk experiment](../cold-parallel-extraction-2026-10-10/README.md).
Reconstruction verified all 241 recorded source hashes and both executable hashes.
Six source files differ from the formatted `6735f6db` checkout only through
pre-commit formatting. The strict whitespace-only checker refused optional commas
and return semicolons; the [manual review](raw/provenance-manual-review.json)
and full diff preserve its resolution. These are binaries from the pinned prior
source epoch, not a claimed rebuild from the formatted commit.

During the fourth case, the `/tmp` executable directory disappeared. The cause
was not established. The runner stopped on the missing executable, preserving
the incomplete batch and its first three complete case audits. An independent
[post-interruption audit](raw/interruption-audit.json) checked native source,
runner and fixture identities. Each accepted sample already carried matching
before/after executable hashes; the missing final full-batch executable audit
is not fabricated or labeled successful.

The exact preserved source was then rebuilt with Rust/Cargo 1.88.0, locked offline
release settings and the recorded `external-source` features. The new binaries
and build target reside on disk outside `/tmp`. Construction used `--no-run`;
no full test suite ran. The [recovery executable manifest](raw/recovery-executables.json)
pins new hashes and the matching source identity. The interrupted fourth case
restarted in full and completed. During the fifth case, a pre-process eviction
left 1,572,864 resident bytes; the unchanged zero-residency gate stopped that batch.
The final admission policy then restarted both remaining cases with bounded
eviction retries. The fifth case's partial results are excluded in full.

The accepted result therefore comprises three complete cases from the initial
epoch, one from the first recovery batch, and two from the final batch. No paired
comparison crosses epochs. Both interrupted batches remain explicitly incomplete.

Additional retained setup failures include a `/tmp` user-quota failure, a smoke
directory-name refusal, a Cargo argument error during executable construction,
and report-helper corrections. None is hidden as a successful benchmark result.
Tooling and evidence moved to an owned disk directory after the quota failure.
The fixture root has a 24 GiB apparent-size cap; source files use hard links and
successful outputs are removed only after verification and drain.

[The retention manifest](raw/retention-manifest.json) maps original paths to
retained files with SHA-256 hashes, lengths and compression encoding. Large text
is losslessly gzip-compressed and scripts are retained as text. Native binaries,
fixture payloads, compiler caches and reconstructed build trees are excluded.
Rebuild the prior source using its retained base revision, patch and new modules;
the corpus manifest reproduces source selection from the pinned commit. The
retained preparation, calibration, runner and summarizer scripts record commands,
environment allowlists, sampling rules and failure handling.

This was measurement-only work, so output verification and evidence audits replace
an unnecessary full-suite rerun. [AGENTS.md](../../../AGENTS.md) now records the
user's policy: focused checks during experiments, benchmark evidence before broad
validation, and full-suite validation after batching retained improvements at an
integration checkpoint.
