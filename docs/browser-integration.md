# Browser authorisation for reVault

Status: experimental, disabled in ordinary builds. Use synthetic lockboxes only
until an independent focused security review approves real-secret delegation.
The implementation supplies a generic reVault integration; application login,
passkeys, HTTP authentication, CSRF middleware and any particular consumer are
outside its scope. Safari, Windows and macOS adapters are later milestones.

## Boundary and threat model

An authenticated website requests an unlock through a browser SDK, the reVault
extension, a Native Messaging executable and the existing Session Agent. Only
the agent resolves local paths, accesses local profile grants, obtains Auto Open
credentials and encrypts the content key. The adapter has no key cache. There is
no localhost HTTP listener. Portable browser WASM is unchanged and cannot access
the OS credential store or Session Agent.

The integration protects against unpaired websites, webpage origin spoofing,
iframe requests, recipient/lockbox substitution, modified signed metadata,
modified ciphertext, stale challenges and replay. A logged-in browser session
never substitutes for explicit unlock consent. Every unlock displays a native
reVault approval dialog, including when the local lockbox is already open.

The local OS account and installed extension are trusted. A compromised account,
extension, approved website or approved server can undermine the protection.
Same-user native code can already use reVault's ordinary local APIs; extension
IDs and process arguments are not a security boundary against that account.
This does not protect contents from the server after authorised release. A
content key grants access to **all contents**, including every credential class
in the lockbox. A read-only server handle cannot prevent a malicious recipient
from implementing its own reader or retaining the delivered key.

## Build and install on Linux

Build both the existing CLI/agent and the new thin executable with the opt-in
feature, from `rust/`:

```console
cargo build -p revault_cli --features browser-integration --bins
cargo xtask build-browser
```

The browser build uses the pinned `browser/package-lock.json`, generates the
JavaScript protobuf codec, and bundles the SDK and extension scripts. Node/npm
and `protoc` are needed for generation. Normal Rust consumers use checked-in
protobuf types and do not need `protoc`. Regenerate those types with
`cargo xtask generate-browser-protocol` after changing the schema.

Install `/usr/bin/zenity` using your Linux distribution's package manager. It is
the trusted approval UI, launched directly without a shell or Pango markup.
Denial has default keyboard focus. Missing GUI support returns
`approval_required`; it never approves automatically. An unavailable or
interactive OS secure store may require separate local action. Silent unlocking
is not promised.

Load `browser/chromium/` as an unpacked extension in a browser supporting that
development mode. Load `browser/firefox/manifest.json` through Firefox's temporary
add-on workflow for development. Browser-store signing and distribution are not
part of this experimental installer. Obtain the actual Chromium extension ID
from its extension-management page. Firefox's manifest specifies
`browser@revault.onepub.dev`.

From the location where the executable will remain installed:

```console
revault-browser install CHROMIUM_EXTENSION_ID browser@revault.onepub.dev
revault-browser hosts
```

The installer writes `dev.revault.browser.json` to the standard per-user native
host directories for Google Chrome, Chromium and Firefox. Each manifest permits
exactly the supplied extension ID; wildcards are refused. It also records the
accepted caller identities locally. `hosts` reads back the installed manifests.
Repeating installation is idempotent. A different extension ID does not inherit
an existing pairing. `REVAULT_BROWSER_INSTALL_ROOT` can stage the three standard
relative directories under an isolated root for packaging/tests; normal installs
use the user's home directory.

Stop an older Session Agent and open the synthetic lockbox with the new CLI:

```console
lbx session stop
lbx /absolute/path/synthetic.lbox open
```

The browser helper deliberately does not start another process as a key cache.
The new agent starts its dedicated `browser.sock` alongside its existing socket,
in the same owner-only directory with the same peer UID/GID checks. Ordinary
agent IPC remains unchanged. The feature must be built into the CLI that starts
the agent, not just the helper.

## Explicit pairing

Confirm the server's long-lived **Ed25519 signing key** through an independent
trusted setup channel. Do not obtain the trust anchor solely from the same
webpage whose identity is being verified. Subsequent short-lived requests are
signed by that key, including the per-process encryption key and challenge.

Create a local JSON pairing file (never send it through the webpage protocol):

```json
{
  "origin": "https://secrets.example.com",
  "application_id": "example-service",
  "application_name": "Example service",
  "lockbox_id": "CANONICAL-LOCKBOX-ID",
  "lockbox_path": "/absolute/canonical/path/service.lbox",
  "profile": "default",
  "server_signing_key": ["REPLACE WITH 32 INTEGER BYTES FROM THE VERIFIED KEY"],
  "extension_id": "ACTUAL-EXTENSION-ID",
  "disclosure": "Contains diagnostic and staging credentials. Both will be available to this server."
}
```

The signing-key line above is a placeholder, not a valid key. Supply exactly 32
integer bytes. The canonical lockbox ID and path must identify the same existing
file. The local profile must already have a grant. The disclosure is required;
list every credential class the recipient will gain access to.

