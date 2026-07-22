# Binding conformance contract

Every language binding must run the same contract against the real native
library or WASM module. A symbol-count check, parser test, or create/commit
smoke test is not conformance.

## Archive lifecycle

The test creates an archive with a deterministic content key and exercises:

- create/open with content key, password, contact key, signing key, and options;
- files, replacement rules, permissions, directories, parent creation, rename,
  delete, recursive removal, symlinks, existence, stat, range reads, and full
  reads;
- logical and physical content streams, extraction, storage length, and commit;
- normal and secret variables, sensitivity, listing, update, and deletion;
- workload profile, worker policy, runtime options, cache statistics, import
  statistics, reset, page inspection, recovery reports, and file inspection;
- password and contact key slots, slot listing, deletion, owner signing keys,
  owner inspection, and lockbox identifiers;
- form definitions, revisions, resolution, records, field updates, field
  reads, record listing, and record deletion;
- recovery scanning and salvage for intentionally damaged archive copies;
- contact and signing key generation, serialization, import/export, encryption,
  decryption, wrapped-contact accessors, fingerprints, and all key formats.

Each structured result is decoded from its private FlatBuffer table. Tests
must assert field values, not merely that a pointer or map was returned.
The authoritative operation-to-result mapping is
`bindings/flatbuffers/results.tsv`. Every buffer-returning operation is classified
as raw bytes, UTF-8, or an exact FlatBuffer table type; unclassified operations
fail `revault-tool bindings check`.

Every package suite must begin with a `NATIVE` evidence record containing the
target, delivery kind, installed artifact path, SHA-256, and `installed`
status. The verifier rejects release runs that omit this record. Build-tree
paths and `REVAULT_LIBRARY` are forbidden in package acceptance jobs.
Rust is source-native and emits `SUITE` records for `public_api_suite` and
`vault_api`, followed by a `SOURCE` record containing the packed `.crate`
archive hash. It does not emit a fictitious C-library record.

## Vault lifecycle

The test creates a temporary vault and exercises:

- open/create, replace, default paths, password changes, and structure version;
- profiles, profile emails, profile generations, rotation, and profile history;
- private keys, contacts, contact signing keys, existence, listing, loading,
  storing, deletion, and restoration;
- forms, form aliases, definitions, resolution, listing, and seeding;
- known lockboxes, access labels, remembered passwords, backups, restore, and
  backup manifests;
- local-vault lockbox creation/opening for password, content-key, and contact
  protection;
- agent operations, sleep support, platform-secret-store status and scope,
  and all vault/owner/lockbox key cache operations.

## Interoperability

The archive and vault produced by each language are opened and verified by
every other language in the matrix. This all-pairs check catches ABI layout
errors, incorrect byte lengths, buffer ownership bugs, and FlatBuffer decoding
differences that a single ring edge could miss. Each check emits an `INTEROP`
record consumed by `revault-tool e2e verify-interop`; self-opening does not
count.

Agent and platform-secret-store cases run in service-enabled containers. They
are separate from filesystem-only cases but are mandatory for a full suite.

## Required result

Each language runner must fail on any missing operation, unexpected error,
incorrect concrete result, leaked returned buffer, or interoperability mismatch.
There are no smoke-only or structural-only language stages.

Each of the 94 claimed host combinations is run by the same Rust orchestrator
after the native and publication-tree artifacts have been downloaded:

```text
revault-tool e2e package-conformance --language python \
  --target linux-x86_64-gnu --archive native --packages packages \
  --repository . --work package-conformance-python-linux-x86_64-gnu
```

The command creates isolated package-manager caches, packs where the ecosystem
has a package archive format, installs into a clean consumer, runs the full
language test, and hashes the carrier found under that installation.

The complete matrix contains sixteen languages and therefore 480 directed,
non-self artifact paths: `16 * 15 * 2`, where the two artifacts are the archive
and vault produced by each language.
