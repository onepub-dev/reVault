# Borrowed raw page body — rejected result

The fixed trial completed all 112 processes (96 measured, 16 preparation) in
1146.349 seconds. Persisted-byte/type/mode checks, final identities and owned
cleanup passed. The frozen usefulness gate failed: the prespecified compressed
Interactive median RSS ratio was 0.9666, above the required 0.95. Reverted only
the two Cow expressions; page.rs is byte-identical to retained checkpoint
`4957aa5a`. No extra samples, qualification tests or WASM regeneration were run
for the rejected change. The proposed golden-vector test was not applied.

| Source / workflow | Elapsed ratio, 95% interval | Maximum RSS old → new, MB | Median RSS ratio |
| --- | --- | --- | --- |
| Source Zstd / Interactive | 0.9676 [0.9310, 0.9936] | 108.60 → 108.77 | 0.9666 |
| Source Zstd / BulkImport | 0.9804 [0.9669, 0.9940] | 95.24 → 86.68 | 0.9613 |
| Random raw / Interactive | 0.9729 [0.9515, 0.9920] | 83.85 → 87.35 | 1.0583 |
| Random raw / BulkImport | 0.9770 [0.9590, 0.9934] | 72.93 → 75.35 | 1.0178 |

MB means bytes / 1,000,000. All aggregate elapsed intervals indicate improvements,
and all elapsed upper/max-RSS guards pass, but this does not override the frozen
memory requirement. Compressed Interactive still reaches 108.77 MB; raw median
RSS increased 5.8% and 1.8%. The overall write target remains open. These results
describe a rejected experiment, not the current runtime.

Baseline `175400ae` is the retained page-flush implementation; trial `1087a467`
was built from `4957aa5a` plus the two-expression diff. The initial build output
was captured by the tool, not redirected; a second cached build produced the
persistent cargo log. Both invocations are distinguished in the evidence. The
artifact was initially placed under repository-root target, then moved to the
intended rust/target path with the same hash before freeze; no duplicate remains.
The experiment did not start until corrected paths and identities passed review.

See [summary](raw/metrics-summary.json), [completion](raw/paired-complete.json),
[identities](raw/paired-identities.json), [build and rejected source diff](raw/build-new/)
and [frozen protocol](raw/frozen-protocol.md). Read-only inspection launcher
diagnostics are preserved separately; they were outside trial/summary output.
No architecture, release, ZIP/PGP or global performance acceptance is claimed.

All 3,720 benchmark command files are losslessly packaged in
[command-evidence.tar.gz](raw/command-evidence.tar.gz), with every extracted byte
verified against [the manifest](raw/command-evidence.sha256). Original logs remain
in ignored local storage. Build/source hashes and the rejected patch preserve
the measured epoch; the current runtime is restored. No archive/corpus bytes
are committed.

The following original protocol records the pre-result design and decision rule.


The retained page-flush ownership trial leaves compressed single-file Interactive
peak RSS at 108.47 MB, while the other three measured cohorts are below 100 MB.
In encode_page_body_plaintext, the compress=false branch clones object_stream
only to copy it into the final body. Replace this temporary copy with a borrowed
Cow slice; the compressed branch owns the existing encoder output. Header bytes,
encoded length, checksum, padding, encryption and compressed output are unchanged.
Do not combine this with other page/queue/cache changes.

Preserve measured page-flush binary 175400ae as baseline. Freeze 112 fresh public
CLI add processes: the same two pinned 1 GiB sources × Interactive single-file /
BulkImport one-file-directory workflows × old/new × 14 rounds. Exclude the first
two rounds per cohort/variant: 16 preparation, 96 measured. Alternate order by
round + case + workflow parity. Use six adjacent two-round paired log-ratio blocks,
10,000 seeded bootstrap draws and parity views. Seed is 0x6b2175d9 followed by
UTF8 label XOR/multiply 16777619 modulo 2^32; zero becomes 1.

Retain only if ALL four aggregate elapsed upper 95% bounds are ≤1.05, ALL four
maximum RSS ratios are ≤1.05, AND the prespecified source-zstd-1gib /
single-file-interactive median RSS ratio is ≤0.95. No outcome-selected cohort,
threshold change or extra samples. Absolute ≤100 MB measured maxima remain the
separate write-memory target, not implied by the median benefit criterion.

Keep the established public lifecycle: old binary creates fresh empty plaintext
unsigned Zstd3 archives outside timing and independently extracts both variants;
time whole add/commit/drop, verify exact listing, cmp bytes, regular file/size/0644,
and drain archives/outputs outside timing. Pin current trial source/build/binary,
preserved baseline provenance, runner/protocol/analysis/helper and source identities.
CPU 0–4, Auto workers, source zero-resident admission, clean environment and short
owned agents remain unchanged. Enforce finite GNUtime metrics, combined 24 GiB
owned budget including new binary/prior contexts, existing archive/output reserves,
30-minute work cap with 60-second cleanup reserve and bounded per-command timeout.
No builds/tests/runtime edits during timing. Helper-agent RSS is excluded.

If retained, verify raw body bytes against a stable format vector, existing page
size/codec matrices and encrypted/plain roundtrip/integrity tests, plus public
write lifecycle and pressure/abort safety. Rebuild tracked WASM from current source
and verify existing portable lifecycle. No full suite per individual experiment,
small-file/native-layout performance, ZIP/PGP or release claim.
