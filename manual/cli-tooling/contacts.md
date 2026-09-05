---
description: Manage the public Profiles belonging to people you share with.
---

# Contacts

A Contact is another person's public Profile stored in your Vault. Contacts make it easy to grant that person access without passing keys on every command.

Read [Sharing](sharing.md) before adding a Contact. Saving a public key is easy; establishing that it belongs to the intended person is the important part.

## Manage Contacts

```bash
lbx vault contact list
lbx vault contact exchanges
lbx vault contact verify <exchange-id> alice
lbx vault contact remove alice
```

Start a reciprocal invitation using one explicitly selected Profile:

```bash
lbx vault contact exchange alice@example.com --profile default \
  --key-server https://keys.example.com
```

Follow [Sharing](sharing.md) to accept and compare the shared fingerprint.
Both public keys are saved together only after local verification. Pending
invitations are listed by `exchanges`, not by `contact list`.

Grant the verified Contact access to a Lockbox with:

```bash
lbx shared.lbox access grant alice
```

Removing a Contact from your Vault does not alter Lockboxes and cannot revoke copies already held by that person. Revoke the relevant Lockbox access entries separately, then rotate or replace shared material when the threat calls for it.
