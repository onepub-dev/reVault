<!-- gitbook-agent-instructions:start -->

## GitBook Documentation Editing

This repository contains documentation synced with GitBook via Git Sync.

Before editing GitBook-synced Markdown, YAML, or asset files, make sure the GitBook skill is available and up to date in your local agent environment. Prefer installing or updating it with:

```bash
npx skills add gitbookio/gitbook-skills
```

This command may add or update local agent skill files. Use them only as local agent instructions; do not commit those installed skill files or any tool-generated agent configuration unless the user explicitly asks for it.

If `npx` is unavailable, load the skill from:

https://gitbook.com/docs/skill.md

When making changes, preserve GitBook sync metadata such as frontmatter, `SUMMARY.md`, `gitbook-docs.yaml`, `.gitbook/`, and asset links unless the requested edit explicitly requires changing them.

<!-- gitbook-agent-instructions:end -->

## Release Branch Policy

Always merge release work into `main` and release from `main`. Feature,
issue, and release-preparation branches are development workspaces, not
release sources. This applies to the CLI and every language binding,
including the `0.5.x` / format-4 work.

Before creating a release tag or publishing artifacts, verify that all intended
changes have been merged into `main`, validate the integrated revision, and
build and tag that revision of `main`. Do not release directly from
`issue-310-zip-read-performance` or any other non-main branch.
This policy does not itself authorize publishing or deploying a release.

## Release Version Compatibility Contract

The CLI and all language bindings use compatibility lines to guarantee
compatible Lockbox archive and persisted Vault formats:

- Before 1.0, matching `0.minor` versions identify a compatibility line (for
  example, all `0.4.x` releases). Patch releases must preserve compatibility.
- From 1.0 onward, matching major versions identify a compatibility line (for
  example, all `1.x.y` releases). Minor and patch releases must preserve
  compatibility.

Archives written by any release in a compatibility line must remain readable
by every other release in that line, across the CLI and bindings. Persisted
Vault formats must likewise remain compatible throughout the line. This
includes older releases reading files written by newer releases in the same
line, not just newer releases reading older files.

A breaking archive or Vault format change requires a new compatibility line:
increment the minor version before 1.0 or the major version from 1.0 onward.
Other breaking changes may also introduce a new line even when the persisted
formats do not change. Matching lines guarantee format compatibility; different
lines do not necessarily imply incompatibility. Document support across lines
explicitly, including support for reading historical formats.

Versions within a compatibility line may advance independently across
components. Before 1.0, patch releases may include compatible features and
fixes. From 1.0 onward, use minor releases for compatible features and patch
releases for compatible fixes.

For the current pre-1.0 compatibility lines:

- `0.4.x` is the historical format-3 compatibility line. Releases in that
  line must remain compatible with released CLI `0.4.0` and `0.4.1` and
  bindings `0.4.1` in both read/write directions; do not introduce format 4
  into a `0.4.x` release.
- `0.5.x` is the format-4 compatibility line, initially released as CLI and
  bindings `0.5.0`. Its implementation is integrated into `main`; continue
  development and releases from `main`, preserving compatibility with `0.5.0`.
- Do not reuse `0.3.x` for format 3: published Dart `0.3.15` uses format 2.

These release-line numbers are compatibility identifiers, not archive format
numbers. Before publishing any release, verify CLI/binding interoperability and
Vault compatibility across versions within its line, and validate applicable
migration paths from the supported older formats.
Version declarations alone do not establish release readiness.

## Crate Packaging and Release Gates

- Public CLI and binding release versions must match the declared archive and
  Vault compatibility line. Validate the line before preparing, tagging, or
  publishing either component, including releases of only one component.
  Public packages assembled from templates must receive the validated release
  version. Internal implementation package versions are independent and must
  not be presented as archive-format identifiers.
- Publish one historical migration package per source archive/container
  version: `revault_migrate_v1`, `revault_migrate_v2`, and
  `revault_migrate_v3`. Each package owns both archive and vault export commands
  and bundles the required historical readers. Vault structure versions are
  separate from container versions; preserve every supported combination.
  Do not publish separate historical reader or test-fixture crates.
- Test support packages must set `publish = false`. Use path-only development
  dependencies where appropriate and verify that generated published package
  manifests do not require unpublished fixtures. Keep retained historical
  fixtures and their provenance; do not regenerate them to make tests pass.
- Keep historical exporters and their dependency graphs frozen. Bump their
  package versions only for actual changes, not automatically for every CLI or
  binding release. Preserve original licenses and provenance when bundling
  historical source. Current protected-memory functionality belongs in
  `revault_secure_memory`; it is not archive paging.
