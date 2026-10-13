# Release preparation — 2026-10-13

Status: in progress; no release-ready claim or publication authorization.

Brett requested preparation for release after accepting the current performance checkpoint. Performance experiments remain paused. The working scope is the current default format-4 implementation on `issue-310-zip-read-performance`, for compatibility line `0.5.x`, under the release-preparation instruction. Alternative archive layouts remain experimental and are not activated by this work. Published baseline checked on GitHub: CLI `0.4.1` and bindings `0.4.1`.

The candidate is not yet frozen: `cc2909a0` is the last committed performance checkpoint, followed by uncommitted security remediation. Final evidence must identify the actual committed candidate, features, toolchain and artifacts. Earlier checks cannot qualify later edits. Preserve the unrelated dirty main checkout.

## Required work and evidence

| Requirement | Current disposition | Completion evidence |
| --- | --- | --- |
| Security remediation | Active review covers private extraction, agent transport/process identity, secret lifetimes, codec ownership and password derivation budgets. | Resolve confirmed findings, review changes, run focused validators and ordinary persisted lifecycles; retain limitations and failed attempts. |
| Default runtime integration | Current default/native library suites and ordinary package targets pass at recorded epochs; subsequent narrow fixes have focused qualification. | Complete current-source API, Vault, CLI, migration and relevant feature suites with exact counts and ignored-test accounting. |
| Binding consistency | Additive policy/budget/alias mappings, parity and recorded carrier/package lifecycles pass; platform and final dependency routing remain open. | Binding declaration/parity checks plus persisted CLI-to-binding and binding-to-CLI round trips. |
| Migration fixtures | Source inventory lacks archive format 4 and Vault structure 3/container 4, both required by retained fixture assertions. | Add only the new generators and immutable fixtures with producing source/version provenance; run the entire retained matrix and subsequent mutation checks. Never regenerate historical fixtures to pass. |
| Compatibility contract | No stable 0.5 release appears in the checked release list; 0.4 is format 3. | Validate migration from supported historical formats, documented read support, matching CLI/binding archive exchange and persisted Vault compatibility. Do not claim old 0.4 readers can read format 4. |
| Compression dependency packaging | Workspace uses a patched vendored `zstd-complete`; patches do not propagate to independently published Cargo consumers. | Establish corrected distributable dependency route and validate isolated packaged consumers. Local workspace success is insufficient. Dependency publication requires separate authorization. |
| Supported platforms | Linux focused tests and Windows library cross-checks provide partial evidence only. | Actual supported Windows/macOS runtime CI, native package/install lifecycles, permission/agent tests and portable WASM checks on the final candidate. Address platform behavior regressions before readiness. |
| Dependencies and quality | Rustls advisory update passed an earlier audit; later dependencies/features changed. | Final locked dependency audit, required lint/static checks, formatting through tracked hook and appropriate post-format verification. |
| Candidate preparation | No final source/artifact identity exists yet. | Commit and push isolated work, run reviewed prepare-only release workflow, retain package hashes and install/conformance results. Do not invoke publish, create release tags or deploy. |
| Documentation | Existing plan contains historical failures and experimental architecture gates. | Reconcile current release scope, format contract, migration/support matrix, release notes and known limitations against evidence. Preserve historical reports unchanged. |

## Performance claims

The accepted checkpoint is a limited four-cohort plaintext extraction result, not a universal speed or 100 MB guarantee. The recent security changes are not timed by that report. Release preparation does not restart optimization, reinterpret old gates or advertise unmeasured speedups. If a mandatory release resource check remains unmet, report it explicitly rather than weakening the requirement.

## Publication boundary

Local release builds, tests and nonpublishing candidate preparation are authorized. Publishing packages, release tags or production deployments requires explicit user authorization after a concrete candidate is ready to review.

The active remediation record is the [security review](../security-review-2026-10-12/README.md); the [archive-v4 plan](../../archive_v4_plan.md) retains historical performance and integration evidence. Release preparation must selectively port the reviewed release-infrastructure corrections from main without replacing format-4 runtime sources or touching unrelated main-checkout work.
## Current preparation checkpoint

