# Page flush ownership — retained public write result

The fixed 112-process trial completed in 1164.438 seconds: 96 measured and 16
preparation runs. Every independent persisted-byte/type/mode check, final identity
check and owned-agent cleanup passed. All frozen timing and maximum-RSS guards
passed, and both named raw cohorts exceeded the required 5% median RSS reduction.
Retain consuming packer ownership instead of two deep payload clones. No samples
were repeated.

| Source / workflow | Elapsed ratio, 95% interval | Maximum RSS old → new, MB | Median RSS reduction |
| --- | --- | --- | --- |
| Source Zstd / Interactive | 0.9977 [0.9848, 1.0097] | 121.45 → 108.47 | 11.9% |
| Source Zstd / BulkImport | 0.9633 [0.9511, 0.9762] | 110.08 → 97.21 | 13.4% |
| Random raw / Interactive | 0.9567 [0.9487, 0.9668] | 108.61 → 87.70 | 12.9% |
| Random raw / BulkImport | 0.9173 [0.9066, 0.9297] | 94.24 → 75.25 | 25.0% |

MB means bytes / 1,000,000. Three cohorts now have all measured runs below 100 MB;
compressed Interactive still reaches 108.47 MB, so the overall write target is
not complete. Compressed BulkImport and both raw aggregate intervals establish
elapsed improvements. Compressed Interactive does not. Raw Interactive odd-round
upper bound 1.0002 prevents claiming speedup in every stratum; all parity upper
bounds remain below the 1.05 guard.

The preserved baseline is cache8 binary `cccc2f00`; measured trial `175400ae`
was built from checkpoint `7ca719b4` plus the two-file ownership transfer. The
original write/preparation/pressure ordering and post-success reference publication
remain intact. Payloads keep wiping Drop ownership; no new arbitrary retry policy
is introduced. The default-layout writer is measured; no native-layout speedup,
small-file performance, ZIP/PGP parity, architecture or release claim follows.

After retention, two uncalled private helpers were removed and the approved packer
ownership/accounting test was added. These changes do not alter the measured
release path. All nine focused release test executions passed: default/native packer ownership,
pressure commit/abort, injected storage failures, packed deletion survivors,
neighbor reuse, file-writer commit/reopen and two public CLI lifecycle tests.
The default pressure/fault tests exercise the changed legacy flush path; no
arbitrary retry guarantee is inferred from them. Current-source WASM release
build and wasm-bindgen 0.2.128 generation passed. Only the generated .wasm changed;
JS and declarations remain identical. Both portable password and setter
commit/reopen lifecycles passed using cached Node v24.19.0. Initial PATH exit 127
and a harness assertion against a nonexistent cache_stats accessor remain in
separate logs; corrected checks use the actual generated API. Source hashes were
stable across generation, with test addition and dead-helper removal distinguished
from the frozen measured epoch. See [focused logs](raw/focused/) and
[WASM provenance](raw/wasm/notes.txt).
The earlier broad API timeout limitations remain; no full suite is repeated for
this experiment.

See [summary](raw/metrics-summary.json), [completion](raw/paired-complete.json),
[identities](raw/paired-identities.json), [build provenance](raw/build-new/) and
[frozen protocol](raw/frozen-protocol.md). Read-only inspection emitted sandbox
stream-FD diagnostics after completion, outside trial/summary output; the exact
context is retained in [the incident note](raw/inspection-launcher-diagnostic.txt).

All 3,720 benchmark command files are losslessly preserved in
[command-evidence.tar.gz](raw/command-evidence.tar.gz); every extracted byte was
verified against [its manifest](raw/command-evidence.sha256). Originals remain
in ignored local storage. [Validated pre-format sources](raw/preformat-checkpoint-sources.tar.gz),
[hashes](raw/preformat-checkpoint-sources.sha256), and
[diff](raw/preformat-checkpoint.diff) preserve this epoch before hook formatting;
they do not claim post-hook byte identity. No archive/corpus payload is committed.

The following original protocol records the pre-result design and decision rule.


After retained cache8, default CLI single-file add still reaches 122.47 MB for
source Zstd and 108.81 MB for random raw. Source inspection finds two deep
payload copies in legacy FilePageWriter::flush: pending().to_vec(), then cloned
PageObjects. Transfer the operation-local packer's owned payloads into the page
and retain only object IDs and small segment contexts. Reset packer accounting
to its empty four-byte stream prefix while preserving page size.

Page sizing, allocation, durable preparation, write/pressure flushing and final
segment publication retain their existing order. No references are published
until the write/pressure path succeeds. The writer is discarded on error; the
existing public operation/transaction abort and recovery rules remain unchanged.
FileHandle dirty input is not cleared before successful completion, but no new
promise of arbitrary in-place retry after an error is introduced. Payloads retain
their normal wiping Drop ownership. Do not change page encoding, caches, workers,
compression, profile selection or the rejected submission-window policy.

Preserve cache8 binary cccc2f00 as baseline. Freeze 112 fresh processes: the same
two pinned 1 GiB sources × Interactive single-file / BulkImport directory workflows
× old/new × 14 rounds. Exclude the first two rounds per cohort/variant (16 prep,
96 measured). Alternate order by round + case + workflow parity. Use six adjacent
two-round paired log-ratio blocks, 10,000 seeded draws and parity views. Seed is
0x6b2175d9 followed by UTF8 label XOR/multiply 16777619 modulo 2^32; zero becomes 1.

Retain only if ALL four aggregate elapsed upper 95% bounds are ≤1.05, ALL four
maximum RSS ratios are ≤1.05, AND BOTH named random-raw workflow median RSS ratios
are ≤0.95. No extra samples or outcome-dependent threshold adjustment. These are
intermediate criteria; the 100 MB whole-process write target remains separate.

Same complete public add/commit/drop timing and independent public verification
as prior fixed trials: old CLI creates fresh empty plaintext unsigned Zstd3 archives
outside timing and extracts every result, cmp verifies source bytes, exact listing
and file type/size/0644 checks pass, output/archive drained outside timing. Keep
CPU 0–4, Auto workers, zero-resident source admission, clean environment and short
owned agent paths. Pin build/source/binary/runner/protocol/analysis identities,
enforce finite GNUtime resource metrics, 24 GiB combined owned budget with archive
and extraction reservations, and existing 30-minute / 60-second-reserve bounded cleanup.
No builds/tests/source edits during timing. Helper-agent RSS is excluded.

If retained, focused checks cover consuming packer accounting/ownership, packed
neighbor byte preservation, pressure commit/abort/reopen, existing injected I/O
failures across storage operations, handle lifecycle and public CLI writes.
Regenerate tracked WASM for current internal runtime source and verify its existing
portable lifecycle. No per-experiment full-suite rerun or release claim.
