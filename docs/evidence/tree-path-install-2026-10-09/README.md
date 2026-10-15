# Whole selected-tree atomic path installation

The private test-only shared-tree adapter now installs the complete extent-copy
result through the existing atomic replacement primitive. It retains an exclusive
source file lock through rename and parent-directory synchronization, preserves
permissions, independently reopens and verifies the replacement, and checks both
source and temporary path identities before publication. No public format or API
is activated. The inherited copy refuses unsupported access roots.

Before rename, returned errors erase and synchronize only the owned temporary
inode, remove its path and synchronize the directory. Cleanup errors preserve
both causes. A replaced temporary path refuses cleanup instead of erasing the
substitute. After rename, errors preserve the installed archive; directory-sync
failure reports uncertain durability and requires reopening before continuing.

Six focused tests pass, including:

- All 16 modes: two consecutive installations retain lineage, stable repeated
  size, filesystem metadata, files, normal/secret variables, current definitions
  and historical form captures. Fresh handles reopen each result; subsequent
  mutation remains writable. Unix permissions are retained.
- Four representative modes at four returned-error boundaries: original bytes
  remain before rename; a complete successor remains after rename. No owned
  temporary path remains after successful cleanup.
- Missing signer, symlink and externally replaced source paths refuse safely.
  A substituted temporary path and the displaced complete archive remain intact.
- 96 fresh child-process exits: all 16 modes at temporary creation, copying,
  dependency construction, verification, rename and directory synchronization.
  A parent process independently verifies old or complete new selected state.
  These exits bypass Rust destructors; they are not physical power-loss tests.

The first run failed five tests because fixture transfer used an ordinary read
through a guard covering secret ranges. The child exited with status 101 rather
than the requested checkpoint status 71 for the same setup failure. Transfer now
copies the completed fixture's backing image, matching existing fixture patterns;
production guarded reads and installer logic were unchanged. The corrected run
passes six tests and 96 process exits; strict Clippy passes. Six existing whole-tree
copy controls also pass. Commands and initial/final results are in
[validation.txt](validation.txt).

This component does not adopt or erase abandoned temporary files after process
loss. Authenticated resumable temporary ownership remains a separate design task.
Both copies coexist until replacement; source-plus-destination space, CPU, peak
RSS, mixed aging, region-loss behavior and Windows/macOS durability are unqualified.
It assumes stable storage and cooperative exclusive locks, not protection against
arbitrary concurrent filesystem modification. No complete-format gate is closed.

Post-format validation on `f0dbd0b2` (installer `0b173f77` plus main alias fix
`ee3cbd39`) passes six installer tests, all 96 process exits and strict core
Clippy. The merged CLI passes 14 alias tests, eight completion tests and strict
CLI Clippy. Raw post-format and merged-CLI logs are retained alongside this
report. Main contains only the alias fix; experimental installer changes remain
on the performance branch.
