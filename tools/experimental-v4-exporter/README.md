# Experimental format-4 migration exporter

This isolated adapter reads the unreleased format-4 checkpoint
`a7a9f4d85b0fa716bf9e402de5f29ff5a6f61e9b` and writes authenticated logical
migration artifacts for the current importer. It is separate from the current
archive reader and is never selected automatically or installed from crates.io.
The released format-3 exporter remains separate and unchanged.

Build from the repository root, choosing a new directory on a volume with enough
space for the archived source and Rust build:

```sh
./tools/build_experimental_v4_exporter.dart /path/to/new-exporter-build
```

The script archives the exact checkpoint and compiles standalone adapters against
that source using its pinned Rust toolchain. It preserves archived dependency
versions while adding the adapter, then repeats the build with `--locked`. The
source revision, adapter hashes, resolved Cargo lock hash, and executable hashes
are recorded in `provenance.json`. Resolution is offline; required crates must
already be in the Cargo cache. Existing output directories are preserved.

The resulting executables implement migration IPC protocol 3:

* `revault-migrate-archive-experimental-v4`: native format 4, migration schema 3.
* `revault-migrate-vault-experimental-v4`: container format 4, Vault structure 3,
  migration schema 3.

Select the executable explicitly with the current CLI's `--exporter` option.
Migrate an old Vault first, preferably to a separate output directory. Then use
the migrated Vault when migrating its archives. For example:

```sh
lbx doctor migrate vault --output /path/to/new-vault \
  --exporter /path/to/new-exporter-build/revault-migrate-vault-experimental-v4
lbx doctor migrate lockbox /path/to/old.lbox --output /path/to/new.lbox \
  --exporter /path/to/new-exporter-build/revault-migrate-archive-experimental-v4
```

The archive exporter uses the supplied Vault's contact credentials. For a
password-only archive, provide its password through the existing
`LOCKBOX_PASSWORD` environment mechanism, inherited by the archive exporter.
Vault and migration-artifact passwords and contact private-key records use framed
stdin. These credentials are never placed in command arguments.
Historical recovery and lazy writes operate only on a private working copy.
Keep the originals and compare migrated logical content before replacing them.
The current migration command verifies its authenticated artifact and destination
before completing replacement.

Migration establishes the new integrity graph for the imported state. It cannot
retroactively establish that an old archive was free of partial replay or other
tampering before export. The old reader's security limitations remain relevant to
the source. The pinned adapter supports the checkpoint's experimental encoding;
this is not a promise that every earlier experimental format-4 revision is
readable. Full-archive rollback still requires external freshness state.

This tooling does not authorize a release or resume performance work.
