# Binding changelog

## 0.3.12

- Fix the C# package's private FlatBuffers namespace so the public
  `Revault.Revault` class no longer prevents the binding from compiling.
- Add a runnable Dart vault-and-lockbox quick start and clarify installation,
  platform support, package examples, and manual links across the language
  binding READMEs.
- Declare Dart WebAssembly support and correct Dart and Go source formatting
  required by their registry validation gates.
- Publish the Composer companion repository through Packagist's update API and
  retain automatic updates through its GitHub integration.
- Harden the binding release promotion workflow and parallelize the Linux CLI
  build and test phases.

## 0.3.0

- Document every supported facade member across C, C++, C#, Dart, Go, Java,
  Kotlin, JavaScript/TypeScript, Lua, PHP, Python, Ruby, Rust, Swift, and WASM.
- Make low-level native operation layers internal where each language permits.
- Enforce public API and canonical schema documentation in the binding gate.
- Treat the resulting source-level visibility changes as a pre-1.0 minor
  release for every binding package.

## 0.2.0

- Initial complete lockbox and vault binding release.
