# Release preparation — 2026-10-13

Status: in progress; no release-ready claim. Brett has authorized completing preparation and then releasing reVault 0.5 once the required gates pass. The separately approved corrected Zstd dependency is published.

Brett requested preparation for release after accepting the current performance checkpoint. Performance experiments remain paused. The working scope is the current default format-4 implementation on `issue-310-zip-read-performance`, for compatibility line `0.5.x`, under the release-preparation instruction. Alternative archive layouts remain experimental and are not activated by this work. Published baseline checked on GitHub: CLI `0.4.1` and bindings `0.4.1`.

The security checkpoint `788d4a03` was followed by registry routing and retained-fixture qualification at `0300ee65`, installed-consumer checks at `5cdeda3d`, and the agent deadline correction at `352a180a`. The first 0.5 candidate `1acd23f2` entered prepare-all run `38367390629`; its Linux native conformance exposed a missing explicit headless fixture policy. That candidate and the later `f67a01dc` candidate are not qualified; the latest package correction checkpoint below supersedes their current status. Final evidence must identify the actual committed candidate, features, toolchain and artifacts. Earlier checks cannot qualify later edits. Preserve the unrelated dirty main checkout.

## Current qualification checkpoint

Candidate `c17c21128730f66e5b8a4bed4ddd6c3b101f0555` must not be promoted:
the fixed comparison stopped on an observed **148,746,240-byte CLI RSS** peak
during the 10,000-file plaintext extraction used to verify a warm-up write.
The command exited successfully; the frozen 100,000,000-byte resource guard
stopped the run before content verification. Cleanup passed. The retained 528
observations form an incomplete failed campaign, not a comparison pass.

The [extraction worker correction](../extraction-worker-memory-2026-10-14/README.md)
removes four unnecessary complete metadata copies while preserving worker count,
authentication, page-cache ownership and extraction policy. Ten focused extraction
tests and all-target Clippy pass; exact checkpoint resource qualification and a
replacement immutable candidate remain required. Current c17 CI is independently
pending its Linux secure-API step; success there cannot override this resource
failure. The three c17 lockfile audits are clean at the recorded official database
revision. The earlier six-case screen remains a limited positive for its workloads.

## Historical rustdoc correction checkpoint

Candidate `f3b69ee0bbc99117616b6a3e125111de319fb445`, prepared from historical-reader correction `682fd600`, is ineligible for promotion because its required rustdoc job encountered colliding library output names. The [focused workflow correction](../rustdoc-reader-isolation-2026-10-13/README.md) passes all six affected commands while retaining warnings-as-errors and every package. Replacement candidate CI is required. Its fresh CLI build passed; all three exact candidate locks passed advisory audit against clean official database revision `d8325cf4f578e2df4cc5b98a30e9f46b3b88e9f0` (1,309 advisories, no ignored advisories or findings). No timing samples were run on f3.

## Historical fourth-candidate qualification

