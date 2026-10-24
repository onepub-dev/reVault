# Reciprocal contact exchange

Every binding delegates exchange signing, validation, HTTP and persistence to
Rust. Each person shares one Profile's encryption and signing public keys and
independently verifies the other person's identity. The relay does not establish
identity for either participant.

Open a `ContactExchange` session using an existing Vault **file** path and its
Vault passphrase. The session owns a protected copy of that passphrase, but no
long-lived Vault lock. The caller still owns its input and must wipe or close it.
Network calls and application prompts run without holding a Vault handle.
Closing the session releases its credential copy without deleting contacts or
cancelling invitations. Do not close it concurrently with an operation.

Both Profiles need an email configured before starting. The relay argument is
an HTTPS origin, without a path, query or fragment; loopback HTTP is supported
for local testing. Invitations expire after 24 hours unless a lifetime from
1–168 hours is selected.

## Two-person workflow

For example, Alice's Dart application creates an invitation:

```dart
final session = ContactExchange.open(vaultFile, vaultPassphrase);
try {
  final offered = session.invite(
    'https://keys.example.com', 'default', 'bob@example.com',
  );
  // Deliver offered.invitation only to Bob. Its holder can accept the offer.
  // If offered.relayPending is true, session.list() retries delivery.
} finally {
  session.close();
}
```

Bob accepts the complete invitation URL with his selected Profile:

```dart
final session = ContactExchange.open(vaultFile, vaultPassphrase);
try {
  final reply = session.accept(invitationReceivedFromAlice, 'default');
  displayForIndependentComparison(reply.fingerprint);
} finally {
  session.close();
}
```

Each application uses online `list` to fetch the reciprocal response. The user
compares both identities and **all 16 fingerprint groups** over a known phone
call, in person, or an independently authenticated messenger. Only after that
comparison does Alice save Bob:

```dart
final session = ContactExchange.open(vaultFile, vaultPassphrase);
try {
  session.verify(
    exchangeId, 'bob', fingerprintComparedWithBob,
    ComparisonChannel.knownPhoneCall,
  );
} finally {
  session.close();
}
```

Bob verifies separately, using local contact name `alice`. An accepted response
is not a trust decision. `verify` atomically stores both keys and the signed
transcript. Existing different keys are refused; their explicit replacement
requires a separate, independently checked decision.

Use `verifyAuthor('alice', receivedArchivePath)` to compare an archive's
authenticated signer with Alice's saved signing key. The Vault must contain a
Profile capable of opening that archive. A successful open by itself does not
prove that Alice authored it.

## Status and recovery

Statuses contain the exchange ID, relay origin, invitation URL, signed claimed
emails, expiry, complete fingerprint once available, acceptance status, local
verification status, local contact name, and `relayPending`. They do not contain
management capabilities or private keys. Treat invitation URLs as sensitive.

`relayPending` means local state was durably prepared but delivery or completion
acknowledgement needs retrying. Online `list` retries it; offline listing never
contacts the relay. Local storage and changed-pinned-transcript failures are
reported as errors. Verified contacts remain usable even if acknowledgement
fails. Once both signed bundles are pinned, verification can finish offline,
including after expiry.

Cancel applies only to an unaccepted invitation created by this Vault. Forget
removes local exchange state and its management capability, preserving verified
contacts. Cancel an active invitation before forgetting it if it should become
unavailable. A session close performs neither operation.

## Language entry points

| Binding | Open and cleanup | Verification channel |
| --- | --- | --- |
| Dart | `ContactExchange.open(path, passphrase)`; `close()` | `ComparisonChannel.inPerson` |
| JavaScript / TypeScript / hosted WASM | `api.contactExchange(path, passphrase)`; `close()` | `ComparisonChannel.inPerson` |
| Python | `with ContactExchange.open(path, passphrase) as session` | `ComparisonChannel.IN_PERSON` |
| Java / Kotlin | `api.contactExchange(path, passphrase)`; `close()` / resource scope | `Revault.ComparisonChannel.IN_PERSON` |
| C# | `using var session = api.OpenContactExchange(path, passphrase)` | `ComparisonChannel.InPerson` |
| Swift | `try api.contactExchange(path, passphrase)`; `defer { session.close() }` | `.inPerson` |
| Go | `revault.OpenContactExchange(path, passphrase)`; `defer session.Close()` | `revault.InPerson` |
| C++ | `revault::ContactExchange(path, passphrase)`; RAII | `revault::ComparisonChannel::in_person` |
| PHP | `$api->contactExchange($path, $passphrase)`; `free()` in `finally` | `ComparisonChannel::InPerson` |
| Ruby | `api.contact_exchange(path, passphrase)`; `close` in `ensure` | `Revault::ComparisonChannel::IN_PERSON` |
| Lua | `api:contact_exchange(path, passphrase)`; `close()` after `pcall` | `revault.ComparisonChannel.in_person` |
| Rust | `contact_exchange(path, &passphrase, command)`; credentials owned by caller | `ComparisonChannel::InPerson` |
| C | `contact_exchange_open`; `contact_exchange_execute`; `contact_exchange_free` | Private operation field `channel` |

Rust takes one typed command per call and does not create a session handle:

```rust,no_run
use revault_api::vault::{contact_exchange, ContactExchangeCommand, SecretString};
use std::path::Path;

fn inspect_saved(path: &Path, passphrase: &SecretString) -> Result<(), Box<dyn std::error::Error>> {
    let statuses = contact_exchange(path, passphrase,
        ContactExchangeCommand::List { offline: true })?;
    for status in statuses {
        println!("{} verified={}", status.id, status.verified);
    }
    Ok(())
}
```

For C, requests are UTF-8 JSON up to 16 KiB matching the documented Rust
`ContactExchangeCommand` fields. Replies are the `ContactExchangeStatusList`
table in the supplied FlatBuffers schema. Free every result with `buffer_free`;
a null buffer/handle reports failure through the existing structured error API.
The higher-level bindings expose ordinary domain objects and keep that transport
private. The language conformance runners exercise session open, offline list
decoding and cleanup; native and CLI integration tests exercise reciprocal
exchange and persisted archives in both directions.

See the [protocol and deployment design](../docs/reciprocal_exchange.md) for
signed transcript rules, relay replication, terminal receipts, and fencing.
