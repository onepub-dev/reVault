# Page body lifetime — frozen fixed public write trial

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