The fourth immutable candidate is `7cb57d5667c6308b0ee91793f195b84db5378800`,
in [prepare run 38386958471](https://github.com/onepub-dev/reVault/actions/runs/38386958471).
It must not be promoted: a fresh clean-database audit identified vulnerable ruzstd
0.8.3 in the historical format-1 migration readers. The current format-4 decoder
and facade/fuzz audits were unaffected. Five explicit compatibility reader packages
now preserve the distinct historical implementations while routing to patched
ruzstd 0.9.1. The [remediation checkpoint](../historical-migration-decoder-2026-10-13/README.md)
records nine decoder passes, six selected/retained migration passes, five clean
Clippy package gates, unchanged historical fixtures and clean audits of all three
locks. A replacement committed candidate is still required.

The exact fourth-candidate CLI also passed the bounded six-case resource screen:
two protected owner-signed writes and four plaintext many-file extractions, with
verified bytes, modes, identities and cleanup. The largest conditional conservative
CLI-plus-owned-agent bound was 95,035,392 bytes, including setup observations.
These are limited workload results, not the full release performance gate or a
universal 100 MB guarantee. The full fixed ZIP/PGP comparison has not started;
the new historical dependency fix must be reflected in final candidate identity.
Sanitized [write](../historical-migration-decoder-2026-10-13/resource-preflight/write-resource-summary.json)
and [extraction](../historical-migration-decoder-2026-10-13/resource-preflight/extract-resource-summary.json)
summaries are retained; raw records were preserved with a verified relocation map.

The user-requested [archive limits manual](../../../manual/maintain-and-recover/archive-limits.md)
distinguishes field encodings, practical metadata limits and extraction defaults.
Post-release issues 330 and 331 cover broader limits investigation and CLI numeric
extraction overrides; they do not restart optimization during this release.

## Historical third-candidate qualification

The third immutable candidate is `353ded7da454de970a075819add09073e4190e28`,
prepared from correction checkpoint `ec8a0729` in
[run 38381172126](https://github.com/onepub-dev/reVault/actions/runs/38381172126).
It is not qualified. The required Swift compile gate rejected an unused extraction
result. Windows x64 and ARM64 installed-package checks each reported 22 passing
Vault integration tests and three failures: two tests attempted a second filesystem
read while holding the archive's mandatory writer lock, followed by one poisoned
test-mutex cascade. The archive opens had succeeded; the failures were at the
independent byte assertions, not at recovery admission.

The correction checks Swift's returned Boolean and compares extracted bytes before
the policy extraction overwrites the file. Swift 6.1.3 package inspection, build
with warnings denied, and its one test pass in the recorded isolated environment.
The Vault tests now drop the first recovered handle without committing, compare
persisted bytes independently, then reopen through the same credentials and owner
for mutation and commit. Runtime authentication and file locking are unchanged;
the two corrected Linux lifecycles and the Vault integration-target Clippy gate pass. Replacement Windows CI remains required to qualify the platform correction.

The two corrected Vault lifecycles, the original poisoned-cache test in a fresh
process, and one supplementary cache lifecycle pass locally; Vault integration-test
Clippy also passes with warnings denied. All registry-native validation commands
listed in the workflow pass at this correction epoch. See the
[curated correction results](raw/candidate-353-corrections/checkpoint.json),
[local command/artifact results](raw/candidate-353-corrections/local-results.json)
and [independent CI results](raw/candidate-353-corrections/independent-ci.json).
Exact `353ded7d` Endian Interop passed all six jobs with current-writer fixture
steps; Agent IPC passed on Linux, macOS and Windows. These are scoped positives,
not qualification of the failed candidate or its replacement.

Local registry validation also found that the workflow's repository-root Ruby gem
command produced an incomplete artifact despite exiting successfully. Validation
now builds from the package directory, matching the existing publication and
installed-consumer paths, and checks actual gem payload entries. The corrected gem
contains the facade, implementation modules and native shim. The exact commands, source epochs and artifact hashes are recorded in the
curated results; none of these commands publishes a package.

The exact third-candidate CLI passed an untimed process-accounting admission:
two ordinary 8 MiB plaintext and encrypted/owner-signed lifecycles, 24 thread-clone
calls, zero non-thread child-creation calls, verified bytes/modes and owned-service
cleanup. This is neither a timing result nor the 100 MB gate. No resource samples
have run. The six-case screen remains disarmed and must be pinned to the corrected
candidate before execution; the full fixed comparison additionally requires that
screen, frozen inputs and the declared ZIP/PGP interpretation.

## Earlier package correction checkpoint

The second candidate `f67a01dc` (prepare run `38373486321`) is not qualified.
Its required gates exposed Go/Python formatting, Ruby/Lua alias facade mappings,
a missing `gdbus` utility in Linux images, and fresh Rust 1.88 consumer resolution
of a newer incompatible UUID. The correction batch fixes those specific causes.
The Linux Vault dependency now constrains UUID below 1.27; ordinary edition-2021
resolver-2 consumers remain supported. Root, facade and E2E lock changes add only
the direct dependency edge and retain UUID 1.26.0.

The extracted `revault-api 0.5.0` package passed its ordinary persisted archive/
Vault lifecycle and a separate installed-package interop invocation on Rust 1.88.
Ruby and Lua each passed all 229 conformance checks, including persisted aliases,
with an owned Secret Service whose bus-owner PID was verified. These local runs
use the previously qualified native library in pinned Trixie containers; final
Bookworm package/platform CI remains required. Tooling, Go, Python and JS/TypeScript
checks passed at their recorded source epochs. See [curated results](raw/package-gate-corrections/result.json).

The actual bounded mutation-sequence fuzz job passed at `f674b7ab` in
[run 38373225520](https://github.com/onepub-dev/reVault/actions/runs/38373225520);
`f67a01dc` changes only release versions/build manifests from that checkpoint.
The earlier fuzz installation failure below is historical. No release resource
samples have started. A replacement immutable candidate, final platform/package
gates and the required resource/comparator acceptance still precede publication.

## Required work and evidence

| Requirement | Current disposition | Completion evidence |
| --- | --- | --- |
| Security remediation | Active review covers private extraction, agent transport/process identity, secret lifetimes, codec ownership and password derivation budgets. | Resolve confirmed findings, review changes, run focused validators and ordinary persisted lifecycles; retain limitations and failed attempts. |
| Default runtime integration | Current default/native library suites and ordinary package targets pass at recorded epochs; subsequent narrow fixes have focused qualification. | Complete current-source API, Vault, CLI, migration and relevant feature suites with exact counts and ignored-test accounting. |
| Binding consistency | Additive policy/budget/alias mappings, parity, registry routing and recorded carrier/package lifecycles pass; final replacement platform/package qualification remains open. | Binding declaration/parity checks plus persisted CLI-to-binding and binding-to-CLI round trips. |
| Migration fixtures | Approved new archive-4 and Vault-3/container-4 fixtures are generated; all 21 historical files remain unchanged. The corrected retained suite passes with explicit authenticated export/refusal coverage for immutable signed v2/v3 originals; separate known-owner v2/v3 positive migration tests pass. | Add only the new generators and immutable fixtures with producing source/version provenance; run the entire retained matrix and subsequent mutation checks. Never regenerate historical fixtures to pass. |
| Compatibility contract | No stable 0.5 release appears in the checked release list; 0.4 is format 3. | Validate migration from supported historical formats, documented read support, matching CLI/binding archive exchange and persisted Vault compatibility. Do not claim old 0.4 readers can read format 4. |
| Compression dependency packaging | Approved `zstd-complete 0.3.0` is published with the exact reviewed checksum; workspace, Rust facade and fuzz locks now use the registry route. Fresh registry-only consumer tests pass 2/2 and the protected ABI lifecycle passes 1/1. | Preserve exact artifact and resolution evidence; complete final packaged binding checks. |
| Supported platforms | Linux focused tests and Windows library cross-checks provide partial evidence only. | Actual supported Windows/macOS runtime CI, native package/install lifecycles, permission/agent tests and portable WASM checks on the final candidate. Address platform behavior regressions before readiness. |
| Dependencies and quality | Rustls advisory update passed an earlier audit; later dependencies/features changed. | Final locked dependency audit, required lint/static checks, formatting through tracked hook and appropriate post-format verification. |
| Candidate preparation | Third candidate `353ded7d` has an exact source identity but failed required Swift/Windows test gates; the reviewed corrections require a replacement candidate. | Commit and push isolated work, run reviewed prepare-only release workflow, retain package hashes and install/conformance results. Promote only the successfully qualified immutable candidate through the authorized release workflow. |
| Documentation | Existing plan contains historical failures and experimental architecture gates. The reviewed selected-default v4 wire specification and conformance index are now installed in ARCHIVE_FORMAT.md; experimental layouts remain excluded. | Reconcile current release scope, format contract, migration/support matrix, release notes and known limitations against evidence. Preserve historical reports unchanged. |

## Performance claims

The accepted checkpoint is a limited four-cohort plaintext extraction result, not a universal speed or 100 MB guarantee. The recent security changes are not timed by that report. Release preparation does not restart optimization, reinterpret old gates or advertise unmeasured speedups. If a mandatory release resource check remains unmet, report it explicitly rather than weakening the requirement.

## Publication boundary

Brett explicitly authorized completing release preparation and then releasing reVault 0.5. That authorization does not waive failing gates: candidate preparation and qualification come first, followed by promotion of the verified immutable artifacts. Performance optimization remains paused.

The active remediation record is the [security review](../security-review-2026-10-12/README.md); the [archive-v4 plan](../../archive_v4_plan.md) retains historical performance and integration evidence. Release preparation must selectively port the reviewed release-infrastructure corrections from main without replacing format-4 runtime sources or touching unrelated main-checkout work.
## Earlier security and format checkpoints

The user confirmed current format 4 for the 0.5 line and separately approved
both the corrected dependency publication and durable, clearly marked public
test-only signing material for the new immutable fixture. `zstd-complete 0.3.0`
was published successfully; an independent registry download matches the exact
approved SHA-256 `acce3e08f5aa28b5dc1738f46c3a8a65e0daa28dca2dae579cc48bc398958e9d`.
The withdrawn 0.2.1 artifact was not published. See [publication evidence](raw/published-codec/result.json).
Direct registry dependency/standalone consumer checks and fresh retained-fixture
qualification passed at `0300ee65`; the historical signed fixtures without their
original owner key retain explicit authenticated export/refusal coverage, not a
false successful-import claim. No reVault package is published. The subsequent
`5cdeda3d` checkpoint adds the persisted oversized-form refusal lifecycle and an
installed Rust facade create/reopen gate. Its release driver build, required
Clippy check and two driver unit tests passed; the actual installed consumer
execution remains a candidate package gate.

All four explicitly selected mandatory compression regression tests passed once
at the post-hook runtime checkpoint. The subsequent resource preflight has not
run: it will be re-frozen on the final registry-routed CLI after remaining fixes.
The exact-current-writer endian workflow passed all six jobs after binding fixture
cache keys to the producer commit. The first run's cached format-4 artifact had
unknown producer provenance and failed opening; that evidence remains preserved.
At exact `0300ee65`, Endian Interop passed all six jobs. Agent IPC and Unicode
passed on Linux and Windows but failed on macOS. The Unicode diagnostic identifies
`Session Agent read deadline: Invalid argument (os error 22)`; the nonblocking
absolute-deadline correction passed 13 Unix unit tests, Vault library Clippy
with warnings denied, and the normal CLI agent-cache lifecycle (one pass; the
separate serial lifecycle remains ignored in that invocation). The subsequent candidate run passed the actual macOS platform runtime job
`115196263197`, including agent IPC and filesystem checks; that closes this
specific runtime regression, not the entire candidate matrix. The constructor enables nonblocking I/O once; poll waits use the
same absolute deadline, including partial progress, interrupted waits and buffered
replies after peer closure. No deadline error is suppressed. The Rust workflow recorded 16 passing jobs, the same macOS IPC failure,
and three jobs cancelled by a newer branch workflow; cancelled jobs are incomplete.
The historical fuzz job failed before execution because its pinned tool was already installed;
`5cdeda3d` makes that installation idempotent without changing its version. No fuzz
execution pass follows from that setup correction alone; the later actual successful mutation-sequence run is recorded in the package checkpoint above.


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
The two explicitly approved current-format fixture generators completed; historical files remained unchanged. The corrected retained matrix passes with immutable historical export/refusal distinguished from separate positive known-owner migration. All four scheduled compression regression checks passed at their recorded pre-routing source epoch. External crash corpora and unrelated fixture-maintenance writers remain excluded.

Both required Clippy scopes passed in the isolated Rust 1.88 snapshot with
warnings denied: production workspace and bindings/tooling/WASM wrappers.
The final CLI all-target lint also passed after fixture cleanup and logging fixes.
Workspace rustdoc passed with warnings denied and all features, documenting the
two historical duplicate-name readers separately without dropping any selected
package. Original output-collision failures remain recorded. The earlier standalone Rust facade documentation attempt was blocked by its stale lockfile. Registry routing and that lockfile are now corrected; its final static/package qualification remains separately tracked.
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
0.3.0 release with public API notes included in its package. Selected packaged
qualification passed 777 library, 14 workspace and six documentation tests, with
explicit exclusions recorded. A mismatched ordinary fixture assertion was corrected
and passed separately; exact-package consumers passed. Final prose-only packaging
preserves those runtime/test/dependency identities. Rust 1.87 library compilation
passes. Final artifact SHA-256 is
`acce3e08f5aa28b5dc1738f46c3a8a65e0daa28dca2dae579cc48bc398958e9d`.
This is not an unfiltered package-suite claim. The exact artifact was explicitly approved, published, and independently downloaded with the same checksum. Fresh registry-only consumer tests pass 2/2; the protected ABI lifecycle passes 1/1. All three current lockfile advisory audits report zero vulnerabilities and warnings.

The earlier retained-key action was rejected by automatic approval review and held. Brett subsequently explicitly approved clearly public test-only signing material for the new immutable v4 fixture. Generation has completed with all 21 historical files unchanged; the corrected retained matrix and separate known-owner v2/v3 lifecycles pass. Final candidate platform/package CI, inter-version compatibility and release resource gates remain outstanding. Performance optimization stays paused; release is authorized only after qualification.

## Mandatory final resource verification inventory

P5 requires verification on the final release candidate, not extrapolation from
accepted optimization trials. Run only after builds and tests finish, with a
quiet host, frozen source/artifact identities and a predefined protocol. A
failed gate remains a release blocker; performance optimization stays paused.

| Contract or existing evidence | Required interpretation |
| --- | --- |
| G11 plaintext read/write versus ZIP; protected read/write versus PGP; 100 MB total CLI RSS | Still requires current-candidate qualification. Compression-only ZIP is not the protected PGP control, and helper-excluded process RSS is not total CLI memory. |
| Realistic primary corpus | Disk-backed 1 GiB single-file and 10,000 files of 64 KiB (625 MiB), including build and independent reopen/extraction; preserve compression/security distinctions. |
| Primary comparison statistics | The plan specifies 30 paired observations plus three warm-up pairs, alternating order, paired log ratios and 10,000 fixed-seed bootstrap draws; A3 upper 95% bound at most 1.00; A4 upper bound at most 1.05 against candidate A with matched policies. A4 is not an established PGP comparison threshold. Freeze exact applicable controls before execution. |
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
At that time, pending approval was suspended until a corrected replacement was qualified. Published 0.2.0 exposed mutable `ReusableVec` behavior, requiring a new dependency compatibility version. The subsequently corrected and explicitly approved 0.3.0 publication is recorded in the current checkpoint above; 0.2.1 remains withdrawn.
All original package and integration results remain preserved.

At that checkpoint, source publication, platform CI and resource measurements were
held until focused normal/property validation of the ownership correction. Performance
optimization remains paused. This historical result superseded the then-current pass-only descriptions; later corrected-source results are recorded above. It is not a release-ready conclusion.

## Registry and platform-correction checkpoint

Published Zstd 0.3.0 is now resolved from the registry by the workspace, standalone
Rust facade and fuzz lockfiles. All three advisory audits pass with no warnings;
the fresh registry-only codec consumer passes 2 tests and the protected ABI
lifecycle passes 1. Standalone facade rustdoc passes with warnings denied. Its
current source version is still pre-candidate; these results do not qualify the
future version-staged 0.5 package.

The focused platform-fixture tranche passes 8 password/access CLI tests, 8
completion tests, 3 page-reclamation tests and 2 Linux headless tests. The Unix
transport tranche passes 10 tests and Vault library Clippy. Android compilation
and actual Windows/macOS behavior remain CI requirements; operation-only error
context has been added to diagnose the unresolved macOS Unicode initialization
error without logging endpoints or credentials. Error kinds are preserved; raw
OS codes remain in display text but are no longer returned by raw_os_error for
wrapped transport errors.

Wire-spec review identified two further form defects: collection capacity was
not admitted against remaining bytes, and u16 string lengths could truncate.
Both are corrected without changing previously valid encodings. Two ordinary
admission validators, three persisted form lifecycles and API library Clippy pass.

Approved current fixtures were generated with reconstructable producer-source
evidence and all 21 historical files unchanged. The retained suite passes 3
tests with its maintenance writer ignored; separate known-owner v2 and v3
migration lifecycles each pass. Historical originals whose owner key was never
retained are explicitly export/refusal evidence, not fictitious successful
imports or permission to replace an established owner.

See the [curated correction summary](raw/platform-corrections/summary.json).
Original failed attempts and source/artifact manifests remain in owned evidence
storage. Raw credential-bearing fixture output is not copied into this summary.
Final platform, package, wire-documentation and resource gates still precede
release promotion.

## First 0.5 candidate conformance correction

Prepare-all run `38367390629`, source `1acd23f2ef9c6e0094fd3d575b0d88180d71a2f9`,
created no release tags or published reVault packages. Linux x86-64 and AArch64
native-package conformance failed when their CI Session Agent requested sleep
inhibition unavailable from the runner's D-Bus policy. The conformance fixture now
explicitly disables both sleep inhibition and process suspend termination,
including the installed Rust consumer. Dedicated capability/refusal tests remain
unchanged; these package lifecycles do not establish suspend protection. Preserve
the failed candidate rather than interpreting setup failure as package success.

The same candidate exposed Windows alias fixtures that assumed `/tmp` was a
fully qualified path, an omitted generated Swift alias-list decoder, and required
Dart formatting. The correction uses each carrier's owned absolute artifact path,
retains exact alias identity/path assertions, adds the generator's alias-list
mapping, and normalizes only the generator-owned Dart outputs during generation.
The facade's required Dart format, fatal analysis and documentation checks pass;
required tooling Clippy and binding inventory checks pass. The inventory reports
230 ABI declarations and 229 operations; older workflow labels saying 219 are not
an authoritative count.

The local C package compiled and passed its core and agent phases. Its platform
phase twice failed because `--login` left Secret Service unregistered and D-Bus
activation collided with the first keyring daemon. A private HOME/XDG retry
preserved that diagnosis. The fixture correction owns a foreground unlocked
keyring, private temporary state, nonactivating readiness and exact bus-owner PID
checks, with child cleanup on success or error. The corrected installed run passed all 229 required operations: 242 PASS rows,
four artifact rows and one native-artifact row. Core/agent/platform/last-error
phase row counts were 201/35/9/1. An intervening run passed all phases but the
verifier rejected its install path under rust/target; the final run used an owned
/tmp install root and passed the unchanged verifier. Tooling build and all-target
Clippy passed. Private Vault/keyring/runtime state and owned processes were
cleaned up. See [focused checkpoint evidence](raw/candidate-carrier-corrections/result.json).

Both corrected local loopback publication lifecycles passed (2/2) after making
their headless policy explicit. The original CI failure retains only safe status
and byte counts, so its underlying child stderr cannot be reconstructed. The
helper audit found no additional agent-using fixture omission; dedicated suspend
capability and refusal tests are unchanged. Actual Windows and Swift execution
and the full replacement candidate remain required.

CLI publication must include `revault_archive_v3_reader`,
`revault_vault_v3_reader`, and `revault_migrate_archive_v3` before the current
migration/CLI crates. Verify both `revault-migrate-archive-v3` and
`revault-migrate-vault-v3` binaries and their handshakes after registry publication.
Final installed Rust verification must remove candidate-only core path patches
and exercise persisted creation and independent reopen through registry packages.
