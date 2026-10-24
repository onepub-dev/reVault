# Reciprocal invitation exchange

Issue: https://github.com/onepub-dev/reVault/issues/226

## Integration checkpoint — 25 October 2026

The user confirmed full reciprocal exchange as the intended behavior,
superseding the issue's proposed receive-only fallback. Current-main reconciliation,
short-lived CLI Vault handles, request limiting, terminal receipts, replica
snapshots and operator-facing fencing guidance are implemented locally.

The CLI and all language facades now delegate to the same Rust exchange
workflow. Native sessions retain a protected Vault passphrase, release every
Vault handle before HTTP, and return before application-driven comparison
prompts. Each facade has exchange session lifecycle conformance coverage.
Cross-language release qualification remains in progress; the
original branch's older tested/review labels do not establish release readiness.

Publication has been explicitly authorized, subject to completing the security
release gates. No release or security code has been pushed. This checkpoint is
not a release-readiness assertion; production deployment remains separate.

The exchange is invitation-scoped and always reciprocal. Each participant
selects one local profile; both its encryption and hybrid signing public keys
are transferred. Public directory discovery and receive-only invitations are
not part of this protocol.

The protocol record is called an invitation. Binding `ContactExchange` sessions
are only process-local credential owners, not live peer connections. Signed
profile labels and generation are retained in the contact transcript; the
complete signing/encryption keys identify that remote profile. Email and labels
are claims, not a verified directory identifier. Rotation to different keys
requires another independently verified exchange and explicit contact
replacement; automatic trust in a new generation is not implied.

## Authentication

The inviter signs an offer containing wire version 1, a random 256-bit exchange
identifier, its complete bundle, the intended recipient email, creation time
and expiry. The recipient signs the exact offer and its own complete bundle.
Signatures require both existing Ed25519 and ML-DSA-65 algorithms.

The shared verification value is the full SHA-256 digest of the versioned,
role-ordered transcript, including both signatures. It is displayed as 16 groups
of four hexadecimal characters. No bits are truncated. Each participant pins
its own contribution locally, retrieves the other contribution, checks all
signatures and then compares the entire shared value over an independently
authenticated channel. Each saves its own trust decision.

This is full-fingerprint authentication, not an RFC 6189 short-authentication-
string protocol. Its security rests on signature verification, collision
resistance of the full digest, local pinning, and correct independent comparison.
It does not claim the usability/security properties of a short phrase or PIN.
Replacing it with a truncated digest would require a separately reviewed
commitment-based protocol and a new protocol version.

Wire records are bounded JSON. Signed messages use the exact tuple ordering,
domain strings and struct field ordering in
`revault_publish_protocol::exchange::model`, serialized by `serde_json::to_vec`.
Unknown fields, noncanonical public keys and malformed identities are rejected.
Changes to this canonical encoding require a protocol version change.

The relay sees public identities and relationships. Profile names and email
addresses are signed claims, not independently verified identity assertions.
The invitation holder can consume its acceptance slot; identity substitution
is detected by the independent comparison, not prevented by an email lookup.
Signatures prove possession of signing keys and bind the advertised encryption
keys; they do not independently prove possession of the encryption private keys.

## Delivery and capabilities

Clients use HTTPS with redirects disabled. HTTP is restricted to loopback for
local operation and testing. The invitation identifier appears in the URL
fragment, which a normal browser does not transmit in its request to the
landing page. A separate owner management capability is never in that URL.
The recipient generates a distinct management capability before accepting.
Management capabilities are persisted in the encrypted local vault, and only
their hashes are persisted by the relay.

- Create is idempotent for the same offer and owner capability.
- Inspect returns only the offer to the invitation holder.
- Accept atomically freezes one signed reply. Identical retries require the
  original recipient capability; different responses are refused.
- Poll requires an owner or recipient management capability.
- Complete records separate acknowledgements, not local trust.
- Cancel requires the owner capability and refuses an accepted invitation.

Local state is committed before network mutations, allowing accept/create
retries after lost responses. Inviter polling also retries an unacknowledged
create. A relay response cannot change a pinned offer or accepted reply.
Once both bundles are locally available, verification can finish without the
relay, including after the invitation expires.

The default lifetime is 24 hours, configurable to seven days. After both
acknowledgements, the relay replaces the public identity payloads with a receipt
containing capability hashes, transcript hashes and expiry. Completed requests
remain retryable until expiry. Cancellation similarly leaves a receipt so retries
cannot resurrect an invitation. Cleanup does not erase backups or logs.

The CLI resolves the Vault passphrase into protected memory once, then opens the
Vault only for individual local reads and commits. It drops the handle before
relay requests and before asking the user to compare identities or fingerprints.
No writable Vault handle spans those waits. Reopening also rechecks pinned state
before writing, rather than overwriting a concurrent change.

## Library workflow

