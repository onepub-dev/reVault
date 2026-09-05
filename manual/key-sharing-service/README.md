---
description: What the optional public-key sharing service does.
---

# Key sharing service

The reVault key-sharing service delivers invitation-scoped, reciprocal public
Profile bundles. It does not provide a searchable public directory or decide
whether a key belongs to the person you intended.

The normal flow is:

1. Alice runs `lbx vault contact exchange` with Bob's email and her selected Profile;
2. Alice sends the generated invitation URL to Bob;
3. Bob runs `lbx vault contact accept`, explicitly selecting his reply Profile;
4. the server retains both signed bundles while Alice retrieves Bob's reply;
5. both users independently compare one shared fingerprint and verify locally.

Both encryption and signing public keys travel in each direction. This flow
does not email-verify identity; the signed email claims are checked during the
independent comparison. Public invitation capabilities cannot retrieve the
recipient's reply or manage the exchange.

Read [Sharing](../cli-tooling/sharing.md) for the complete user workflow and its trust warnings.

## Running your own service

Organisations may operate an internal key-sharing service. Reciprocal
invitations use `POST /v2/exchange` on one explicitly selected server with
persistent storage. Existing publication topology/replication does not replicate
these invitations: route this endpoint to one active instance.

Build this branch to use the new invitation endpoint; an older published
package may not provide it. The server's installation command is:

```bash
cargo install revault_key_server
```

Inspect the exact options belonging to the installed version before building an operator configuration:

```bash
revault_key_server --help
revault_key_server run --help
```

The [Configuration](configuration.md) and [Topology server](../topology-server.md) pages describe clustered deployments. Keep replication tokens out of source control and use HTTPS between every public or replication endpoint.

The service holds public keys and short-lived exchange state, not private Profile keys or Lockbox contents. Even so, monitor it as an internet-facing security service and keep it patched.
