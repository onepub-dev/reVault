# Archive v4 design and delivery plan

Created: 2026-09-26. Status: implementation started on the existing issue-310
branch; architecture selection and release qualification are outstanding.
This plan implements the [project goals](../manual/project-goals.md).
The [documentation map](documentation_map.md) assigns authority and cleanup work.

## Current position and next work

Updated 2026-09-27. P2 is still an architecture comparison, not a selected format.
The production default writer has bounded staging; candidate C has authenticated
membership, separated metadata mirrors, resumable allocation/erasure, file/range
reads and [independently protected physical packs](evidence/independent-file-packs-2026-09-27/README.md)
with [tested updates and explicit read-only salvage](evidence/candidate-file-lifecycle-2026-09-27/README.md)
and [source-preserving file compaction](evidence/candidate-compaction-2026-09-27/README.md).
C remains test-only and lacks complete public
record/access semantics. The [current scorecard](archive_v4_evaluation.md#current-scorecard)
states which gates remain failed or incomplete.

1. Finish the shared-control architecture experiment against the
   [whole-format eligibility findings](archive_v4_evaluation.md#whole-format-eligibility-review).
   The [fresh dense file image](evidence/dense-file-image-2026-09-27/README.md) now
   connects authenticated controls, credential bootstrap, a bounded catalogue,
   denser independent fragments and verified file/range reads. It remains file-only
   with [persisted metadata updates](evidence/dense-metadata-update-2026-09-27/README.md).
   Renames/permission changes now connect the catalogue, ownership graph, compact
   preparation, mirrored publication and retirement through interruption tests.
   Payload mutation remains unconnected. The first edit retains two extra 64 KiB
   control allocations: both retained small fixtures grow from 320 to 448 KiB
   (1.935× ZIP), stable through 100 metadata edits and 100 unchanged repeats.
   [Split slots were ruled out for the primary corpus](evidence/inline-slot-feasibility-2026-09-27/README.md):
   its 29/37 KiB encoded catalogues cannot fit a 24 KiB slot. Compare paged
   catalogue COW with the [read-only paged cost model](evidence/paged-catalogue-cost-2026-09-27/README.md),
   budgeting actual hybrid publication bytes, complete file/pack leaves and staging
   space together on retained records before implementing another writer.
   Integrate typed catalogue overflow, public record semantics, payload updates,
   overflow transitions and compaction/installation before qualification. Preserve the existing candidate as the control.
   Five retained images now match the size model; both small fixtures are actual
   320 KiB files (1.382× ZIP), with full persisted-byte verification. Budget
   temporary metadata, sealed-length/cleanup and any relocation publication; this
   size result does not qualify mutation, CPU/RSS or full public semantics.
   [Read-only salvage](evidence/dense-salvage-2026-09-27/README.md) now verifies
   intact neighbours independently. A highly compressible stress case loses
   512 files per destroyed physical region versus C's 64; one-byte corruption
   loses one file. On the retained primary corpus the corresponding losses are
   190 plaintext / 176 protected files versus C's 64. Explicitly decide this
   trade-off independently of the per-fragment random-read bound.
2. Retain the [failed mixed aging qualification](evidence/mixed-file-aging-2026-09-27/README.md):
   all four 1,000-operation modes preserve contents and erasure, but unpadded
   control-space growth at cycle 421 exceeds the first-100 high water. Diagnose
   placement in the whole-layout comparison; do not mask it with compaction.
   Extend packed-file process death and source-handle orchestration. File-only
   compaction passes source-preserving failures and 16 process deaths; resource
   probes cover 100k files and 1 GiB, with platform/headroom checks remaining.
3. Run one declared whole-layout comparison addressing fixed control costs,
   metadata access/cache policy and normal-open padding verification together.
   Retain the failed results: C is 3.77× ZIP raw / 4.02× compressed on 8 MiB reads;
   small files remain 6.08× ZIP after compaction; encrypted range-open regresses.
   Raw 64 MiB passes its size subcase at 1.016× ZIP. No A3/A4 pass is claimed.
   Preserve source, authority, erasure and padding guarantees; do not resume an
   unlimited sequence of buffer/encoder changes or silently replace workloads.
4. Finish migration/owner fixtures, bidirectional CLI/binding/Vault compatibility
   and release qualification. Obtain a corrected published compression dependency
   and verify isolated packaged builds before removing the temporary vendor patch.
5. Complete H1's remaining platform/binding packaging and remote CI qualification.
   The separate #322 branch at `d8dfa6cf` now passes Linux locked, denied and hung
   credential-store tests, existing-keyring interoperability, a clean release
   install without D-Bus development packages and the installed headless lifecycle.
   Do not mix unfinished v4 into format-3 `0.4.x`.

The [checkpoint history](archive_v4_history.md) preserves earlier decisions and
measurements without competing next-work instructions. Guarantees and numeric
budget changes still require explicit review; none is implied by a green test.

## Outcome and scope

Deliver an archive format that makes mirror and other mutations physically
safe, supports efficient first reads and updates, and preserves security,
recoverability and cross-language compatibility. Evaluate the whole archive
design before continuing isolated performance tuning.

Retain the v4 transaction work already on `main` as the starting point. Compare
the current data layout, experimental indexed blocks, and a simpler pack/extent
layout against the same contract. A wholesale rewrite is a candidate outcome,
not a prerequisite or a decision already made.

Cover files, directories, symlinks, variables, forms, access metadata, mirror
definitions and Vault operations that use the archive engine. Independent Vault
storage redesign, new sharing services, passkeys, deduplication and unrelated
features are outside this work.

Also deliver the independent installation/portability fix in
[#322](https://github.com/onepub-dev/reVault/issues/322): the CLI must install and
work on desktop and headless systems. Track this as H1 below. It does not require
an archive redesign and must not wait for the v4 layout decision.

## Starting evidence

This is the original planning snapshot from source/document review. The
[implementation evidence](evidence/v4-baseline-2026-09-26.md) records subsequent
executed suites and reproduced blockers; no new comparative timings are claimed.

| Area | Recorded state | Evidence |
| --- | --- | --- |
| Main | `0f9a137e`; v4 transactions, accounting, rollback, compaction and external backing-store support landed | [Transaction protocol](../manual/develop-with-revault/transactions.md), [implementation map](transaction_recovery.md) |
| Performance branch | `issue-310-zip-read-performance`, HEAD `8fb653e8`; latest production commit `00a351ff`; native blocks remain opt-in | Branch file `rust/revault_lockbox_api/benches/results/issue310-resume-checkpoint.txt` |
| Uncommitted work | Parallel encoder-pool prototype in four files; no established performance win | Same checkpoint; inspect the actual diff before touching it |
| Native correctness | Last checkpoint reports two failing recovery tests; default suite passed in that recorded run | `issue310-resume-checkpoint.txt` and `issue310-resume-audit-2026-09-22.txt` |
| Read objective | Recorded large compressed native reads remained approximately 3.8–3.9 times ZIP duration in one comparison | `issue310-shared-window-notes.txt`; this is not a claim about every workload |
| Write objective | Protected compressed native/default gate still open; encrypted-only upper 95% bound was +10.876%, above +5% | `issue310-layout-gate-62180a3-notes.txt` |
| Release fixtures | Missing retained v4/current Vault-container fixtures; non-v1 archive export path still invokes the current reader | [Retained fixtures](../rust/revault_migration/tests/retained_fixtures.rs) |
| Documentation | Conflicting format descriptions and duplicated historical overviews | [Cleanup inventory](documentation_map.md) |
| Headless installation | #322 reports `cargo install revault_cli` v0.0.17 failing because `libdbus-sys` requires `dbus-1.pc`; current Linux Vault dependencies include `dbus`, `secret-service`, `zbus` and `keyring` | [Issue #322](https://github.com/onepub-dev/reVault/issues/322), [Vault manifest](../rust/revault_vault_api/Cargo.toml); runtime behaviour and other platforms still need investigation |

The issue-310 evidence files above live on that branch, not necessarily on
`main`. Find the persistent worktree with `git worktree list`, or inspect the
files with `git show issue-310-zip-read-performance:<path>`. Do not rely on an
agent's absolute local path or on ignored build artifacts surviving.

Original requirements remain useful provenance:
[crash-safe redaction #303](https://github.com/onepub-dev/reVault/issues/303),
[ZIP read performance #310](https://github.com/onepub-dev/reVault/issues/310),
[abandoned allocations and full retirement #313](https://github.com/onepub-dev/reVault/issues/313).
Issue #322 also explicitly asks whether other target platforms have equivalent
constraints; the fix must include that audit rather than assuming Linux is the
only affected environment.

## Acceptance contract

Goal IDs refer to the project goals. This table is the initial scorecard; P1
defines the detailed matrix and P2 fills measured evidence. Do not weaken a
gate after seeing a failing result without an explicit recorded change of scope.

| Gate | Acceptance | Initial status |
| --- | --- | --- |
| A1 — committed state and ownership (G2, G3) | Every physical range is live, retained control/history, zero reusable space, or durably tracked pending work. Faults select a complete old/new state. Cleanup is resumable; no abandoned payload remains after completed reclamation. | Implementation exists; full matrix to revalidate |
| A2 — damage and authorization (G2, G3) | Intact independently recoverable content retains provable authorization despite unrelated damage. Never authorize content using its own untrusted checksum or resurrect uncommitted/deleted records. Document limits where authority is lost. | Design unresolved; native failures recorded |
| A3 — reads (G4) | Unencrypted first-read parity with ZIP on each designated primary workload; 10% faster remains the aspiration. Separate raw/compressed and small/large workloads, fresh handles, cold I/O and warm decoded caches. | Not met; exact primary workload selection in P1 |
| A4 — open and writes (G4) | Retain the recorded native/default non-regression gate: upper 95% bound of relative duration change no greater than +5% on designated comparable cases. Include protected modes, streaming and real CLI operations. | Open; protocol and cases to freeze in P1 |
| A5 — space and resources (G3, G4) | No content leaks or no-change allocation accumulation. Separate retained history from live/free bytes. Bound parser/cache/worker memory; establish numeric limits for growth, read/write amplification, recovery time and compaction headroom before selection. | Limits/budgets to specify in P1 |
| A6 — migration and interoperability (G1, G6) | Supported historical formats migrate with full logical verification and source preservation. Retained fixtures and all released CLI/binding/carrier combinations pass the compatibility matrix. | Release blockers remain |
| A7 — maintainability and specification (G7) | One reviewable layout/protection model, explicit ownership and persistence ordering, normative specification, decision records and reproducible evidence. | Documentation consolidation started |
| A8 — installation and session independence (G1, G5, G6, G7) | Normal CLI installation and core workflows succeed on supported headless and desktop targets. Optional credential services are not mandatory build/runtime requirements; absent, locked or denied stores have bounded, actionable behaviour and secure explicit-credential alternatives. | #322 open; H1 pending |

For A3, P1 must operationally define “10% faster” and statistical parity rather
than switching between throughput and latency measures. For A4, compare equally
protected/durable modes; do not compare encrypted writes to unprotected ZIP as
if the work were equivalent. A confidence interval exceeding a threshold is a
failure to establish the gate, not proof of that worst-case slowdown.

## Milestones and dependencies

| Milestone | Main deliverable | Depends on | State |
| --- | --- | --- | --- |
| P0 — preserve and reproduce | Exact baseline/evidence inventory and reproduced blockers | This plan | Complete: prototype preserved, blockers reproduced, deterministic append/reuse CLI abort coverage passes |
| P1 — define contracts | Security model, evaluation contract and proposed design decisions | P0; drafting can begin immediately | Drafts written; mode assignment, budgets and decisions await review |
| P2 — compare architectures | Same-workload results for three candidates | Frozen P1 contracts | A/B and file-only C comparisons retained; independent shared packs/deletion pass correctness; full C lifecycle/recovery/record matrix, resource evidence and contract review remain |
| P3 — select and specify | Accepted decision records and complete v4 wire specification | P2 | Pending |
| P4 — implement and harden | Selected implementation with green correctness and performance gates | P3 | Pending |
| P5 — qualify release | Migration, compatibility, platform and documentation evidence | P4; fixture/tooling repairs can start earlier | Pending |
| H1 — fix #322 | Headless/desktop installation and credential-backend validation | Can start after its own baseline reproduction; independent of P1–P4 | Independent branch: Linux dependency fix and targeted tests pass; full platform/store matrix outstanding |

Assign a named owner and reviewer when starting each milestone. The product
maintainer owns changes to guarantees and primary targets; storage/security
reviewers own protocol and trust analysis; performance and release maintainers
own reproducibility and interoperability evidence. No dates or staffing are
assumed by this plan.

### H1 — Fix headless installation and audit other targets (#322)

Local checkpoint: `issue-322-headless-cli` at `d8dfa6cf`. Linux uses pure-Rust
D-Bus with a five-second total credential-operation deadline, including bus
handshake and prompts. Private-service tests cover healthy interoperability,
locked/denied stores and hung authentication, session negotiation, methods and
prompts. The release CLI installs and passes a persisted-byte headless lifecycle
in an offline Linux container without D-Bus development packages. Vault tests,
strict Clippy and Windows GNU cross-check pass. Evidence is retained on that
branch in `docs/evidence/headless-credential-deadlines-2026-09-27/`.
Actual macOS/Windows credential-service behaviour, binding packages, remote CI
and appropriate-release-line integration remain unqualified; A8 is not complete.

This is a separate issue branch/worktree owned by the CLI/platform maintainer.
It may proceed alongside the archive milestones. The issue is a build failure,
so detecting an absent desktop service only at runtime does not fix it.

* Reproduce the reported Cargo installation in a minimal Linux environment
  without `libdbus-1-dev`, a graphical session, a session bus or Secret Service.
  Record which direct/transitive features pull in `libdbus-sys`, and distinguish
  build-time native requirements, dynamic runtime libraries and service access.
* Audit equivalent installation, credential-store and session assumptions on
  macOS and Windows, including SSH/noninteractive use, CI/service accounts and
  locked or unavailable stores. Record applicability explicitly; do not assume
  Linux environment-variable handling proves other targets work. Audit affected
  native binding carriers; document WASM/platform exclusions rather than adding
  a desktop service dependency to portable code.
* Choose a dependency/backend arrangement that preserves desktop integration
  and the normal headless installation path. Evaluate pure-Rust backends and
  optional platform features. Adding a requirement to install D-Bus development
  packages, or vendoring C D-Bus, is not the intended resolution. Avoid relying
  only on a special `--no-default-features` workaround for ordinary installation.
* Define unavailable/disabled/locked/denied credential-store behaviour. Explicit
  credentials must support core commands without a desktop session. An operation
  that specifically requests an unavailable Auto Open backend should return an
  actionable error. Never fall back to plaintext credential persistence or wait
  indefinitely for an interactive prompt in unattended execution.
* Add clean-environment installation and public CLI lifecycle coverage. Create
  a Vault/Lockbox, add data, independently reopen/read and compare bytes, then
  exercise the supported session and explicit credential flows. Cover no TTY,
  no bus/store, and a working desktop store; verify bounded failure for unavailable
  requested capabilities. Inspect packaged native dependencies as well as
  source builds so developer-machine libraries cannot hide the defect.
* Update installation/Auto Open/platform documentation and CI images. Keep a
  minimal-environment CI lane without desktop development packages; existing
  lanes that install `libdbus-1-dev` cannot establish this acceptance criterion.

Exit: A8 passes with reproducible build and runtime evidence on the supported
target matrix; desktop credential integration still works. The fix may ship
independently on the appropriate release line once its checks pass. If ported
to format-3 `0.4.x`, transfer only the compatible fix and validate that branch;
do not pull v4 code with it. P5 includes A8 as a release gate, not a reason to
delay H1 until archive selection.

### P0 — Preserve work and reproduce the baseline

* Record exact commits, feature flags and uncommitted diffs in both worktrees.
  Preserve the parallel prototype separately; do not discard it or continue its
  tuning as the next default task. Keep the format-3 release branch isolated.
* Build matching default and native-layout configurations. Reproduce the two
  recorded recovery failures, then run the relevant baseline suites. Distinguish
  existing failures, environment limits and new regressions.
* Reproduce failed mirror preparation through the public CLI with deterministic
  source changes after writes, followed by independent reopen/content checks.
  Audit coverage against #313 rather than assuming the core reproduction covers
  the CLI path. Exercise appended and reused space.
* Check retained-fixture routing and inventory failures. Record the complete
  historical exporter and current Vault-container matrix.
* Produce a short evidence index with commands, hashes and feature sets. Make
  active issue tracking reflect the remaining work without rewriting history.

Exit: another contributor can reproduce each known blocker and locate every
retained change. No performance result depends on a missing old executable.

### P1 — Resolve the requirements before selecting the layout

Create `docs/security_model.md` and `docs/archive_v4_evaluation.md`. Define
protection modes, attacker capabilities, metadata leakage, security boundaries,
recovery trust and workload budgets. Keep ordinary transaction recovery distinct
from salvage of damaged archives.

Draft three decision records:

| Decision | Questions and candidate approaches |
| --- | --- |
| Signing and independent recovery | Must normal open eagerly verify all content? How does a surviving file/extent prove owner authorization and committed membership when another file or the TOC is damaged? Compare existing full-content verification with authenticated per-object commitments under a signed root. A Merkle-style structure is an option, not a chosen design. Specify deletion freshness, rollback/replay limits and recovery when proofs are lost. |
| Physical layout and access units | Choose compression frame, authentication block, packing and allocation units together. A 16 KiB verification block does not provide a Zstd restart point. Define small-file packing, raw range reads, compressed range amplification, metadata opening, padding and remote range-request cost. Keep one descriptor model across protection modes. |
| Allocation, history and reclamation | Compare conservative base-free-range reservation with explicit bounded reservations. Define ownership of control allocations, durability ordering, history retention, checkpoint cost, tail trimming and verified compaction. Measure rollback of a tiny failed update against a large free-space inventory. |

Preserve all established security guarantees while evaluating alternatives.
Any change to eager verification, damage detection timing, recipient access,
padding defaults or owner authorization needs an explicit product/security
decision before activation. No benchmark licenses weakened authentication,
skipped wiping, reduced durability or an extra default full-frame codec pass.

Exit: every acceptance gate has a test/measurement method and a fixed target or
documented limit. Open choices have named decision owners. No target required
for selection is left as “improve performance.”

### P2 — Compare complete designs

| Candidate | Purpose | Principal questions |
| --- | --- | --- |
| A: existing data pages with v4 transactions | Control and possible final design | Can the goals be met without the native block extension? Which whole-page/frame costs remain unavoidable? |
| B: experimental native indexed blocks | Evaluate retained issue-310 work | Do raw range gains survive compressed, small-file, signed-recovery and aged-archive workloads without unacceptable complexity or write cost? |
| C: simpler pack/extent layout with the same transaction guarantees | Test whether a different layout removes recurring work | Can independently protected packs/extents, compact indexes and explicit recovery commitments simplify ownership and reduce I/O/copies? What are packing, redaction and recovery costs? |

Use bounded prototypes with the same required guarantees. A prototype missing
signing, rollback or redaction can inform a cost breakdown, but cannot win a
production comparison against a complete implementation. Keep raw results and
rejected experiments. Do not spend an unlimited series of micro-optimizations
making one candidate look acceptable before comparing the alternatives.

The evaluation document must cover:

* Synthetic small files (including the existing 512 × 4 KiB case), a large
  small-file tree (100,000 entries), 1/8/64 MiB files, and a GB-class streaming
  case with explicit memory accounting.
* Compressible, incompressible and mixed contents plus a pinned real source
  tree/package corpus; synthetic credentials only.
* Create, open/list, first byte, whole-file/range reads, full extraction, initial
  mirror import, unchanged repeat, additions, larger/smaller replacements,
  removals, refusal/abort, recovery and compaction.
* At least 100 update cycles for leak/accounting assertions and a 1,000-cycle
  aging experiment. Exercise fragmented free space and accumulated history.
* All encryption/signing combinations, raw/Zstd, default privacy padding, and
  a separately reported optional no-padding case. Test both ordinary defaults
  and explicit cache/worker policies; identify native and WASM constraints.
* Fresh handles over warm OS caches, genuinely cold I/O where reproducible,
  and repeated reads over warm decoded caches as separate results. Record
  network request/byte amplification for remote-range use independently.

Report end-to-end duration, first-byte latency, CPU, peak RSS, archive size,
bytes read/written/decoded/zeroed, allocations, sync counts, recovery work and
compaction disk headroom. Separate payload, metadata, padding, free space and
retained history. Stage profiles explain costs; their overlapping medians are
not additive end-to-end measurements.

Build before timing, pin source/features/dependencies/corpora, alternate paired
candidate runs, retain sample distributions and uncertainty, and verify persisted
bytes outside the timed section. Record hardware, cache policy, worker count,
OS state and uncontrolled load. No owned build/test overlaps timed runs. Use
Rust for new runners. State differences in codecs, protection, durability and
CRC/authentication coverage when comparing ZIP or encrypted archive tools.

Exit: a bounded comparative report explains which design meets each gate and
where it fails. If none passes, return to the specific unmet P1 decision;
do not activate the least-bad candidate or quietly drop a workload.

### P3 — Select the design and write the normative specification

Select a design using the complete scorecard, including complexity and recovery
cost. Record rejected alternatives and trade-offs in accepted decision records.
Update `rust/revault_lockbox_api/ARCHIVE_FORMAT.md` with the chosen v4 encodings,
endianness, authenticated identities/lengths, nonce rules, limits, padding,
compression, indexes, history and feature/version rules. Link the maintained
transaction protocol and add conformance vectors.

Define what every v4 reader must understand before any `0.5.x` release. A later
writer flag must not introduce output unreadable by earlier clients in that
line. Audit whether any v4 artifacts have already been distributed; specify
their support/migration explicitly rather than assuming unreleased means unused.

Exit: an independent implementation reviewer can determine valid/invalid bytes,
publication authority, recovery behaviour and compatibility from the documents
and vectors. No unresolved format-level decision is hidden in a feature flag.

### P4 — Implement the selected design and prove its invariants

Implement cohesive changes on isolated branches/worktrees, preserving unrelated
work. Retain measured improvements where they fit the selected architecture;
remove rejected production paths after replacement coverage exists.

Exercise create → unchanged repeat → add → replace larger/smaller → remove →
independent reopen for every record family and mixed transactions. Verify file,
variable and form bytes, metadata, permissions, mirror ownership and access
semantics. Test shared-page neighbours, streams, handles, free-space reuse,
readonly opens, cancellation and stale/external storage handles.

Inject returned failures and real process death around intent, reservations,
payload/root/auth writes, sync, publication, zeroing, checkpoints, truncation and
replacement. Interrupt recovery again. Check complete allocation accounting and
physical erasure where the public CLI cannot observe them; document that testing
exception. Keep public CLI setup and separate CLI persisted-content verification
for end-to-end tests.

Only after the architecture is selected, profile remaining gate failures and
perform targeted optimization. Each experiment states the unmet gate, expected
effect, controls, memory/security costs and stopping condition. Stop when the
gate passes or evidence redirects the design; a microbenchmark win alone does
not close a milestone.

Exit: no known correctness failures in the selected configuration, and all
required A1–A5/A7 results pass on the same source revision. Run tests and checks
before Rust formatting; use the tracked commit hook without bypassing it.

### P5 — Qualify migration, bindings and the release

Repair historical fixture-reader routing. Retain immutable v4 archive fixtures
and the current Vault-structure/container combination alongside all historical
fixtures. Include supported modes, mirror failures/bloat, secure-page retirement,
non-default owner identity, interruption/resume and replacement recovery.

Migration must preserve identity, established owner, access semantics, creation
options and all committed records. Verify all logical contents before installing
or reusing resumed output; ignore abandoned payloads. Keep originals/backups and
encrypted intermediates. Publish required historical exporter dependencies in
verified order and exercise the exact installed tooling.

Rebuild every distributed native/WASM carrier and matching language package.
Test cross-language and cross-version reads/writes within the compatibility line,
including older readers consuming newer output, plus supported platform and
endianness combinations. Keep format-3 `0.4.x` separate from format-4 `0.5.x`;
never reuse Dart's published format-2 `0.3.x` line for format 3.

Run affected core, CLI mirror/migration, migration-exporter, Vault, binding and
platform crash/recovery suites and executable conformance tests. Re-run the final
performance matrix on the release candidate. Complete the documentation cleanup
and any outstanding security review required by the selected design/release.
Include H1's minimal headless installation and desktop/runtime capability matrix;
a release builder with all native development packages installed is insufficient.

Exit: the release checklist links exact revisions, commands, artifacts and
passing results for A1–A8. A package bump, green unit suite, closed issue or
unchanged ABI does not substitute for this evidence.

## Progress discipline

Update this current-work list and the evaluation scorecard when scope changes.
Append dated evidence to the [history](archive_v4_history.md), keeping source,
commands, failures and limits reproducible. Historical “next” notes are not the
active plan. A completed component or resource win does not close a release gate.
