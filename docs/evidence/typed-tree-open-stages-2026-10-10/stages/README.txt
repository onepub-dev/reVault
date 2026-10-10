Tree open stage diagnostic — 2026-10-10

The new test-only `TreeImage::open_observed` path and `REVAULT_TREE_OPEN_DIAGNOSTIC` path were exercised in the issue-310 performance worktree at HEAD 4db3b974977a47eb6f9ec314ef5fbd0df5e2df5c. The diagnostic calls the same validation path as `TreeImage::open`, records stage durations for 100 warm opens per fixture, verifies the archive contents before and after the repeated-open loop, and checks that archive bytes remain unchanged. It excludes drop and verification timing and includes stage-observer overhead. This is diagnostic stage timing only; it is not a ZIP comparison, benchmark qualification, or function-level CPU profile.

Commands run, pinned Rust 1.88.0:
- `CARGO_TARGET_DIR=/tmp/revault-tree-open-stages-20261010/target cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --lib typed_tree_single_traversal -- --nocapture --test-threads=1` — passed (1 test).
- `CARGO_TARGET_DIR=/tmp/revault-tree-open-stages-20261010/target cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --lib typed_credential_open -- --nocapture --test-threads=1` — passed (2 tests).
- `CARGO_TARGET_DIR=/tmp/revault-tree-open-stages-20261010/target cargo +1.88.0 clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings` — passed.
- `CARGO_TARGET_DIR=/tmp/revault-tree-open-stages-20261010/target cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --lib --no-run` — passed; executable copied to `bin/revault_lockbox_api_tests` and preserved separately from the earlier final baseline.

Raw outputs are `test-traversal.log`, `test-credential-open.log`, `clippy.log`, and `build.log`. Diagnostic exact commands are recorded in `commands.txt`; raw run stdout/stderr and parsed 100-observation JSON are `small512.*`, `raw8m.*`, `compressed8m.*`, and `raw64m.*`.

Median durations across 100 opens, in microseconds:

fixture | storage handle | codec | authenticated traversal and reclaimed | typed validation | pack padding | value key | owner payload verification | total
small512 mixed compressed | 17.850 | 0.050 | 560.108 | 231.833 | 20.760 | 0.140 | 0.090 | 833.246
raw8m random | 15.365 | 0.050 | 304.445 | 19.285 | 6.200 | 0.110 | 0.080 | 347.460
compressed8m pattern | 15.455 | 0.050 | 249.139 | 4.471 | 5.941 | 0.130 | 0.080 | 276.149
raw64m random | 17.455 | 0.060 | 1160.861 | 145.722 | 48.541 | 0.140 | 0.090 | 1375.154

All four JSON records state repeats=100 and verified=true. Their tree archive SHA-256 values match the previous frozen final audit. Fixture pre-run hashes from that audit are in `fixture-hashes-before.txt`; post-run hashes and the source file checksum manifest are in `fixture-hashes-after.txt`. The per-case source inventory SHA-256 is unchanged, every source file passed `sha256sum -c` (`source-verify-*.log`), and the diagnostic independently verified the content before and after the 100 opens.

Frozen diagnostic test executable SHA-256: dd18de118ad90c5609820f3a1ae49a72286b297e7bc67e639653a16f43c64dd5. Exact uncommitted source patch: `source.diff`, SHA-256 a08bae5d6294eb1ac3137c56264263af5bfb999d62e77ba556e6549afdedf846. Current modified source file hashes: tree_probe.rs 0ad24904362c00d9878cedaeeed443ce7a100d209e3d01e264f66c2528c1777f; tree_image.rs 2a4e6a9dceaabbbb15adf9257a10319d47201e672227efc9f7a57ff965e63d8b. `git diff --check` passed; worktree contains only those two parent-added uncommitted source files. No formatting or source edits were made by this runner.
