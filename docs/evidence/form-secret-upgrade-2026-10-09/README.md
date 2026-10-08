# Public cross-record form secret upgrade baseline

This is a public Rust API baseline, not a new typed adapter or production behavior
change. All **3 form_capture_history integration tests** pass, including the new
cross-record upgrade fixture in every archive mode; targeted strict Clippy passes.
No experimental archive internals construct or inspect the fixture.

One form type starts with a secret field and captures a historical value, then
removes/recreates that field as normal text in later definitions. Target and normal
sibling records, an absent-field record and an unrelated type are added. After
commit and independent reopen, setting the target secret appends a definition
revision and advances every same-type record's reference, including the absent
record. The normal sibling becomes secret with identical bytes; the already-secret
historical capture retains its bytes but receives the current label/kind. Other
captured fields keep their historical metadata, the target override wins and the
unrelated type stays unchanged. The absent capture is not invented.

A repeated secret set preserves the definition revision count and all logical
record state. This does not claim the public setter is a persisted byte no-op.
Normal downgrade and unknown-field refusals preserve committed image bytes.
Every committed transition is independently reopened; complete upgraded snapshots
are compared after repeat/refusal. Synthetic secrets are accessed only through
scoped SecretString callbacks. No release, migration or performance qualification
follows from this baseline.

Commands (archive worktree rust/, pinned Rust1.88.0):

```bash
cargo test -p revault_lockbox_api --release --features external-source --test form_capture_history -- --test-threads=1 --nocapture
cargo clippy -p revault_lockbox_api --features external-source --test form_capture_history -- -D warnings
```

Source hashes/logs are in checks/. The exact pre-format source snapshot uses
.rs.txt so the formatting hook cannot alter frozen evidence. Post-format checks
will be recorded separately. Later typed mutation must preserve these cross-record
semantics atomically with bounded selected-source staging; it is not implemented
by this baseline.
