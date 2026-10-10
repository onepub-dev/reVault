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

- `0.4.x` is the format-3 compatibility line: released CLI `0.4.1` and
  bindings `0.4.1`. Main records CLI `0.4.1` and bindings `0.4.2-dev.1`,
  excluding unfinished format-4 work. Keep compatibility with released CLI
  `0.4.0` and all other releases in this line in both read/write directions.
- `0.5.x` is reserved for format 4 on `issue-310-zip-read-performance`.
  The v4 transaction-recovery implementation is deferred there alongside the
  performance work; publish a stable release only once format 4 is ready.
- Do not reuse `0.3.x` for format 3: published Dart `0.3.15` uses format 2.

These release-line numbers are compatibility identifiers, not archive format
numbers. Before publishing any release, verify CLI/binding interoperability and
Vault compatibility across versions within its line, and validate applicable
migration paths from the supported older formats.
Version declarations alone do not establish release readiness.

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

Archive-v4 implementation belongs on `issue-310-zip-read-performance` in its
existing worktree. Find it with `git worktree list`; read that branch's plan
and evidence before making implementation changes. Preserve unrelated work.

The [September 27 checkpoint](docs/history/archive-v4-checkpoint-2026-09-27.md)
records the earlier pause and usage constraint. Revalidate its state and the
applicable constraint on an explicit resume. Documentation maintenance does
not authorize resuming paused implementation. See the
[v4 plan entry point](docs/archive_v4_plan.md) for document locations.
