# Branch cleanup — 24 October 2026

The user authorized merging or deleting stale branches, retaining only explicit
holdbacks. No release or security code was pushed. Local main remained
`7245ec7323193ecd77026b36bbafd66c51fc99f5` during the audit; remote main remained
`26ca0bd61be5f4fa6e8ffe892c1fd5fad0275f32`.

Deleted 115 remote branches and 35 local branches. The machine-readable
[deletion inventory](branch-cleanup-2026-10-24.json) records every ref, exact tip
and disposition. Existing worktrees were detached at the same commit when
necessary; no worktree or unfinished tracked/untracked edit was removed.

## Evidence

- 89 of the 160 audited refs were ancestors of local main; additional branches
  had only patch-equivalent commits.
- Aggregate patches for code-quality remediation, #189 and #233 exactly matched
  squash commits `1ef1e734`, `9b8aea5b` and `d81c6d39`, respectively.
- #270 is integrated as `08024bb3`, adapted to the target-first CLI.
- #4142's Java-only plaintext opening is superseded by the integrated binding
  implementation across languages.
- Dependency updates were already integrated or superseded. The obsolete
  sysinfo update targeted a dependency removed from the active implementation;
  remaining historical exporter dependencies must stay frozen.
- Old release and candidate branches contained obsolete version preparation,
  generated API documentation, or CI adjustments superseded by current main.
  Existing release tags were preserved.

Remote deletion used exact expected-tip leases and an atomic push. The original
GitHub ruleset was saved, only the reviewed refs were temporarily excluded, and
the original protection was restored and verified after deletion.

## Intentionally retained

| Branch | Reason |
| --- | --- |
| `main` | Integrated development and release source; private release work remains local. |
| `issue-226-reciprocal-exchange` | Active full reciprocal exchange integration; not yet ready to merge. |
| `feature/browser-integration` | Experimental browser-authorized delegation; substantial changes against older APIs need separate integration and security review. |
| `feature/phone-device-approval` | Unfinished device-approval work, including storage/migration changes, needs a deliberate design and compatibility review. |
| `issue-28-wasm-unlock-design` | Browser unlock design retained with the browser experiment; not presented as current implemented behavior. |

These are the only five branch names remaining locally and remotely. The local
phone branch also preserves a later patch-equivalent commit than its remote
backup. No unreviewed feature was merged merely to eliminate its branch.
