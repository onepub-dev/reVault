# reVault for .NET

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

`Revault.Api` provides .NET classes for Lockboxes, Vaults, keys, and the
session agent, together with the matching reVault engine.

## Installation and native runtime

```shell
dotnet add package Revault.Api --version 0.3.11
```

`Revault.Load(nativeLibraryPath)` can load a native library supplied by the
application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the libraries
installed by NuGet under `runtimes/<RID>/native`. .NET selects the runtime
identifier for the current operating system and architecture and copies the
matching library to the build or publish output. Library selection is shared
by the process and must happen before the first reVault operation.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```csharp
using Revault;

var runtime = Revault.Load();
using var signing = runtime.GenerateProfileSigningKeyPair();
using var publicSigningKey = signing.PublicKey();
using var vault = runtime.OpenOrCreateVault(root, vaultPassphrase);
using var session = runtime.OpenLockboxSession();
using var box = session.OpenWithPassword(path, lockboxPassword);
box.SetOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.AddFile("/hello.txt", "hello\n"u8.ToArray(), replace: false);
box.SetVariable("owner", "alice");
box.SetSecretVariable("token", "secret"u8.ToArray());
box.WithSecretVariable("token", token => token.Length);
box.Commit();
```

`Revault.Load` does not open a Vault or Lockbox. A Vault passphrase, Lockbox
password, and content key are distinct caller-owned secrets. Native failures
are `RevaultException`; `AgentSession` explicitly controls temporary agent
content-key entries. Dispose all owned objects. Secret callbacks receive a
read-only span backed by a temporary buffer that is cleared
immediately after the callback returns.

## Create, open, and replace

`Vault.Open` and the Lockbox open methods require existing resources and never
create them. Use open or create only when either outcome is acceptable. The
replace APIs are destructive and should be called only after an explicit user
decision. `Commit` persists pending Lockbox changes; `Dispose` releases the
handle and key but does not commit or delete the archive.

Open the resulting archive with its Lockbox password or a credential resolved
from the Vault. A profile signing key becomes an owner key only after
`SetOwnerSigningKey` assigns that role.

## Secret values and resource lifetime

Keep vault passphrases, Lockbox passwords, and content keys in mutable buffers
and clear arrays owned by the caller in a `finally` block. Secret callbacks
receive a temporary `ReadOnlySpan<byte>`; never capture it, convert it to a
retained string, or return a reference to its backing memory.

All values that own native resources implement `IDisposable`. Use `using`
declarations for Vaults, Lockboxes, key pairs, public keys, and agent handles. A typed
`RevaultException` includes structured details suitable for error handling and
user guidance.

## Optional session agent

Normal Vault and Lockbox operations stay within this process and never start or
consult the agent. Use `AgentSession` when Lockbox keys need to be shared
across processes or remain available after the process that opened the Lockbox
exits. Closing an agent entry forgets
only that cached key; it does not delete the Lockbox or remove a password
remembered by the Vault.

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

## API documentation and support

The public classes are documented in the NuGet XML documentation generated from
this package. The [.NET conformance program](../e2e/csharp/Program.cs) is the
executable reference for complete workflows, cleanup, and typed errors. The
[example index](../API_EXAMPLES.md) tracks method coverage across releases.
Report missing or placeholder class/method documentation in the
[reVault issue tracker](https://github.com/onepub-dev/reVault/issues).
