# Decoder overlapping-copy trial — not promoted

Date: 2026-10-02. Isolated Rust 1.88 experiment against the frozen
`transition88` reader. Main decoder code remains unchanged by this trial.

The hypothesis was to grow copies from an initialized periodic prefix instead
of repeatedly copying offset-sized chunks during overlapping matches. Existing
full-match reservation, ring-buffer primitive, format, window bounds, checksum
calculation and wiping were retained. Trial source, patch and hashes are saved.

Debug workspace tests pass (5), decoder tests pass (10, including 192 overlap
reference cases across wrapped/unwrapped buffers), and three targeted streaming
tests pass. Release reVault format tests pass 241 with 7 ignored; strict Clippy
passes. Checks include malformed/truncated streams and output bounds.

Four cases each ran 30 pairs after three warmups against the original reader,
same archives/protection/units, CPU 2 and exact-byte verification. Input and binary
hashes remained unchanged. Samples are in [measurements/](measurements/), with
the complete [metric table](overlap-paired-results.tsv).

| Case | Trial/original read-only time, paired 95% interval |
| --- | --- |
| Compressed 8 MiB plaintext | 1.021 [0.983, 1.068] |
| Compressed 8 MiB encrypted/signed | 0.997 [0.989, 1.006] |
| Small mixed files | 0.999 [0.989, 1.011] |
| Raw control | 0.998 [0.992, 1.005] |

There is no demonstrated improvement in targeted read work. Some total/open and
RSS metrics move favorably even for the raw control, which never uses this loop;
those changes cannot be attributed to the proposed decoder optimization. The
trial is retained as rejected evidence and is not promoted.

## Setup failures and checksum finding

Automatic approval review initially applied the stale read-only scope and
rejected execution. Verification of actual human approval records resolved it;
one same-command retry was accepted, without a bypass. Missing upstream corpus
files were restored only in the isolated test copy, with source hashes retained.
The standalone release vendor test build timed out at 600 seconds before tests
ran. Debug vendor tests supplied safety coverage; performance binaries remained
release builds. An initial fixture window mismatch was corrected in the test
compressor only; the decoder's 256 KiB bound remained unchanged.

A new negative test discovered that the original full-buffer decoder calculates
and exposes Zstd content checksum values but does not automatically compare them.
The original decoder reproduces the flipped-checksum failure in
[overlap-checksum-baseline.log](overlap-checksum-baseline.log). Negative-test source
and trial failure are preserved. Final copy-equivalence tests check calculated/
stored getter values and actual malformed/truncated input errors; they do not
pretend the missing baseline comparison existed.

reVault stored-fragment authentication remains in force before decoding. The
missing full-buffer comparison is a separate correctness change with separate
validation and measurement, not part of this copy experiment.
