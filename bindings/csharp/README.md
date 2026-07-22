# reVault for .NET

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. `Revault.Api` provides owned .NET classes and ships the
matching native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
dotnet add package Revault.Api --version 0.2.0
```

```csharp
using Revault;

var vault = new Vault();
using var box = vault.CreateLockbox(new byte[32]); // load a real key securely
box.AddFile("/hello.txt", "hello\n"u8.ToArray(), replace: false);
box.SetVariable("owner", "alice");
box.SetSecretVariable("token", "secret"u8);
box.WithSecretVariable("token", token => token.Length);
box.Commit();
```

Dispose all owned objects. Secret callbacks receive a read-only span backed by
a temporary buffer that is zeroed immediately after the callback returns.
