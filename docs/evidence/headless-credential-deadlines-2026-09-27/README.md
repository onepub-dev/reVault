# Headless credential-store deadlines (#322)

The Linux credential path now has an overall five-second deadline and refuses
locked Auto Open credentials without requesting a desktop unlock prompt. It retains
encrypted Secret Service sessions and existing credential attributes. The optional
system-bus connection also has a bounded authentication handshake.

The private provider tests exercise real D-Bus transport, independent DH/HKDF/CBC
session encoding, and the public Vault credential API. They use only synthetic
credentials, isolated buses and child process environments; neither the host bus
nor a real desktop keychain is involved. Test output is checked not to contain the
synthetic credential. Actual keyring-client interoperability is tested in both
directions so backend replacement does not strand existing credentials.

Validation includes the full Vault suite, the existing headless CLI lifecycle,
stalled system-bus setup, Windows cross-compilation, dependency inspection and
strict affected-package Clippy. The CLI test independently reopens persisted data
and compares its bytes after create, replacement and deletion. The full Vault run passed 25 unit tests, nine private-service integration tests,
23 Vault integration tests and two doctests. The new system-handshake timeout and
existing logind/inhibitor tests both pass separately. The CLI lifecycle, Windows
check and strict Clippy pass. Post-hook validation passes all 26 unit tests, nine private-service tests, 23 Vault
integration tests, two doctests and strict Clippy.

The normal CLI graph still contains `secret-service` and `zbus`, with no
`libdbus-sys` in normal/build dependencies. The old Linux keyring client is used
only as a test interoperability control; native macOS/Windows keyring use remains.
A fresh-target, offline release-mode `cargo install --locked --offline --path
revault_cli` also succeeds in the retained `rust:1.88-bookworm` image without
`libdbus-1-dev`. Registry sources are mounted read-only; host credentials and
compiled artifacts are not supplied. The installed executable links no libdbus.
The first runtime attempt failed because the existing test harness writes below
the read-only source mount. Giving only that fixture directory a writable isolated
mount resolves the harness issue; the unchanged installed release executable then
passes the complete headless lifecycle in 20.81 seconds. Both logs are retained.

The image digest remains
`sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`.
The source revision for this build is `1baec9c7` (subsequent evidence-only amendments
do not change Rust source). Remote CI, real macOS/Windows store behavior and packaged
binding interoperability are not claimed by these local checks.

See [test scope and operational behavior](../../testing/headless-installation.md).
