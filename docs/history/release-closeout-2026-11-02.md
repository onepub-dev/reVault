# Release closeout checkpoint — November 2, 2026

This continues the [November 1 checkpoint](release-closeout-2026-11-01.md).

## Integrated release corrections

- #346 consolidates crates.io publication under the top-level
  `revault-crates-release.yml` workflow and the `release` environment. CLI and
  binding build pipelines remain separate. Candidate verification, dependency
  ordering, partial-publication recovery and immutable public versions remain
  enforced. All 69 tooling unit tests and 21 integration tests passed; workflow
  lint introduced no new findings. The operator confirmed updating all fifteen
  active crates' trusted publishers. Actual registry authorization remains to be
  exercised by the next publication.
- #345 stamps only explicitly marked installation examples in packaged READMEs.
  Dependency versions and historical Ruby runtime notes remain unchanged.
  The Rust source guide uses published 0.6.0 until the generated 0.6.2 package is
  public; Rust 0.6.1 was not published. All 71 tooling unit tests, 21 integration
  tests and the 56-page manual check passed.
- Concurrent dependency updates #335 and #337 are integrated. The rcgen 0.14
  update required changing the HTTPS test's `key_pair` access to `signing_key`.
  The actual local HTTPS certificate/restart test and key-server all-target
  compilation passed in the current main worktree with a fresh target directory.

## Candidate and disclosure state

Candidate `41306873278` was cancelled before publication after the README
stamping defect was found. The Windows step marked unsuccessful during that
cancellation returned no diagnostic logs; no independent test failure was
established from it.

Fresh bindings 0.6.2 candidate
[41361552039](https://github.com/onepub-dev/reVault/actions/runs/41361552039)
uses source `3e39710f20473514db9b3a183783b7eabbc300e6` and immutable ref
`release-candidates/3e39710f2047-1793575171355512884`. At this checkpoint its
guard has passed and platform/installed-package qualification is running.
It is not yet qualified or published.

The eight private advisory drafts now contain the prepared release guidance.
Their affected-version and first-fixed-version mappings were preserved.
They remain private pending successful publication and public-package checks.
No open Dependabot alerts affect active manifests; the remaining alerts concern
the two retained historical evidence snapshots, which were not rewritten.

Merged issue branches 342, 344, 345 and 346 and their clean worktrees were
removed. Exact temporary deletion exemptions were removed and repository
branch protection restored. Candidate refs remain available for provenance.
No production deployment was performed.

## Later November 2 corrections and integration

Main now includes #277 Profile rename, #312 remembered-Lockbox resolution,
#331 numeric extraction limits (including short binary suffixes such as `10M`),
and #348 relative symlink restoration. The integrated CLI changes passed seven
focused CLI tests and the 56-page manual validator. The symlink correction then
passed 17 API extraction tests and five CLI extraction tests. It rejects restored
targets outside a selected subtree before writing output and does not change
the archive format.

The #215 dependency audit and #347 individual alert dispositions are integrated.
Historical migration dependencies are explicitly excluded from the pure-Rust
objective; replacing ring when a qualified alternative becomes viable remains
an open goal. The completed branches were removed with exact temporary deletion
exclusions, and the original repository rule was restored.

Candidate `41361552039` completed unsuccessfully. Earlier watcher summaries
missed failures already present in that run; the final API result and direct job
logs supersede those summaries. Both Linux native jobs lacked `cmake`; Clippy
rejected `Iterator::last` on a double-ended iterator; the CLI coverage gate lacked
session-protection commands because `headless_init` was absent from its runner.

#349 adds the missing native prerequisite, fixes the iterator, includes
`headless_init` and `extraction_limits` in the required CLI runner, and names
commands in spawn-failure diagnostics. Validation passed five headless tests
with successful command coverage, two xtask E2E tests, six tooling E2E tests,
strict Clippy and workflow YAML parsing. The cached Debian image was confirmed
to lack cmake. Actionlint was unavailable locally.

Replacement bindings 0.6.2 candidate
[41392531422](https://github.com/onepub-dev/reVault/actions/runs/41392531422)
uses main source `c3ee581695259267f26f58205db2eeffdcbce559` and immutable ref
`release-candidates/c3ee58169525-1793596270708044619`. Qualification is running;
no publication or disclosure is claimed. The eight advisories remain private
until package availability and the coordinated closeout requirements are met.

Candidate `41392531422` was cancelled after its source and installed-API checks
reported the new `rename_profile` method missing from the public API review
manifest. Both native Linux conformance jobs and strict Rust formatting/Clippy
passed. The method is now explicitly classified with the existing CLI-only
mixed key/password-profile lifecycle operations; no foreign rename operation is
claimed. The complete binding-surface check passed (375 public Rust names,
241 ABI declarations and 240 operations).

The active replacement is
[41393746205](https://github.com/onepub-dev/reVault/actions/runs/41393746205),
from main `e9414556b89c885317dbb09c58585d3b3da87e8c`, immutable ref
`release-candidates/e9414556b89c-1793597004630157232`. It remains qualification,
not publication.

The original candidate's secure-API job was not hung: it passed at
2026-11-02 03:23:46 UTC after approximately four hours. Its 732 core unit tests
took 14,224.53 seconds; 20 tests were ignored. Many recovery/interruption tests
across archive modes accounted for the duration. No required tests were removed
or reclassified to accelerate the replacement candidate.
