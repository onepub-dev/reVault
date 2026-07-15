# reVault / Lockbox

reVault creates portable archives that are secure and compressed.

The reVault archives are called 'Lockboxes'.

You can think of a Lockbox as a zip file on steriods. 

A Lockbox can be used to store:
* files
* directories
* symlinks
* file/directory permissions
* variables
* forms (collection of variables)


A lockbox is encrypted, compressed and signed using modern encryption and compress
techniques.

reVault is designed to be simple and safe to use, avoiding overly complex 
terminology.

reVault works on Linux, Windows and MacOS.

reVault ships as a:
  * cli
  * apis
  * libraries for multiple languages



- `revault_cli`, a CLI for everyday use.
- `revault_lockbox_api`, for creating lockboxes.
- `revault_vault_api`, for creating and managing vaults.

Terminology is explicit:

- A **lockbox** is a portable archive and uses the `.lbox` extension.
- A **key** is used to lock and unlock lockboxes. Keys are more secure than passwords.
- A **vault** is where we store information about lockboxes such as the keys
used to open and close a lockbox. The vault also holds a contact list which helps us share 
lockboxes with other people.
  

> **Pre-release:** the Rust implementation is currently alpha software. The
> archive format and public APIs may change between releases. Do not use it as
> the only copy of important data, and keep vault backups and profile recovery
> material offline.

## Installation

The best way to start with reVault is to install the CLI tooling.


### crates.io
The easist way to install the reVault CLI is from crates.io

```
cargo install revault_cli
```


### repo

You can also install the CLI by cloning the github repo.

```bash
git clone https://github.com/onepub-dev/reVault.git
cd reVault/rust
cargo xtask install-cli
```

# Initialise your vault

In order to create a lockbox you must first initialise your Vault.

When you install the reVault CLI, Cargo installs both `lockbox` app and its alias `lbx` into `~/.cargo/bin`.
Ensure that directory is on your `PATH`.

To initialise your vault run:

```bash
lbx vault init
```
The init command will create the keys need to create/open and close lockboxes.
The keys are stored in your reVault vault.

The init command will also dump those keys to the terminal. You need to backup those
keys safely.
* If you loose the keys you will not be able to access any lockbox - there is no way to recover from this!!!
* If some else gets access to those keys then they have access to all of your lockboxes.


## CLI Quick Start

Create a lockbox for the default vault profile: (recommended)

```bash
lockbox create secrets.lbox
```

For a lockbox protected by a passphrase instead of a vault profile:

This form is less secure than using a profile.

```bash
lockbox create --password secrets.lbox
```

Open it for normal commands:

```bash
lockbox open secrets.lbox
```

The open is cached in a per-user in-memory agent which will automatically close
it after about 30mins.  Lock it when you are done:

```bash
lockbox close secrets.lbox
```

Add files:

```bash
lockbox add --recursive secrets.lbox ./project /project
lockbox add secrets.lbox ./generated.env /secrets/prod.env
```

List and extract:

```bash
lockbox list secrets.lbox /
lockbox list secrets.lbox '/project/**/*.rs'
lockbox extract secrets.lbox /project/README.md ./out/README.md
```

See [docs/cli_how_to.md](docs/cli_how_to.md) for command-focused examples.

## Profiles And Sharing

Create another local profile:

```bash
lockbox vault profile create laptop
```

List profiles and export a public key:

```bash
lockbox vault profile list
lockbox vault profile export ./laptop.pub --name laptop
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
lockbox vault profile backup ./default.profile-backup
```

Private vault keys are stored inside the local vault as secret variable records,
not as normal files in the vault lockbox. See the CLI help and
[CLI how-to](docs/cli_how_to.md) for vault backup and profile recovery.

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

## Key Server, Topology, and Replication

`revault_key_server` is a short-lived rendezvous service for sharing candidate
contact public keys. A publisher uploads a payload, verifies their email, and
shares the resulting publish code with the recipient. The recipient uses that
code to receive the candidate key, then verifies its fingerprint independently
before trusting it. The server does not establish profile trust and is not a
store for private keys.

### Single server

A first deployment uses one server with `server_id = 0`. Publish codes include
that server id as their first digit, followed by a random code body. Using a
self-routing code from the start keeps pending publishes compatible if the
deployment later gains standby servers.

### Topology servers

Servers publish a topology document containing the known server URLs and the
primary/failover route for each publish-code owner id. Clients use it as follows:

- For a publish, discover the cluster through a topology endpoint, select a
  key-server member, and keep that choice sticky locally.
- For receive and delete, read the publish code's first digit, contact that
  owner id's primary server, then use only its configured failovers.
- Do not fail over after a rate-limit response; hopping servers must not bypass
  abuse controls.

DNS can provide normal host resolution, but it is not the routing mechanism.
Round-robin DNS alone can send a receive request to a server that does not own
the corresponding publish code.

### Replication and failover

Each server is authoritative for the codes carrying its own server id. It sends
signed state events—published payloads, receive counts, payload lifecycle
tombstones, and rate-limit blocks—to configured standby peers through the separate
`/v1/replicate` endpoint. Events carry an origin server id, epoch, and sequence
number, so standbys apply duplicates idempotently and never replicate received
peer events again.

Replication does not make the cluster hot/hot. A standby keeps replica state
but serves an owner's payloads only after an operator explicitly promotes it for
that owner id. This avoids two servers consuming a single-use publish at the
same time during a network partition. See the [key-server deployment guide](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/README.md), [configuration reference](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/KEY_SERVER_CONFIG.md), and [redundancy design](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/REDUNDANCY.md) for the deployment and recovery procedures.

## Rust Library

Use `revault_lockbox_api` when you need the portable storage engine:

```rust
use revault_lockbox_api::{Lockbox, LockboxOpen, LockboxPath, LockboxProtection, SecretVec};
use std::path::Path;

let key = SecretVec::try_from_slice(b"correct horse battery staple")?;
let signing_key = revault_lockbox_api::OwnerSigningKeyPair::generate()?;
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

Use `revault_vault_api` for native applications that want the local vault and
open-cache behavior:

```rust
use revault_vault_api::{local_vault, SecretString};

let vault = local_vault();
let password = SecretString::try_from_bytes(b"pw".to_vec())?;

vault.create_lockbox_with_password("secrets.lbox", &password)?;

let lockbox = vault.open_lockbox_with_password("secrets.lbox", &password)?;
let mut lockbox = lockbox;
lockbox.add_file_from_path("notes.txt", &revault_lockbox_api::LockboxPath::new("/notes.txt")?)?;
lockbox.commit()?;
```

## Documentation

- [CLI how-to](docs/cli_how_to.md): command examples.
- [Lockbox Session Agent](docs/lockbox_session_agent.md): local open cache
  lifecycle, protocol, and security model.
- [Archive format](rust/revault_lockbox_api/ARCHIVE_FORMAT.md): lockbox archive details and page
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
cargo xtask generate-api-docs
```

## License

Dvault Source Available License 1.0 - see [LICENSE](LICENSE).

The source remains available for inspection, modification, and redistribution.
Derivative works must publish their corresponding source code in a publicly
accessible repository such as GitHub or a similar service. The license also
restricts third parties from offering dvault, modified dvault, or substantially
similar dvault-derived functionality as a hosted, managed, or
network-accessible service without a separate written license.