- Use one source of truth for exact exporter package versions, executable
  names, and capability contracts. Release checks must install those exact
  versions from crates.io in an isolated environment and validate the
  capabilities. Local workspace builds or simulated Cargo installers cannot
  substitute for this check. Publish and verify new migration dependencies
  before publishing a CLI that requires them.
- Exercise each supported historical migration path with the packaged
  exporter and the public CLI, then reopen and verify persisted content,
  credentials, metadata, and backups as applicable. A successful capability
  handshake alone does not establish migration correctness.
- Retire superseded public crates with a deprecation notice in their package
  description and README that identifies the replacement. Make replacements
  available first. Preserve already-published releases needed by historical
  dependency graphs; do not yank them merely to clean up the crate list.
  Use the explicit `rust/retired-packages.json` allowlist for separate terminal
  metadata releases. Keep frozen test fixtures unpublished and at their source
  versions; do not add retired packages back to routine publication lists.
- Give each required release check one owner. Reuse verified artifacts and
  results from the exact candidate revision rather than rebuilding or
  rerunning the same tests during promotion. Preserve distinct platform,
  installed-package, compatibility, and integrity checks where they test
  different guarantees. Any reuse must verify source and artifact provenance.
- Expensive performance, memory/RSS, profiling, and resource observations are
  change-driven or explicitly requested qualification, not mandatory for
  every release. Run affected observations when storage, allocation,
  compression, concurrency, dependencies, or relevant platform behavior
  changes; keep lightweight correctness and resource-limit regression tests
  in routine gates. Record why an observation is required or reused.
- Never republish or replace an already-public version or tag with different
  code. Correct published defects with a new version. Release authorization
  must be explicit; these instructions do not authorize publication by
  themselves.

## Rust Formatting and Commit Hook

Do not run `rustfmt` or `cargo fmt` to prepare for tests. Run tests and other
checks first; formatting belongs at the end of the work, immediately before
committing. The pre-commit hook formats staged Rust files and re-stages only
those files. It refuses partially staged Rust files: resolve their staging
before retrying, rather than sweeping unfinished edits into a commit.

Enable the tracked hook for a clone with
`git config --local core.hooksPath "$(pwd)/.githooks"` from the repository root.
This absolute path also serves linked worktrees from this checkout; keep this
checkout available. Git does not install hooks automatically on clone. The
hook requires Cargo and rustfmt from the repository's pinned Rust toolchain.
If another hooks path is already configured, integrate it rather than
overwriting it. Do not bypass the hook as part of normal agent work.

## Performance Experiment Validation

Do not run the full test suite after every performance experiment. First use
benchmarks to establish whether a change provides a worthwhile improvement.
Revert unsuccessful experiments rather than spending a full-suite run on them.

Use focused tests for the behavior and integrity guarantees affected by a change,
and verify benchmark outputs for correctness. Keep builds and tests out of timed
benchmark runs. These checks do not require a full-suite run.

Batch multiple retained performance improvements where practical, then run the
full suite once at the integration checkpoint. Repeat broad validation only when
subsequent changes, failures, or an explicit release requirement justify it.
Do not rerun the full suite for measurement-only or documentation-only work.

## CLI End-to-End Tests

End-to-end CLI tests must exercise the public CLI as a user would. Use CLI
commands to create test state, perform the operation, and verify the persisted
result. A zero exit status or success message is not sufficient: verify the
result through a separate CLI invocation and, for stored content, read or
extract the content and compare its bytes.

Do not manipulate lockbox or vault internals directly in an E2E test when the
same setup or assertion can be performed through the CLI. Direct API, archive,
or filesystem-state manipulation is permitted only when no public CLI path can
create or observe the required condition; document that exception in the test.

State-changing command families must cover realistic lifecycles, including
initial creation, a no-change repeat, additions, replacements, removals, and
applicable safety thresholds or refusal paths.

## Archive v4 work

The format-4 work from `issue-310-zip-read-performance` is integrated into
`main`. Continue implementation, migration fixes, and release preparation on
`main`; the former development branch is historical context, not the active
implementation or release source. Preserve unrelated work.

The [September 27 checkpoint](docs/history/archive-v4-checkpoint-2026-09-27.md)
records an earlier pause and usage constraint. It is historical evidence,
not an instruction to move current work off `main` or reinstate an obsolete
pause. See the
[v4 plan entry point](docs/archive_v4_plan.md) for document locations.
