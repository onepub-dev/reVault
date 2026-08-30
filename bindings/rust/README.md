# reVault for Rust

reVault is an encrypted `Lockbox` archive and persistent `Vault` for files,
credentials, keys, and typed records. This source-native crate re-exports the
reviewed `Lockbox`, `Vault`, and `AgentSession` facade over the native core; it
does not use the C ABI and never discovers a shared library through an
environment variable. See the
[reVault manual](https://docs.revault.onepub.dev/).

```toml
[dependencies]
revault-api = "0.3.11"
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [source-native conformance example](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/rust/src/main.rs)
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
debug output, ordinary strings, or unnecessarily long-lived byte buffers.
