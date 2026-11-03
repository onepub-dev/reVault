# Core test runtime audit — November 3, 2026

Issue: [#354](https://github.com/onepub-dev/reVault/issues/354).
Baseline source: `d249065dc3ea304af03c2742934d140bd34c9341`.
The initial scope was duplicate-case removal, optimized test-build measurement,
and investigation of inner-loop costs, without splitting or sharding the suite.
The subsequent ownership decision below moves unreleased storage experiments
to their dedicated branch; it does not shard production tests.

## Where the release time went

The [bindings 0.6.2 candidate](https://github.com/onepub-dev/reVault/actions/runs/41411932831)
spent 13,868.10 seconds (3 hours 51 minutes) in the lockbox API unit-test binary:
733 passed, 20 ignored, zero failed. That was approximately 98% of the required
package test-command window. The seven documentation tests took 4.21 seconds;
the integration targets after the unit binary took about 67 seconds in total.
This was principally execution time, not compilation or artifact upload.

Libtest prints a warning after a test has run for 60 seconds. Warning-to-completion
timestamps provide approximate lower bounds, not exact per-test or CPU timings.
These tests ran concurrently; their durations must not be added together.

| Test | Lower-bound wall time | Primary fault cases | Interrupted recovery cases |
| --- | ---: | ---: | ---: |
| Selected-source replacement | 112.6 min | 3,288 | 1,020 |
| Form definition / empty record creation | 97.8 min | 3,168 | 1,164 |
| Form field upgrade | 94.1 min | 2,232 | 672 |
| Form moves | 56.5 min | 1,488 | 504 |
| Form deletion | 42.8 min | 1,704 | 564 |
| Variable mutation / deletion | 41.8 min | 3,432 | 2,724 |
| Variable moves | 20.3 min | 1,512 | 504 |

These seven tests executed 16,824 primary faults and 7,152 interrupted recoveries.
Each primary scenario mutates an image, recovers it, verifies the stored content
and erasure guarantees, then repeats recovery and verification for idempotence.

An important scope distinction: the slow `candidate_files` adapters and their
publication, allocation and recovery machinery are gated by `#[cfg(test)]` in
`rust/revault_lockbox_api/src/file_format/mod.rs`. They are experimental storage
correctness tests, not the shipped archive implementation. The default unit
binary nevertheless runs them during release qualification. This change retains
that coverage; it does not silently exclude experimental correctness tests.

## Genuine duplication and its replacement

`CrashStore` models volatile writes, durable state, torn writes, uncertain syncs,
and truncation failures. The old standard matrix runs three write-prefix lengths
times two persistence flags at every operation index.

| Operation | Relevant parameter | Equivalent combinations removed | Retained cases |
| --- | --- | --- | ---: |
| Write / append | Prefix: 0, 97, full | Writes never inspect `persist_sync` | 3 of 6 |
| Sync | Whether pending writes became durable | Sync never inspects prefix | 2 of 6 |
| Truncate | Whether truncation became durable | Truncate never inspects prefix | 2 of 6 |

The representative `(prefix=97, persist=true)` is retained for every operation,
preserving the existing deterministic interrupted-recovery samples. Those inner
loops use only `persist=true`; they retain every write prefix and one equivalent
prefix for sync/truncate. Short-write prefix aliases are deliberately retained
so sampling does not change.

The successful run now records operation kind, write offset and length, and
truncate length. A replay checks the trace through its selected fault and fails
if that fault was not reached. Small regression tests compare all original and
retained volatile/durable outcomes, including subsequent cleanup, and verify
that trace changes and unreached faults are rejected. No cryptographic mode,
failure index, content assertion, erasure assertion, or idempotence check is
removed.

## Fixtures and other potential costs

The seven large tests already build their immutable baseline once per mode,
outside the inner failure loops. The selected-source baseline contains two tiny
files, two form definitions, three records and a retained variable. Rebuilding
that fixture for each fault is not the problem.

Each scenario still needs an independent volatile/durable state. Copy-on-write
simulation could reduce copying but would need separate proof of power-loss
semantics; it is not part of this change. Salvage verification and normal reader
verification exercise different contracts. Recovery repetition verifies
idempotence. Neither is removed as apparent duplication.

The previous test profile optimized external dependencies at level 3 while
leaving workspace code unoptimized. The selected-source signed modes repeatedly
verify real hybrid Ed25519 / ML-DSA-65 signatures. Generic cryptographic code can
be instantiated in the calling workspace crate, so dependency optimization
alone does not guarantee an optimized call path. Secure-memory access also
uses a global heap lock and page-permission changes. Measurements must establish
the relative costs rather than assuming the bottleneck is fixture allocation.

## Release-cycle performance and fuzz audit

- `release-candidate.yml` calls the Rust correctness workflow; it does not call
  the performance or fuzz workflows.
- `performance.yml` has only `workflow_dispatch`. It owns compression corpus
  observations, 100,000-file mirror indexing, failover load and large-store replay.
- `fuzz.yml` owns bounded mutation-sequence fuzzing on changes, a schedule and
  manual dispatch. It excludes release-candidate branch pushes.
- The Rust workflow's fuzz-harness compile job explicitly excludes the
  `Release candidate` workflow. It compiles harnesses, not fuzz campaigns.
- CPU/RSS probes, extended metadata/overflow aging, thousand-cycle qualification,
  and production-scale compression cases are ignored in ordinary API tests.
- The core cycle does retain bounded regression cases recovered from prior
  fuzz failures, plus deterministic statistics/corpus-generator unit tests.
  These do not run fuzz searches, benchmarks, or timed performance thresholds.
- Required ignored CLI session/compatibility and key-server network tests are
  correctness checks. The network gate explicitly skips heavy failover load
  and large persistent-store replay.

No performance or fuzz workload was found inside the core release cycle, so
none needed moving. Their existing separate workflows remain intact.

## Local measurements and validation

Measurements are recorded after compilation, on the same host and with isolated
timing windows. They are not predictions of GitHub runner wall time. Debug
assertions and integer-overflow checks must remain enabled in optimized tests.

### Unchanged source, before test-case reduction

Host: AMD Ryzen 7 3700X (8 cores / 16 threads), 62 GiB RAM, Linux,
Rust 1.88.0. These targeted measurement binaries enabled `external-source` in
both profiles; the selected tests themselves exercise the shared experimental
tree implementation. These are single local observations, not a statistical
cross-platform guarantee or a measured reduction in complete CI duration.

| Test / profile | Wall | User CPU | System CPU | Result |
| --- | ---: | ---: | ---: | --- |
| Connected replacement, all modes; old O0 workspace | 36.06 s | 31.98 s | 4.07 s | Passed |
| Same unchanged test; O2 workspace, checks enabled | 11.07 s | 4.81 s | 4.26 s | Passed |
| Complete original selected-source fault matrix; O2 | 812.75 s | 183.76 s | 593.37 s | Passed; 3,288 primary + 1,020 recovery cases |

The small unchanged test improved by 3.26x. The optimized large matrix still
spent approximately 73% of elapsed time in kernel CPU and incurred 10,613,230
minor page faults, zero major faults, and a peak RSS of 97,948 KiB. The large O0
matrix was not timed to completion; there is no claimed O0/O2 ratio for it.

A separate `strace -f -c` run of the smaller optimized test recorded **2,217,628
`mprotect` calls**. Only 59 `mmap`, 32 `munmap`, 10 `mlock`, and three `futex` calls
were recorded. The three futex calls can include libtest's parent waiting for
its worker; their accumulated blocked time does not demonstrate lock contention.
Tracing increased wall time to 72.52 seconds, so that duration is not used for
the untraced speed comparison. Host perf counters were unavailable
(`perf_event_paranoid=4`); no host security settings were changed.

The caller-side generic optimization caveat is documented in the
[Cargo profile guide](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides-and-generics).
The retained `[profile.test]` uses `opt-level=2`, `debug-assertions=true` and
`overflow-checks=true`. External dependencies retain their previous level 3.
Production profiles and cryptographic parameters are unchanged.

### Protected fingerprint reservation

The test helper `selected_source::prepare_texts` reserved
`PreparedSecurePages::new(4096)` even when replacing only a few dozen text
segments. That reserves 128 KiB of guarded fingerprints. Both the secure heap's
write access and its secure read scopes protect/unprotect the full allocation
and its canaries, page by page, even for a small append or fingerprint comparison.
This explains a concrete source of protection-call amplification.

An exact-capacity experiment computes the required fingerprint count from the
already-verified layouts' extent counts before preparation. The same helper
still enforces its 4,096-page limit, uses protected memory and checks canaries;
no page-protection operation is suppressed. The other experimental form builders
also reserve 4,096 fingerprints; their allocation design is a separate possible
follow-up, not silently changed here.

Reproduction: build the API library test binary with `cargo +1.88.0 test
-p revault_lockbox_api --lib --features external-source --no-run`, then time its
exact `selected_source_capability_and_connected_replacement_all_modes` or
`selected_source_replacement_guarded_atomic_recovery_faults` test using
`/usr/bin/time -v`, `--exact --nocapture --test-threads=1`. The original O0
profile and O2 override were built into separate target directories before any
timed runs. Exclude compilation from runtime measurements.

The exact-capacity experiment was **reverted**. Three alternating untraced
before/after runs of the smaller all-mode test were:

| Pair | Before | Exact capacity |
| --- | ---: | ---: |
| 1 | 7.50 s | 7.53 s |
| 2 | 7.57 s | 7.42 s |
| 3 | 7.49 s | 7.48 s |
| Mean | 7.52 s | 7.48 s |

All six runs passed. The observed difference is not a demonstrated useful
improvement. The candidate trace recorded 2,182,812 `mprotect` calls: only 34,816
fewer (1.57%). Its traced wall time increased to 83.45 seconds; traced durations
are diagnostic, not performance comparisons. Large protected allocations remain
a possible contributor elsewhere, but changing this one reservation did not
explain the test's overall cost. No changes to secure-memory protections,
allocation policy, fingerprint capacities, or cryptographic implementation are
retained. The temporary signing-timer probe was prepared but never applied or
committed.

### Retained changes: focused validation

The three fault-matrix/trace regression tests passed. The unchanged smaller
all-mode replacement test also passed against the retained changes.

The selected-source exhaustive matrix was then timed alone with one test thread,
using the same O2 profile, checks and `external-source` feature as its original
baseline. All assertions passed, including the new fault-trace checks.

| Metric | Original matrix | Distinct cases |
| --- | ---: | ---: |
| Primary faults | 3,288 | 1,508 |
| Interrupted recoveries | 1,020 | 804 |
| Total executed scenarios | 4,308 | 2,312 |
| Wall time | 812.75 s | 372.63 s |
| User CPU | 183.76 s | 83.60 s |
| System CPU | 593.37 s | 288.97 s |
| Minor page faults | 10,613,230 | 5,800,444 |
| Peak RSS | 97,948 KiB | 99,704 KiB |

This is a 54.15% wall-time reduction (2.18x faster) for that local optimized
test, with 46.33% fewer equivalent scenarios. Do not multiply this ratio by the
smaller test's O0/O2 ratio to predict complete CI time: their CPU/system cost
mixes differ. The retained scope is test-profile optimization and duplicate
fault removal across the seven form/variable tests. The full default-feature
unit binary, integrations and platform checks are assigned to CI for one broad
validation pass, rather than repeating them locally and in CI.

### CI dependency maintenance

The initial branch CI run found an existing stale path dependency in the
independent fuzz workspace: `rust/fuzz/Cargo.lock` still recorded
`revault_secure_memory 0.1.0` after the source package moved to `0.1.1` in #352.
The harness compile job correctly refused to update it under `--locked`.
Only that lock entry was synchronized; unrelated dependency versions did not
change. Locked offline metadata and the exact CI command,
`cargo +1.88.0 check --locked --manifest-path rust/fuzz/Cargo.toml`, then passed
(7.75 seconds for the latter). The separate fuzz campaign had already passed;
its unlocked invocation did not expose the stale checked-in lockfile.
## Subsequent decision: separate experimental storage

The user requested that both experiments B and C live on one branch. Both are
preserved at `d236d32e` on `issue-315-experimental-storage`, including the matrix
deduplication and test-profile improvements measured above. The production
cleanup removes their implementations, dedicated tests, and benchmark targets.
Historical evidence and frozen migration packages remain unchanged.

After removing C, the compiled inventory listed 426 tests. A fresh compile of
the exact preserved `d236d32e` source with the same `external-source` feature
lists 759 tests. The final separated binary lists 393 tests: 366 experimental
tests moved off the production branch, with the unrelated output-reservation
test retained under its new module name. The comparison has 367 old names absent
and precisely one new name (that rename). This source-matched inventory replaces
the initial comparison against an older cached binary.

During B separation, the unrelated output-reservation regression was moved into
`lockbox::output_reservation_tests`. Production file-identity, relocation and
cache-history checks were retained. The cache-history test now exercises two
ordinary decoded pages, preserving its bounded-history and hit-count assertions.
The API inventory guard still rejects exposure of experimental paths and
requires a private declaration when excluded source exists; absent source no
longer requires a fictitious module declaration.

Unicode CI exposed a separate stale baseline pin: `77cfd919` was a format-3
reader despite the step being labelled format 4. The workflow now pins the
published CLI 0.5.0 commit `b086e6ad19938fa337e4e9956b3ad9b06771cc6d`.
The format-5 refusal and unchanged-vault assertions are retained.
### Production integration checkpoint

The retained library suite with `external-source` passed: **388 passed, zero
failed, five ignored**. Test execution took **29.76 seconds** locally; the
command including compilation took 81.25 seconds. This is not a matched-machine
speedup claim against the earlier GitHub runner. The final binary lists 393 tests.
All-targets Clippy initially passed without denying warnings. Review against the
release gate identified three remaining experiment-only helpers
(`seal_prepared_content_secure`, `Storage::append_pair`, and
`AtomicFileReplacement::existing_for_test`); these were removed as well.
They were not pre-existing production warnings: their callers were all in C.

The six private-boundary guard tests passed. The actual read-only binding
inventory check also passed, including 241 ABI declarations and 240 operations
across the generated binding surfaces. No binding API changes were required.

Local Windows test compilation stopped in the `alloca` dependency because
`x86_64-w64-mingw32-gcc` is not installed. It did not reach project code and is
not a Windows pass; Windows CI remains required. File-identity support used by
non-Unix tests has been retained. Frozen migration package source was not changed.
### Published-reader compatibility correction

The pinned published CLI 0.5.0 rejects format 5 with its historical generic
passphrase/damage error (exit 1), not the newer structured exit 14. This is the
known diagnostic limitation documented by #340, not acceptance of format 5.
The baseline test now verifies its exact version, first creates and lists its
own Vault to prove the credential environment works, then checks refusal of the
format-5 Unicode Vault and byte-for-byte non-mutation. The current CLI separately
reopens the Unicode alias and reads the expected stored value.

That focused test passed in 12.63 seconds. The separate current-reader
future-Vault test was strengthened to require exit 14 and the supported version;
it passed in 41.69 seconds with its existing upgrade guidance and unchanged-file
checks. Neither result substitutes for Windows/macOS CI.

Final strict Clippy checks passed with `-D warnings`: core all-targets with
default features, core all-targets with `external-source`, and tooling all-targets.
No warnings remained. Removing the three unused test helpers does not change
the retained test inventory or production code paths.
