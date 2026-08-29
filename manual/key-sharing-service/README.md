---
description: What the optional public-key sharing service does.
---

# Key sharing service

The reVault key-sharing service is a temporary delivery channel for published public Profiles. It makes exchanging a key convenient; it does not decide whether that key belongs to the person you intended.

The normal flow is:

1. the owner associates an email address with a Profile and runs `lbx vault profile publish`;
2. the service verifies access to that email address and returns a publish code;
3. the owner sends the publish code to the recipient;
4. the recipient runs `lbx vault contact receive <publish-code> <contact-name>`; and
5. the recipient independently compares the Profile fingerprint with the owner.

Read [Sharing](../cli-tooling/sharing.md) for the complete user workflow and its trust warnings.

## Running your own service

Organisations may operate an internal key-sharing service. The server is a Rust executable and supports standalone and replicated deployments. Install the current published crate with:

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
