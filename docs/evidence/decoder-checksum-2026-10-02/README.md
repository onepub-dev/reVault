# Full-buffer decoder checksum and failure wiping

Date: 2026-10-02. Promoted locally after separate validation and measurement.
This is a correctness correction, not a claimed decoder speedup.

The [overlap experiment](../decoder-overlap-trial-2026-10-02/README.md) proved that
the original full-buffer decoder calculated Zstd frame checksums without comparing
them. Full-buffer decoding now compares each completed frame's stored/calculated
checksum before advancing to the next concatenated frame or returning success,
with an explicit mismatch error. Checksum-absent valid frames still work.
Streaming/incremental APIs retain their previous separate contract; this does
not claim they delay output until the final checksum is available.

The fresh reVault fallback also used an ordinary vector whose length could reset
on a decode error, leaving partial decoded output unwiped. It now uses a fully
sized `ZeroizingBytes` destination and `decode_all`; on failure the writable
length remains intact until drop wipes it. The existing scoped workspace path
already retained this pattern. No stored-fragment authentication, padding, window
limit or codec format was weakened. The rejected overlap-copy code is absent.

## Validation

- Pinned Rust 1.88 vendor workspace tests: 6 passed, including corrupt first,
  middle and last concatenated frames and valid checksum-absent frames.
- Vendor decoder tests: 9 passed; three targeted streaming tests also pass.
- reVault compression tests: 20 passed, covering both failure paths, bounds,
  scoped/fresh equivalence and scoped scratch cleanup.
- Format regressions: 241 passed, 7 manual probes ignored.
- Strict Clippy: passed.
- Main-worktree compression/format/Clippy reruns pass after promotion; all five
  source hashes match the isolated proof before and after qualification.

Tests assert rejection and scratch release; they do not inspect freed memory.
The fresh fallback's error wipe is supported by the fixed writable-length/
zeroizing-drop implementation and existing buffer tests. Exact five-file source
is in `source-files.tar`; source, lockfile, toolchain and executable proof is in
[checksum-source-proof.json](checksum-source-proof.json). Logs remain alongside it.

Wiping qualification is limited to the fresh output destination and the scoped
workspace arena. A subsequent source review found that the ordinary vendor
decoder's owned `RingBuffer::drop` deallocates without zeroization and owned
`ReusableVec::drop` drops an ordinary vector. Those pre-existing internal-history
lifetimes are not corrected by this patch. Full decoder-memory wiping remains
an explicit qualification gap; the tests do not prove that broader guarantee.

## Separate paired comparison

Four cases each ran 30 pairs/three warmups against the original `transition88`
reader, using identical fixtures, CPU 2 and unchanged byte/authentication checks.
All source/archive/binary identities pass. Full samples are in
[measurements/](measurements/) and the [metric table](checksum-paired-results.tsv).

| Case | Corrected/original total time, paired 95% interval |
| --- | --- |
| Compressed 8 MiB plaintext | 0.981 [0.965, 0.997] |
| Compressed 8 MiB protected | 0.990 [0.982, 0.999] |
| Small mixed files | 0.995 [0.991, 0.999] |
| Raw control | 0.996 [0.990, 1.002] |

No material total regression is observed. Small-file read-only time rises about
0.9% [0.4%, 1.3%] while its total time falls about 0.5%; both are retained.
Open/RSS movement in the unaffected raw control limits causal attribution, so
these numbers are not presented as a checksum speedup. Earlier large/range ZIP
gaps remain. The subsequent [FSE arithmetic experiment](../decoder-fse-2026-10-02/README.md)
retains table entries and the newly enforced checksum behavior.
