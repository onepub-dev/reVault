# Explicit single-file CLI extraction — retained streaming profile

All eight descriptive baseline and 56 fresh paired processes completed with
persisted bytes, mode 0644, cold-archive admission, identities and owned-session
cleanup verified. Retain the command-local File branch change from ReadMostly
to ExtractMany. The existing native Auto cache targets reduce peak RSS by 67–69%
on these two 1 GiB plaintext workloads; both frozen timing/RSS gates pass.

| Workload | New/old elapsed ratio, 95% interval | Maximum old → new RSS, bytes |
| --- | --- | --- |
| Source Zstd | 0.9861 [0.9764, 0.9953] | 236,036,096 → 77,271,040 |
| Random raw | 0.9784 [0.9687, 0.9883] | 242,716,672 → 74,915,840 |

Median elapsed changed from 3.750 to 3.685 seconds and 3.045 to 2.965 seconds,
respectively. The raw odd-round interval [0.9669, 1.0012] does not establish a
speedup for every stratum. Every new measured process stayed below 100 MB; the
aggregate upper timing intervals meet the predeclared 1.05 bound. No resampling
or threshold changes were made. See [the complete audit](raw/metrics-summary.json).

The CLI still verifies authenticated content and preserves replacement/refusal
semantics. Global ReadMostly, interactive/range reads, explicit cache policies,
WASM, and the reverted many-file worker policy are unchanged. This is a large
explicit full-file result, not encrypted, small-file, native-layout or whole-G11
qualification. Helper-agent RSS is excluded from the process metric.

The exact frozen [paired protocol](raw/paired-protocol.md), baseline protocol and
runner snapshots, pinned old/new optimized build provenance, full sample rows
and identity manifests are retained below. Runtime source was frozen throughout
timing. Focused CLI validation is recorded separately after the measurement.

## Historical frozen experiment description


Measure the exact public branch `lockbox ARCHIVE extract /PATH DESTINATION`.
The `LockboxEntryKind::File` branch in `commands/files.rs` selects ReadMostly;
the `--to` archive/directory branch instead selects ExtractMany. Do not infer
one branch's cost from the other or change the global ReadMostly policy.

Reuse the already-created default-A plaintext one-file 1 GiB source-Zstd and
random-raw archives from the native/public epoch. Verify their pinned archive
and source hashes and discover the exact stored pathname through public recursive JSON listing;
perform an untimed explicit public extraction and independently compare every
persisted byte before admitting timing. Do not rebuild archives or the baseline
CLI. Use the restored valid baseline CLI binary `b0fb3237…` and its original
credential-checkpoint build provenance; the later worker-budget trial was reverted.

First run eight descriptive whole-process baselines: one excluded preparation
and three measured processes for each of the two 1 GiB cases. Preserve outputs
only until independent `cmp`, exact regular-file/size checks, expected mode 0644
under controlled umask 0022 and draining pass.
Use a fresh destination, clean isolated environment, short owned agents with
actual session roundtrip/cleanup, CPU 0–4 and zero-resident archive admission.
Report full CLI time/CPU/RSS and exclude helper-agent RSS explicitly. Reuse the
existing 24 GiB combined owned budget; no new corpus/archive duplication.

Only if this baseline establishes excess memory consistent with the selected
profile, propose a minimal command-local streaming-profile change; the paired
intervention supplies the stronger causal evidence. Preserve
range/interactive API semantics, permissions and refusal behavior. Before timing
such a trial, freeze 56 fresh processes with two preparation and twelve measured
rounds per case/variant, alternating order and the existing six-block seeded
paired analysis. Retain only if every new measured RSS is below 100 MB and each
aggregate elapsed-ratio upper 95% bound is at most 1.05, with all output/identity
checks passing. After retention, run focused normal output/refusal checks.
This experiment qualifies large full-file extraction on these
two workloads, not small-file or random-range performance. No runtime edit until
the baseline, and no full API rerun for each experiment.

## Completed baseline and frozen trial

All eight descriptive processes passed persisted-byte/mode, cold-archive,
identity and owned-session cleanup gates. The actual stored path was obtained
from public recursive JSON listing: `/file-000000.bin` in both archives. Source
Zstd measured median elapsed was 3.76 seconds with maximum RSS 235,876,352 bytes;
random raw was 3.02 seconds with maximum RSS 244,584,448 bytes.

The trial changes only the CLI explicit File branch from ReadMostly to the
existing ExtractMany streaming profile. Directory/whole-archive extraction,
global ReadMostly API behavior, replacement/refusal logic and all authentication
remain unchanged. This does not restore the rejected many-file worker policy.
The new optimized CLI build and one-file diff are in `raw/build-new/`; old and
new binaries are retained separately. The conditional 56-process protocol and
its predeclared gates above remain unchanged. No builds/tests overlap timing.

The paired analysis uses six adjacent two-round blocks of old/new log elapsed
ratios, 10,000 seeded bootstrap draws and separate even/odd-round views. Seeds
start at `0x6b2175d9`, then fold the UTF-8 case/comparison label with XOR and
multiplication by 16777619 modulo 2^32 (zero becomes one). The analysis script
and shared statistics helper are frozen with the runner before timing.

## Focused validation and preservation

Release CLI tests `add_accepts_jobs_option_for_large_files` and
`read_only_archive_supports_cli_listing_and_byte_exact_extraction` passed. The
latter now checks positional output, refusal preserving a distinct destination
sentinel, explicit overwrite restoring bytes, and unchanged read-only archive
bytes. Its strengthened final rerun passed; the earlier version also passed.
The initial wrong-working-directory Cargo invocation failed before tests and is
preserved separately. No full-suite rerun was performed for this experiment;
the earlier integration timeout limitations remain.

Command logs are losslessly packaged in [command-evidence.tar.gz](raw/command-evidence.tar.gz)
with an [extracted-byte SHA-256 manifest](raw/command-evidence.sha256). Originals
remain in ignored local build storage. Focused logs and sample rows remain directly
readable. The [pre-format source snapshot](raw/validation/pre-format-source.tar.gz)
and hashes preserve the checked epoch; the commit hook may subsequently apply
formatting-only changes. Baseline and paired protocol snapshots preserve the
pre-result text. No payload or archive bytes are included in this checkpoint.
