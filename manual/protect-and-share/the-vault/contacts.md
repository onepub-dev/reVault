---
description: Manage the public Profiles belonging to people you share with.
---

# Contacts

A Contact is another person's public Profile stored in your Vault. Contacts make it easy to grant that person access without passing keys on every command.

Read [Sharing](sharing.md) before adding a Contact. Saving a public key is easy; establishing that it belongs to the intended person is the important part.

## Manage Contacts

```bash
lbx vault contact list
lbx vault contact receive <publish-code> alice
lbx vault contact remove alice
```

For a public Profile exchanged as a file:

```bash
lbx vault contact import alice ./alice.pub \
  --fingerprint <fingerprint-code> \
  --fingerprint-channel in-person
```

Grant the Contact access to a Lockbox with:

```bash
lbx shared.lbox access grant alice
```

Removing a Contact from your Vault does not alter Lockboxes and cannot revoke copies already held by that person. Revoke the relevant Lockbox access entries separately, then rotate or replace shared material when the threat calls for it.

