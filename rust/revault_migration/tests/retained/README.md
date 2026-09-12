# Retained migration fixtures

These native bytes are permanent compatibility inputs, kept in Git. Normal tests
must never recreate them or update their expected results. All credentials and
content here are synthetic and public; never use these keys or passwords elsewhere.

Each `archive-vN` or `vault-vN` directory contains:

- `native.bin`: the original Lockbox file, with its original format and history.
  The `.bin` suffix avoids the repository's ignore rule for user `.lbox` files.
- `manifest.json`: native and container versions, producing library versions,
  SHA-256 checksum, public fixture password, and content inventory.
- `expected.json`: canonical logical records captured from the source export.
  Byte values, including private keys, are represented by length and SHA-256.
  File frame hashes cover every byte, including frame identity and offset.

Archive fixtures contain descriptions, directories, permissions, empty and Unicode
files, a binary file spanning multiple migration frames, symlinks, normal/secret/
empty variables, every form field kind, definition revisions, forms attached to
old and new revisions, both mirror removal policies, mirror rules and host
identity, password/contact slots, and committed replacements and deletions.
Vault fixtures contain key/signing generations, contacts with and without signing
keys, form definitions, known Lockboxes, access labels, remembered passwords, and
key-directory backups. Vault v3 also contains a password Profile. Vault structure v2 has two fixtures,
one for container v1 and one for container v2. Mirrors written
by the earliest v1 writer predate `strict`; v2 and v3 fixtures enable it.

Run from `rust/`:

```sh
cargo test --locked -p revault_migration --test retained_fixtures
```

The release candidate's `rust-checks` migration group already runs all tests in
`revault_migration`, including this suite. No fixture-generation environment
variable is set in CI. The suite requires every archive version from 1 through
`LOCKBOX_FORMAT_VERSION` and every vault structure version from 1 through
`CURRENT_VAULT_STRUCTURE_VERSION`. Increasing a version without adding its fixture
fails the release tests. Keep current-format fixtures now so they become historical
inputs automatically at the next format change.

The tests check checksums and source versions, export using the appropriate
historical reader, upgrade the migration schema, compare the source records to the
saved snapshot, import into current native format, reopen and export again, and
compare all records. Subsequent persisted mirror and vault mutations verify that
the migrated files remain usable. Native header version/mode and public commit
history are not compared as logical data: migration creates a new commit chain.
The key-directory generation advances on import; all other exported key-directory
bytes must match exactly. Source snapshot checks still include the original
key-directory generation.

## Adding a version

1. Keep all existing fixture files unchanged. Do not replace v1 with a file made
   by the current writer, or relabel header bytes to simulate an older format.
2. Retain the exact historical reader/exporter dependencies before changing a
   native reader. Register the new historical version in `export_fixture`.
3. Add a generator using the writer for the new native format and include every
   newly supported content kind. Record exact producing library versions. The
   existing generators are examples; the v3 generator refuses to run after its
   format constants have advanced.
4. Generate only the new directory with the explicit ignored test:

   ```sh
   REVAULT_APPEND_FIXTURE=archive-v4 cargo test -p revault_migration \
     --test retained_fixtures generate::append_retained_fixture -- --ignored --exact
   ```

   This example requires registering the v4 writer first. The generator refuses
   to overwrite an existing directory or file. It is never a release prerequisite.
5. Review the content inventory, saved logical records, checksums and native
   versions. Extend inventory assertions for new record families and any deliberate
   logical schema transformation. Commit native bytes, manifests, expected records,
   generator and tests together.
6. Run the complete retained-fixture matrix. A failure is a compatibility defect
   to investigate; do not regenerate old fixtures to make it pass.

Archive format and vault structure versions are independent. The matrix key
includes the artifact kind, structure/native version, and container version. Keep
every historical combination, including vault structure v2 in containers v1 and
v2. The current vault/container combination is required even when only the archive
version has advanced. The corpus and dedicated test are excluded from the published
crate package, but always run from the repository during release preparation.
