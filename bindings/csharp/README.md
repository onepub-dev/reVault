# reVault for .NET

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. `Revault.Api` provides owned .NET classes and ships the
matching native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
dotnet add package Revault.Api --version 0.2.0
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```csharp
using Revault;

var runtime = Revault.Load(); // loads the installed carrier only
var vault = runtime.ReplaceVault(root, vaultPassphrase);
using var box = vault.OpenLockboxWithPassword(path, lockboxPassword);
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
