# Contributing and releasing language bindings

The language bindings are handwritten public facades over one generated native
ABI. Changes must keep the language APIs, generated FlatBuffer models, native
carriers, examples, and registry metadata synchronized.

## Documentation and examples

Each binding owns a language-specific `README.md`. It should contain:

- the package name and supported platforms;
- the ecosystem-native installation command;
- a complete runnable first example;
- the language's resource-lifetime and secret-handling conventions;
- links to version-matched examples shipped in the package; and
- links to the shared format, security, and migration documentation.

Ship at least one runnable example in the registry package whenever that
ecosystem supports examples. Keep exhaustive cross-language method coverage in
[`API_EXAMPLES.md`](API_EXAMPLES.md) and link to its stable GitHub URL from
published packages. Do not use a relative link that escapes a registry package.

Consumer READMEs should not contain release credentials, staging paths, or
operator-only commands.

## Generated code

Do not hand-edit generated FlatBuffer sources. Regenerate them from the shared
schema and review both the schema change and generated diff. Public facade
changes should preserve the terminology in `docs/terminology.md` and update the
conformance adapters for every affected language.

## Validation

Before publishing, run the binding lint workflow and the installed-package
conformance matrix. A package is releasable only when it installs from its final
registry layout and passes the interoperability suite against the other
bindings.

The canonical automation is in:

- `.github/workflows/bindings-lint.yml`
- `.github/workflows/bindings-native-preflight.yml`
- `.github/workflows/bindings-native-release.yml`

## Release process

Release versions come from a signed `revault-api-vX.Y.Z` tag or an explicit
publication retry using the same retained preflight artifacts. The release
assembler rewrites staged manifests to that version; do not prepare a release
by editing every source manifest manually.

The native preflight must complete before publication. It builds the supported
carriers, assembles registry-native packages, installs those packages in clean
consumers, and runs the cross-language interoperability suite. The release
workflow then publishes immutable assets and promotes registry packages.

Publication retries must use the exact successful preflight run and source SHA.
Never replace a public version or move a tag after a registry or release asset
has accepted it. Correct published defects with a new patch version.

Registry credentials and trusted-publisher policies belong to the protected
GitHub `release` environment. Long-lived credentials should be limited to
registries that do not support short-lived or trusted authentication.

For package coordinates, companion repositories, trusted publishers, and
registry-specific prerequisites, see the
[bindings distribution guide](README.md#distribution-and-publication).
