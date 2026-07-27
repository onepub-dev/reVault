# Package registry quality audit

This document tracks how each reVault binding is evaluated by its public
package registry. A registry validation failure blocks publication. Numeric
scores are recorded only where the registry actually publishes one; other
registries expose validation, search, provenance, documentation, compatibility,
or maintenance signals instead.

## Current position

| Binding | Public package site | Evaluation model | Repository status |
| --- | --- | --- | --- |
| Dart | pub.dev | 160-point `pana` report | 150/160; every recoverable category is full |
| Swift | Swift Package Index | Public score based on ten repository traits | README, DocC, platform declaration, and test target present |
| Go | pkg.go.dev | Documentation, license, tagged/stable version, and build-context signals | Package overview and module metadata present |
| JavaScript | npm | Keyword relevance, downloads, dependents, recency, and provenance; no subjective quality score | README, keywords, links, engines, types, and trusted publication configured |
| WebAssembly | npm | Same npm signals | README, keywords, links, engines, types, and trusted publication configured |
| Python | PyPI | Metadata validation, classifiers, project links, wheel compatibility, and trusted-publisher provenance | README, license reference, classifiers, keywords, links, and platform wheels configured |
| Java | Maven Central | Mandatory POM metadata, signatures, source and Javadoc artifacts | Complete name, description, URL, license, developer, and SCM metadata configured |
| Kotlin | Maven Central | Same Maven Central requirements | Complete POM metadata and Java runtime dependency configured |
| C# | NuGet | Package validation plus README, license, repository, framework, and package metadata signals | README, XML docs, tags, repository, license, and native runtime assets configured |
| Ruby | RubyGems | Gem validation and package-page metadata; no numeric score | README, license, author, support, documentation, source, and changelog links configured |
| PHP | Packagist | Composer schema, installability, metadata, release activity, downloads, dependents, and advisories | Keywords and documentation, source, issue, and security links configured |
| Lua | LuaRocks | Rockspec validation and installability; no numeric score | Summary, detailed description, homepage, license, dependency, and native layout configured |
| Rust | crates.io and docs.rs | Cargo package validation, discoverability metadata, documentation build, downloads, and dependents | README, rustdoc, license file, keywords, categories, repository, and MSRV configured |
| C | GitHub, Homebrew, vcpkg, Debian, and RPM | Formula/port/package policy and installation tests | ABI header docs, license, metadata, target constraints, and smoke tests configured |
| C++ | ConanCenter, vcpkg, Homebrew, Debian, and RPM | Recipe review, metadata completeness, build matrix, and package tests | RAII docs, Conan topics/description, vcpkg metadata, and package-manager tests configured |

## Dart score

The package was measured locally with `pana` 0.23.14, the analyzer used by
pub.dev. The result is 150/160:

- 20/30 conventions: valid pubspec, README, and changelog; the ten unavailable
  points require an OSI-approved license.
- 20/20 documentation: documented public facade and a package example.
- Platform points for the explicitly supported Linux, macOS, and Windows
  targets.
- 50/50 static analysis: no errors, warnings, formatting issues, or core lints.
- 40/40 dependency health: current dependencies, current Dart/Flutter support,
  and compatible lower bounds.

The project uses the reVault Source Available License 1.0. It must not be
relabelled as an OSI license to gain points. A licensing decision is the only
legitimate route to the final ten pub points.

## Swift score

Swift Package Index scores whether a repository is archived, its license,
release count, stars, dependency count, recent activity, documentation,
contributors, test targets, and README. Repository-controlled traits are now
present: README, DocC documentation, a test target, bounded dependencies, a
declared macOS platform, and current activity. Release count, stars, and
contributors grow through normal project use. License points depend on whether
the current source-available license meets the index's App Store compatibility
test and must not be optimized by inaccurate metadata.

## Non-numeric registry signals

- npm search uses package name, description, README, and keywords without a
  subjective quality ranking. Trusted publishing supplies visible provenance.
- PyPI surfaces standards-based metadata, project URLs, compatible wheel tags,
  and trusted-publisher provenance. It does not publish a package quality score.
- Maven Central enforces complete POM metadata and signed main, source, Javadoc,
  and POM artifacts.
- NuGet emphasizes a rendered package README, license, repository metadata,
  target frameworks, validation, signing, and ownership signals.
- RubyGems, Packagist, and LuaRocks validate their native manifests and surface
  documentation, source, support, release, dependency, and adoption metadata.
- crates.io validates Cargo packages and uses README, rustdoc, keywords,
  categories, license, repository, releases, downloads, and dependents as
  discoverability and trust signals.
- pkg.go.dev highlights package documentation, recognized licensing, tagged
  versions, v1 stability, and supported build contexts.
- ConanCenter and vcpkg use recipe/port review and build matrices rather than a
  consumer-facing numeric score. Homebrew, Debian, and RPM likewise enforce
  package policy and installation behavior.

## Automated gate

`revault-tool bindings check` now verifies the registry-facing metadata and
quality files alongside the public API documentation gate. Registry-native dry
runs are also enforced by `bindings-lint.yml`: the workflow holds the pinned
`pana` 0.23.14 result to a maximum ten-point deduction, performs a Dart
publication dry-run, validates both npm payloads, builds and checks the PyPI and
NuGet artifacts, validates Composer and LuaRocks metadata, builds the Ruby gem,
packages the Rust crate, and verifies that the Swift package manifest can be
loaded before compiling and testing it. The npm reports and built Python,
NuGet, and Ruby packages are retained as a CI artifact for inspection.

These checks validate the source publication surfaces. The native release
workflow separately installs the assembled native-carrier packages and runs
the complete package conformance matrix, because only staged release payloads
contain every platform library.

## Remaining decisions and external signals

- Relicensing would affect pub.dev, Swift Package Index, and license visibility
  elsewhere. It is a product/legal decision, not a release-engineering fix.
- A v1 Go tag should be published only when the API is genuinely stable.
- Stars, downloads, dependents, contributors, likes, and release history cannot
  be manufactured by package metadata.
- NuGet signing and registry ownership/prefix indicators should be enabled once
  the registry identities and signing policy are finalized.
- Package-page scores must be rechecked from the staged release payload because
  native carrier assembly happens after source manifests are copied.

## Primary criteria

- [pub.dev scoring](https://pub.dev/help/scoring)
- [Swift Package Index scoring](https://swiftpackageindex.com/blog/revealing-and-explaining-package-scores)
- [npm search and package selection](https://docs.npmjs.com/searching-for-and-choosing-packages-to-download/)
- [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)
- [Python project metadata](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/)
- [Maven Central requirements](https://central.sonatype.org/publish/requirements/)
- [NuGet package README](https://learn.microsoft.com/nuget/nuget-org/package-readme-on-nuget-org)
- [RubyGems specification](https://guides.rubygems.org/specification-reference/)
- [Composer schema](https://getcomposer.org/doc/04-schema.md)
- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [pkg.go.dev package signals](https://pkg.go.dev/about)