The codec ownership correction is implemented and focused-qualified. Borrowed
workspace storage is now fixed-capacity with pre-mutation admission and explicit
failure propagation. Current-source compilation, 20 ordinary storage/ownership/
workspace checks and a fresh protected public CLI lifecycle passed. The corrected
full default API library suite passed: 711 passed, zero failed and 18 explicitly ignored
(729 complete inventory), with a normal test-process exit 0 in 3703.93 seconds.
All 568 frozen source paths and the executable identity remained unchanged.
The reporting wrapper subsequently exited 255: its ignored-reason parser omitted
18 result lines and its postterminal manifest callback used a stale path. Raw
libtest output and an independent identity/count reconciliation substantiate the
pass; both the original wrapper failure and corrective classification are retained
in [default API evidence](../security-review-2026-10-13/raw/full-runtime-api-recheck/).
The native-layout API library suite also passed: 718 passed, zero failed and
18 ignored, with all 736 unique names matching inventory and the terminal summary.
The test process exited 0 in 3779.99 seconds; source and binary identities remained
unchanged. See [native library evidence](../security-review-2026-10-13/raw/full-runtime-api-native-recheck/).
These are library harnesses, not complete package suites. The subsequent ordinary
API integration targets passed 74 tests, with one explicitly ignored architecture
transfer test; external-source (15) and native-layout (1) targets ran with their
required features enabled. API doctests passed 10/10, page library tests 23/23 and
page doctests 1/1. The recovery integration initially counted only files where the
report deliberately includes directories; its corrected exact path/type/content
assertions passed 9/9, and the focused committed-membership test passed. Two
unnecessary ordinary copies of the recovery content key were removed in favor
of a synchronous borrow; the original failure and source delta are retained.

Current Vault library tests passed 35/35, private Linux Secret Service integration
9/9 and Vault doctests 2/2. Tooling unit tests passed 58/58 and release-candidate
tests 13/13. Migration library tests (2) and the selected ordinary archive-v4,
migration-flow, padding and Vault-v3 integrations (11) passed; there were no
migration doctests. No retained-fixture writer or external corpus was executed.
Commands, inventories, source identities and terminal outputs are in
[package integration evidence](../security-review-2026-10-13/raw/release-integration-final/).

The remaining Vault integration target exposed a real backup-recovery gap:
recovering a content key discarded the backed-up access directory, while v4
correctly requires that directory to match the latest authenticated commit.
The initial result was 21 passed and two failures: one recovery error and a
subsequent poisoned test-environment mutex. A narrow correction carries the
wrapped directory into authenticated opening, requires exact identity,
generation and digest, and stages repair for a later normal commit. Cache
publication and writable cleanup follow successful authentication and owner
admission. Normal signed/unsigned recovery and unsigned-recipient lifecycles
passed in a first 24/1 focused run; its remaining new fixture incorrectly combined
two recipient mutations without an intervening commit. A later compile-only
test error used a nonexistent contact-key Clone method. A further fixture held
its Vault's exclusive lock during an independent open; that first assertion
failure caused seven subsequent poisoned-mutex failures. These attempts remain
preserved. After releasing the fixture handle correctly, the exact primary
validator passed, as did eight fresh-process follow-ups (seven formerly poisoned
cases and an explicit unsigned-contact repeat). Together with 17 unaffected
passes at identical runtime, all 25 unique Vault integration cases are covered;
the repeated unsigned case is not double-counted. Refusals preserve archive bytes
and cache state; matching-owner repair commits and reopens without the backup.
See [final focused recovery evidence](../security-review-2026-10-13/raw/vault-backup-owner-admission-final/).
Earlier full-library passes predate this narrow fix; they are not represented as
executions of its final source. Three additional focused API checks passed for
password slots, primary-directory fallback and successful unwrap reuse under a
shared password budget. Required API/Vault all-target Clippy now passes with
warnings denied; vendored dependency warnings remain recorded separately.

Final default-feature checking caught an obsolete feature guard on the private
backup parser; its availability now matches the unconditional authenticated-open
caller. Archive creation also replaces the preexisting hard-link requirement
with atomic exclusive rename on Linux, Android, macOS, iOS and Windows, using
the existing held-parent publication primitive. Other targets retain their prior
publication behavior. No overwrite or copy fallback was introduced. Successful
rename disarms temporary-name cleanup, so a reused old name is never removed.
Ordinary public archive locking/non-clobber tests passed 6/6, private archive
permissions 1/1 and private extraction staging 3/3. Windows/macOS runtime evidence
is still required. Original compile/lint failures and the final pass are preserved
in [publication qualification](../security-review-2026-10-13/raw/archive-publication-final/).
Fresh native/WASM artifact and local package validation now pass against this
corrected pre-hook runtime. The generated WASM SHA-256 is
`9191f2ec8a27e63f20c5098df811dd380a2646abaf5e68df0221abd3fecfadf5`;
its password, budget and cache-setting lifecycles pass and all four generated
binding artifacts are installed exactly. A real Linux native archive at local
version `0.5.0-dev.1` was built, staged and assembled into npm packages. Hosted
WASM package conformance passes, including persisted aliases and read-only alias
dispatch through WASM. The archive SHA-256 is
`854c65adb5b530ca2a937fac1396740c4d1a0e0c1b49575c83c94d8feabe4d44`.

