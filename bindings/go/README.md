# reVault for Go

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

The Go package provides Lockbox, Vault, key, and session agent APIs that follow
Go conventions and use the matching reVault engine.

## Installation and native runtime

```shell
go get github.com/onepub-dev/revault-api@v0.3.11
```

The module contains a static library for each supported Go operating system
and architecture. Target specific cgo files link the matching library into the
application, so users do not install a separate SDK and `REVAULT_LIBRARY` does
not apply. A working cgo toolchain is required when building the application.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```go
_, err := revault.Load()
if err != nil { log.Fatal(err) }
box, err := revault.Create(make([]byte, 32)) // use a real content key securely
if err != nil { log.Fatal(err) }
defer box.Close()
if err := box.AddFile("/hello.txt", []byte("hello\n"), false); err != nil { log.Fatal(err) }
if err := box.SetVariable("owner", "alice"); err != nil { log.Fatal(err) }
if err := box.SetSecretVariable("token", []byte("secret")); err != nil { log.Fatal(err) }
if err := box.WithSecretVariable("token", func(token []byte) error {
    // token is cleared immediately after this callback.
    return nil
}); err != nil { log.Fatal(err) }
if err := box.Commit(); err != nil { log.Fatal(err) }
```

`Revault.Load` verifies the linked engine; it does not open a Vault or Lockbox.
`OpenVault` opens existing
persistent metadata; `ReplaceVault` is the explicit destructive constructor.
Vault passphrases, Lockbox passwords, and content keys are distinct caller
owned byte slices. Native failures return `NativeError`. `SessionAgent` is
explicit and caches only temporary content keys; it does not delete files or
persistent credentials. Secret callbacks receive a temporary byte slice;
never retain it.

## Core API concepts

- `Revault` verifies the engine linked into the application.
- `Vault` stores Profiles, Contacts and remembered Lockbox access.
- `Lockbox` owns an open archive until `Close`.
- `SessionAgent` explicitly caches selected content keys.

## API documentation and support

Use [pkg.go.dev](https://pkg.go.dev/github.com/onepub-dev/revault-api) for the selected module's class and method reference and the [Go repository](https://github.com/onepub-dev/revault-api) for releases. The [method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md) and [Go conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/go/conformance.go) cover the complete common operation inventory.

## Create, open, and replace

Open functions require an existing Vault or Lockbox and never create one.
Functions that open or create may create persistent state; replace functions are
destructive. Keep those choices explicit in application control flow.
`Commit` writes pending Lockbox changes. `Close` releases the handle and
content key held by this process; it does not commit or delete a file.

Use a path based open function with the Lockbox password, a profile key, or
access resolved from the Vault.

## Credentials, errors, and cleanup

Vault passphrases, Lockbox passwords, and 32-byte content keys are separate
byte slices owned by the caller. Clear them after use and avoid converting them
to immutable strings. A profile signing key becomes a Lockbox owner key only after
explicit assignment.

Check every returned `error`. Native failures are represented by `NativeError`
with stable structured details; the package does not expose mutable global
last error state. Close every owned value with `defer` immediately after a
successful constructor.

Secret callbacks receive a temporary mutable slice that is cleared after
return. The callback must not retain the slice or a subslice.

## Optional session agent

Ordinary Lockbox opens never start or consult the agent. Use `SessionAgent`
when Lockbox keys need to be shared across processes or remain available after
the process that opened the Lockbox exits. Closing an entry forgets the cached
content key, not the Lockbox file or a credential stored in the Vault.

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

## Version specific references

The [Go conformance program](../e2e/go/conformance.go) demonstrates complete
ownership and failure handling. The [example index](../API_EXAMPLES.md) tracks
method coverage, while pkg.go.dev renders the comments belonging to the
selected module version.
