# reVault for Swift

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

SwiftPM provides a `RevaultAPI` product with Lockbox, Vault, key, and session
agent classes for macOS and Linux.

## Installation and native runtime

```swift
.package(url: "https://github.com/onepub-dev/revault-swift", exact: "0.3.11")
```

On macOS, the published Swift package downloads the release's
`RevaultC.xcframework` as a SwiftPM binary target. On Linux, it uses a system
library target, so install the matching reVault C SDK where the compiler and
dynamic linker can find it. The Swift API links through `RevaultC`; it does not
use `REVAULT_LIBRARY` or select a library at runtime.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```swift
let runtime = Revault()
let signing = try runtime.generateProfileSigningKeyPair()
let publicSigningKey = try signing.publicKey()
let box = try runtime.lockboxCreate(Data(repeating: 0, count: 32))
try box.setOwnerSigningKey(signing) // Profile becomes this Lockbox's owner
try box.addFile("/hello.txt", Data("hello\n".utf8), false)
try box.setVariable("owner", "alice")
try box.setSecretVariable("token", Data("secret".utf8))
try box.withSecretVariable("token") { token in token.count }
try box.commit()
try box.free()
try publicSigningKey.dispose()
try signing.dispose()
```

`Revault()` loads the installed runtime facade; loading does not open a Vault or
Lockbox. A Vault passphrase, Lockbox password, and content key are distinct
caller-owned secrets. Native failures are thrown as `RevaultError`. Agent use
is explicit via `AgentSession`; closing an entry forgets only temporary cached
content keys. Secret callbacks receive a temporary raw buffer that is zeroed
after return; do not convert it to a retained `String` or `Data`.

## Core API concepts

- `Revault` is the native runtime entry point.
- `Vault` stores Profiles, Contacts and remembered Lockbox access.
- `Lockbox` owns an open archive and keeps its content key in this process.
- `AgentSession` explicitly caches selected content keys.

## API documentation and support

The [Swift package repository](https://github.com/onepub-dev/revault-swift)
contains the release README and DocC catalogue. Build DocC for the selected
package version for exact class and method signatures. The
[method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [Swift conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/swift/Sources/RevaultConformance/main.swift)
cover the common operation inventory.

## Create, open, and replace

Open methods require existing Vaults and Lockboxes and never create missing
state. Use open or create only when creation is acceptable and replacement
methods only after an explicit destructive choice. `commit()` persists
pending Lockbox changes; `close()` or `dispose()` releases handles and keys
held by this process without deleting the archive.

Open that archive with its Lockbox password, a profile key, or access resolved
from the Vault. A profile signing key becomes a Lockbox owner key only after
`setOwnerSigningKey`.

## Secrets, errors, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are distinct
secrets owned by the caller. Keep them in mutable buffers, avoid conversion to
`String`, and clear them after use. Secret callbacks receive a temporary
buffer that is cleared after return and must not escape the closure.

Use `defer` immediately after acquiring every owned Vault, Lockbox, key,
secret, or agent value. Native failures throw `RevaultError` with structured
details and recovery guidance.

## Optional session agent

Ordinary Lockbox opens never start or consult the agent. Use `AgentSession`
when Lockbox keys need to be shared across processes or remain available after
the process that opened the Lockbox exits. Closing an entry forgets the cached
key; it does not delete the Lockbox or credentials stored in the Vault.

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

Missing or placeholder DocC class/method documentation is a binding defect.
