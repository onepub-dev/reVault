# Complete interrupted tree-copy admission

The test-only shared-tree adapter can now resume installation of an explicitly
selected, complete replacement in the source directory. It does not discover,
adopt or erase arbitrary partial temporary files. The source and candidate must
be distinct regular files; symlinks, same paths and (on Unix) same-inode hard
links refuse. Exclusive source and replacement locks remain held during validation,
rename and directory synchronization.

A valid successor alone is insufficient: an ordinary edit is also a successor.
Admission requires the current source commitment, exactly the next generation,
exact sealed physical lengths, identical selected payload counts and deterministic
compaction placements. All source and candidate payload extents are rehashed in
at most 64 KiB guarded chunks. Only physical placements are normalized before
comparing every typed catalogue row, retaining files, nodes, permissions, normal
and secret variables, definitions and historical form captures. Both publications
are rechecked after validation. Authentication follows the selected mode; checksum-
only plaintext does not acquire owner authentication from this adapter.

On admission, source permissions are applied and the candidate synchronized before
rename. Path identities are checked immediately before publication. A failure
never erases either file. An error after rename leaves the installed successor;
a failed parent sync reports uncertain durability. A missing candidate after an
already completed rename is an error, not an inferred successful retry receipt.

Five focused tests pass, with 48 fresh-process exits across all 16 modes at verified,
installed and directory-synced boundaries. Fresh parent readers verify complete
state; pre-rename process loss retries the same candidate successfully. Additional
cases refuse truncated, stale-source, ordinary edited, corrupt and trailing-byte
candidates without changing either file. All-mode normal completion and a repeat
with a missing candidate are checked. Strict library/test/bench Clippy passes.
Raw outputs: [tests](test.log), [Clippy](clippy.log).

Commands use pinned Rust 1.88.0 from the performance worktree's rust directory:

```text
cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --lib whole_tree_resume -- --nocapture --test-threads=1
cargo +1.88.0 clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
```

The tests use private synthetic fixtures because no public CLI writes this format.
These are process-exit tests, not physical power-loss qualification. Stable storage
and cooperative locks remain preconditions. This is complete-copy admission, not
a durable ownership journal: partial-copy cleanup, post-rename receipts, automatic
recovery discovery, access roots, cross-platform durability and complete CPU/RSS/
aging/size qualification remain outstanding. No public format/API is activated.
