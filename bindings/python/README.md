# reVault for Python

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

Platform wheels include the reVault engine and expose Python classes for
Lockboxes, Vaults, keys, and the session agent.

## Installation and native runtime

```shell
python -m pip install revault-api
```

PyPI publishes a wheel for each supported operating system and architecture.
The wheel stores the matching library inside `revault_api/_native`, and the
package loads it through `ctypes`. Installing from a source archive does not
supply a library unless the package is assembled with the matching native
files.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```python
from revault_api import Revault, SecretString, Vault

runtime = Revault.load()                 # loading does not open a Vault
signing = runtime.generate_profile_signing_key_pair()
public_signing_key = signing.public_key()
with runtime.lockbox_create(bytes(32)) as box:  # Lockbox held by this process
    box.set_owner_signing_key(signing)  # Profile becomes this Lockbox's owner
    box.add_file("/hello.txt", b"hello\n", False)
    box.set_variable("owner", "alice")
    box.set_secret_variable("token", bytearray(b"secret"))
    size = box.with_secret_variable("token", lambda token: len(token))
    box.commit()
public_signing_key.close()
signing.close()

with SecretString("Vault passphrase") as vault_passphrase:
    persistent = Vault.open_or_create("/tmp/revault-vault", vault_passphrase)
    persistent.close()
```

`Revault.load(native_library_path=...)` can load a native library supplied by
the application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the
library inside the installed wheel. A bare name uses the operating system
search path.

The value passed to a secret callback is a temporary `bytearray`; it is
cleared after the callback. Do not convert it to a retained `str` or `bytes`.
`SecretString` and `SecretBytes` own mutable buffers and wipe them on
`close()`. Ordinary Lockbox operations never contact the agent.

## Core API concepts

- `Revault` loads the runtime.
- `Vault` owns persistent local state.
- `Lockbox` owns an open archive.
- `AgentSession` caches selected content keys.

Prefer context managers and close each object independently.

## API documentation and support

The wheel ships typed public classes and method docstrings. Use
`help(revault_api)` or an API documentation generator for the installed
version. The
[method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [Python conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/python/conformance.py)
cover the common operation inventory.

## Create, open, and replace

Open methods require existing Vaults and Lockboxes; they never create missing
state. Use open or create only when creation is acceptable and replacement
methods only after an explicit destructive choice. `commit()` persists
pending Lockbox changes. `close()` releases the handle and content key held by
the current process but does not commit or delete the archive.

Open that archive using its Lockbox password, a profile key, or a credential
resolved from the Vault. A profile signing key becomes a Lockbox owner key only
after explicit assignment.

## Secrets, exceptions, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are different
secrets. Use `SecretString`, `SecretBytes`, and mutable `bytearray` values;
close or clear them promptly. Python `str` and `bytes` objects cannot be
reliably wiped.

Use context managers for every Vault, Lockbox, key, secret, and agent value.
Secret callbacks receive a temporary `bytearray` that is cleared after return
and must not escape. Native failures raise typed exceptions with structured
details and recovery guidance.

## Optional session agent

Ordinary Lockbox opens never start or consult the agent. Use `AgentSession`
when Lockbox keys need to be shared across processes or remain available after
the process that opened the Lockbox exits. Closing an entry forgets the
agent's cached key; it does not delete the Lockbox or a credential stored in
the Vault.

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

Missing or placeholder package docstrings are binding defects; the executable
conformance program is the worked reference for each operation.
