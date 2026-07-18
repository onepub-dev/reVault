# reVault for Rust

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This source-native crate re-exports the complete
`revault_lockbox_api` and `revault_vault_api`; it does not use the C ABI. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```toml
[dependencies]
revault-api = "0.2.0"
```

```rust
use revault_api::lockbox::{Lockbox, LockboxPath, SecretString, VariableName};

let key = [0_u8; 32]; // load a real content key securely
let mut box_ = Lockbox::create(&key)?;
box_.add_file(&LockboxPath::new("/hello.txt")?, b"hello\n", false)?;
box_.set_variable(&VariableName::new("owner")?, "alice")?;
let token = SecretString::try_from_slice(b"secret")?;
box_.set_secret_variable(&VariableName::new("token")?, &token)?;
box_.commit()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Secret types zero their owned storage. Avoid exposing secret values through
debug output, ordinary strings, or unnecessarily long-lived byte buffers.
