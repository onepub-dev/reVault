# CI/CD and phone approval

CI must be treated as an untrusted requester, not as a child vault stored on a
developer workstation. There are no deployed `beget` vaults to preserve and the
old workflow has been removed.

## Threat model

Assume an AI tool or compromised process can read every developer file and can
invoke the local credential store. It must not be able to approve itself. Phone
recipient, transport, and response-signing private keys therefore live only on
the phone, encrypted by the platform Keychain/Keystore and released after local
user authentication. Secure Element backing is used where an algorithm is
supported, but is not a CI requirement and cannot currently hold the hybrid
post-quantum recipient key on typical phones.

The relay receives opaque, end-to-end encrypted envelopes. It never receives a
vault password, content key, provider token, or phone private key. STUN/ICE is
not required: push notification plus an HTTPS mailbox works across carrier NAT,
corporate firewalls, sleeping phones, and CI runners more reliably than a
peer-to-peer channel.

## Enrollment

1. The phone creates independent hybrid recipient and transport identities, a
   response signing identity, a random device id, and a random mailbox id.
2. An authenticated QR/pairing transcript transfers only the public enrollment
   record to the desktop vault.
3. The owner gives the device a recognisable name. Multiple phones can be
   enrolled for one lockbox; devices are not contacts because they also have a
   mailbox, signing identity, capabilities, platform, and revocation state.
4. The owner adds a separately named source policy and explicitly selects the
   lockbox ids and operations it may request.

```console
lockbox vault device enroll alice-phone.device.json
lockbox vault source add production-deploy.source.json
lockbox vault device list
lockbox vault source list
```

The JSON files are versioned pairing/policy records. They contain public data,
but the pairing channel still must be authenticated to prevent key substitution.

## Interactive request

The requester creates a random request id, independent challenge, one-time reply
key, two-minute expiry, and an operation digest. It includes the lockbox's
recipient-wrapped content-key slot and encrypts the whole request to the phone's
transport key. The relay indexes the ciphertext only by opaque capability
hashes.

The phone verifies all of the following before showing an Approve button:

- device id, source id, lockbox id, action, expiry, and operation digest;
- the enrolled source policy;
- a local desktop signature or a provider-signed OIDC workload token;
- the provider-specific repository/project and workflow/ref/environment claims;
- that the request id has not already been consumed.

The prompt shows the owner-assigned source name and verified repository,
workflow, ref, environment, and commit. Requester-supplied text is visibly
labelled unverified and is never used for policy decisions.

On approval, the phone unwraps a candidate device slot locally and returns only
the content key in a response encrypted to the one-time reply key. The response
is signed by the enrolled phone and binds request id, challenge, source,
lockbox, operation digest, issue time, and expiry. Relay response fetch is
atomic, and both clients retain replay caches. A captured response therefore
cannot authorize another request or be consumed twice.

## Provider OIDC policies

OIDC trust is configured per provider and per named source. Signing-key
validation uses the issuer discovery document and JWKS; policy evaluation then
requires the exact issuer and an allowed audience.

GitHub Actions policies use stable `repository_id` values and constrain
`job_workflow_ref`, `ref`, and `environment`. Display-only context can include
`repository`, `actor`, and `sha`, but mutable names do not replace stable ids.

GitLab policies use stable `project_id` values and constrain `ref`,
`environment`, and the provider's job/pipeline context. Generic providers use
an issuer, audiences, and exact allowed values for named claims. This adapter
model is deliberate: claim names and security semantics are provider-specific.

Signing into a provider on the phone is useful for selecting repositories and
building the policy, but it does not authenticate a later CI job. Every job must
present its own short-lived, provider-signed workload token. No authentication
key is stored on the developer desktop.

## Unattended CI

Some deployments cannot wait for a phone. They use a separately declared
`unattended` source and a hybrid recipient private key generated inside the CI
provider's protected secret store. Hardware backing is optional because typical
hosted runners do not provide a portable hardware keystore. This is a different,
weaker policy: compromise of that provider secret can open the explicitly
allowed lockboxes until access is rotated.

Interactive OIDC mode is preferred when a person must authorize each run. The
OIDC token authenticates the job; it is not itself an unlock key.

## Relay limits and operations

The key server enforces envelope size and lifetime, per-source minute/hour
request quotas, per-source and per-device pending limits, per-device push
quotas, and `429 Retry-After` responses. Production deployment must share this
state across replicas and persist pending ciphertext until expiry; the in-memory
store is suitable only for a single process and tests. Push payloads contain
only a mailbox wake-up hint, never key material.

Revocation has two layers. Revoking a device/source prevents future approvals
and retains its audit record. Cryptographic revocation also rotates the affected
lockbox content key and removes that recipient slot.
