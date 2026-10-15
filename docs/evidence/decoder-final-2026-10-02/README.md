# Final combined decoder qualification

Date: 2026-10-02. Main source frozen after qualification at approximately
16:16 UTC. It combines the accepted frame-checksum correction, output-buffer
wiping, equivalent FSE table arithmetic and state-update inline hint.

## Declared-length failure correction

Independent source review found that both fresh and scoped decoders could return
a successfully decoded vector shorter than the declared length, transferring it
out of `ZeroizingBytes` before the outer public caller rejected the mismatch.
That error then dropped an ordinary vector. Both wrappers now check exact output
length before transferring ownership, so the zeroizing guard retains all output
on this failure. Larger output already failed within the bounded decoder.

The new 131,072-byte regression covers declared lengths one byte too short and
one byte too long in fresh and scoped modes, subsequent valid decoding/reuse,
and scoped scratch release. It checks rejection, not post-free memory contents;
wiping follows the retained guard and existing buffer tests. Ordinary
vendor-owned internal-history wiping remains a separate pre-existing gap.

## Main-worktree validation

- Rust 1.88 release compression tests: **21 passed**.
- Release format tests: **241 passed, 7 ignored**.
- Strict Clippy: **passed**.
- The six changed decoder/consumer source hashes are identical before and after
  qualification and were independently checked again while creating
  `source-files.tar`.

The saved logs and hash manifests accompany the exact six-file source tar.
The combined source, harness, lockfile and toolchain identities are recorded in
`inline-final-combined-source.sha256` and `inline-final-harness-source.sha256`;
the final verification logs check main and isolated copies. `final-batch-scope.txt`
records exact controls and invocation semantics. The complete pinned source
snapshot, including vendor test fixtures and `PATCHES.md`, is preserved separately
as `final-pinned-source.tar` (SHA-256
`46acdcbc467ed02aa8a240d915992623662aebe203876d1df78f1d96ca7d02be`).
The earlier 16,000-cycle archive aging run remains at its separately documented
checkpoint; it was not repeated or relabeled as a final decoder run.

## Performance status

The [isolated inline trial](../decoder-inline-2026-10-02/README.md) predates the
declared-length checks. Its compressed plaintext read gain and compiler/RSS
caveats remain attached to that source.

The [final combined comparison](measurements/) completed before 16:40 UTC, with
30 pairs and three warmups per case against the accepted FSE baseline. Both
executables and ambient metadata use Rust 1.88. All byte checks and source/archive
inventories pass. Ratios below are final/baseline duration, not isolated estimates
of either the inline hint or the new length check.

| Case | Read ratio, paired 95% interval | Total ratio, paired 95% interval |
| --- | --- | --- |
| Compressed 8 MiB plaintext | 0.975 [0.967, 0.982] | 0.956 [0.949, 0.963] |
| Compressed 8 MiB protected | 0.978 [0.972, 0.985] | 0.961 [0.955, 0.968] |
| Small mixed files | 0.971 [0.966, 0.976] | 0.958 [0.950, 0.964] |
| Raw control | 0.990 [0.981, 0.997] | 0.979 [0.970, 0.986] |

No measured read/total regression is observed in this bounded comparison.
Unchanged raw work also becomes faster, so common runtime/build effects limit
causal attribution. The approximately 0.60 process-RSS ratios remain unexplained
and are not claimed as a memory optimization. These results do not close the
large/range ZIP gaps or establish whole-format/release qualification.

## Next bounded correctness step

Qualify erasure of the ordinary decoder's owned plaintext history and literal
buffers, including replacement/reallocation, before making a full wiping claim.
Keep this separate from public archive activation and the fresh C-to-tree export
work. Acceptance should observe storage immediately before release through a
test hook, covering successful decode, checksum/malformed-input failure,
concatenated frames and growth; never inspect freed memory. Retain fresh/scoped
byte equivalence, window limits, the current compression/format regressions and
strict Clippy, then report any cost in a matched pinned comparison. No such
implementation or broader wiping qualification occurred in this session.