```console
revault-browser pair /path/to/local-pairing.json
revault-browser list
```

The native dialog shows the exact HTTPS origin, local application name, lockbox,
entire-lockbox scope, disclosure and full pinned key in hexadecimal. Confirm the
key out of band before approving. Pairing produces a local ID. It refuses an
existing mapping: replacing server identity requires explicit revocation and
another approval. HTTP, URL paths, userinfo and noncanonical origins are refused;
nondefault HTTPS ports are supported and form part of the exact origin.

```console
revault-browser revoke PAIRING_ID
revault-browser uninstall
```

Revocation is checked again after approval and after loading/encrypting the key.
`uninstall` removes native host manifests and all local browser pairings. Remove
the browser add-on separately. Replay tombstones remain. Neither action can
revoke a content key a server has already received.

## Protocol and cryptography

The schema is [`browser.proto`](../rust/revault_browser_protocol/proto/browser.proto).
Only `GetCapabilities`, `RequestUnlock` and `CancelRequest` exist. JSON is limited
to `{ "payload": "BASE64_PROTOBUF" }` inside the browser's native-endian 32-bit
length framing. Decoded protobuf is capped at 16 KiB, JSON at 24 KiB. Lengths are
checked before transport allocation. Unknown JSON keys, invalid base64, malformed
frames and unsupported versions fail closed. Stdout contains protocol responses
only. Error states never include OS errors, paths, private keys or payloads.

The extension gets the origin from the browser's MessageSender URL/origin and
checks it against the current top-level tab. It rejects nonzero frame IDs and
messages from other extensions. All page-supplied origin, frame and extension
fields are overwritten. Browser-page messages are untrusted even after login.

The server signs `SIGNING_DOMAIN || canonical UnlockRequest protobuf bytes`,
where `SIGNING_DOMAIN` is `reVault browser request v1` followed by one zero byte.
Canonical encoding is the checked-in prost encoder's field order, omitting
ordinary default scalar fields. Use the Rust `validation::signing_bytes` helper;
other language implementations must match it. There are no maps in the schema.
The agent verifies with the local pinned Ed25519 key, using strict verification.
Requests last at most 120 seconds and cannot be issued in the future.

Encryption is RFC 9180 HPKE Base mode using DHKEM(X25519, HKDF-SHA256),
HKDF-SHA256 and ChaCha20Poly1305. The `hpke` crate implements the construction.
The HPKE info is `reVault browser unlock v1` followed by one zero byte. The entire
canonical `UnlockRequest` is authenticated additional data. This binds version,
origin, application, boot ID, recipient key ID/public key, request ID, challenge,
lockbox, scope, issue time and expiry. The envelope carries the encapsulated key,
32-byte ciphertext, detached tag and public binding metadata.

Only the 32-byte lockbox content key is sealed. Vault passphrases and profile
private keys never enter browser messages. The payload is encrypted/decrypted
in the existing `SecretVec` protected allocator. Recipient keys at rest in the
receiver process also use that allocator. Cryptographic libraries necessarily
use transient native working values: the HPKE X25519 private-key wrapper zeroizes
on drop, and explicit seed/serialization buffers are wiped. This does not claim
that every register, stack temporary or third-party key-schedule allocation is
locked memory. Review those library boundaries before production enablement.

HPKE Base mode does not independently attest the agent's identity to the server.
The server must check its own stored challenge and successfully open the actual
lockbox before considering access available. An envelope's expiration limits
acceptance; it cannot revoke a content key after delivery.

## Browser SDK

The bundled ESM SDK is `browser/dist/sdk.js`, with declarations in
`browser/src/sdk.d.ts`. It is separate from portable WASM.

```javascript
import { getCapabilities, requestUnlock, cancelRequest } from './revault-browser.js';

const capabilities = await getCapabilities();
// Obtain SignedUnlockRequest protobuf bytes from your authenticated server.
const result = await requestUnlock(serverRequestBytes);
if (result.state === 'ok' && result.envelope) {
  // POST result.envelope as protobuf through your session-authenticated,
  // CSRF-protected application endpoint. Do not store the envelope.
}
// To cancel, use the server request's request_id:
await cancelRequest(requestId);
```

States include `extension_missing`, `native_helper_missing`, `upgrade_required`,
`vault_locked`, `approval_required`, `denied`, `expired`, `unpaired_recipient`,
`lockbox_unavailable`, `secure_store_unavailable`, `invalid_request`, `replayed`,
`cancelled`, `busy`, `internal_error` and `ok`. A missing or incompatible agent
needs local action. Cancellation is scoped to the origin and installed extension.
Cancellation before registration is remembered briefly. Once an envelope has
been delivered, cancellation cannot recall it.

## Generic server contract

