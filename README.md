# reVault / Lockbox

reVault is a local-first encrypted storage toolkit. Its portable Lockbox format
stores files, symlinks, and variable values in an encrypted `.lbox` container.
The Rust workspace provides:

- `lockbox`, a CLI for everyday use.
- `lockbox_core`, the portable storage library.
- `lockbox_vault`, the native vault and open-cache layer for desktop/server
  tools.

Terminology is explicit:

- A **lockbox** is the portable `.lbox` container.
- A **vault** is the user's local private store. It may contain private keys,
  trusted recipient keys, key-directory backups, and local open-cache state.

These terms are used consistently throughout the CLI and library APIs.

> **Pre-release:** the Rust implementation is currently alpha software. The
> archive format and public APIs may change between releases. Do not use it as
> the only copy of important data, and keep vault backups and identity recovery
> material offline.

## Installation

The current pre-release CLI is installed from the repository with Cargo:

```bash
git clone https://github.com/onepub-dev/reVault.git
cd reVault/rust
cargo install --path lockbox_cli
```

Cargo installs both `lockbox` and its short alias `lbx` into `~/.cargo/bin`.
Ensure that directory is on your `PATH`.

Initialize the local vault before creating identity-protected lockboxes:

```bash
lbx vault init
```

The vault stores local private identities, contacts, key-directory backups, and
the metadata needed for the session agent. On Linux, macOS, and Windows,
reVault can store the vault passphrase in the platform credential store when
that store is available. If it is unavailable, the CLI continues securely but
prompts for the vault passphrase when needed.

> Status: the Rust implementation under `rust/` is the production direction for
> the first Lockbox format release. The format is still pre-1.0, so breaking
> changes are allowed while the design is finalized. Third-party cryptographic
> review is still a release blocker.

## CLI Quick Start

Create a lockbox for the default vault identity:

```bash
lockbox create secrets.lbox
```

For a lockbox protected by a passphrase instead of a vault identity:

```bash
lockbox create --password passwords.lbox
```

Open it for normal commands:

```bash
lockbox open secrets.lbox
```

The open is cached in a per-user in-memory agent for a short sliding TTL. Lock
it when you are done:

```bash
lockbox close secrets.lbox
```

Add files:

```bash
lockbox add secrets.lbox ./project
lockbox add-file secrets.lbox ./generated.env /secrets/prod.env
```

List and extract:

```bash
lockbox ls secrets.lbox /
lockbox ls secrets.lbox /project --glob '**/*.rs'
lockbox extract secrets.lbox /project/README.md ./out/README.md
```

See [docs/cli_how_to.md](docs/cli_how_to.md) for command-focused examples.

## Identities And Sharing

Create another local identity:

```bash
lockbox vault identity create laptop
```

List identities and export a public key:

```bash
lockbox vault identity list
lockbox vault identity export laptop ./laptop.pub
```

Import a contact public key after independently verifying its fingerprint:

```bash
lockbox vault contact import alice ./alice.pub \
  --fingerprint <fingerprint> \
  --fingerprint-channel phone-call-to-owner
```

Create a lockbox for a recipient:

```bash
lockbox create --for alice shared.lbox
```

Grant or revoke access on an existing lockbox:

```bash
lockbox access grant shared.lbox alice
lockbox access list shared.lbox
lockbox access revoke shared.lbox alice
```

Exporting a private key is supported for backup and migration, but treat the
output as a secret:

```bash
lockbox vault identity backup ./default.identity-backup
```

Private vault keys are stored inside the local vault as secret variable records,
not as normal files in the vault lockbox. See the CLI help and
[CLI how-to](docs/cli_how_to.md) for vault backup and identity recovery.

## Variables

Variables are encrypted metadata, not files. They do not appear in file
listings and are loaded only when variable commands or APIs request them. The
canonical command is `variable`; `variables` and `var` remain compatibility
aliases.

Plain variables are for values that are useful configuration but not high-value
secrets:

```bash
lockbox variable set secrets.lbox DATABASE_URL --value 'postgres://localhost/app'
lockbox variable set secrets.lbox DATABASE_URL='postgres://localhost/app'
lockbox variable get secrets.lbox DATABASE_URL
```

With a session-default lockbox, assignment syntax is also supported:

```bash
lockbox variable set DATABASE_URL='postgres://localhost/app'
lockbox variable set DATABASE_URL 'postgres://localhost/app'
```

