# Current native, candidate and public extraction comparison — 2026-10-11

## Completed comparison — 2026-10-12

The fixed component matrix completed 480 accepted samples (400 measured,
80 preparation); the separate public CLI matrix completed 64 (48 measured,
16 preparation). Sources, executable identities, persisted bytes and output
admission/drain gates were verified. [The summary](raw/metrics-summary.json)
retains paired intervals and parity views. These are current-source measurements,
not reuse of historical executable results.

| Component median seconds | A | B | C1 | Cauto | ZIP |
| --- | --- | --- | --- | --- | --- |
| Source 64 MiB | 0.219 | 0.240 | 0.259 | 0.214 | 0.139 |
| Source 1 GiB | 3.399 | 3.316 | 4.078 | 3.733 | 2.181 |
| Random 64 MiB | 0.192 | 0.148 | 0.115 | 0.115 | 0.085 |
| Random 1 GiB | 2.540 | 1.904 | 1.767 | 1.767 | 1.272 |

All archive variants remain slower than ZIP in this component screen. Native B
improves the raw workloads and source 1 GiB against A; Cauto improves compressed
workloads against C1. This does not select an architecture or equate codecs,
security or recovery guarantees. Native component access uses Interactive;
public `extract --to` uses ExtractMany, so the gap is not solely process startup.

The original public source 64 MiB elapsed observations are confounded: its agent
socket pathname was 108 bytes, exceeding the Linux 107-byte pathname limit.
Normal secret-activity registration waited about three seconds for unsuccessful
agent startup. Other case paths fit. Preserve those observations, but do not use
them to qualify extraction timing. Component timings are unaffected. Public RSS
reached roughly 237 MB at 1 GiB and justified the subsequent
[corrected matched cache trial](../native-extract-cache-budget-2026-10-11/README.md),
which preflights real agent roundtrips in short isolated contexts. No security
activity registration was disabled to remove the confound.

The following frozen protocol records the intended comparison; the explicit
qualification above takes precedence over any general timing acceptance claim.


Historical protocol status: reviewed and frozen before large fixture creation and timing; base `f6f44173`.
Builds and tiny untimed smoke checks are separate from measurements. This follows
the retained opt-in adaptive policy; it does not reopen threshold tuning or
select an archive architecture. The continuing user goal stops below 10% usage
remaining or above 99% after reset. Reading at protocol preparation: 95% remaining.

## Workloads and endpoints

Four one-file workloads use the existing pinned source/random corpora: source
Zstd at 64 MiB and 1 GiB, and incompressible random/raw at 64 MiB and 1 GiB.
Retain original corpus hashes plus each exact prefix hash. All archives are
plaintext, unsigned, with default size padding where the format supports it.
Native and candidate Zstd use level 3; ZIP uses its existing Deflate default for
source and Stored for random/raw. This compares the normal format/codec paths,
not identical codecs or identical security/recovery guarantees.

Five component variants run against identical logical bytes:

| Variant | Build and access |
| --- | --- |
| A | Current CLI/library defaults, `issue303-protocol`, no native-block feature |
| B | Same source with explicit `native-block-layout` |
| C1 | Typed-tree candidate, explicit one worker (current default) |
| Cauto | Same candidate fixture, opt-in descriptor-aware workers zero |
| ZIP | Current-source evaluator using the locked Rust ZIP dependency |

Create A/B fixtures through their public CLI binaries with explicit encryption,
signing and compression choices. Persisted bytes must be verified by a separate
public CLI extraction before component timing. The matching evaluator opens the
exact same archive read-only using `Lockbox::open(Unencrypted)` and invokes
`extract_file_to_writer`. Retain native default cache/worker settings; do not add
an extra whole-archive audit or silently bypass normal open work. Report the
observed cost of each API, including its native metadata authentication/open.

All component variants use the same sink: output directory and regular file
creation, writes capped at 64 KiB, flush/close, archive drop, no fsync in the timer.
Native partial writes count only successful bytes. ZIP reads through EOF/CRC.
Independent source/output byte comparison and owned output drain happen after
timing, before cleanup. C uses 64 KiB logical units; capture actual chunk codec
census. Archive bytes, creation time/RSS and setup results are descriptive only;
candidate construction is not a public write benchmark.

