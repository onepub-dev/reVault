# Main integration — 2026-10-09

Merge of performance `8c196c5c` with main `6343b3fe`. Code merges without
conflicts; documentation resolutions retain the advanced branch plan/evidence
and main's approved goals, release/hook guidance and documentation cleanup.
Historical G1–G7 references in active plans/decisions map to the new G1–G11 goals.
G3/G11's scale, ZIP/PGP read/write and 100 MB total CLI RSS requirements remain
unqualified; older incremental resource proposals do not supersede them.

Pre-commit validation uses Rust 1.88.0 and the performance worktree's own target:

* CLI aliases and completion integration suites: 8 passed each.
* Strict core Clippy with external-source, library/tests/benches: passed.
* Strict CLI Clippy, binaries/tests: passed.
* Six focused whole-tree compaction controls: passed.
* Relative documentation links: 341 across 64 changed/manual pages, 39 unique
  navigation entries; whitespace check allows the existing WiX CRLF convention.

The full serialized format run was intentionally interrupted during historical
form fault matrices because the merge does not change experimental file-format
code. Its partial log is retained and **is not a completed pass**. An initial
scoped run also selected the larger functional fixture and was interrupted;
the final six-control run explicitly excludes that fixture and the fault matrix.
No test assertions or source exclusions were weakened. No benchmark was run.

```text
cargo +1.88.0 test -p revault_cli --test aliases --test completion
cargo +1.88.0 clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
cargo +1.88.0 clippy -p revault_cli --bins --tests -- -D warnings
cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --lib whole_tree_compaction -- --test-threads=1 --skip whole_tree_compaction_returned_failures_durably_discard_owned_destination --skip whole_tree_compaction_large_selected_state_file_store_all_modes
```

The user explicitly resumed implementation, with a **25% remaining Codex usage**
stop condition replacing the older 70% threshold. Existing native recovery
failures and all whole-format qualification gates remain open.
