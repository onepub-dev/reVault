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
