# Page flush ownership — frozen fixed public write trial

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