`ContactExchange` is a process-local session for an existing Vault **file**,
such as `~/.local/share/lockbox/vault/local-vault.lbox`. It never creates a Vault
or starts the Session Agent. The native constructor validates the supplied
Vault passphrase, copies it into protected memory and closes the Vault. The
caller owns its input passphrase and must wipe that input when no longer needed.
Closing the session wipes its protected copy; it does not cancel invitations,
forget exchanges, or delete contacts. Do not close a session concurrently with
an operation using it.

All facades provide invite, accept, list, verify, verify-author, cancel, forget,
and close operations (using each language's naming conventions). Rust exposes
`contact_exchange` with `ContactExchangeCommand`; C uses the bounded private
JSON operation transport and a typed FlatBuffer result. Protocol signing,
validation, HTTP, file locks, and atomic persistence belong to Rust.

`invite` takes a relay HTTPS origin, one existing profile name, intended
recipient email, and a lifetime of 1–168 hours (default 24). `accept` takes the
complete invitation URL and one existing local profile. Both profile emails
must be configured first. `list(offline=true)` reads saved public status;
online listing synchronizes replies and retries pending acknowledgements.
`relayPending` means the local preparation was persisted but relay delivery
needs retrying. Local storage errors and changed pinned transcripts are errors,
not pending delivery. No status contains a management token or private key;
the invitation URL is still sensitive because its holder can accept the offer.

Only call `verify` after comparing both identities and all 16 groups of the
fingerprint through `ComparisonChannel` (in person, a known phone call, or an
independently authenticated messenger). A status with `accepted=true` does not
mean verified. Each participant must verify separately. `verify` saves both
public keys and the transcript atomically; existing different contacts are
refused. `verifyAuthor` opens a received archive using the Vault's local keys
and compares its authenticated signer to the saved contact signing key.

Cancel applies only to an unaccepted invitation owned by this Vault; forgetting
removes local exchange state and preserves any verified contact. Forgetting an
active exchange also discards its management capability, so cancel it first if
it should no longer be available. Expired exchanges with both signed bundles
can still be verified offline.

## Persistence and capacity

The relay is single-active-instance. It writes each record through a temporary
file, syncs it, atomically replaces the committed record, and syncs the directory
on Unix. Startup validates records and rebuilds the expiry and identity indexes.
Exchange snapshots use the authenticated replication transport. Their merge is
monotonic: an accepted response cannot change, completion flags cannot regress,
and a terminal receipt cannot be replaced by an earlier payload. Failed disk
writes do not consume a replication sequence and suppress subsequent retries.

`exchange_owner_id` selects one active exchange owner (default server 0). Other
servers refuse public exchange operations until explicitly promoted using the
existing `promoted_owner` configuration. All servers in the exchange deployment
must agree on this owner. Fence the old active server before promoting a standby;
this is controlled failover, not automatic leader election or active-active service.
Keep the pinned relay HTTPS origin stable when moving it to the promoted server.

When replicas are configured, a public exchange operation succeeds only after
all configured peers have durably accepted its current snapshot. A failed peer
causes a retryable error even if the local mutation committed. Client retries
resend that state. Explicit peer resynchronization also includes exchanges and
receipts. Deploy compatible servers together; older peers reject the new event.
This trades availability during a peer outage for durability of acknowledged
exchange state. A standalone relay retains only its local durability guarantee.

Count, reserved byte capacity and per-signing-identity admission limits are
configurable. Reserve a full maximum-sized record before admitting an exchange
so later acceptance cannot be displaced by new requests. Active records are
never LRU-evicted. Expiry uses an ordered index rather than scanning all
invitations on each request.
The HTTP endpoint also applies the existing per-peer request token bucket before
JSON decoding and signature verification. Receipt slots remain reserved until
expiry, bounding retry-state storage and preventing completion from bypassing
admission limits.

The default count ceiling is one million; the default 1 GiB byte budget admits
8192 records at the 128 KiB reservation. Raising capacity needs operational
measurement, filesystem planning and reverse-proxy abuse limits. Identity
quotas cannot prevent an attacker generating many identities.

Local invitation records are ordinary encrypted files inside the existing
vault container. The archive/vault wire format is unchanged. Contact
verification writes both existing public-key record types, an exchange
transcript record and local verification state in one vault commit. Existing
different keys are never silently replaced. Removing a contact also removes
its attached exchange transcript; forgetting an invitation preserves its
verified contact.

## Validation

Protocol tests cover full-fingerprint comparison, signature/context mutation,
key substitution, reflection, expiry, malformed wire records and unsafe URLs.
Relay tests cover durability, idempotent acceptance, management capability
separation, cancellation, capacity refusal and expiry, payload removal on
completion, receipt retries after restart, monotonic replica application and
acknowledged delivery through the HTTP replication endpoint before promotion.

Vault tests reopen persisted state and compare both public keys. CLI tests
create independent users, exchange/verify through the HTTP service, exercise
retries, wrong recipient and fingerprint refusal, explicit contact replacement
and removal, and read exact archive bytes in both directions. A separate CLI
invocation checks each archive's signer against the exchanged contact key.

These automated checks are implementation evidence, not an independent
cryptographic review or a production capacity certification.
