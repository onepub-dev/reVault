# Item descriptions

Issue: https://github.com/onepub-dev/reVault/issues/333

## Model

Descriptions are multiline explanatory text on variables, form definitions,
field definitions, and form records. A field value has no separate description.
Definition guidance explains how to fill out a form; record guidance explains
the particular instance. Neither replaces the other.

Definition and field guidance are revisioned. A changed description creates a
new definition revision; identical updates and clearing absent field guidance
are no-ops. Callers revising native definitions supply the complete fields, including guidance.
CLI structural revisions preserve guidance for retained field IDs because field
specifications do not carry descriptions.
Copying a definition copies its exact revision's field guidance. Reimporting an
existing revision with conflicting guidance is rejected.

`forms show` displays latest guidance and the record's stored revision separately.
Descriptions of variables and records move and disappear with their owners.
Empty optional descriptions differ from absence. Existing form-definition
descriptions continue to use an empty string for no guidance.

## Persistence and compatibility

Store descriptions directly on their owners in authenticated encrypted storage.
Variables carry an optional description beside the normal or secret value;
field definitions and form records carry their own optional description.
There are no synthetic variables, hashed owner indexes, or JSON field maps.
Each optional description distinguishes absence from an explicitly empty string.
Multiline UTF-8 text uses the existing text validation and per-description limit.

This changes the persisted archive/container format to 5 and starts the 0.6.x
CLI and binding compatibility line. Variable and form node encodings advance to
version 2. Format-4 readers must reject these files. Format-5 readers require
explicit migration of format-4 archives and Vault containers; opening a file
never silently upgrades it. Vault structure version remains 3.

The isolated `revault_migrate_v4` package owns both historical archive and Vault
exporters. The current migration schema is 4 and carries native descriptions;
historical exporters retain their published schema-3 dependency. Existing
retained fixtures and historical readers remain unchanged. The initial 0.6.0
release uses published 0.5.0 as a migration source, not a same-line baseline.
Once 0.6.0 is published, later 0.6.x candidates must test bidirectional
compatibility against that released baseline.

The short-lived hidden-variable implementation was not released. It has no
supported persisted compatibility contract and is not a migration source.

## API and display

Rust exposes typed description methods. Foreign bindings share two additive
ABI operations accepting a validated JSON request and returning the existing
OptionalString FlatBuffer. Errors retain the existing error types and channel.
FlatBuffer variable, field-definition and record models also expose description
and presence fields. Presence distinguishes absent text from an empty string.
Generated encoders and all language models must carry both fields.
Lockbox reads work on read-only handles; mutations require writable handles and
commit. Vault definition mutations commit immediately.

`description get` preserves bytes without an added newline. `forms show` prints
full guidance. Tables preview the first line, JSON retains full text, and TSV
escapes backslash, tab, CR and LF. Descriptions are shown without opting into
secret-value access and must not contain those secret values.
