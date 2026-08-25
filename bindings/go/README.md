# reVault for Go

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Go package links the version-matched native SDK.
See the [reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
go get github.com/onepub-dev/revault-api@v0.2.0
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```go
runtime, err := revault.Load() // verifies the installed carrier
box, err := revault.Create(make([]byte, 32)) // use a real content key securely
if err != nil { log.Fatal(err) }
defer box.Close()
_ = box.AddFile("/hello.txt", []byte("hello\n"), false)
_ = box.SetVariable("owner", "alice")
_ = box.SetSecretVariable("token", []byte("secret"))
_ = box.WithSecretVariable("token", func(token []byte) error {
    // token is cleared immediately after this callback.
    return nil
})
_ = box.Commit()
```

`Revault.Load` does not open a Vault or Lockbox. `OpenVault` opens existing
persistent metadata; `ReplaceVault` is the explicit destructive constructor.
Vault passphrases, lockbox passwords, and content keys are distinct caller
owned byte slices. Native failures return `NativeError`. `SessionAgent` is
explicit and caches only temporary content keys; it does not delete files or
persistent credentials. The module contains a version-matched static carrier
for each supported Go target, so cgo links the carrier without a separate SDK
installation. Secret callbacks receive a temporary byte slice; never retain
it.
