# revault_lockbox_api

`revault_lockbox_api` is the portable storage engine for reVault `.lbox` files.

It owns the encrypted lockbox file format and the Rust API for creating,
opening, reading, mutating, and recovering lockboxes. It deliberately does not
manage the local vault, active sessions, auto-open behavior, command-line
prompts, or agent caching. Those concerns live in higher-level crates such as
`revault_vault_api` and `revault_cli`.

## Purpose

Use `revault_lockbox_api` when an application needs direct access to the lockbox file
format:

- create password, content-key, or contact-protected lockboxes
- store files, directories, symlinks, variables, and form records
- read and list lockbox contents without the CLI
- update lockbox contents with explicit owner signing
- inspect public lockbox metadata
- recover intact records from damaged lockbox files
- build native, server-side, or WASM wrappers around the core API

## Storage Capabilities

A lockbox is an encrypted archive with structured metadata. It can hold:

- **Files**: in-memory bytes, reader streams, and files imported from the host
  filesystem.
- **Directory-like paths**: lockbox paths are canonical internal paths such as
  `/docs/readme.txt`; directories are represented by the paths of stored
  entries.
- **Permissions**: file mode bits can be stored with file records and restored
  by extraction policies on platforms that support them.
- **Symlinks**: lockbox symlinks store an internal source path and internal
  target path.
- **Variables**: named metadata values that are not exposed as files and do not
  appear in normal file listings.
- **Secret variables**: variable values stored in secure memory while opened and
  accessed through callback APIs.
- **Forms**: versioned form definitions plus form records with normal and
  secret fields. This supports structured data such as login records without
  flattening everything into files.
- **Key slots**: password and contact key slots can open the same random
  lockbox content key.
- **Key-directory backups**: higher-level vault code can keep encrypted backup
  copies of key-directory data for recovery flows.

## Encryption And Authentication

Lockbox storage uses a random content key to encrypt file data and private
metadata. Passwords and contacts do not encrypt archive contents directly; they
open or wrap the content key.

- **Content encryption**: pages are encrypted and authenticated with
  ChaCha20-Poly1305.
- **Password slots**: pass phrases are stretched with Argon2id and used to wrap
  the content key with ChaCha20-Poly1305.
- **Contact slots**: contact sharing uses a hybrid pre-quantum and
  post-quantum key wrap: X25519 plus ML-KEM-768 derive the wrapping key, then
  ChaCha20-Poly1305 encrypts the content key.
- **Commit authentication**: owner commits are signed with both Ed25519 and
  ML-DSA-65. Verification requires both signatures to be present and valid.
- **Explicit write authority**: `Lockbox::open` returns a read-only handle;
  mutating an existing lockbox requires `Lockbox::open_for_write` and an
  owner signing key.

The hybrid design keeps a mature pre-quantum primitive in the path while adding
post-quantum protection. For key wrapping, an attacker should need to break both
the X25519 and ML-KEM-768 sides to recover the wrapping key. For commit
authentication, requiring both Ed25519 and ML-DSA-65 avoids treating a commit as
authentic unless it verifies under both the established classical signature and
the post-quantum signature.

## Compression And Layout

Lockbox compresses before storing encrypted payloads when compression is useful.
High-entropy data is detected and left uncompressed to avoid wasting space or
CPU. Repeated or text-like data is stored with zstd compression.

The archive layout is optimized for both small and large records:

- small files and metadata can be packed into shared pages;
- larger file payloads can be split across compression frames;
- decoded page and compression-frame caches accelerate repeated reads;
- workload profiles tune import, read-mostly, and extraction-heavy workloads;
- recovery scanning can rebuild intact files, symlinks, variables, and forms
  from damaged lockboxes when enough records survive.

## Integration Surface

The optional `vault-integration` feature exposes a narrow API used by
`revault_vault_api` for content-key caching and key-directory backup recovery.
Normal callers should use the standard `Lockbox` open/create APIs.

## Archive Format

The on-disk `.lbox` format is described in
[ARCHIVE_FORMAT.md](ARCHIVE_FORMAT.md). That document covers the fixed header,
pages, page objects, commit roots, commit authentication, TOC, variables, forms,
key directories, and recovery rules.

Implementation notes such as page-cache boundaries, compaction flow, key
removal maintenance, and recovery scan behavior are in
[IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md).

## Create And Read A Lockbox

```rust
use revault_lockbox_api::{
    Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretString,
};

fn main() -> revault_lockbox_api::Result<()> {
    let path = std::path::Path::new("example.lbox");
    let pass_phrase = SecretString::try_from_bytes(b"correct horse battery staple".to_vec())?;
    let signing_key = OwnerSigningKeyPair::generate()?;

    let mut lockbox = Lockbox::create_file(
        path,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;

    lockbox.add_file(
        &LockboxPath::new("/notes/hello.txt")?,
        b"hello from reVault",
        false,
    )?;
    lockbox.commit()?;

    let opened = Lockbox::open(path, LockboxOpen::Password(&pass_phrase))?;
    let bytes = opened.get_file(&LockboxPath::new("/notes/hello.txt")?)?;

    assert_eq!(bytes, b"hello from reVault");
    Ok(())
}
```

## Reopen For Mutation

Opening a lockbox with `Lockbox::open` is intentionally read-only. To
modify an existing lockbox, reopen it for write and provide the owner signing
key that should sign the next commit.

```rust
use revault_lockbox_api::{
    Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretString,
};

fn main() -> revault_lockbox_api::Result<()> {
    let path = std::path::Path::new("mutable.lbox");
    let pass_phrase = SecretString::try_from_bytes(b"correct horse battery staple".to_vec())?;
    let signing_key = OwnerSigningKeyPair::generate()?;

    Lockbox::create_file(
        path,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;

    let mut lockbox = Lockbox::open_for_write(
        path,
        LockboxOpen::Password(&pass_phrase),
        &signing_key,
    )?;

    lockbox.add_file(&LockboxPath::new("/updated.txt")?, b"updated", false)?;
    lockbox.commit()?;

    Ok(())
}
```

In a real application, keep the owner signing key somewhere durable and
protected. The local vault does this for CLI-managed lockboxes.

## Worked Examples

The `examples/` directory contains runnable examples for the main archive
features:

- `files_and_directories.rs`: add in-memory and host files, then list and read
  them.
- `symlinks_and_permissions.rs`: preserve executable permissions and store a
  lockbox symlink.
- `variables.rs`: store normal and secret variables.
- `forms.rs`: define a form, create a record, and store normal and secret
  fields.
- `recovery.rs`: damage a lockbox, scan it, and salvage intact records.

Run one with:

```sh
cargo run -p revault_lockbox_api --example files_and_directories
```

## Benchmarks

Core archive performance benchmarks live in `benches/`. See
`BENCHMARKS.md` for Criterion usage, the reproducible PGP comparison harness,
sample results, and profiler notes.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.
