# Item descriptions

Issue: https://github.com/onepub-dev/reVault/issues/333

## Model

Descriptions are multiline explanatory text on variables, form definitions,
field definitions, and form records. A field value has no separate description.
Definition guidance explains how to fill out a form; record guidance explains
the particular instance. Neither replaces the other.

Definition and field guidance are revisioned. A changed description creates a
new definition revision; identical updates and clearing absent field guidance
are no-ops. Structural revisions retain guidance for retained field IDs.
Copying a definition copies its exact revision's field guidance. Reimporting an
existing revision with conflicting guidance is rejected.

`forms show` displays latest guidance and the record's stored revision separately.
Descriptions of variables and records move and disappear with their owners.
Empty optional descriptions differ from absence. Existing form-definition
descriptions continue to use an empty string for no guidance.

## Persistence and compatibility

Use encrypted hidden normal variables, following existing Lockbox description
metadata. Keys under `/.revault/descriptions/` contain a kind and a SHA-256 digest
of the canonical owner identity. Variable identities are canonical names;
record identities are canonical paths; field-map identities are definition
type ID and revision. The digest is an index, not a secrecy mechanism. The
archive's existing encryption and authentication protect both keys and values.

Field guidance is a JSON map keyed by field ID. Validate fields, text, and the
encoded map before changing the definition. Normal-variable limits apply,
including a one-MiB aggregate encoded map limit per revision. Metadata remains
excluded from ordinary variable listing and export.

This preserves the format-4 wire layout and requires no new compatibility line.
Released 0.5.0 clients can read and ordinarily update these archives and Vaults.
They do not implement the new metadata lifecycle: old-client owner renames or
deletions can leave orphan annotations, and old-client definition revisions do
not inherit field guidance. Use updated clients for these operations. The
released-baseline compatibility test verifies preservation through ordinary
writes in both archive-creation directions, including Vault field guidance.

## API and display

Rust exposes typed description methods. Foreign bindings share two additive
ABI operations accepting a validated JSON request and returning the existing
OptionalString FlatBuffer. Errors retain the existing error types and channel.
Lockbox reads work on read-only handles; mutations require writable handles and
commit. Vault definition mutations commit immediately.

`description get` preserves bytes without an added newline. `forms show` prints
full guidance. Tables preview the first line, JSON retains full text, and TSV
escapes backslash, tab, CR and LF. Descriptions are shown without opting into
secret-value access and must not contain those secret values.