This gate found two binding defects: four Ruby alias shim functions bypassed
the selected-library resolver, and the hosted JS dispatcher omitted the four
persistent-Vault alias names and the renamed read-only handle. Both now follow
the existing carrier/dispatcher contracts. The Rust ABI and WASM binary did not
change for these facade repairs. Failed link, dispatcher and launcher preparation
attempts remain distinguished from the passing final package gate. Ruby itself
is unavailable locally; shim compilation does not replace its runtime CI.
See [WASM qualification](../security-review-2026-10-13/raw/final-wasm-artifact/summary.json)
and [package qualification](../security-review-2026-10-13/raw/final-package-artifact/summary.json).
Only curated result descriptions and source/artifact hashes are retained for this
last artifact tranche. Automatic approval review rejected copying raw scripts
and logs because limited marker checks did not establish absence of synthetic
credentials; no such raw copy executed. The original owned evidence remains
outside the repository. This limitation is explicit, not a claim of transcript
erasure. No publication or full-platform/final-candidate acceptance is implied.
Retained-fixture generation and its signed-v4 matrix remain permission-held.
Four scheduled compression regression checks remain for quiet-host validation;
fixture-maintenance writers and external crash corpora are not enabled.

Both required Clippy scopes passed in the isolated Rust 1.88 snapshot with
warnings denied: production workspace and bindings/tooling/WASM wrappers.
The final CLI all-target lint also passed after fixture cleanup and logging fixes.
Workspace rustdoc passed with warnings denied and all features, documenting the
two historical duplicate-name readers separately without dropping any selected
package. Original output-collision failures remain recorded. The standalone Rust
facade documentation command is blocked before compilation by its stale lockfile
under `--locked`; corrected registry dependency routing remains a release blocker.
The reviewed 17 source files and workflow were transferred byte-for-byte only
after native-library completion, with exact preimage checks and preserved
[consolidation evidence](../security-review-2026-10-13/raw/post-native-consolidation/).
Focused bootstrap (1), encoder (4), variable (1) and alias ABI (1) checks passed;
test-module preservation was independently checked. No full library repeat is
claimed or required solely for those syntax, test and module-order changes.

The earlier Vault library capture omitted the new local secret-activity module;
the complete current-source 35/35 repeat above supersedes that incomplete source
inventory without erasing its evidence. The CLI main
lifecycle target passed 78/78. Initial CLI setup failures remain recorded: obsolete
form selectors, long agent paths, explicit nonexistent exporter overrides and
assumed exporter locations. Corrected compaction and completion each passed, and
aliases passed 21/21. The corrected migration target passed all 11 cases in the
pinned isolated build. Its final cleanup-guard run passed 11/11 in 194.29 seconds,
with 17 owned-agent namespaces and 17 confirmed stop-and-cleanup successes.
Initial isolated add-syntax and mixed-compiler preparation failures remain
separate evidence, not product regressions or successful qualification.

Two earlier local-loopback publication tests printed ephemeral synthetic profile
backup keys and verification tokens through their test helper. Three task-owned
raw logs were replaced with sanitized summaries; separate exit-status files
remain. The prior tool transcript cannot be recalled. No user credentials were
involved, and the synthetic material must not be reused. Original raw hashes and
independently retained body counts are unavailable and are not reconstructed.
The corrected helper emits only fixed operation/status diagnostics and output
lengths. Fresh guarded runs executed both exact ignored bodies successfully:
two-server 1/1 in 5.97 seconds and failover 1/1 in 3.36 seconds, each with one
confirmed public agent stop and fixture cleanup, no self-skip and no cleanup
failure. A zero-execution invocation is preserved separately. Migration assertion
failure messages now likewise suppress arbitrary captured output. The scoped
marker audit includes actual Vault backup labels; it does not prove secret
absence or transcript erasure. Read-only process inventory found no live process
matching the exact isolated CLI binary; historical worktree agent identity was
not reconstructed from missing binary hashes.

Fresh native ABI compilation passes. Current native carrier checks passed C (197),
C++ (189), JavaScript (182), Python (287), Dart (181), Java (182) and Go (229) reported
checks. Go used an owned isolated Secret Service fixture; private fixture state was
removed afterward. Those carrier runs predate the final backup/publication
corrections; the new native archive is qualified by the final hosted WASM package
lifecycle above. These Linux results do not establish Windows/macOS runtime
qualification. Evidence is under
[current validation](../security-review-2026-10-13/raw/).

An isolated current-source WASM release build and binding generation passed, as
did ordinary password, explicit password-budget validation and cache-setting
commit/reopen lifecycles. That earlier capture predates the final backup/publication corrections and is
superseded by the fresh artifact/lifecycle and package results above. Tracked
generated artifacts now match that completed generator output. The initial manifest capture used the wrong working directory
and is retained as invalid; an already-valid pre-API runtime manifest verifies the
source remained unchanged. The replacement manifest is explicitly retrospective.

