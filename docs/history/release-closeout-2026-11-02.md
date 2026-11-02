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

## Android acceptance blocker

Candidate `41393746205` was cancelled after its Android 16 KiB acceptance job
exceeded the 45-minute limit. Native C/C++ acceptance passed on that emulator;
the following Flutter acceptance command produced no output before timeout.
The old runner buffered child output, so these logs do not identify whether
compilation, installation or runtime operation stalled. The 4 KiB job passed.
The candidate's remaining secure-API job was deliberately cancelled, not failed.
Its final result was 57 passed jobs, three cancelled and 18 skipped.

#350 adds live Flutter output, bounded subprocess execution, failure logcat and
non-secret lifecycle markers. Local analysis, compilation, subprocess success,
failure and timeout probes, and workflow parsing passed. These checks do not
establish Android runtime correctness.

Focused diagnostic
[41400976269](https://github.com/onepub-dev/reVault/actions/runs/41400976269)
uses retained publication trees from `41393746205`, with diagnostic code from
main `108d6228aad78acc3e5befe011f8aa61d5be3802`. It cannot publish or replace
release qualification. Another candidate is required after resolving this
blocker. No 0.6.2 package or advisory publication is claimed.

## Later Android diagnosis

Candidate `41401952839` (main `b15d1f92b46bab3a392bc68a41b67801cc4f89e9`)
was cancelled after Android 16 KiB Flutter acceptance timed out. The APK built
in 177.6 seconds and installed in 577 ms, but no test started before the
20-minute command deadline. Native C/C++ acceptance passed. This locates the
stall after installation; it does not establish its underlying cause.

Diagnostics now include verbose Flutter output, full Android logcat, activity
and window state on failure, native crash-buffer capture, and retained native
executables with ELF headers. Local Dart analysis/compilation and workflow
parsing passed. The first combined diagnostic failed during emulator readiness.
Subsequent diagnostic-only native crashes used inherited NDK 27.3 despite
installing NDK 28; the release workflow already explicitly selected NDK 28.2.
The diagnostic was corrected to match that selection. Those crashes must not
be attributed to the release toolchain.

Both fresh-runner attempts of corrected diagnostic
[41409573929](https://github.com/onepub-dev/reVault/actions/runs/41409573929)
passed the combined C/C++ and Flutter lifecycle on 16 KiB pages, using retained
packages from `41401952839`. Flutter completed in approximately 155 and 178
seconds. Retained executables show 16 KiB ELF load alignment. The original
Flutter launch stall was not reproduced or conclusively explained; #350 remains
open. Another full candidate must pass before publication. No release gate was
removed, and no package or advisory was published during diagnosis.

## Qualified 0.6.2 candidate and partial publication

Candidate [41411932831](https://github.com/onepub-dev/reVault/actions/runs/41411932831)
passed from main `a3fcdbd9559ebed61557d5ac7d6ad102785100e4`. Both Android
runtime gates passed. The secure-API job passed in 3h56m; its API unit binary
took 13,868.10 seconds, with 733 passed and 20 ignored. Across that job, 819
tests passed and 21 were ignored. Preserve this run's timestamped log for the
user-requested test-runtime audit after publication, covering duplicate work,
fixture reuse and other improvements without losing important coverage.

Authorized publication run
[41443362028](https://github.com/onepub-dev/reVault/actions/runs/41443362028)
and nested run `41443487958` completed unsuccessfully only because Rust
publication was rejected. All other publication/promotion jobs, including
Java and Kotlin Maven publication, succeeded. The crates workflow successfully
uploaded `revault_lockbox_api` 0.0.66, then crates.io rejected
`revault_publish_protocol` 0.0.60 with a crate-specific 403. Token exchange had
succeeded less than a minute earlier. `revault_secure_memory` 0.1.0 was already
public and was reused. The public Rust facade 0.6.2 remains unavailable.

The operator has been asked to verify that crate's trusted publisher:
repository `onepub-dev/reVault`, workflow `revault-crates-release.yml`,
environment `release`. Browser access is unavailable to the agent, and the
registry's configuration API requires authenticated crate-owner access.
Once corrected, resume with:

```bash
revault-tool release publish --candidate 41411932831
```

The release driver resumes failed publication jobs and preserves already-public
versions. No new candidate or repeated four-hour qualification is needed for
this registry configuration correction. Never replace the public versions or
move their tags.

Initial public-package verification confirms the sealed candidate's artifact
digests and source/version markers. All eight npm integrity values and all six
Ruby platform gem hashes match the frozen candidate payloads. NuGet adds its
registry signature; its signed catalog hash verifies and its remaining package
contents match the candidate, including the recorded source commit. Remaining
registry propagation checks and Rust publication must finish before coordinated
advisory disclosure. The eight advisories remain private. No production server
deployment was performed. The fully merged Android diagnostic worktree/branch
was removed; #350 remains open for the unexplained intermittent startup stall.

Follow-up registry checks completed: Maven Java/Kotlin 0.6.2 are available;
Packagist lists v0.6.2 with the PHP tag's packaging commit; Go proxy info, mod
and ZIP are available, the ZIP passes integrity checking, and sum.golang.org
records its checksums. The earlier Go timeout and Kotlin propagation gap are
resolved. No local Go consumer ran because this environment has no Go binary.

The public Go and PHP tag trees match the sealed candidate's prepared trees
byte for byte. Swift's differences are its promoted manifest and header line
endings; its XCFramework reference/checksum matches the public release asset.
Homebrew resolves the candidate template to 0.6.2 with four platform asset
hashes matching the release. These companion tag commits are packaging commits,
not the main source SHA. Evidence is retained locally under
`/tmp/revault-062-public-check/`. Rust trusted-publisher authorization remains
the publication blocker; advisory disclosure and the test-runtime audit remain
pending completion of publication.
