# FSE state-update inline hint

Date: 2026-10-02. The isolated change adds only `#[inline]` to
`FSEDecoder::update_state`, preserving all operations and bounds checks. The
baseline contains the accepted checksum and FSE arithmetic changes. This trial
does not contain the subsequent declared-length/output-wiping correction.

## Validation and paired measurements

The isolated trial passes 9 vendor decoder tests, 6 workspace tests, 3 targeted
streaming tests, 20 release compression tests, 241 format tests (7 ignored), and
strict Clippy. Four cases each have 30 paired observations after three warmups;
archives, units, protection and byte/authentication checks are unchanged.

| Case | Inline/baseline read duration, paired 95% interval |
| --- | --- |
| Compressed 8 MiB plaintext | 0.956 [0.946, 0.965] |
| Compressed 8 MiB protected | 0.983 [0.966, 1.003] |
| Small mixed files | 0.995 [0.962, 1.028] |
| Raw control | 0.994 [0.988, 1.001] |

The narrow accepted result is approximately 4.4% lower compressed plaintext
read time. Other read intervals include parity. Total/open time and process RSS
also move in the raw control; those changes are not attributed to this hint.
In particular, the roughly 0.60 RSS ratios across cases are unexplained and are
not presented as a memory optimization.

## Provenance limits

The [read-only audit](inline-provenance-audit.txt) verifies both resource executables and the common runner
were built with Rust 1.88.0, with matching runner source/hash and unchanged
fixtures, invocation parameters and RSS measurement method. The trial's sample
metadata reports ambient Rust 1.94.1 because its `rustc --version` subprocess ran
outside the pinned checkout. This metadata discrepancy does not describe the
compiler that built either executable. Preserve it with the audit; do not
silently rewrite the original observations.

The first smoke invocation omitted the required unit argument and failed setup;
the corrected exact-unit smokes passed before measurement. Both logs remain.
Main-source final qualification and any combined measurement with the independent
declared-length correction are recorded separately; this isolated result does
not establish those later source identities, ZIP parity or release qualification.
