# Wiped reusable raw read buffer

Status: retained after a fixed isolated comparison. The candidate reuses a bounded allocation for uncompressed plaintext fragments, including incompressible raw fallbacks in compression-enabled archives. Each call rereads stored bytes and verifies their existing SHA-256 commitment before the visitor receives them. Internal authenticated padding is still checked. Every return or error wipes the entire allocation, including spare capacity; unwind drops a wipe-on-drop local buffer. Only zeros survive between calls. Encrypted/compressed fragments keep the existing decoder path; no payload is cached and archive bytes, algorithms and worker policy are unchanged.

## Comparison

Control is the post-format compact-cache reader at `f57a12ca`, executable SHA-256 `3b1caba961df69bf5e0a0601e81ae8c41be581f43bbcc26fcca99d75bd1772aa`. Candidate is the exact retained unformatted three-file trial, executable `389c4d29f7ba62ecfd0fa1be6302d5f13b66602a3b807c250b0189c7d70737de`. Parent verified both identities, adapter embedded paths and source snapshots before approving timing. Rust 1.88.0, CPU 2, one worker, identical archive fixtures, fresh processes/handles and warm OS caches; three warmups then 30 measured paired triples per case, with alternating backend order. Byte validation occurs outside timing. Ratios use the existing fixed-seed 10,000-resample paired log bootstrap.

| Fixture | Buffer/control total [95% CI] | Buffer/ZIP total [95% CI] | Buffer median total (ms) |
| --- | --- | --- | --- |
| small512-mixed-compressed-stream | 0.9952 [0.9802, 1.0105] | 1.0338 [1.0107, 1.0563] | 3.115 |
| raw8m-random-stream | 0.9799 [0.9723, 0.9878] | 3.6663 [3.5953, 3.7310] | 5.715 |
| compressed8m-pattern-stream | 0.9920 [0.9770, 1.0096] | 2.4437 [2.3811, 2.5071] | 3.723 |
| raw8m-random-range | 0.9741 [0.9558, 0.9937] | 5.9312 [5.6954, 6.1864] | 0.436 |
| raw64m-random-stream | 0.9852 [0.9818, 0.9880] | 3.6194 [3.5551, 3.6686] | 43.038 |
| raw64m-random-range | 0.9874 [0.9755, 1.0010] | 10.5591 [10.2705, 10.8908] | 0.803 |

Raw full-stream total time improves 2.0% at 8 MiB and 1.5% at 64 MiB; 8 MiB range improves 2.6%. Small-file, compressed-stream and 64 MiB range intervals include no change, so no gain is claimed there. All six cases still fall short of ZIP parity.

All inputs, source and executable hashes remain stable. All six cases have 99 verified samples including warmups, with the expected byte counts and unchanged retained inputs. The [summary](raw/metrics-summary.json), [sample audit](raw/final-audit.json), exact [source patch](raw/source.patch), source snapshots, runners, host context, identity records, raw samples and test logs are retained. Generic harness names `primary`/`packed` identify the compact-cache control; `other`/`tree` identify this buffer candidate. These results do not qualify ZIP parity, public v4 activation, G11, scale, encrypted reads, cold storage or another platform.

## Validation

Strict Clippy passes. Focused release checks pass: eight data-extent tests, four selective-read tests, 19 authenticated-index tests (one manual probe ignored), and three external-source controls. New private tests use the actual fragment encoder and check exact returned bytes, partial read failures, wrong commitments, callback errors/unwind and authenticated nonzero internal padding. They cover 64/192/256 KiB incompressible raw fallbacks and grow/shrink reuse with bounded capacity. Private state requires internal fixtures because public CLI commands do not expose this experimental buffer.

Review found geometric Vec growth could exceed the intended bound; the measured candidate reserves exactly before resizing. Two preliminary compile errors (missing test fixture traits and borrow ordering) are retained separately from final passing logs. Formatting is deferred until after tests and measurement, as required by the commit hook.

The first smoke launcher used a nonexistent fixture directory and failed before comparison; its log is retained under `raw/smoke-fixed/`. Corrected smoke checks under `raw/smoke-fixed-corrected/` all pass. This setup correction produced no timed samples; the fixed measured batch ran once.

Commit `311bbeef` passes post-format strict Clippy, eight data-extent tests and four selective-read tests. Documentation relative-link checks pass, including the current evidence pages. [Final validation logs](postformat/) are retained. These checks validate the formatted code; no post-format benchmark was run.
