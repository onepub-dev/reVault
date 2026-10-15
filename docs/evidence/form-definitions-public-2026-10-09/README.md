# Public definition, resolution and empty-record baseline

One public API integration test passes all 16 archive modes; targeted strict
Clippy passes. This is existing behavior before the test-only typed adapter,
not public activation or release compatibility qualification. The fixture uses
public methods to construct, commit and independently reopen all state.

Repeated define appends a revision even with equal contents; exact snapshot import
is idempotent and conflicts refuse. Explicit existing type IDs retain the previous
alias, but an invalid supplied alias still refuses. Imported revision10 is selected
as numeric maximum and all revisions list oldest first. Only each type's latest
alias participates in resolution; old alias remains resolvable here solely because
a different imported type has it. Ambiguous, missing and invalid aliases return
their respective errors. Latest definitions list in stable type-ID order.

Resolution tries the accepted36-character hex/dash type-ID grammar before alias
lookup, including uppercase normalization. Consequently a valid hex-shaped alias
whose corresponding type is absent causes repeated define(alias) to create two
random types, not revise the earlier alias. Conversely define(existing-type-ID
string) revises that type and keeps its established alias. These surprising existing
behaviors are retained explicitly; no production correction is included.

Creation stores an empty record despite a required field, captures the then-current
revision, and creates missing parents. Later definition edits do not change that
record's captured revision. Duplicate record and ambiguous-reference creation
refuse; no refused parent appears after reopen. Full definition/record metadata and
exact import byte-noop are checked through separate public opens.

The new focused test is
`public_form_definition_resolution_revision_and_empty_creation_all_modes` in
`tests/form_capture_history.rs`. Commands use pinned Rust1.88.0, release
`external-source`, serial test execution and targeted strict Clippy for that
integration target. Initial source hashes and exact logs are retained in `initial/`.
The tracked hook formatted the integration test at `a944aec3`; all four
post-format public history tests and targeted strict Clippy pass. Exact logs and
source identities are under `postformat/`. Next implement the bounded typed
resolver/revision/empty-record adapter over one authenticated selected snapshot;
strict migration-import and historical capture admission remain distinct.
