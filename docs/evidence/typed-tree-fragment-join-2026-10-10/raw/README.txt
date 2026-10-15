Ordered fragment staging/join diagnostic — 2026-10-10

Candidate under test: the two-file working-tree patch in source.patch replaces per-fragment BTreeMap insert/remove staging with bounded Vec staging plus a file-ID-to-index map and ordered append/join checks. The new malformed-join regression test corrupts experimental decoder records through test-only direct format manipulation; no public CLI can create those deliberately invalid records. After each malformed in-memory record sequence is refused, the unchanged original archive is independently reopened and its bytes checked. This exception is limited to constructing malformed test fixtures.

Validation (Rust 1.88.0, release, external-source):
- typed_tree_fragment_join — 1 passed. Covers the 9 specified fault classes across 16 modes.
- typed_tree_single_traversal — 1 passed.
- typed_credential_open — 2 passed.
- typed_tree_filesystem_growth_deletion_permissions_and_copy_loss_all_modes — 1 passed, 16 modes.
- Strict core Clippy (`--lib --tests --benches -- -D warnings`) — passed.
- Optimized core test binary built with `--no-run`.
Raw validation logs are in logs/.

Frozen binaries:
- bin/before SHA-256 b6c1b4f1e45e4c73c65babf32e5b3592b9804fdf64ded0c6f70316c2368636ae (commit d6ae8e909b9a51d4e882ec200ef8267b14ea75d7).
- bin/after SHA-256 d2f48af30e0544696efab2adb0ae2956f27d737a0a7784f09387f6974359195e.
- source.patch SHA-256 3bbd64ee8e032bfae4afa8a36ff86bdc5148fdbd39c3afa89f235e582cd8866d.
Candidate source, test, lockfile, executable, fixture, and runner hashes are retained under paired/hash-before.json and paired/hash-after.json. The successful batch reports stable frozen inputs and fixtures.

Paired repeated-open diagnostic only; this is not a ZIP comparison or release qualification. Each fixture ran 3 warmup pairs and 30 measured before/after pairs, alternating order, with 100 opens in each fresh process pinned to CPU 2. The paired unit is mean per-open time within each process. The fixed-seed bootstrap used 10,000 resamples and reports the geometric mean paired after/before ratio with 95% interval. Every process verified its fixture. There were zero failed or incomplete pairs; no adaptive reruns were made.

Fixture | before mean/open ms | after mean/open ms | after/before [95% CI]
512 x 4 KiB patterned-compressed | 0.914173 | 0.822633 | 0.89974 [0.87283, 0.93284]
8 MiB random raw | 0.299599 | 0.279808 | 0.93392 [0.93014, 0.93780]
8 MiB patterned compressed | 0.244970 | 0.230522 | 0.94120 [0.91476, 0.96536]
64 MiB random raw | 1.382691 | 1.277467 | 0.92068 [0.88140, 0.96308]

All four intervals are below 1, showing faster observed open time in these fixed diagnostic cases. Full per-process observations, raw stdout/stderr, source/executable/fixture hashes, host context, and bootstrap summaries are under paired/. Dart+dcli runner: run-paired.dart. Its driver output is driver.log.
