# Borrowed raw page body — frozen fixed public write trial

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
