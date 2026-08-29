# reVault binding package managers

reVault is developed in Rust and provides bindings for the languages below.
This table links each binding to its primary package manager or package
registry and records the package name used by reVault.

| Language | Package manager or registry | reVault package |
| --- | --- | --- |
| JavaScript | [npm](https://www.npmjs.com/) | `@onepub-dev/revault-api` |
| Python | [PyPI](https://pypi.org/) (`pip`) | `revault-api` |
| TypeScript | [npm](https://www.npmjs.com/) | `@onepub-dev/revault-api` (includes type declarations) |
| Java | [Maven Central](https://central.sonatype.com/) | `dev.onepub:revault-api` |
| C# | [NuGet](https://www.nuget.org/) | `Revault.Api` |
| C++ | [ConanCenter](https://conan.io/center/) and [vcpkg](https://vcpkg.io/) | `revault-api` |
| C | [vcpkg](https://vcpkg.io/), [Homebrew](https://brew.sh/), Debian APT, and RPM | `revault-api` / native SDK |
| PHP | [Packagist](https://packagist.org/) (`Composer`) | `onepub/revault-api` |
| Go | [Go modules](https://go.dev/ref/mod) and [pkg.go.dev](https://pkg.go.dev/) | `github.com/onepub-dev/revault-api` |
| Rust | [crates.io](https://crates.io/) (`Cargo`) | `revault-api` |
| Kotlin | [Maven Central](https://central.sonatype.com/) (`Gradle` or Maven) | `dev.onepub:revault-api-kotlin` |
| Lua | [LuaRocks](https://luarocks.org/) | `revault_api` |
| Ruby | [RubyGems](https://rubygems.org/) | `revault_api` |
| Dart | [pub.dev](https://pub.dev/) (`pub`) | `revault_api` |
| Swift | [Swift Package Manager](https://www.swift.org/package-manager/) | [`onepub-dev/revault-swift`](https://github.com/onepub-dev/revault-swift), product `RevaultAPI` |
| WebAssembly | [npm](https://www.npmjs.com/) | `@onepub-dev/revault-api-wasm` |

JavaScript and TypeScript share one npm package. C and C++ do not have a
single language-wide registry, so the table lists the package managers used by
the reVault release process. Swift Package Manager resolves `RevaultAPI` from
the dedicated `onepub-dev/revault-swift` publication repository.
