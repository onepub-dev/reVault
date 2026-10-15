# Page-body lifetime — rejected result

The fixed trial completed all 112 processes (96 measured, 16 preparation) in
1146.681 seconds. Persisted-byte/type/mode checks, final identities and owned
cleanup passed. The frozen usefulness gate failed: compressed Interactive median
RSS increased 1.44%, instead of falling at least 5%. Compressed BulkImport maximum
RSS increased 6.61%, also failing the 5% nonregression guard. Reverted only
`drop(body)`; page.rs is byte-identical to checkpoint `574ebafc`. No extra
samples, qualification tests or WASM regeneration were run for this rejected
change. The conditional golden-vector test was not applied.

| Source / workflow | Elapsed ratio, 95% interval | Maximum RSS old → new, MB | Median RSS ratio |
| --- | --- | --- | --- |
| Source Zstd / Interactive | 0.9707 [0.9371, 1.0035] | 108.52 → 112.73 | 1.0144 |
| Source Zstd / BulkImport | 0.9606 [0.8990, 1.0022] | 96.89 → 103.29 | 1.0772 |
| Random raw / Interactive | 1.0046 [0.9901, 1.0217] | 87.52 → 86.04 | 0.9224 |
| Random raw / BulkImport | 0.9995 [0.9872, 1.0154] | 68.66 → 71.24 | 1.0186 |

MB means bytes / 1,000,000. All aggregate elapsed upper bounds pass the 1.05
guard, but none establishes an aggregate speedup and timing cannot override the
memory gates. Raw Interactive median RSS improves 7.76%; this was not the
prespecified useful workload. Compressed Interactive still reaches 112.73 MB.
These results describe a rejected experiment, not the current runtime. The
retained page-flush improvement and the open overall write target are unchanged.

Baseline `175400ae` is the retained page-flush implementation. Trial `a58b538d`
was built from `574ebafc` plus the one-line lifetime change. The original release
build output was captured directly and records 36.38 seconds. Zeroization stayed
before deallocation; the trial changed only allocation lifetime, not encoded
bytes or protection policy.

See [summary](raw/metrics-summary.json), [completion](raw/paired-complete.json),
[identities](raw/paired-identities.json), [build and rejected source diff](raw/build-new/)
and [frozen protocol](raw/frozen-protocol.md). Read-only inspection launcher
stream-FD diagnostics are preserved separately and occurred outside redirected
trial/summary output. No architecture, release, ZIP/PGP or global performance
acceptance is claimed.

All 3,720 benchmark command files are losslessly packaged in
[command-evidence.tar.gz](raw/command-evidence.tar.gz), with every extracted byte
verified against [the manifest](raw/command-evidence.sha256). Original logs remain
in ignored local storage. Build/source hashes and the rejected patch preserve
the measured epoch; the current runtime is restored. No archive/corpus bytes
are committed.

The following original protocol records the pre-result design and decision rule.


The rejected borrowed-body experiment missed its frozen memory gate. This is a
separate lifetime hypothesis: encode_page_with_format wipes body after producing
stored_body, but retains its allocation through final page allocation. Pinned
zeroize 1.9.0 wipes Vec capacity without deallocating it. Add only drop(body)
immediately after body.zeroize(), before stored-body validation and final-page
allocation. The body has no later reads. Full wipe precedes release; encoded
bytes, checksums, padding, encryption and validation order otherwise stay unchanged.
Do not reintroduce the rejected Cow branch or combine other allocation changes.

Preserve measured retained page-flush binary 175400ae as baseline. Freeze 112
fresh public CLI add processes: the same two pinned 1 GiB sources × Interactive
single-file / BulkImport one-file-directory workflows × old/new × 14 rounds.
Exclude the first two rounds per cohort/variant: 16 preparation, 96 measured.
Alternate order by round + case + workflow parity. Use six adjacent two-round
paired log-ratio blocks, 10,000 seeded bootstrap draws and parity views. Seed is
0x6b2175d9 followed by UTF8 label XOR/multiply 16777619 modulo 2^32; zero becomes 1.

Retain only if ALL four aggregate elapsed upper 95% bounds are ≤1.05, ALL four
maximum RSS ratios are ≤1.05, AND the prespecified source-zstd-1gib /
single-file-interactive median RSS ratio is ≤0.95. No extra samples, alternative
post-result gate or outcome-selected cohort. Absolute ≤100 MB measured maxima
remain a separate whole-process target, not inferred from median benefit.

Keep established public lifecycle and settings: fresh empty plaintext unsigned
Zstd3 archives created by old binary outside timing; complete add/commit/drop
timed; old binary independently lists/extracts both variants; cmp bytes, exact
listing, regular file/size/0644, archive/output drain and final hashes required.
CPU 0–4, Auto workers, 8 MiB single-file target/BulkImport Auto, zero-resident
source admission, clean environment and short owned-agent contexts unchanged.
Pin source/build/binary/runner/protocol/analysis/helper identities. Enforce finite
GNUtime metrics, combined 24 GiB owned budget including rejected/prior contexts
and new binary, archive/output reservations, 30-minute work cap with 60-second
cleanup reserve and existing bounded command timeout. No builds/tests/source
edits during timing. Helper-agent RSS is excluded.

If retained, use existing page size/codec/protection matrices, exact uncompressed
wire vectors, roundtrip/refusal, wiping and pressure/abort tests, plus public CLI
lifecycle. Rebuild tracked WASM from current source and verify portable lifecycle.
No full suite per individual experiment, small-file/native-layout speedup,
ZIP/PGP parity, architecture or release qualification is claimed.
