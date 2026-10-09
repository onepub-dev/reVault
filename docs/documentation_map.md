# Documentation ownership

Updated: 2026-10-09. Use one source for each kind of information. Historical
proposals and measurements are evidence of earlier work, not current contracts.

| Question | Source |
| --- | --- |
| What outcomes and boundaries matter? | [Project goals](../manual/project-goals.md), G1–G11. |
| How do users operate reVault? | Pages listed in [manual/SUMMARY.md](../manual/SUMMARY.md). |
| Which releases interoperate and how do users migrate? | [Compatibility](../manual/develop-with-revault/compatibility.md), [migration procedure](../manual/maintain-and-recover/migrating-between-versions.md), and the release contract in [AGENTS.md](../AGENTS.md). |
| What are the transaction and recovery guarantees? | [Transactions and recovery](../manual/develop-with-revault/transactions.md). |
| Where is current archive-v4 work? | [V4 work entry point](archive_v4_plan.md), which identifies the implementation branch and its plan/evidence. |
| How are code changes prepared? | [AGENTS.md](../AGENTS.md) and the tracked commit hook. |
| What did earlier investigations find? | [Historical records](history/README.md), [benchmark history](benchmark_history.md), dated reports and the implementation branch's retained evidence. |

## Cleanup completed

- Moved the September 27 handoff out of agent instructions and into a dated
  historical record. Agent guidance retains a short work-location and pause pointer.
- Preserved the uncommitted September 26 plan as a historical baseline. Its old
  goal numbering and milestone states are no longer presented as current on main.
- Retired the obsolete implementation overview and design narrative. Their old
  paths remain useful entry points; Git history preserves the previous bodies.
- Kept the performance report as a historical record, without presenting its
  unversioned measurements as current results.
- Replaced the mixed-version migration narrative with a short contract and links.
- Retained the archive-format reference with an explicit historical notice.
  Its body has not been certified as a complete specification of a particular
  released format, and it must not be used as the v4 wire specification.

## Material to retain

Keep historical format references, migration fixtures, audit inputs, benchmark
corpora, dated reports and design evidence. Do not delete them just because
their conclusions have been superseded. Record producing revisions and limits
when available; do not invent missing provenance.

The hidden `manual/docs/`, `manual/rust/` and `manual/epage_file/` trees, and older
manual paths outside published navigation, still need a dedicated duplicate
audit. Some tooling or incoming links may depend on them. This cleanup does not
claim that those trees have been reconciled.

## Maintenance rules

Keep implementation status and experimental results out of the project goals.
Mark proposals, historical records and release-qualified claims distinctly.
Update incoming links before removing a path; retain a short pointer when an
old URL remains useful. Preserve GitBook metadata and navigation.

For v4, read the plan, decisions and evaluation documents in the implementation
worktree before changing them. Main's historical snapshots are not a substitute
for that branch's actual state, and documentation cleanup does not resume work.
