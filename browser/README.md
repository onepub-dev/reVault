# reVault browser delegation

Experimental Chrome/Chromium and Firefox extensions and a ciphertext-only browser
SDK. No WASM secret operations, monitor code, or passkey implementation are included.

See [build, pairing, protocol, security and server integration](../docs/browser-integration.md).
From `rust/`, run `cargo xtask build-browser` to generate the unpacked extension
bundles and `browser/dist/sdk.js`; run `cargo xtask test-browser` for relay tests.
Real-secret delegation requires an independent focused security review.
