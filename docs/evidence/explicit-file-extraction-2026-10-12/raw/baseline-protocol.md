# Explicit single-file CLI extraction — bounded baseline proposal

Measure the exact public branch `lockbox ARCHIVE extract /PATH DESTINATION`.
The `LockboxEntryKind::File` branch in `commands/files.rs` selects ReadMostly;
the `--to` archive/directory branch instead selects ExtractMany. Do not infer
one branch's cost from the other or change the global ReadMostly policy.

Reuse the already-created default-A plaintext one-file 1 GiB source-Zstd and
random-raw archives from the native/public epoch. Verify their pinned archive
and source hashes and discover the exact stored pathname from that manifest;
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