[`Receiver`](../rust/revault_browser_protocol/src/receiver.rs) owns a fresh
recipient encryption key and random 32-byte boot ID for each process/lock epoch.
It creates short-lived challenges and keeps their session bindings server-side.
`issue` accepts a signing callback, keeping the long-lived pairing key separate
from lockbox-decryption material. `accept` consumes a matching challenge before
decryption and rejects wrong-session, changed, expired or replayed envelopes.

The embedding application must:

1. Authenticate users and derive an unguessable 32-byte session binding on the
   server. Never accept that binding directly from a request body.
2. Expose challenge issuance only to authenticated users. Apply rate limits.
3. Accept protobuf envelopes only over an authenticated, CSRF-protected endpoint;
   impose a body limit before reading/decoding. Bind to that endpoint's current
   authenticated session and call `Receiver::accept` atomically.
4. Import `UnlockMaterial` with `open_lockbox`. It opens a read-only handle and
   verifies the actual lockbox ID, without any server-side Session Agent cache.
   Keep no password or profile private key on the server.
5. Drop the handle, all decrypted application data and `Receiver` on explicit
   lock/shutdown. Construct a new receiver when unlocking is next needed. Do not
   persist recipient keys, unlock material, envelopes or decrypted secrets.
6. On process restart, begin without an unlocked handle even if browser login
   sessions remain valid. Issue a fresh challenge and require fresh native consent.

The reVault receiver API does not implement your HTTP session or CSRF policy.
Returning `ok` from the browser SDK is not evidence that the server accepted an
unlock. A user authenticated to the application still must approve reVault.

## Recovery and lifecycle

* **Server identity rotation:** revoke the mapping, independently verify the new
  signing key, then pair again. Ephemeral recipient keys rotate each boot and
  never replace the pinned signing identity.
* **Content-key rotation:** use reVault's existing access/key-rotation workflows.
  The agent validates a cached key against the current file and otherwise opens
  the current profile grant. Existing server handles/keys must be discarded.
  Changing a lockbox ID requires a new explicit mapping.
* **Agent restart:** replay hashes and a clock high-water mark survive on disk.
  The journal contains no keys or plaintext. Expired entries are pruned; clock
  rollback behind the recorded high-water mark fails closed.
* **Suspend:** the existing sleep watcher clears local cache entries and advances
  the browser authorisation epoch. Pending approvals cannot release keys after
  that event. Platforms lacking a functioning sleep watcher require review;
  existing agent configuration still controls general sleep protection.
* **Concurrency:** at most eight unlock approvals, twelve browser IPC connections,
  1,024 short-lived cancellations and 4,096 live replay hashes are retained. Limits
  fail closed with `busy` or transport failure. No unbounded key cache is added.
* **Lost browser/device:** revoke its pairing locally. Restoring login credentials
  does not restore delegation consent. Application login recovery belongs to the
  application; vault/profile recovery remains reVault's existing recovery flow.

## Verification and review gate

From `rust/`:

```console
cargo test -p revault_browser_protocol
cargo test -p revault_vault_api@0.0.10 --features browser-integration browser::
cargo test -p revault_cli --features browser-integration --test browser_delegation
cargo xtask test-browser
cargo xtask check-required
cargo xtask clippy-advisory
```

The policy tests substitute native approval and key sources only inside the
private Rust test module; production has no environment variable or browser
operation for automatic approval. CLI lifecycle tests create a synthetic profile
lockbox through public commands, read back stored bytes through a separate CLI
invocation, delegate into the receiver, compare bytes, discard the receiver and
handle, reject replay, then approve a fresh request and compare bytes again.
The documented exception is extracting the content key through the narrow Rust
profile-grant API because the public CLI intentionally cannot export raw keys.

The `browser_native` target in `rust/fuzz` exercises framing, JSON/base64 and
protobuf parsing. Run a bounded coverage-guided campaign with the repository's
cargo-fuzz toolchain, e.g. `cargo fuzz run browser_native -- -max_total_time=60`.
The normal protocol suite also runs a deterministic malformed-input corpus.

The [`synthetic_receiver`](../rust/revault_browser_protocol/examples/synthetic_receiver.rs)
example is a local receiver for a synthetic lockbox, not an HTTP/login service.
Its signing key is deliberately public test material. The example and tests must
never be used for real secrets. A real-browser/native-dialog acceptance run must
verify the chain in both Chrome/Chromium and Firefox on the target desktop.

Independent focused review is still required before real-secret deployment.
Review origin/MessageSender handling and navigation, exact extension host
registration, trusted UI and cancellation races, pairing/revocation atomicity,
replay persistence and rollback, suspend behavior, HPKE canonical binding and
library secret temporaries, and the consumer's session/CSRF/restart policy.
Passing automated tests does not replace that review or browser-store review.

References: [RFC 9180](https://www.rfc-editor.org/rfc/rfc9180),
[HPKE in-place encryption](https://docs.rs/hpke/0.13.0/hpke/fn.single_shot_seal_in_place_detached.html),
[Chrome Native Messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging),
[Firefox Native Messaging](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging).