A freshly fetched Cargo advisory audit passed for the current lockfile: 423
dependencies, zero advisories and no warnings. The database revision is
`57dcc0f2192e025138907a495c1ac9a7e1e3e123`, dated
`2026-10-12T20:06:48+02:00`. See the
[fetch and audit evidence](../security-review-2026-10-13/raw/final-cargo-audit-fetch/).
Any later dependency-routing change requires its own applicable audit.

The old Zstd 0.2.1 proposal is withdrawn. The replacement is a source-incompatible
0.3.0 proposal with public API notes included in its package. Selected packaged
qualification passed 777 library, 14 workspace and six documentation tests, with
explicit exclusions recorded. A mismatched ordinary fixture assertion was corrected
and passed separately; exact-package consumers passed. Final prose-only packaging
preserves those runtime/test/dependency identities. Rust 1.87 library compilation
passes. Final artifact SHA-256 is
`acce3e08f5aa28b5dc1738f46c3a8a65e0daa28dca2dae579cc48bc398958e9d`.
This is not an unfiltered package-suite or complete caller-integration claim, and
no publication is authorized.

The retained v4 fixture still needs permission for durable, clearly public
synthetic signing material. Automatic approval review rejected that action; it
remains on hold. Historical fixtures and their hashes remain unchanged. Do not
substitute another key-storage route. Final migration, platform CI, dependency
routing and release resource gates remain outstanding. Performance
optimization stays paused; no release or deployment is authorized.

## Mandatory final resource verification inventory

P5 requires verification on the final release candidate, not extrapolation from
accepted optimization trials. Run only after builds and tests finish, with a
quiet host, frozen source/artifact identities and a predefined protocol. A
failed gate remains a release blocker; performance optimization stays paused.

| Contract or existing evidence | Required interpretation |
| --- | --- |
| G11 plaintext read/write versus ZIP; protected read/write versus PGP; 100 MB total CLI RSS | Still requires current-candidate qualification. Compression-only ZIP is not the protected PGP control, and helper-excluded process RSS is not total CLI memory. |
| Realistic primary corpus | Disk-backed 1 GiB single-file and 10,000 files of 64 KiB (625 MiB), including build and independent reopen/extraction; preserve compression/security distinctions. |
| Primary comparison statistics | The plan specifies 30 paired observations plus three warm-up pairs, alternating order, paired log ratios and 10,000 fixed-seed bootstrap draws; A3 upper 95% bound at most 1.00 and protected A4 guard at most 1.05. Freeze exact applicable controls before execution. |
| Existing public ZIP and protected-ZIP screens | Three measured repetitions per group are descriptive only. Protected ZIP comparisons deliberately have unequal protection; neither establishes the PGP gate. |
| Existing helper-memory and many-file screens | Eight and 16 descriptive operations respectively; earlier helper inventory limitations remain explicit. They predate security changes and do not qualify this candidate. |
| Accepted four-cohort extraction experiment | Its 112 operations use 12 measured pairs per cohort and retention-specific gates. It does not replace the final ZIP/PGP or complete workload matrix. |
| Range, mutation, aging and scale requirements | Inventory applicable existing checks from the plan, including authenticated range behavior and no-change/reclaim/aging cycles. No current claim establishes the million-entry goal. Historical proposed incremental memory budgets do not replace the 100 MB release requirement. |

The final comparator and workload protocol is not yet frozen. In particular,
current PGP comparison evidence is missing. Historical experimental B/C layout
failures remain historical; default-path release preparation does not activate
those layouts or authorize a new optimization campaign.
## Historical pre-correction full-integration failure — 2026-10-13

At the pre-correction checkpoint, the full Vault library suite passed 35/35 tests. The public CLI package
run exited 101 at its first failing target, `cli_flow`: 67 passed and 11 failed.
Earlier targets recorded 43 passing binary-unit tests with one declared ignore,
one passing agent-flow test with one declared ignore, and 21 passing alias tests.
Later CLI targets were not run. The earlier inventory's zero-ignore claim was
incomplete; actual terminal target summaries above are authoritative.

Several normal CLI operations reported `static workspace vector moved` in the
vendored codec, with allocator corruption or a child SIGSEGV in some failures.
These were confirmed release blockers that triggered the correction described above. Agent startup and signed/unsigned status
failures are also preserved and are not assumed to be cascades without evidence.
No crash or vulnerability reproduction probe was run. Static review identifies
that borrowed workspace storage is represented as a `Vec` and exposed through
mutable Vec access: post-operation identity assertions cannot prevent an earlier
reallocation/deallocation. The replacement ownership boundary was then designed;
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

At that checkpoint, source publication, platform CI and resource measurements were
held until focused normal/property validation of the ownership correction. Performance
optimization remains paused. This result supersedes earlier pending/pass-only
checkpoint descriptions; it is not a release-ready conclusion.
