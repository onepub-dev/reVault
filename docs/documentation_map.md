# Documentation ownership and cleanup

Reviewed: 2026-09-26. This is the documentation maintenance plan, not a format
specification. The [project goals](../manual/project-goals.md) define purpose;
the [v4 plan](archive_v4_plan.md) defines the current engineering sequence.

## The documents we need

Keep one authoritative document for each question. Link to it instead of
copying its requirements into another overview.

| Question | Authoritative location | Action and owner role |
| --- | --- | --- |
| Why does reVault exist, and what outcomes matter? | [Project goals](../manual/project-goals.md) | Created. Product maintainer owns changes to goals and priorities. |
| What work happens next, and what blocks v4? | [V4 plan](archive_v4_plan.md) | Created. V4 maintainer keeps milestones and evidence current. |
| What threats and guarantees does each mode cover? | [Security model](security_model.md) | Draft created; security review and protocol decisions outstanding. Existing audit notes are inputs, not the normative model. |
| What exactly is stored, and what must readers/writers do? | [ARCHIVE_FORMAT.md](../rust/revault_lockbox_api/ARCHIVE_FORMAT.md) | Storage maintainer replaces the historical body with the selected v4 specification after the design decision. Include encodings, limits, commitments, version/feature negotiation and conformance vectors. |
| How do transactions and recovery work? | [Transactions and recovery](../manual/develop-with-revault/transactions.md) | Retain and update. Storage maintainer owns the phase/order contract; the wire specification links here rather than maintaining a second prose protocol. |
| How do we compare candidates and prove the release gates? | [Evaluation contract](archive_v4_evaluation.md) | Draft methodology, proposed resource budgets and current scorecard created; freeze before comparative timing. |
| Why was a consequential design chosen? | [Signing/recovery](decisions/001-v4-signing-and-recovery.md), [layout](decisions/002-v4-layout-and-access-units.md), [allocation/history](decisions/003-v4-allocation-history-and-reclamation.md) | Proposed records created. None is an accepted protocol change yet. |
| Which versions interoperate, and how do users migrate? | [Compatibility](../manual/develop-with-revault/compatibility.md), [migration procedure](../manual/maintain-and-recover/migrating-between-versions.md), [engineering migration contract](format_versioning_and_migrations.md) | Release maintainer reconciles these distinct public/protocol/release roles and the repository compatibility contract. Keep version matrices in one place, with links elsewhere. |
| How do people use the product? | Pages listed in [manual/SUMMARY.md](../manual/SUMMARY.md) | Documentation maintainer retains the task-oriented manual; API generation remains the signature authority. |
| Which installation/session capabilities work on each platform? | Installation and session pages in the manual; H1's platform matrix in `docs/archive_v4_evaluation.md` | CLI/platform maintainer documents headless and desktop support, optional credential stores, native dependency requirements and tested exceptions as part of [#322](https://github.com/onepub-dev/reVault/issues/322). |

The security/evaluation documents and decision drafts now exist. Their proposed
status is deliberate: an unresolved design has not been approved. Codex owns the
initial drafts and evidence; product/security/release reviewers remain unassigned.

## Cleanup inventory

