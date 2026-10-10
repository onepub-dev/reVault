# Archive v4 work and planning

This page locates the v4 work; it is not a new implementation plan or an
assertion that a format has been selected or qualified for release.

## Main remains on format 3

Main continues the `0.4.x` compatibility line. The v4 transaction-recovery
implementation introduced by `4954bf11` and merged through `7711ff9b` is deferred
to `issue-310-zip-read-performance`, where it is already present in the branch
history. Main retains format-3 cleanup recovery, but not v4 preparation rollback,
tail reclamation or the public verified-compaction command.

This separation does not downgrade existing format-4 Lockboxes or Vault
containers. Use a matching v4 build to read or recover those files; keep their
originals and backups. Format-3 commands reject newer containers and cannot
resume a migration whose destination is format 4.

The performance worktree and its active changes remain authoritative for v4.
Reintegrating it will require explicit format selection, compatibility and
migration validation, and release qualification.

The implementation branch is `issue-310-zip-read-performance`. Its plan and
evidence are ahead of the old main-checkout draft. Use `git worktree list` to
find the existing checkout, then read these files **on that branch**:

- `manual/project-goals.md` — the goals used by that branch's evaluation.
- `docs/archive_v4_plan.md` — implementation sequence and current recorded scope.
- `docs/archive_v4_evaluation.md` — acceptance criteria and retained evidence.
- `docs/decisions/` — proposed and accepted design decisions.
- `docs/archive_v4_history.md` — previous branch checkpoints.

The [branch plan](https://github.com/onepub-dev/reVault/blob/issue-310-zip-read-performance/docs/archive_v4_plan.md)
is available on GitHub when that branch has been pushed. For local work, the
existing worktree and its actual Git state take precedence over remote status.

Main's [project goals](../manual/project-goals.md) now use G1–G11. Historical
plans and branch evidence may use earlier numbering. Reconcile goal references
explicitly before updating requirements; do not silently reinterpret old gates.

## Historical context

- [September 26 planning baseline](history/archive-v4-plan-2026-09-26.md).
- [September 27 pause and handoff](history/archive-v4-checkpoint-2026-09-27.md).

These snapshots preserve earlier requirements, work locations, evidence and
limitations. Their branch heads, pending changes and milestone states are dated.
Reading or relocating them does not resume the paused implementation or
establish release readiness. Recheck the applicable pause/usage constraint and
actual branch state on an explicit resume.

See the [documentation map](documentation_map.md) for maintained entry points
and the distinction between product goals, implementation plans and history.
