Shared-tree open-stage diagnostic — 2026-10-10

Scope: single-version stage timing of the shared publication-tree open path. This is not a ZIP comparison and makes no before/after claim. The only data reads are the four existing fixture archives and source payloads. Each probe process does a full content verification before and after the 100 observed opens; drop and those verifications are outside the measured interval. Observer overhead is included.

Source at run: HEAD 7abbf536414cd644baec037c46bc732a3655c658 with parent-added uncommitted diagnostics in `tree_image.rs`, `publication_anchor/shared/tree.rs`, and `publication_anchor/shared/tree/update/source.rs`. The exact binary patch is `source.patch`, SHA-256 f54aeb141085b182aa96f7133feae45cf1d1178cd46155dc6232d3c6eb6c9add. No formatting or source edits were made by this runner. The earlier rejected plaintext-borrow experiment is absent; its restoration is confirmed by the base source hash 0d2f3dfa60e4ebc7b596fa55253f96a88b70f2e5c9fe9bdecacdbe5f7d7dc21a before this diagnostic patch.

Validation, pinned Rust 1.88.0, existing isolated target `/tmp/revault-tree-open-stages-20261010/target`:
- Full shared tree module filter `file_format::publication_anchor::shared::tree::tests::`: 17 passed, 1 ignored, 637 filtered; includes corruption, copy-loss, ownership/recovery and interruption tests. The suite took 282.63 seconds.
- `typed_tree_single_traversal`: 1 passed.
- `typed_credential_open`: 2 passed.
- Strict core Clippy (`--features external-source --lib --tests --benches -- -D warnings`): passed.
- Optimized release test binary build (`--release --features external-source --lib --no-run`): passed, reusing existing target.

Raw logs: `test-shared-tree.log`, `test-traversal.log`, `test-credential.log`, `clippy.log`, and `build.log`. An initial compile attempt before the parent fixed one omitted no-op callback failed with E0061 at `shared/tree/update/source.rs:34`; the exact captured error is retained in `initial-compile-error.log`. After that parent fix, the complete shared-tree suite passed. A harness-only Dart parse error was fixed before any probe process started; the final Dart+dcli script is `run-detail.dart`.

Diagnostic executable: `bin/tree_open_detail`, SHA-256 d0ea3455b68a541c2e90d9c669d0daef439d28e0d9d6bfda6ecb53b6ffe579b3. Current source hashes at run are listed in `run/hash-before.json`; `run/hash-after.json` says stable=true. Cargo.lock SHA-256 is 30f035feb33237411a1fe46b2f13656454e7062954223677d2ca569ba7e5a081. Four fixture artifacts and all source-file hashes were captured before and after. Each `verified=true` record's tree archive hash matches the frozen fixture identity. Host and load context are in `run/host-before.json` and `run/host-after.json`; every process was pinned to CPU 2.

Median durations across 100 observed opens, microseconds:

| Fixture | Total open | Selected publication | Index walk/decode | Ownership graph | Reclaimed space | Typed validation | Pack padding | Storage handle |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 512 x 4 KiB compressed-pattern | 1141.004 | 105.392 | 525.269 | 2.290 | 106.197 | 360.926 | 24.686 | 11.975 |
| 8 MiB random raw | 329.635 | 85.442 | 120.907 | 11.666 | 74.781 | 19.361 | 6.451 | 6.930 |
| 8 MiB patterned compressed | 252.674 | 83.817 | 71.922 | 0.740 | 76.087 | 4.375 | 5.911 | 6.810 |
| 64 MiB random raw | 1337.027 | 84.291 | 820.733 | 142.808 | 78.031 | 147.887 | 50.411 | 8.710 |

Per-fixture raw JSON in `small512-mixed-compressed.json`, `raw8m-random.json`, `compressed8m-pattern.json`, and `raw64m-random.json` retains all 100 observations and the complete stage list. Raw process stdout/stderr are the adjacent `.stdout` and `.stderr` files. `run/summary.json` provides the concise result. The initial callback compile failure is retained, not hidden.
