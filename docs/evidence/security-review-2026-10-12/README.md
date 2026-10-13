# Defensive security review — in progress

This review starts at `cc2909a005f4f7065ace3038a1d9459832432402` on the format-4 development branch. It revisits the earlier `a191020a` security checkpoint and subsequent archive, memory, extraction and agent changes. The performance goal remains paused. This is source review and defensive validation, not cryptographic certification, release approval, or exploitation testing.

The user's sensitivity rule is explicit: temporary plaintext requires wiping when its origin or destination is encrypted. Plaintext-only archive buffers do not acquire a new blanket wiping requirement. Existing shared codec scratch can require conservative cleanup where provenance is mixed.

## Confirmed findings and current disposition

| Boundary | Source finding | Status |
| --- | --- | --- |
| Page encoding | Plain `Vec` intermediates and partial object serialization could drop before explicit cleanup on an encrypted-path error. Inner page-compression candidates also outlived outer cleanup coverage. | Sensitivity-aware ownership and checked preallocation implemented; sensitive fallback codec ownership corrected and focused checks pass. Final integration remains pending. |
| Internal extraction buffers | Write-only decoded encrypted content and a partially assembled `read_file` result lacked cleanup on writer/read failure. | Conditional ownership implemented; successful public `Vec` results still transfer ownership. |
| Agent suspend cleanup | Unix and Windows server paths held the key-cache mutex during transport reads and replies, delaying the suspend cache clearer. | Cache critical section narrowed; Unix connect/request budgets and a Windows cancellation watchdog implemented and checked. Windows cancellation is a request, not a guaranteed instantaneous deadline. |
| Agent admission | Unchecked TTL addition could panic; secret read scratch used ordinary stack storage and non-RAII cleanup. | Checked expiry and zeroizing scratch implemented on Unix and Windows; defensive validators pass. |
| Agent process identity | Arbitrary unsigned PIDs were cast to signed Unix PIDs; termination later looked up a reusable PID. | Positive PID validation retained; remote process termination removed. A ready process-local watcher guards secret access and self-termination. Requested unavailable protection fails explicitly; deliberately disabled headless policy remains available. Windows inhibition uses an owned power-request handle, independent of the acquiring thread. |
| Extraction filesystem | Ordinary-mode creation exposed content before final permission restoration; some failure paths left partial files. Path checks followed by pathname operations did not anchor mutable ancestors. | Capability-relative private staging and atomic publication implemented for normal, selected-directory and mirror extraction; focused qualification in progress. Additive file-policy binding exposure is implemented and under qualification. |
| Codec workspace ownership | Borrowed workspace storage exposed an owning mutable Vec facade; later full caller integration demonstrated that after-the-fact growth checks were insufficient. | Replaced with fixed-capacity borrowed storage and pre-mutation admission, explicit capacity propagation, ownership-preserving transfers and poisoned-context refusal after mutation failures. Current ordinary invariants, allocation containment, encrypted CLI lifecycle and default/native API library suites pass. The earlier 0.2.1 package is withdrawn; the exact source-incompatible 0.3.0 replacement has completed its selected standalone/consumer qualification and awaits separate publication approval. Final caller artifact and platform gates remain open. Clear-only routing remains unchanged. |
| Password derivation work | Directory byte limits and fixed KDF parameters did not bound total slot derivations across primary/mirror directories. | Default allowance is 32 KDF attempts per password credential across primary, mirror and backup candidates. Confirmed wrong-password slots are deduplicated; successful unwraps remain available for fallback authentication. Explicit larger API/CLI/binding allowances preserve trusted-archive readability; no archive validity ceiling is introduced. |
| Key working memory | Argon2's allocating convenience API freed password-derived blocks without wiping. Several key wrapping/signing seed locals also had cleanup only on successful paths or no cleanup owner. | Caller-owned wiped Argon2 memory, seed/wrapping-key owners, and cryptographic dependency cleanup features applied; parameters and record formats unchanged. |
| Archive replacement | Temporary archive creation used ambient default permissions, and replacement could lose the existing destination mode/DACL. | Private creation implemented before bytes; replacement/compaction preserve Unix mode or Windows DACL from held source handles before publication. No owner-identity preservation promise is added. Normal lifecycle checks pass. |
| Dependencies | Fresh RustSec audit reports `rustls 0.23.44` affected by RUSTSEC-2026-0285. | Exact compatible `0.23.45` lockfile update applied. The later locked audit examined 423 dependencies with no advisory or warning using database revision `57dcc0f2192e025138907a495c1ac9a7e1e3e123` (2026-10-12). Dependency routing after an approved codec publication will require its own lock/audit check. |

