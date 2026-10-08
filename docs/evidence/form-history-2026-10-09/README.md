# Public form history and namespace baseline

Two integration tests use only the existing public API in all **16 archive
modes**, commit and independently reopen persisted bytes. Both tests and targeted
strict Clippy pass before formatting. No experimental form implementation or
production semantics changed in this baseline.

Updating one field advances the record's definition revision while untouched
values retain their captured label and kind, including a field absent from the
referenced revision. A removed secret field recreated as normal in a later
revision may remain a secret historical capture after another field advances
the record revision. Explicitly setting that recreated field replaces its value
and capture. Direct secret-to-normal definition or value changes refuse without
changing committed bytes. The accepted 36-dash FormTypeId is retained: the public
type accepts a normalized 36-character hex/dash string, not strict UUID syntax.

The separate namespace test preserves a file and form at the same path and a
form whose direct parent is a file. It also verifies that missing ordinary
parent directories are created for a new nested form path. Independent reopen
checks both file bytes and form values. These are observed compatibility states,
not a proposal to strengthen or change public filesystem behavior.

Typed normal-open admission must authenticate captured fields independently of
the current definition's field list. It must still require the exact referenced
type/revision and captured alias. The released migration import currently applies
stricter current-field membership/kind validation; that distinct policy must not
be substituted for normal historical record admission. Typed form snapshots,
selected ownership, full-size metadata, mutation/recovery and public activation
remain subsequent work.

Commands, from the archive worktree's `rust/` directory:

```bash
cargo +1.88.0 test -p revault_lockbox_api --release --features external-source --test form_capture_history -- --test-threads=1 --nocapture
cargo +1.88.0 clippy -p revault_lockbox_api --features external-source --test form_capture_history -- -D warnings
```