| Existing material | Disposition | Required cleanup |
| --- | --- | --- |
| Root README and manual introduction | Keep as entry points | Link the goals page. Describe capabilities briefly; do not maintain a second goals list or release checklist. |
| `docs/implementation_overview.md` | Historical; replace or retire after v4 selection | Its v2/fixed-page descriptions and separate goals list are not current authority. Preserve useful component explanations in a concise implementation map linked to the spec, then replace obsolete body with a pointer. |
| `rust/revault_lockbox_api/ARCHIVE_FORMAT.md` | Keep path; historical body pending replacement | The 96-byte/version-1 header description is not the v4 dual-slot header. Preserve the old revision through Git history or an explicitly versioned historical spec before replacement. Do not declare the existing body to be a complete historical spec without checking it. |
| `docs/performance_review.md` | Historical assessment | Move still-relevant requirements into the evaluation contract. Label measurements by date/revision; retire its competing “current” bottleneck and next-work lists. |
| `docs/design_discussion.md` | Historical idea collection | Extract still-relevant decisions into decision records. Move unrelated future ideas to a backlog; remove the obsolete running implementation narrative after links are updated. |
| `docs/archive_v2_restructure_proposal.md` and summary report | Retain as historical proposal/evidence | Its “v2” is a historical proposal label, not the current release target. Re-evaluate its hypotheses; do not relabel its old measurements as v4 results. |
| `docs/security_audit.md`, key-management, secure-memory and dedupe notes | Retain as audit/design inputs | Consolidate normative guarantees into the security model. Mark old algorithm and implementation claims by revision; an internal audit is not an independent security certification. Dedupe remains separately scoped. |
| `docs/transaction_recovery.md` | Keep as a short pointer and code map | Already delegates the maintained protocol to the manual; use this pattern for other duplicate overviews. Verify source links as implementation changes. |
| `docs/format_versioning_and_migrations.md` | Update in milestone P1/P5 | Remove v3-as-current and retained-reader claims that conflict with the v4 core. Reconcile exporter registry/publication order, fixture readers, and Vault structure versus container versions. |
| `docs/archive_v4_history.md` | Keep as historical evidence | Dated checkpoints moved out of the active plan/scorecard. Their old next-work statements are superseded by the current plan. |
| `docs/benchmark_history.md`, dated benchmark/encoder reports | Preserve evidence | Add an index from the evaluation document. Keep conditions, rejected experiments and raw data; do not rewrite historical findings to match newer results. |
| Issue-310 `benches/results/issue310-*` and resume notes | Preserve branch evidence | Import a curated evidence index tied to exact commits. Resume notes document previous work; the v4 plan controls what happens next. Avoid promoting the full experiment diary into the normative spec. |
| Hidden `manual/docs/*`, `manual/rust/*`, `manual/epage_file/*` | Audit for duplicate/legacy material | These are not listed in the current published navigation. Compare against source counterparts, retain unique information, redirect incoming links, then remove duplicate bodies. Absence from navigation alone is not proof that removal is safe. |
| Older manual trees outside the published navigation | Consolidate in a separate cleanup change | Audit `manual/cli-tooling/`, `manual/apis/`, top-level mirror and key-sharing pages against their published replacements. Some tooling still reads old paths, so update validators and links before deleting them. |
| `manual/.gitbook/vars.yaml` | Reconcile with release scope | Baseline validator currently reports CLI `0.0.14` versus manifest `0.0.18`, and key server `0.0.7` versus `0.0.37`. Resolve which release the manual describes before changing values. |
| GitHub issues #303, #310 and #313 | Keep provenance; reconcile tracking | Preserve original requirements and reproductions. #310 has been reopened and marked in progress; its acceptance gates remain open. Issue state is not evidence that gates passed. |
| #322, CLI installation and Auto Open documentation | Fix and document independently of format selection | H1 in the v4 plan covers build dependencies and runtime availability on headless/desktop Linux, macOS and Windows. Document secure explicit-credential alternatives and retain a minimal-environment CI lane. |

## Cleanup sequence

1. Establish the goals, plan and document ownership now. Add prominent historical
   notices to the most misleading entry points. Keep historical bodies available.
2. In P1, write the security model and evaluation contract, record design
   questions, and reconcile the migration/compatibility narrative. Label all
   claims as current, proposed, historical or unverified.
3. In P3, replace the wire specification with the selected design and record why
   it was selected. Close or supersede alternative design narratives explicitly.
4. In P5, finish the release matrix, migration and binding instructions. Align
   the public manual's version scope and variables with the release it describes.
5. Remove redundant bodies only after their unique information, incoming links,
   GitBook paths and validator dependencies have a replacement. Use short pointer
   pages where old URLs remain useful. Never remove fixtures or benchmark evidence
   merely to make the tree look tidy.

## Maintenance rules

Normative documents state their scope and version. Plans keep status separate
from acceptance criteria. Decision records say proposed, accepted or superseded.
Evidence includes commit, feature flags, workload, environment and limitations.
An unresolved numeric target remains visibly unresolved; it cannot count as a
passed gate.

Changes to a guarantee update the goals/security contract and the affected
specification or decision record together. Ordinary optimization results update
the evaluation scorecard, not the project goals. Validate relative links,
published navigation, preserved frontmatter and the existing manual checks.

This cleanup establishes authority without deleting evidence. It does not claim
that all historical documentation has already been reconciled.

The [selected-source evidence](evidence/selected-source-2026-10-09/README.md) retains
authenticated stored-source preparation, final frozen aggregate outcomes and
modeled failure controls; its component scope does not replace the format gates.