## Frozen component schedule and gates

Each workload has four excluded preparatory rounds and twenty measured rounds.
Every round runs all five variants, rotation `(round + caseIndex) % 5`, reader TID
parity `round % 2`. This gives two samples of each variant at every position in
each measured parity stratum: 480 accepted samples, 400 measured and 80 prep.
Use CPU affinity 0–4 for every variant, fresh processes, zero archive fincore
residency after sync/fadvise eviction, and process physical reads at least half
the smaller of two independent cold full-file scans (scan agreement within 10%).
Admission retry limits remain sixteen for reader parity and three for eviction;
preserve every rejection. Per-sample execution timeout is 180 seconds, with a
90-minute component batch wall limit. A limit hit is incomplete evidence, never
a passing or silently resized workload.

Enforce a 24 GiB owned-artifact cap and free-space headroom including duplicated
sources, staging images, archives, binaries and the next output before GiB
fixtures. Preserve historical evidence; do not reclaim unrelated artifacts.

Freeze source, executable, runner, protocol, fixture and calibration hashes before
timing. Builds/tests/setup do not overlap timed runs. Reject incomplete cases,
changed identities, wrong pool activation, failed byte checks or failed physical
read gates. Retain raw records, failure output and exact launch commands.

Report elapsed, throughput, CPU, extraction and whole-process peak RSS, physical
reads, archive sizes and actual candidate worker activation. Native worker/cache
settings are defaults, not claimed observed pool counts. Use 10,000 seeded paired
log-ratio bootstrap replicates on ten adjacent parity-balanced two-round blocks,
plus ten within-parity observations. Compare A/ZIP, B/ZIP, C1/ZIP, Cauto/ZIP,
B/A and Cauto/C1. Report confidence intervals without multiplicity correction.
No architecture admission follows from this four-workload screen. Useful
improvement means an upper 95% elapsed bound below one in aggregate and both
parity strata for the compared workload; uncertain cases remain uncertain.

## Separate public CLI observations

After component timing completes, measure A/B public extraction of the same four
archives through separate CLI processes: two prep plus six measured rounds per
workload/variant, alternating A/B order (64 processes, 48 measured). Use isolated
HOME/config/cache/Vault/agent paths and disable the platform secret store, so no
user configuration or secrets influence the run. Plaintext unsigned options avoid
credential lookup requirements; retain all normal public startup, permission,
path-installation and command-exit work. Do not add benchmark-only CLI switches.

Use cold archive page-cache admission and the same CPU affinity, external wall/
CPU/maximum-RSS observation, separate persisted output byte comparison and output
drain before cleanup. Public CLI has no internal reader-TID admission or shared
inner timer; report these results descriptively without pretending the component
parity/physical-read gates or precise phase attribution apply. GNU time's block
input counts may be retained as corroboration, not substituted for procfs gates.
Use a 180-second process timeout and 30-minute public batch limit. Public output
must be a new destination and cleaned only after independent byte verification.

## Next decision and limits

Use the measured native/component/public gap to select one demonstrated
bottleneck for a bounded optimisation and focused validation. Do not spend this
milestone on further automatic thresholds. Preserve authentication, wiping,
bounded temporary memory and normal recovery/refusal semantics.

The 100 MB total CLI RSS goal is assessed for these cases only, not qualified at
all supported scales. Encrypted/PGP and writes, full public record integration,
million-entry scale, candidate security/recovery and bidirectional release
compatibility remain open. B's historical recovery failures are not erased by
throughput, and the repaired native graph does not qualify C's separate format.

## Raw sample packaging

Per-sample logs and rows are losslessly stored in [sample-evidence.tar.gz](raw/sample-evidence.tar.gz). Every extracted file was compared against [the SHA-256 manifest](raw/sample-evidence.sha256); original local files are also preserved under ignored build storage. Extract the archive from `raw/` before rerunning the summarizer. Identity manifests, completed summaries, frozen protocols and validation logs remain directly readable. Source hashes describe the measured pre-format epoch; commit-hook formatting does not change the tested behavior.
