---
description: Manage the public Profiles belonging to people you share with.
---

# Contacts

A Contact is another person's public Profile stored in your Vault. Contacts make it easy to grant that person access without passing keys on every command.

Read [Sharing](sharing.md) before adding a Contact. Saving a public key is easy; establishing that it belongs to the intended person is the important part.

## Manage Contacts

### Exchange both public identities

Each person selects one local Profile. The exchange transfers its encryption
and signing public keys in both directions. Set the Profile email first:

```bash
lbx vault profile email default alice@example.com
lbx vault contacts exchange bob@example.com --profile default \
  --key-server https://keys.example.com
```

Send the displayed invitation URL to Bob. On Bob's device:

```bash
lbx vault profile email default bob@example.com
lbx vault contacts accept '<invitation-url>' --profile default
```

Both people run `lbx vault contacts exchanges`, then independently compare
both identities and the **entire** displayed fingerprint over a known phone
call, in person, or an already authenticated messenger. The relay's email
claims alone do not establish identity. After comparison, Alice saves Bob:

```bash
lbx vault contacts verify <exchange-id> bob \
  --fingerprint '<full-compared-fingerprint>' --channel known-phone-call
```

Bob runs the same command with local contact name `alice`. Neither person's
confirmation verifies the other person's Vault automatically. Existing different
contact keys are refused; remove an old Contact explicitly only after checking
the replacement identity. No Vault lock is retained while waiting for the relay
or the comparison prompt.

Invitations expire after 24 hours by default; `--ttl-hours` accepts 1–168.
Run `exchanges --offline` to view saved state without network access. If delivery
or acknowledgement is pending, run `exchanges` again. To abandon an unaccepted
invitation you created, run `cancel-exchange <exchange-id>`. Use
`forget-exchange <exchange-id>` to remove local state after completion; this
preserves the verified Contact.

After receiving an archive, authenticate its author against that Contact:

```bash
lbx vault contacts verify-author alice ./received.lbox
```

### Import and manage Contacts

```bash
lbx vault contacts list
lbx vault contacts receive <publish-code> alice
lbx vault contacts remove alice
```

For a public Profile exchanged as a file:

```bash
lbx vault contacts import alice ./alice.pub \
  --fingerprint <fingerprint-code> \
  --fingerprint-channel in-person
```

Grant the Contact access to a Lockbox with:

```bash
lbx shared.lbox access grant alice
```

Removing a Contact from your Vault does not alter Lockboxes and cannot revoke copies already held by that person. Revoke the relevant Lockbox access entries separately, then rotate or replace shared material when the threat calls for it.