The dependency advisory concerns TLS handshake encryption-level validation, not Lockbox page encryption. The dependency paths include `ureq` through the publishing protocol/tooling and `lettre` through the key server. See the [primary advisory](https://rustsec.org/advisories/RUSTSEC-2026-0285.html).

## Validation so far

`raw/initial-checks` preserves the first exact commands, source hashes, outputs and exit statuses. Vault API `cargo check --tests` passed. Lockbox API's first check found a missing new enum arm in the native file reader; it was corrected, with the failed attempt retained. These checks are preliminary compilation, not security qualification.

The first dependency audit fetched advisory database revision `25a288f68f2d5ebe658b243a334110c7217a2569`, dated `2026-10-12T13:40:46+02:00`, examined 411 dependencies and reported one advisory. It did not modify the lockfile. The subsequent explicit update changed only the rustls version and checksum.

`raw/attempt2` records successful API/Vault test compilation, Windows GNU Vault compilation, and a clean repeat advisory audit. `raw/vault-focused` records 34 passing Vault release library tests, including the new deadline, TTL and process-admission validators. Windows cross-compilation is not Windows runtime validation.

`raw/key-lifetime-checks` records successful API compilation, Windows Vault compilation and seven passing focused API test executions: Argon2 compatibility, hybrid contact wrapping, file-backed credential authority/mirrors, and four signing checks. An initial `key_wrap` filter matched zero tests; the two named contact/credential checks replace that empty selection. Linux Vault compilation caught an explicit socket-to-`OwnedFd` conversion requirement; the source correction was applied and validated in the next batch.

`raw/transport-key-final` records successful Linux API/Vault and Windows GNU Vault checks, 34 passing Vault tests and seven actual key test executions. The literal `key_wrap` filter again selected zero tests; concrete contact and credential tests supply the recorded coverage. Offline resolution added requested cleanup feature dependency edges without package-version upgrades.

`raw/extraction-compile-1` preserves two corrected compile errors and the Windows API environment limitation: `alloca` requires the unavailable `x86_64-w64-mingw32-gcc`. `raw/extraction-compile-2` records passing Linux API/CLI checks but three failures among seven extraction tests. Capability directory handles may be `O_PATH`, so ordinary descriptor chmod returned `EBADF`; the correction uses capability-relative permissions on the held directory. The original failures remain preserved. `raw/extraction-focused-3` subsequently records three private-staging properties, seven extraction tests and three public CLI lifecycle tests passing. Linux API/CLI compilation and Windows GNU API library cross-compilation also pass. Windows test-target compilation still requires the missing C compiler, and no Windows runtime result is claimed. A later portability correction replaces hard-link file publication with the existing exclusive rename primitive; all three private-staging checks pass after that change.

`raw/codec-ownership-compile-1` through `-4` preserve corrected type/fixture preparation failures. Final results are five ownership unit checks and 15 workspace integration checks passing, with one predeclared large-LDM test ignored. `raw/sensitive-codec-checks` records passing API compilation and 13 focused codec/manifest/workspace/staging checks. `raw/sensitive-workspace-integration` records one containment test covering all 22 levels and six ordinary sizes: construction, encoding and drop allocate nothing outside the supplied buffers for admitted workspace sizes, and an independent decoder verifies bytes. This is bounded coverage, not proof for every possible codec input.

`raw/local-secret-activity-checks` and `raw/inhibitor-ownership-checks` each record successful Linux and Windows GNU Vault checks and 35 passing Vault library tests. Tests do not suspend the machine or terminate processes. Runtime Windows and macOS qualification remains pending; cross-compilation alone does not establish platform behavior. Current watcher-status and public headless lifecycle checks pass, including explicit requested-but-unavailable protection refusal and reconfiguration/closure controls. Initial preparation and diagnostic-message failures are retained in `raw/cache-status-headless-checks` through `-4`.

`raw/password-budget-checks-2` records successful default/native API, Vault and CLI compilation plus three ledger checks. `raw/password-budget-lifecycles` records default/native ordinary archive budget checks passing and the initial ReadOnly test-accessor compile correction. `raw/private-archive-budget-checks` then records Linux API and Windows GNU API compilation, private archive creation/replacement, default/native compaction, shared successful backup unwrap, two headless lifecycles and two password-profile lifecycles passing. Sources remained unchanged during each completed batch.

The password limit is **per supplied credential**, not a global CLI limit across locally configured password Profiles. Each Profile credential receives the selected allowance. Authenticated password rotation has a separate existing scope. `open --password-slot-budget ATTEMPTS` is an explicit trusted-archive recovery route; repeated successful KDF attempts count, so the contract is attempts rather than distinct slots.

The exact corrected `zstd-complete 0.2.1` attempt-5 package passed 805 ordinary checks (783 library, 16 workspace and six documentation tests), with seven declared ignores, plus two isolated consumer checks. Its crate SHA-256 is `52692bf396321a0e1322208a02f32fd113b28ccefbbd46fb6b7dc78b46eb5cf9`. That artifact is now withdrawn after caller integration exposed an ownership defect; its earlier qualification is historical, not publication readiness. The workspace still uses the evolving vendor correction, and a replacement artifact requires new review and approval. Earlier package attempts omitted required ordinary fixtures; those failures and exact artifacts are retained. A separately supplied external fuzz-artifact dataset is not part of distributable qualification and its test has an explicit ignored prerequisite. No external crash corpus was obtained or executed. Package qualification and publication approval remain separate gates.

At the earlier focused checkpoint, final binding conformance, generated WASM lifecycle checks, supported-platform runtime CI, dependency audit and broader integration remained pending. The later local tranche below updates those results; platform and full integration gates are not assumed complete. Cleanup claims concern deliberately owned buffers and enabled library cleanup; they do not promise erasure of every compiler-generated stack temporary. No final clean-security or release-readiness conclusion is made.
## Combined release validation checkpoint

The narrow AST-backed private-boundary scanner and selective release-tooling
ports passed all 56 tooling tests. Generator output is reproducible. Separate
documentation-parser regressions pass for TypeScript and C++; comments are not
treated as declarations. Generated JavaScript transport was rebuilt through
the package's `build:transport` recipe after the stale bundle was detected.

The persisted alias ABI lifecycle passed. Current C, C++, JavaScript, Dart, Java and
Python native conformance runs passed; Python was verified with only its
declared FlatBuffers dependency and without NumPy. Initial zero-test selection,
missing tools/dependencies, stale bundle, Python fixture buffer reuse and four
missing result-manifest declarations are preserved in
`raw/security-binding-focused/`. The subsequent `internal check-bindings`
gate passed with 342 Rust mappings, 2,915 documented declarations, 88 result
encodings and 230 ABI declarations / 229 operations. Remaining carrier checks
are not claimed.

Windows owner-only DACL and replacement-preservation tests are implemented and
included in platform CI selectors. Local Windows test compilation is blocked
by the missing MinGW C compiler; no Windows runtime pass is claimed.

The corrected default API library suite completed 711 passed, zero failed and
18 ignored; native-layout completed 718 passed, zero failed and 18 ignored.
Both inventories and runtime source identities were reconciled against raw
libtest output. The default wrapper's postterminal classification failure is
retained separately from the successful test process. These library results
precede the narrowly qualified Vault backup correction described below and do
not imply full package, platform or release qualification. See the
[current release checklist](../release-readiness-2026-10-13/README.md) for exact
package results and outstanding fixture, dependency-routing, artifact, platform
and resource requirements.
### Final local carrier and audit tranche

`combined-alias-batch6` and `combined-alias-batch7` record the final actual
`internal check-bindings` exit 0 after WASM generation. Go completed 225 passing
records, including writable and persisted read-only alias operations, before
its mandatory platform credential-store write failed because the isolated
headless session had no DBus service. No assertion was disabled; this is not a
full Go conformance pass. PHP, Ruby, Lua, .NET, Swift and Kotlin runtime
qualification remain pending because their local toolchains were unavailable.

The portable WASM release build and wasm-bindgen 0.2.128 generation passed.
The first generation command used an incorrect relative executable path and
exited 127; the corrected generation-only command succeeded without rebuilding.
Tracked Rust source and lock hashes were unchanged. The password persistence
test passed, as did unsigned-32-bit password-budget validation, explicit-budget
open/commit/free/reopen byte checks, and Bytes(8192) → invalid-mode refusal → Auto
cache policy with persisted byte verification. All four generated WASM outputs
match the generator output. These portable checks do not claim a filesystem
policy API or an observable cache-stats getter that the portable surface lacks.

The final local advisory audit exited 0 for 423 dependencies against 1,307
advisory entries. Database revision
`57dcc0f2192e025138907a495c1ac9a7e1e3e123`, dated
`2026-10-12T11:06:48-07:00`, and the exact lock hash are preserved. This is a
dependency advisory check, not a proof that every dependency is defect-free.
The full API integration is still running; supported-platform runtime and final
release resource gates remain open.
## Blocking full-integration result — 2026-10-13

The current full Vault library suite passed 35/35 tests. The public CLI package
run exited 101 at its first failing target, `cli_flow`: 67 passed and 11 failed.
Earlier targets recorded 43 passing binary-unit tests with one declared ignore,
one passing agent-flow test with one declared ignore, and 21 passing alias tests.
Later CLI targets were not run. The earlier inventory's zero-ignore claim was
incomplete; actual terminal target summaries above are authoritative.

Several normal CLI operations reported `static workspace vector moved` in the
vendored codec, with allocator corruption or a child SIGSEGV in some failures.
These are confirmed release blockers. Agent startup and signed/unsigned status
failures are also preserved and are not assumed to be cascades without evidence.
No crash or vulnerability reproduction probe was run. Static review identifies
that borrowed workspace storage is represented as a `Vec` and exposed through
mutable Vec access: post-operation identity assertions cannot prevent an earlier
reallocation/deallocation. A replacement ownership boundary is being designed;
increasing estimated capacities alone is insufficient.

The concurrently running default API suite was deliberately interrupted through
its owned child PID after this separate runtime failure was confirmed. It had
reported 288 passes of 729 inventoried tests, no failed result, and no terminal
suite summary. Raw child exit was -15 after 1,448.419 seconds; the runner's generic
`failed` status is an interrupted-run classification here, not an observed API
test failure. No native API run started. Source/lock and binary identities were
unchanged during both captures. See `raw/vault-cli-integration/` and
`raw/full-runtime-integration/api-default/` in the security evidence directory.

The previously tested Zstd 0.2.1 attempt-5 artifact is **not qualified for
publication**: its isolated tests did not establish caller integration safety.
Any pending publication approval is suspended until a corrected replacement
artifact is reviewed and qualified. The published 0.2.0 public `ReusableVec`
exposes mutable Vec behavior, so a safe replacement may require a new dependency
compatibility version. No replacement version or artifact is approved yet.
All original package and integration results remain preserved.

No source checkpoint, platform CI or resource measurements will proceed before
the ownership correction passes focused normal/property validation. Performance
optimization remains paused. This result supersedes earlier pending/pass-only
checkpoint descriptions; it is not a release-ready conclusion.
## Fixed-capacity correction qualification

The replacement storage representation never constructs an owning Vec over borrowed
workspace memory. Capacity is admitted before mutation; errors propagate to the
workspace boundary, which poisons a context after mutation-phase failure. Ordinary
owned allocation paths retain their existing allocation-failure contract. The
published ReusableVec mutable-Vec interface changes, so this requires a new dependency
compatibility line rather than silently replacing the withdrawn 0.2.1 artifact.

Current library, optional-feature library and test-target compilation passed.
A bounded syn audit verifies that the final test-adapter tranche leaves non-test
AST items unchanged relative to the passing production snapshot. This is an explicit
module audit, not a macro-expansion or general Rust reachability proof. An earlier
adapter accidentally touched six fallible statements outside its intended test
scope; compiler evidence, exact correction and subsequent scope audit are retained.

Current-source validation passed five FixedStorage invariants, five workspace
ownership properties and ten ordinary workspace checks. These cover representative
strategies, changing-size reuse, preflight refusal followed by success, caller-buffer
allocation containment and independent decompression. The fresh public CLI lifecycle
then passed 27 steps, including protected create, add/repeat/replace, agent stop,
reopen, extracted-byte comparison, output refusal and persisted removal. Sources
and executables remained unchanged during each run. One initial CLI harness launch
failed before Dart started because its clean PATH omitted Dart; the unchanged
harness passed after correcting launcher PATH. No old crashing input or external
malformed corpus was executed.

Evidence: `raw/codec-test-scope-ast-audit`,
`raw/fixed-storage-invariants-final`,
`raw/codec-fixed-storage-ownership-validation`,
`raw/codec-fixed-storage-ordinary-validation`, and
`raw/codec-fixed-storage-cli-lifecycle`. Full corrected integration, replacement
package qualification, platform runtime evidence and final release gates remain pending.
## Established CLI regression corrections

Three focused public CLI tests now pass on corrected source. The owner-reporting
lifecycle explicitly requests owner signing and separately checks the documented
unsigned password default. The recovery lifecycle expects the current transaction
recovery diagnostic; its test-support interruption remains necessary because public
CLI commands cannot deterministically interrupt persistence. The beget lifecycle
uses the existing short agent-socket helper for both isolated invocations and now
uses public Vault initialization/contact listing and persisted public-key exports.
A byte-preserving installation into a fresh standard Vault directory is documented
because the CLI does not select arbitrary Vault filenames.

Owner reporting passed in 47.60 seconds and recovery in 1.53 seconds. Beget passed
in 16.39 seconds after correcting a new test assertion to the existing JSON Lines
output contract; its initial array-assumption failure is retained. Passed tests
were not repeated for that test-only correction. See `raw/cli-established-regressions`.

The replacement dependency package is proposed as 0.3.0, with exact artifact
SHA-256 `f88cdf477c6aa3c46185ca0d93346d7fb4cba91a9984c8cec69965af279c5987`.
Its packaged targets compile and the declared Rust 1.87 library check passes.
Runtime package qualification is pending. Ordinary test enumeration does not label
ignored cases; a separate `--list --ignored` enumeration records six library and
one workspace ignore. No publication is authorized. Corrected full API integration
is running separately; neither compilation nor focused results establish release readiness.
## Authenticated Vault backup recovery correction

Current package integration found that recovering a content key from a Vault
backup discarded the access directory itself. Format 4 correctly refused normal
opening when neither embedded copy matched the authenticated commit. The fix
retains only the wrapped backup record and accepts it only when archive identity,
generation and digest match the latest authenticated commit root. It does not
select an older publication or weaken header, signature, TOC or extent checks.
Opening does not publish directory repairs: writable recovery marks a normal
transaction dirty, and a later commit publishes it. Read-only recovery does not
modify archive bytes. Owner admission precedes writable cleanup, and Vault cache
publication follows successful opening. Unsigned-recipient write behavior is
preserved; a signed recipient needs the authenticated established owner.

The exact normal validator now covers stale and same-content-key/wrong-identity
backup refusal, unchanged archive/cache on refusal, read-only byte preservation,
wrong-owner refusal and actual-owner commit/reopen. Signed and unsigned Vault
password recovery and unsigned-contact mutation/reopen also pass. All 25 unique
Vault integration cases are covered across the final runtime's full and focused
processes, with duplicates excluded. Earlier fixture errors (transaction boundary,
contact reconstruction and retained Vault handle) and poisoned-mutex cascades are
preserved in the [release validation evidence](../security-review-2026-10-13/raw/).

The broader default/native API library results are 711/0/18 and 718/0/18;
these precede this narrow recovery correction. Separate ordinary API integration
targets passed 74 tests with one declared architecture-transfer ignore, API
doctests 10, page tests 23 plus one doctest, Vault library 35, private Linux secret
service 9 plus two Vault doctests, tooling 58 plus 13 release-candidate tests,
and migration library/integration targets 13. The current release checklist
distinguishes these scopes and pending artifact/platform/resource gates. None
of this authorizes publication or asserts cryptographic certification.
## Final archive-publication qualification

The preexisting private archive creation path depended on hard links. To retain
normal create support on native filesystems without hard links, Linux/Android,
macOS/iOS and Windows now use the reviewed atomic no-replace rename primitive
relative to a held parent directory. Other targets retain their existing
publication behavior. No overwrite or copy fallback is used. The open file stays
locked through publication; after a successful rename, cleanup no longer unlinks
the old temporary name. Private creation and destination permission semantics
are unchanged.

Six public archive locking/non-clobber lifecycles, one private archive permission
test and three private staging tests pass. The initial default-feature compile
failure exposed a stale private backup-parser feature guard and was corrected.
Two helper visibility lints and one test-only assertion lint were corrected;
final API/Vault all-target Clippy passes with denied warnings. All attempts and
source identities remain in [publication evidence](../security-review-2026-10-13/raw/archive-publication-final/).
These Linux results do not establish Windows/macOS runtime qualification. Fresh
native/WASM artifact qualification, platform CI and release resource checks remain
separate gates. The separate package-publication approval and retained signing
fixture action remain on hold; no publication is authorized.
## Final artifact and hosted binding checkpoint

The corrected Linux native carrier builds and packages successfully. Four new
Ruby alias forwarders now resolve through the same selected carrier as allocation
and buffer release, instead of unresolved direct extern calls; output NULL guards
match adjacent wrappers. The hosted WASM facade maps persistent alias operations
to the existing ABI inventory and recognizes the actual read-only Vault class.
The persisted read-only alias lifecycle asserts real WASM dispatch, preventing
silent native-only coverage. No operation admission was weakened or inventory
entry removed to pass.

Fresh WASM direct lifecycles and the complete local hosted-WASM package gate pass.
The native archive and generated WASM identities, failed-attempt classifications,
source hashes and cleanup scope are in the
[final package summary](../security-review-2026-10-13/raw/final-package-artifact/summary.json)
and [WASM summary](../security-review-2026-10-13/raw/final-wasm-artifact/summary.json).
Raw script/log copying was rejected by automatic approval review due possible
synthetic credential content. Curated results and hashes were retained instead;
raw originals were not copied by an alternate route. Windows/macOS and Ruby
runtime validation, registry routing, approved retained fixtures and final
release resource gates remain outstanding. Publication is not authorized.
