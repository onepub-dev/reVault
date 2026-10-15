Plaintext authenticated-index decode borrowing diagnostic — 2026-10-10

Change tested: `authenticated_index::Index::decode` borrows the existing verified input slice for plaintext body decoding, avoiding the duplicate ZeroizingBytes allocation. Encrypted decoding remains owned ZeroizingBytes. This was the sole uncommitted source change at measurement time.

Validation on source commit 65d2d84564a721f9bd0ce305b9cbfd6e88c81ae8 plus the one-file patch:
- Full `file_format::authenticated_index::tests::` filter: 14 passed, 1 ignored (the large resource observation).
- `typed_tree_single_traversal`: 1 passed.
- `typed_credential_open`: 2 passed.
- Strict Clippy with external-source, lib/tests/benches and `-D warnings`: passed.
- Optimized Rust 1.88.0 test binary build using existing target: passed.
The initial filter text `authenticated_index_tests` matched zero tests; the correct module filter above was then run and passed. Logs are under `after/`.

The earlier committed baseline binary remains frozen at `bin/before` (SHA-256 b521c42e9e087646e47163f4bf2e327bf3ed9d811b18ca8556bd288e8e4fff64). The after binary is `bin/after` (SHA-256 fcc6b3c751fe50e1b212e9e8d7a39e707ecd78eb60937e07a0ccb05c79e2a33c). Source patch is `source.patch` (SHA-256 901719cb83c8878b966b369e2d5389b4c96a6284115439445ff745b539e90409); modified Rust source SHA-256 d37f5fb76d2ee9224902acf857414560f66be8fb0c31b477c01274e6828c22cb. Cargo.lock SHA-256 30f035feb33237411a1fe46b2f13656454e7062954223677d2ca569ba7e5a081. Rustc was 1.88.0. The after executable was built before measurements and kept separate from the immutable before binary.

Paired diagnostic protocol, declared before timing:
- Four existing fixtures: 512 x 4 KiB compressed-pattern stream; 8 MiB random raw stream; 8 MiB patterned compressed stream; 64 MiB random raw stream. All are plaintext/default padded.
- Three warmup pairs and 30 measured process pairs per fixture. Each fresh process performed 100 observed opens; pair order alternated before/after. CPU 2 was enforced with taskset. Both variants used the same optimized probe/test harness apart from the plaintext borrowing change.
- Pair metric: arithmetic mean `total_seconds` across the 100 opens in each process. Paired effect is the geometric mean of after/before ratios; 10,000 paired log-ratio bootstrap resamples with the same xorshift64 algorithm/seed as the established runner (`0x7a1286436bde910f`), reporting indices 249 and 9749 for the 95% interval.
- Every process verified fixture content before and after its open loop; the diagnostic checked tree archive hashes. All runs completed; no adaptive reruns. This is a repeated-open diagnostic, not a ZIP comparison or release gate. Observer overhead is included.

Results (lower after/before is faster):

| Fixture | Before mean per-open (ms) | After mean per-open (ms) | After/before ratio [95% CI] | Outcome |
| --- | ---: | ---: | ---: | --- |
| 512 x 4 KiB compressed-pattern | 0.803220 | 0.821615 | 1.02285 [1.01977, 1.02710] | Regression |
| 8 MiB random raw | 0.329592 | 0.335595 | 1.01816 [1.01449, 1.02282] | Regression |
| 8 MiB patterned compressed | 0.255176 | 0.259515 | 1.01696 [1.01212, 1.02206] | Regression |
| 64 MiB random raw | 1.342127 | 1.390735 | 1.03540 [1.02620, 1.05185] | Regression |

All four intervals lie above 1, so this diagnostic shows a repeatable slowdown in the measured open path. Preserve this result; no repeat batch was run to seek a different outcome.

Full summary: `paired/summary.json`. Exact Dart+dcli orchestration script: `run-paired.dart`; complete raw stdout/stderr per process and all 100 stage observations per process are retained in `paired/<case>/`. Hash-before and hash-after JSON both include the two frozen executable hashes, source patch/source/lockfile/script hashes and fixture artifact/source hashes; `stable` is true. Host before/after context is in `paired/host-before.json` and `paired/host-after.json` (CPU2; load average 0.38 1.50 2.97 before, 0.97 1.48 2.89 after). The one-file source patch was uncommitted at measurement time and was subsequently reverted by the parent to commit 65d2d845; current worktree is clean. No formatting or extra source edits were performed by the test runner.
