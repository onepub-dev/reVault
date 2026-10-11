# Encrypted and signed read/write comparison with ZIP

The fixed public CLI comparison completed all **32 accepted operations**:
8 preparation operations excluded and 24 measured operations. Each reported
group has three measured repetitions. Every extracted output was independently
byte-compared with its pinned 1 GiB source; exact file trees, regular-file types,
declared permission modes and cold-input admission passed. Post-run identities
and owned-agent/private-context cleanup passed. These are descriptive medians
and full ranges, not statistical parity or release qualification.

reVault used **ChaCha20-Poly1305 encryption, owner signing and Zstd level 3**,
with normal default padding. ZIP used Info-ZIP `zip -6` and `unzip`, providing
compression and CRC without encryption or owner authentication. The comparison
includes the extra protection but does not claim equivalent security, codec,
recovery or durability semantics.

## Complete public command results

Native write sums **create + open + add** subprocess time and CPU; its RSS is
the maximum subprocess peak, never the sum. Native read is a separate public
extract process. ZIP write/read are separate public zip/unzip processes.
RSS uses decimal MB; input size is 1,073,741,824 bytes (1 GiB).

| Source / operation | reVault seconds, median [range] | ZIP seconds, median [range] | reVault max RSS, MB | ZIP max RSS, MB |
|---|---:|---:|---:|---:|
| Compressible 1 GiB write | 3.82 [3.82–3.97] | 23.08 [22.98–23.28] | 110.39 | 2.86 |
| Compressible 1 GiB read | 4.29 [4.19–4.30] | 4.90 [4.88–4.92] | 86.10 | 3.05 |
| Random 1 GiB write | 8.36 [8.34–8.43] | 24.40 [24.37–24.42] | 91.64 | 2.86 |
| Random 1 GiB read | 3.60 [3.52–3.62] | 5.60 [5.58–7.41] | 83.74 | 2.99 |

reVault had a shorter median in all four groups in this screen, while using
substantially more process memory than Info-ZIP. The overall 100 MB target
remains open: compressed protected write peaked at **110.39 MB**. These three
repetitions do not establish a universal speedup or an inferential bound.

The synthetic Vault and its default profile were initialized outside timing.
The owned agent was already running and a random passphrase was available
noninteractively. Normal per-command Vault opens/password derivation, owner-key
loading and the explicit archive opening remain inside create/open/add timing.
Read follows the verified write and public doctor/list checks with normal
populated session state. This is not a cold-session or separate unlock test.
Process RSS excludes the helper agent and is not total-system memory.

## Output drain, size and CPU

`sync -d` ran separately after each operation. The following totals add its
elapsed time to CLI time. They are output-drained measurements, not a claim of
equal crash durability; byte/tree verification remains outside timing.

| Source / operation | reVault CLI + drain, median seconds | ZIP CLI + drain, median seconds |
|---|---:|---:|
| Compressible 1 GiB write | 3.82 | 23.19 |
| Compressible 1 GiB read | 5.02 | 5.64 |
| Random 1 GiB write | 8.36 | 24.95 |
| Random 1 GiB read | 4.14 | 6.15 |

| Source | reVault archive bytes | ZIP archive bytes | Observed ZIP codec |
|---|---:|---:|---|
| Compressible 1 GiB | 183224704 | 182405087 | Deflate |
| Random 1 GiB | 1074224512 | 1073915836 | Deflate |

| Source / operation | reVault median CPU seconds | ZIP median CPU seconds |
|---|---:|---:|
| Compressible 1 GiB write | 5.86 | 23.07 |
| Compressible 1 GiB read | 3.55 | 4.90 |
| Random 1 GiB write | 7.87 | 24.38 |
| Random 1 GiB read | 2.89 | 5.59 |

Full ranges, exact RSS values, ratios of medians, actual codecs and identities
are in [the audited summary](raw/metrics-summary.json), supported by
[accepted samples](raw/samples.jsonl) and [completion](raw/complete.json).

## Reproducibility and limits

The CLI `0.5.0-dev.1` release-mode development binary has SHA256
`08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`.
It was built at `765e4ac2`; the current `d635f33b` checkpoint adds only
docs/tools/evidence, and runtime/bindings equality was verified before reuse.
[Original build provenance](raw/build-current/) was retained without a rebuild.
Info-ZIP zip 3.0/unzip 6.00 and all helper/script/source identities were frozen.
The [protocol](README.md) remains exactly as frozen, separate from this report.

CPU affinity was 0–4. Native write used normal Auto workers. Native read used
ExtractMany; one file cannot activate its >=256-file parallel dispatcher.
No candidate adaptive-reader setting was enabled. Archive options were checked
with public doctor, membership with public list, and ZIP method with unzip.
Source pages were cold before write; archive pages were cold before read.
This fresh public Info-ZIP screen is distinct from the older Rust ZIP component
benchmark and from the earlier plaintext public comparison. It does not isolate
encryption/signing overhead by dividing results from different runs.

Sources remained mode `0664`. Ordinary native import currently stores `0600`, and
default native extraction restores `0600`; ZIP restores `0664`. Respective modes
were verified under umask `0022`, without changing original sources or pretending
the metadata behavior was equivalent. No claim covers small/many-file inputs,
ZIP+PGP, native-block layout, all formats or a production release.

## Preparation and privacy

The first tiny untimed smoke created the protected archive but add refused with
exit 10 (closed Lockbox). No timed sample or freeze occurred. Public `open`
was added to both smoke and measured write totals, then a fresh tiny smoke
passed. Original script/protocol/analysis and failure statuses remain under
`raw/preparation-v1-*` and the command records. `batch-retry3` names the
preparation revision; it was the **first and only timed batch**, with no
outcome-dependent rerun or extra samples.

Synthetic credentials existed only in process memory/child environments and
owned private Vault storage. Native CLI stdout/stderr were discarded rather
than retained because Vault initialization can emit recovery secrets. Evidence
contains safe argv/exit status and allowlisted observations. Owned agents were
stopped and private Vault/context directories removed after use. No existing
user Vault or credentials were accessed. No runtime changes or qualification
tests were needed for this measurement-only checkpoint.

All 846 command evidence files, including the failed untimed smoke,
are losslessly packaged in [command evidence](raw/command-evidence.tar.gz).
Every extracted byte was checked against [the manifest](raw/command-evidence.sha256);
originals remain in ignored local storage. Native CLI output is represented
only by suppression markers, not secret-bearing recovery output.
