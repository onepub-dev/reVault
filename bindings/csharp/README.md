# reVault for .NET

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. `Revault.Api` provides owned .NET classes and ships the
matching native runtime. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
dotnet add package Revault.Api --version 0.3.11
```

`Revault.Load(nativeLibraryPath)` selects an application-owned carrier.
Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before NuGet runtime
asset discovery. A bare library name delegates to the operating-system search
path. Native selection is process-wide and must happen before the first native
operation.

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```csharp
using Revault;

var runtime = Revault.Load(); // loads the installed carrier only
using var signing = runtime.GenerateProfileSigningKeyPair();
using var publicSigningKey = signing.PublicKey();
var vault = runtime.ReplaceVault(root, vaultPassphrase);
using var box = vault.OpenLockboxWithPassword(path, lockboxPassword);
box.SetOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.AddFile("/hello.txt", "hello\n"u8.ToArray(), replace: false);
box.SetVariable("owner", "alice");
box.SetSecretVariable("token", "secret"u8);
box.WithSecretVariable("token", token => token.Length);
box.Commit();
```

`Revault.Load` does not open a Vault or Lockbox. A vault passphrase, lockbox
password, and content key are distinct caller-owned secrets. Native failures
are `RevaultException`; `vault.AgentSession` explicitly controls temporary
agent content-key entries. Dispose all owned objects. Secret callbacks receive
a read-only span backed by a temporary buffer that is zeroed immediately after
the callback returns.
