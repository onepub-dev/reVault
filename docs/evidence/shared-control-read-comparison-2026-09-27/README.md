# Shared-control first-read comparison

Date: 2026-09-27. Based on `809b6f9a`. Freeze the containing source and executables
before measurement. This is a bounded file-only whole-layout comparison following
[successful metadata-tail lifecycle size checks](../metadata-tail-retirement-2026-09-27/README.md).
It does not select the archive format or qualify the missing public semantics.

## Declared batch

Compare retained packed C against shared-control full-catalogue images after 100
metadata edits and return-inline cycles. ZIP uses the same source bytes. Run the
following finite six cases once, with 30 paired observations after three warmups:

- 512 × 4 KiB mixed bytes, compressed plaintext, whole-file reads.
- The same small corpus, encrypted and owner-signed, whole-file reads.
- 8 MiB seeded incompressible raw plaintext, whole-file reads.
- 8 MiB patterned compressed plaintext, whole-file reads.
- 8 MiB patterned compressed encrypted/signed, whole-file reads.
- The same raw plaintext 8 MiB archive, one 4 KiB range at its midpoint.

Use one pass, CPU 2 affinity, warm OS cache and a fresh process/handle per sample.
Alternate ZIP/C/dense and dense/C/ZIP order. No owned build, fixture creation or
test overlaps timing. Record host load; do not claim cold-I/O or load isolation.
The common Rust runner computes paired log-ratio geometric means and fixed-seed
10,000-resample 95% intervals. Keep every sample and input inventory.

The common resource probe now uses one timing/copy/verification loop for both C
and the dense reader. Open and read CPU/wall time are separate; verification,
archive hashing and source reads occur outside timers. Process peak RSS is
reported alongside its pre-open baseline. It is not the incremental-above-empty
A5 memory qualification. The parent runner is native Rust, avoiding a Python
launcher high-water floor. Every sample separately opens and verifies all source
bytes; range samples additionally verify the requested range.

`compare-existing` admits matching C/dense corpus and protection options; only
ZIP may have reduced protection. It checks all source inventories before and
after sampling and pins archive/public-owner hashes. Dense images have retained
password wrappers while the C fixture has explicit-key access: wrapper validation
remains in dense normal-open time, while password derivation is outside this
explicit-key comparison. ZIP ranges omit full-entry CRC, as in the earlier batch.
Protected-vs-ZIP ratios are descriptive, not equivalent-security A3 claims.

## Admission and interpretation limits

These images still admit at most 1,024 files / 4,096 fragments and a 64 KiB decoded
catalogue. They do not implement directories, links, variables, forms, mirror
ownership, access mutation/overflow, a large metadata hierarchy or a public
migration path. A faster reader does not establish whole-format eligibility.
The batch is not A4's comparison against the same-protection production baseline,
a write/update benchmark or a complete A3/A5 workload matrix. Preserve existing
failed cases and do not extend sampling until a result happens to pass.

The new runner's tests check protection/corpus comparability as well as the
existing reproducible paired interval and chunk-independent corpus generator.
Both C and dense resource paths are smoke-tested with exact-byte verification;
those smoke timings are not included in the measurement batch.

## Frozen results

Source `8a786a83`; [batch manifest](measurements/batch.json) pins every executable
and retained fixture path. Dense images are the verified 100-cycle outputs of
`4b04e382`; C uses the frozen `e23b4d33` executable. All six runs completed 30
pairs after three warmups. Every archive, owner-public file and source inventory
is unchanged at the end. Post-hook tests and strict Clippy pass (attached logs).

Ratios below are dense/control elapsed time, including normal open. Smaller is
better. Intervals are the declared paired 95% bootstrap intervals.

| Case | Dense / ZIP, 95% interval | Dense / C, 95% interval | CPU dense / C | Dense median peak RSS, KiB |
| --- | --- | --- | ---: | ---: |
| Small plaintext | 0.626 [0.601, 0.654] | 0.0266 [0.0258, 0.0274] | 0.0266 | 6,048 |
| Small encrypted/signed | 0.855 [0.841, 0.869] | 0.0238 [0.0234, 0.0242] | 0.0238 | 6,532 |
| 8 MiB raw plaintext | 3.360 [3.296, 3.418] | 0.930 [0.923, 0.939] | 0.930 | 4,552 |
| 8 MiB compressed plaintext | 2.854 [2.728, 2.990] | 0.719 [0.697, 0.743] | 0.719 | 6,256 |
| 8 MiB compressed encrypted/signed | 3.469 [3.312, 3.618] | 0.625 [0.609, 0.643] | 0.625 | 6,372 |
| Raw 4 KiB range | 3.181 [2.884, 3.523] | 0.429 [0.407, 0.462] | 0.430 | 4,572 |

The small plaintext case passes the proposed ZIP parity/10%-faster **subcase**,
while the large plaintext and range cases fail. Protected-vs-ZIP numbers remain
descriptive because their security contracts differ. No full A3 pass or A4
nonregression claim follows. Dense process baselines are about 3.9–4.0 MiB; these
RSS figures are not measurements above an independently opened empty archive.

The whole-layout change fixes the repeated small-file catalogue work: small
plaintext read-only time is 0.0191× C and protected is 0.0165× C. Large raw
read-only time remains 0.969× C even though open is 0.520× C. Large compressed
read-only time is 0.919× C (protected 0.888×), while open falls to 0.418×
(protected 0.395×). This separates the remaining payload-processing cost from
control/catalogue-open costs. CPU ratios closely track elapsed ratios; this batch
is not evidence of a storage-throughput limit.

Retain the large-read failures. A bounded raw/compressed/protected CPU profile is
the next diagnostic for identifying the responsible payload operations before
another implementation change. It will not count as another timing batch or
justify skipping integrity, owner verification, padding or wiping. Full record
semantics, larger catalogue admission, payload mutation/aging and release gates
remain on the canonical plan.
