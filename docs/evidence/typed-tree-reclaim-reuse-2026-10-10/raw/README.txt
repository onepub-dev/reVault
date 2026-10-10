Guarded reclaimed-space buffer reuse diagnostic — 2026-10-10

Candidate under test: two-file working-tree patch in source.patch. The optimized verify_vacant path now allocates one bounded SecureVec with capacity equal to the maximum over eligible chunks of min(span length, 65536), reads chunks with read_at_into while the secure guard is held, checks for zeroes, and drops/wipes on error. No validation/budget/failure checks were removed.

Checks (Rust 1.88.0, release, external-source):
- publication_anchor::shared::ownership::tests:: — 8 passed, including reclaimed chunk edges, short tails, and guard release on failure.
- typed_tree_single_traversal — 1 passed.
- typed_credential_open — 2 passed.
- external_source integration suite — 15 passed (includes short reads, read limits/budget, failures, retry behavior).
- Strict core Clippy, --lib --tests --benches -- -D warnings — passed.
- Optimized test binary built with --no-run.
Raw logs are in logs/.

Frozen executables:
- bin/before: SHA-256 0970b7bf71ff5ea86bbdb159d1df791227da7c2197fd6fa04c8d3cb1da79b903 (commit 866c9aa9bb06f4fe65dd8f33db3d7cbcd4db3ac8).
- bin/after: SHA-256 2fc4e1fdb4074b32315eb06d1558bfd45736dbab9ae0038836dc95980feba601 (same committed base plus source.patch).
- source.patch SHA-256 e5e3d83085a1e8c28384b8c3aaf5490edf06bb77648966dd9548397eb06a1903.
Other current source hashes are recorded in paired/hash-before.json and paired/hash-after.json; frozen inputs and all fixtures remained stable.

Paired repeated-open diagnostic (not ZIP comparison or release qualification): 3 warmup pairs and 30 measured pairs per fixture; before/after order alternated; 100 opens per fresh process; CPU 2 pinned; paired unit is the mean per-open time within each process. The fixed-seed 10,000-resample bootstrap estimates the geometric mean of after/before paired ratios. Every process verified its fixture; zero failures. Do not adaptively rerun.

Fixture | before mean/open ms | after mean/open ms | after/before [95% CI]
512 x 4 KiB patterned-compressed | 0.793785 | 0.775700 | 0.97716 [0.97383, 0.98169]
8 MiB random raw | 0.328176 | 0.301770 | 0.91894 [0.90673, 0.93512]
8 MiB patterned compressed | 0.252928 | 0.228882 | 0.90462 [0.88847, 0.92198]
64 MiB random raw | 1.338895 | 1.319432 | 0.98548 [0.98258, 0.98794]

All intervals are below 1, indicating faster observed total-open time for these four diagnostic cases. Full exact per-process observations, raw stdout/stderr, source/executable/fixture hashes, host context, and bootstrap summaries are under paired/. Script: run-paired.dart. Driver output: driver.log. Initial launcher failures before any samples (Flutter wrapper cache was read-only; later resident-compiler startup and telemetry writes failed in the sandbox) are reconstructed from tool output in launcher-failures.log. The successful run used the existing Dart SDK bin directory first on PATH and CI=true without changing SDK/cache settings. It completed all 132 process pairs with stable hashes and no failures.
