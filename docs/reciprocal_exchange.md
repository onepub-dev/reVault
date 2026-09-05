# Reciprocal invitation exchange

Issue: https://github.com/onepub-dev/reVault/issues/226

The exchange is invitation-scoped and always reciprocal. Each participant
selects one local profile; both its encryption and hybrid signing public keys
are transferred. Public directory discovery and receive-only invitations are
not part of this protocol.

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

The default lifetime is 24 hours, configurable to seven days. Retained payloads
and acknowledgements expire together; completed acknowledgements remain
retryable until expiry. Cleanup does not erase backups or logs.

## Persistence and capacity

The relay is single-active-instance. It writes each record through a temporary
file, syncs it, atomically replaces the committed record, and syncs the directory
on Unix. Startup validates records and rebuilds the expiry and identity indexes.
Existing publication replication does not replicate invitation state.

Count, reserved byte capacity and per-signing-identity admission limits are
configurable. Reserve a full maximum-sized record before admitting an exchange
so later acceptance cannot be displaced by new requests. Active records are
never LRU-evicted. Expiry uses an ordered index rather than scanning all
invitations on each request.

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
separation, cancellation, capacity refusal and expiry.

Vault tests reopen persisted state and compare both public keys. CLI tests
create independent users, exchange/verify through the HTTP service, exercise
retries, wrong recipient and fingerprint refusal, explicit contact replacement
and removal, and read exact archive bytes in both directions. A separate CLI
invocation checks each archive's signer against the exchanged contact key.

These automated checks are implementation evidence, not an independent
cryptographic review or a production capacity certification.
