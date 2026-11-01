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
