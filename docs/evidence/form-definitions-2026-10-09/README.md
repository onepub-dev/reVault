# Definition revision, resolution and empty-record creation

This test-only (`cfg(test)`) shared-tree adapter connects definition mutation,
resolution and empty-record creation to selected ownership and atomic publication.
It does not activate public APIs or select a format. The retained public baseline
is authoritative for successful behavior, including surprising resolver cases.

Definition resolution first interprets valid type-ID grammar, then considers
aliases among the latest numeric revision of each type. It preserves ambiguity,
case normalization for type IDs, revision gaps and stable type/revision ordering.
A type-ID-shaped alias with no matching type does not fall back to alias lookup;
repeated define calls can therefore create distinct types. Defining an existing
type-ID string revises that type and retains its established alias. Supplied aliases
are validated even when the existing type's alias is retained.

Repeated definition calls advance a checked revision even for identical content.
Snapshot import remains separately idempotent and refuses conflicting existing
revisions. Direct same-field Secret-to-normal schema changes refuse; removing the
field in one revision and recreating it in a later revision remains permitted.
Explicit normal-to-Secret schema revision leaves every captured record unchanged.
A subsequent secret setter sees an already-Secret schema and updates only its
target, distinct from the existing automatic cross-record upgrade behavior.

Mutation opens one authenticated snapshot, verifies all selected content, and stages
borrowed definition parts against that same base. The private shared staging helper
serves both import and revision without cloning aggregate metadata or re-resolving
a later snapshot. The writer revalidates the selected commitment before mutation.
Fresh definition texts receive fresh identities; historical revisions remain live.
Checked counter exhaustion refuses before writes.

Empty-record creation resolves the latest definition, builds missing parent metadata
only in the candidate catalogue, and publishes parents and record in one transaction.
Required fields do not create initial captures. File and form namespaces retain the
observed public behavior, including a form at a file path or beneath a file parent.
Refused paths, aliases, duplicate records and text limits leave persistent bytes
unchanged. Explicit full-object getters retain their caller-requested materialization
cost; this is not a new bounded-memory guarantee for arbitrary getters.

## Verification

Initial all-mode lifecycle passes. Four subsequent controls pass with strict Clippy:
revision/resolution/creation, alias/import/refusal behavior, eleven MiB of normal
metadata, and selected-state replacement between preparation and writer admission.
The latter calibrates the initial control reads and switches an independently
committed image at writer admission, observes the newer authenticated root traversal,
and checks zero adapter mutation calls and exact final newer-image identity. The
simulated external commit is not an adapter write. This is a private-format fixture,
not a general concurrent filesystem race proof.

The functional aggregate uses exact one-MiB name, description and nine field labels,
including split UTF-8. All sixteen modes independently reopen and compare every
text, and repeated import remains unchanged. One-over-limit input refuses. Caller
inputs and the requested full normal-metadata getter allocate ordinary memory;
this probe is guarded staging and semantic capacity evidence, not a memory peak,
CPU, concurrency or format-wide capacity pass. No fixed resource batch is added.

The definition/create recovery matrix passes 3,168 transaction cuts and 1,164
interrupted recoveries. The first namespace/name test failed because its fixture
incorrectly expected an empty record name to succeed; the shared public validator
rejects empty or whitespace-only names. Its exact source was reconstructed after
the fixture correction and SHA-256 checked against the retained run manifest. The
corrected fixture asserts empty-name refusal, unchanged archive bytes and no new
parents before testing namespaces with a valid name. Final affected checks pass: four definition controls, the corrected namespace/name
control, all four public form-history tests and strict Clippy. No adapter behavior
was changed and the matrix was not rerun.
The recovery design separately revises a definition and creates a record with new
parents across eight protection/signing/compression/padding representatives. It
checks independently constructed old/new public objects and exact directory sets,
stable existing record identities and payload bytes, retained file/variable neighbors,
abandoned span/tail cleanup, repeated recovery and interrupted recovery. These are
modeled returned failures, not process death or independent region loss.

## Remaining scope

This adapter retains existing experimental row, graph and preparation caps. It does
not tighten historical capture admission to migration-import policy. Whole-tree
compaction/export preserving variables and forms, access roots, broader public
filesystem/mirror integration, native recovery, architecture selection and complete
resource/compatibility qualification remain open. Fresh whole-tree copying must
preserve authenticated publication lineage explicitly rather than reuse a fresh
export's generation-one rule.

## Reproducibility

`initial/`, `controls/`, `recovery/` and `final/` retain logs, HEAD, status and source
hashes for each variant. `final/` additionally retains the tracked source diff and
new source files as `.rs.txt` snapshots, preserving exact bytes outside formatting
hooks. The recovery fixture snapshot matches its recorded pre-correction hash.
Tests use pinned Rust 1.88.0, release `external-source`, serial execution; strict
Clippy covers library, tests and benches with `-D warnings`.

Final commands from `rust/`:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib typed_form_definition -- --nocapture --test-threads=1 --skip typed_form_definition_and_empty_record_atomic_recovery_faults
cargo test -p revault_lockbox_api --release --features external-source --lib typed_form_empty_creation_preserves_file_namespace_and_name_limits_all_modes -- --nocapture --test-threads=1
cargo test -p revault_lockbox_api --release --features external-source --test form_capture_history -- --test-threads=1
cargo clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
```

The recovery run selects `typed_form_definition_and_empty_record_atomic_recovery_faults`
with the same library/release/features/serial flags. Formatting and post-format
affected verification remain pending at this source checkpoint.
