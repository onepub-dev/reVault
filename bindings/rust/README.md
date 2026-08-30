# reVault for Rust

reVault is a fast, local toolkit for creating secure portable archives called
Lockboxes. Each Lockbox is encrypted, compressed, and signed. It can store
files and directory trees, variables such as API keys, and forms such as login
details.

Lockboxes are easy to copy, share, and back up, and they do not require a
hosted service. The engine is designed for speed and effective compression.
Applications can read, write, and seek within stored files without extracting
the archive, and recover data from partial corruption. reVault provides a
command line tool for everyday work and APIs for application code.

Read the [reVault manual](https://docs.revault.onepub.dev/) for the quick start,
core concepts, and security model.

Your Vault holds your profile and contacts. The CLI protects a new Lockbox for
your profile by default, and you can grant access to contacts using their
public keys. Use password access when you do not have a recipient's contact
(public key) details.

This crate implements the `Lockbox`, `Vault`, and `AgentSession` API directly
in Rust. It does not use the C ABI or discover a shared library through an
environment variable.

## Installation and compilation model

```toml
[dependencies]
revault-api = "0.3.11"
```

Cargo compiles the reVault implementation into the application from the Rust
crate dependency. There is no separate native library to install, extract, or
select, and `REVAULT_LIBRARY` does not apply to this binding.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [Rust conformance example](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/rust/src/main.rs)
are maintained in the source repository.

```rust
use revault_api::{Lockbox, Revault, Vault};
use revault_api::lockbox::{LockboxPath, SecretString, VariableName};

let _runtime = Revault::load();
let key = [0_u8; 32]; // load a real content key securely
let mut box_ = Lockbox::create(&key)?;
box_.add_file(&LockboxPath::new("/hello.txt")?, b"hello\n", false)?;
box_.set_variable(&VariableName::new("owner")?, "alice")?;
let token = SecretString::try_from_slice(b"secret")?;
box_.set_secret_variable(&VariableName::new("token")?, &token)?;
box_.commit()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Vault` is the persistent encrypted store; a directory is only its storage
detail. `ProfileSigningKeyPair` and `ProfileSigningPublicKey` name profile
identity keys. A key has the `owner` role only after it is assigned to a
`Lockbox`. The optional Session Agent operations remain under
`revault_api::vault` and are explicit; ordinary `Lockbox` operations do not
contact an agent.

Secret types zero their owned storage. Avoid exposing secret values through
debug output, ordinary strings, or unnecessarily retained byte buffers.

## Core API concepts

- `Revault` initializes facilities shared by the process.
- `Vault` owns persistent local state.
- `Lockbox` owns an open archive.
- Agent functions cache selected content keys.

Each value has its own lifetime.

## API documentation and support

Use [docs.rs](https://docs.rs/revault-api) for the selected crate version. The
[method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md),
rustdoc, and
[Rust conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/rust/src/main.rs)
cover the public operation inventory.

## Create, open, and replace

Open functions require existing Vaults and Lockboxes and never create them.
Use open or create only when creation is acceptable and replacement APIs only
after an explicit destructive choice. `Lockbox::commit` persists pending
changes. When a `Lockbox` value goes out of scope, Rust releases the content
key that it held in this process; it does not commit or delete the archive.

Open the resulting archive with its Lockbox password, a profile key, or a
credential resolved from the Vault. A `ProfileSigningKeyPair` becomes an owner
key only after explicit assignment to the Lockbox.

## Secrets, errors, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are distinct
secrets. Use the owning secret types, keep borrowed views temporary, and
avoid conversion into ordinary `String` or retained `Vec<u8>` values.
Secret owners zero their storage when they are closed or go out of scope.

Operations return `Result` with structured errors. Handle the result before
using a value and preserve the stable category/details when presenting
recovery guidance. Public APIs do not depend on mutable global last error
state.

## Optional session agent

Ordinary Lockbox opens keep their state in this process and never start or
consult the agent.
Use `AgentSession` when Lockbox keys need to be shared across processes or
remain available after the process that opened the Lockbox exits. Closing an
entry forgets the cached key, not the Lockbox file or credentials stored in the
Vault.

## Platform credential store

The operating system credential store can hold the Vault passphrase. The
user's operating system login normally unlocks that store. After login,
another process running as that user may be able to retrieve the passphrase if
the access policy applied to the saved Vault passphrase does not require
approval for each retrieval. Exact access depends on the operating system, the
credential store configuration, and that access policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault
can then provide access to Lockboxes through profile keys or remembered
Lockbox passwords. Both remain encrypted inside the Vault; they are not copied
to the operating system credential store.

Agent expiry improves memory hygiene. It is not an authentication boundary
after login if the saved Vault passphrase can be retrieved without approval.

Missing or placeholder rustdoc on public classes or methods is a binding
defect.
