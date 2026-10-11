# Current native, candidate and public extraction comparison — 2026-10-11

Status: protocol reviewed and frozen before large fixture creation and timing; base `f6f44173`.
Builds and tiny untimed smoke checks are separate from measurements. This follows
the retained opt-in adaptive policy; it does not reopen threshold tuning or
select an archive architecture. The continuing user goal stops below 10% usage
remaining or above 99% after reset. Latest reading: 95% remaining.

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
