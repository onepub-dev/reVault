# reVault API

The reVault API allows you to create and maintain lockboxes and a vault from your own code.

reVault is developed in Rust but provides language bindings for the following languages:

This table links each binding to its primary package manager or package registry and records the package name used by reVault.

| Language    | Package manager or registry                                                   | reVault package                                        |
| ----------- | ----------------------------------------------------------------------------- | ------------------------------------------------------ |
| JavaScript  | [npm](https://www.npmjs.com/)                                                 | `@onepub-dev/revault-api`                              |
| Python      | [PyPI](https://pypi.org/) (`pip`)                                             | `revault-api`                                          |
| TypeScript  | [npm](https://www.npmjs.com/)                                                 | `@onepub-dev/revault-api` (includes type declarations) |
| Java        | [Maven Central](https://central.sonatype.com/)                                | `dev.onepub:revault-api`                               |
| C#          | [NuGet](https://www.nuget.org/)                                               | `Revault.Api`                                          |
| C++         | [ConanCenter](https://conan.io/center/) and [vcpkg](https://vcpkg.io/)        | `revault-api`                                          |
| C           | [vcpkg](https://vcpkg.io/), [Homebrew](https://brew.sh/), Debian APT, and RPM | `revault-api` / native SDK                             |
| PHP         | [Packagist](https://packagist.org/) (`Composer`)                              | `onepub/revault-api`                                   |
| Go          | [Go modules](https://go.dev/ref/mod) and [pkg.go.dev](https://pkg.go.dev/)    | `github.com/onepub-dev/revault-api`                    |
| Rust        | [crates.io](https://crates.io/) (`Cargo`)                                     | `revault-api`                                          |
| Kotlin      | [Maven Central](https://central.sonatype.com/) (`Gradle` or Maven)            | `dev.onepub:revault-api-kotlin`                        |
| Lua         | [LuaRocks](https://luarocks.org/)                                             | `revault_api`                                          |
| Ruby        | [RubyGems](https://rubygems.org/)                                             | `revault_api`                                          |
| Dart        | [pub.dev](https://pub.dev/) (`pub`)                                           | `revault_api`                                          |
| Swift       | [Swift Package Manager](https://www.swift.org/package-manager/)               | `RevaultAPI`                                           |
| WebAssembly | [npm](https://www.npmjs.com/)                                                 | `@onepub-dev/revault-api-wasm`                         |

JavaScript and TypeScript share one npm package. C and C++ do not have a single language-wide registry, so the table lists the package managers used by the reVault release process. Swift Package Manager resolves `RevaultAPI` directly from the reVault Git repository.

<br>



TODO: update these with the latest api.

Use `lockbox_core` when you need the portable storage engine:

```rust
use lockbox_core::{EnvName, Lockbox, LockboxPath, LockboxProtection, LockboxUnlock, SecretVec};
use std::path::Path;

let key = SecretVec::try_from_slice(b"correct horse battery staple")?;
let mut lockbox = Lockbox::create_file(
    Path::new("secrets.lbox"),
    LockboxProtection::ContentKey(key.try_clone()?),
)?;

lockbox.add_file(&LockboxPath::new("/docs/a.txt")?, b"alpha", false)?;
lockbox.add_file(&LockboxPath::new("/docs/b.txt")?, b"bravo", false)?;
lockbox.set_env(&EnvName::new("DATABASE_URL")?, "postgres://localhost/app")?;
lockbox.commit()?;

let reopened = Lockbox::open_file(
    Path::new("secrets.lbox"),
    LockboxUnlock::ContentKey(key),
)?;
let file = reopened.get_file(&LockboxPath::new("/docs/a.txt")?)?;
let env = reopened.get_env(&EnvName::new("DATABASE_URL")?)?;
```

Use `lockbox_vault` for native applications that want the local vault and unlock-cache behavior:

```rust
use lockbox_vault::{local_vault, SecretString};

let vault = local_vault();
let password = SecretString::try_from_bytes(b"pw".to_vec())?;

vault.create_lockbox_with_password("secrets.lbox", &password)?;
vault.unlock_lockbox_with_password("secrets.lbox", &password)?;

let mut lockbox = vault.open_lockbox("secrets.lbox")?;
lockbox.add_file("notes.txt", "/notes.txt")?;
lockbox.commit()?;

vault.lock_lockbox("secrets.lbox")?;
```