Secret variables are for passwords, API tokens, signing keys, and similar
material. They use secure-memory handling in the variable path and must be provided
through an explicit source:

```bash
lockbox variable set secrets.lbox --secret API_TOKEN --interactive
lockbox variable set secrets.lbox --secret API_TOKEN --file ./api-token.txt
lockbox variable set secrets.lbox --secret API_TOKEN --stdin
lockbox variable set secrets.lbox --secret API_TOKEN --from-env API_TOKEN
```

Avoid passing secrets as command-line arguments. Shell history and process
inspection can expose argv. Prefer `--interactive`, `--stdin`, `--file`, or
`--from-env`.

Sensitivity is chosen when a variable is created. Updating the value preserves
that sensitivity. To change plain to secret, or secret to plain, delete and
recreate the variable.

## Avoiding Leaks

Use these defaults unless you have a specific reason not to:

- Use `lockbox open` and the local agent instead of repeatedly passing
  passwords or private keys.
- Use secret variables for credentials and tokens.
- Do not store secrets as normal files when variable semantics are appropriate.
- Do not pass secret values on the command line.
- Keep private key exports offline, short-lived, and permission-restricted.
- Prefer recipient keys over shared passwords for team access.
- Run `lockbox close secrets.lbox` when an open should no longer be cached.
- Use `lockbox visualize secrets.lbox` for diagnostics; it intentionally avoids
  printing file paths, file contents, variable names, or variable values.

Lockbox protects data inside the container. It cannot protect a secret after you
write it to a normal terminal, shell history, clipboard, exported file, build
log, or process environment controlled by other tooling.

## Rust Library

Use `lockbox_core` when you need the portable storage engine:

```rust
use lockbox_core::{Lockbox, LockboxOpen, LockboxPath, LockboxProtection, SecretVec};
use std::path::Path;

let key = SecretVec::try_from_slice(b"correct horse battery staple")?;
let signing_key = lockbox_core::OwnerSigningKeyPair::generate()?;
let mut lockbox = Lockbox::create_file(
    Path::new("secrets.lbox"),
    LockboxProtection::ContentKey(key.try_clone()?),
    &signing_key,
)?;

lockbox.add_file(&LockboxPath::new("/docs/a.txt")?, b"alpha", false)?;
lockbox.add_file(&LockboxPath::new("/docs/b.txt")?, b"bravo", false)?;
lockbox.commit()?;

let reopened = Lockbox::open(
    Path::new("secrets.lbox"),
    LockboxOpen::ContentKey(&key),
)?;
let file = reopened.get_file(&LockboxPath::new("/docs/a.txt")?)?;
```

Use `lockbox_vault` for native applications that want the local vault and
open-cache behavior:

```rust
use lockbox_vault::{local_vault, SecretString};

let vault = local_vault();
let password = SecretString::try_from_bytes(b"pw".to_vec())?;

vault.create_lockbox_with_password("secrets.lbox", &password)?;

let lockbox = vault.open_lockbox_with_password("secrets.lbox", &password)?;
let mut lockbox = lockbox;
lockbox.add_file_from_path("notes.txt", &lockbox_core::LockboxPath::new("/notes.txt")?)?;
lockbox.commit()?;
```

## Documentation

- [CLI how-to](docs/cli_how_to.md): command examples.
- [Lockbox Session Agent](docs/lockbox_session_agent.md): local open cache
  lifecycle, protocol, and security model.
- [Archive format](rust/lockbox_core/ARCHIVE_FORMAT.md): lockbox archive details and page
  layout.
- [Rust development](docs/rust_development.md): build, test, and API docs.
- [Dependency review](rust/docs/dependency_review.md): dependency and security
  review notes.

## Development

Run the Rust checks:

```bash
cd rust
cargo fmt --all
cargo check --workspace --all-targets --all-features
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Generate Rust API docs:

```bash
cd rust
tools/generate_api_docs.sh
```

## License

Dvault Source Available License 1.0 - see [LICENSE](LICENSE).

The source remains available for inspection, modification, and redistribution.
Derivative works must publish their corresponding source code in a publicly
accessible repository such as GitHub or a similar service. The license also
restricts third parties from offering dvault, modified dvault, or substantially
similar dvault-derived functionality as a hosted, managed, or
network-accessible service without a separate written license.
