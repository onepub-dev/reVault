# reVault for Go

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Go package links the version-matched native SDK.
See the [reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
go get github.com/onepub-dev/revault-api@v0.2.0
```

```go
box, err := revault.Create(make([]byte, 32)) // load a real key securely
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

Install the platform SDK before building so cgo can locate `revault_api`.
Secret callbacks receive a temporary byte slice; never retain it.
