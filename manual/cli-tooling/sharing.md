---
description: Exchange trusted public keys and share a Lockbox.
---

# Sharing

A Lockbox may grant access to more than one Profile. To share one, first save the other person's public Profile as a Contact, then grant that Contact access.

Public keys are safe to share. Private keys and Profile backups are not.

## Establish trust

The difficult part of exchanging a public key is proving who owns it. reVault gives the key a fingerprint: a short representation you can compare through a second, independent channel.

Exchange both encryption and signing public keys with one invitation. Alice
selects the Profile she wants to share and the invitation relay:

```bash
lbx vault profile email default alice@example.com
lbx vault contact exchange bob@example.com --profile default \
  --key-server https://keys.example.com
```

Alice sends the generated invitation URL to Bob. Bob selects his Profile and
accepts; this returns his complete public bundle automatically:

```bash
lbx vault profile email default bob@example.com
lbx vault contact accept '<invitation-url>' --profile default
```

Alice retrieves the reply with `lbx vault contact exchanges`. Neither person
needs to stay online while waiting. Invitations expire after 24 hours by default;
`--ttl-hours` accepts 1–168 hours. Running `accept` again with the same invitation
and Profile safely retries a lost response. `exchanges` retries pending delivery.

Both users now see the same full shared fingerprint. It covers both identities,
both encryption keys, both signing keys, Profile generations and this particular
exchange. Compare every group and both identities through an independently
trusted phone call, messenger conversation, or in person.

Each person confirms locally, choosing their own Contact name:

```bash
# Alice, using the shared fingerprint Bob independently supplied:
lbx vault contact verify <exchange-id> bob \
  --fingerprint '<full-shared-fingerprint>' --channel known-phone-call

# Bob, using the shared fingerprint Alice independently supplied:
lbx vault contact verify <exchange-id> alice \
  --fingerprint '<full-shared-fingerprint>' --channel known-phone-call
```

Omit `--fingerprint` and `--channel` for interactive prompts. All 256 bits
(16 groups of four hexadecimal characters) must match; a short prefix or PIN is
rejected. Do not copy your own displayed code into the confirmation without
comparing it with the other person.

The invitation service is not an identity authority. Names and email addresses
in this flow are signed claims, not server-verified identities. Possession of
an invitation URL permits acceptance; an intercepted invitation can be consumed
by someone else, but cannot establish trust without your independent comparison.
If that happens, reject the comparison and start a fresh invitation.

Keys awaiting verification are kept separately from usable Contacts. Confirming
on Alice's device does not mark Bob's Contact verified. Saving a Contact commits
both keys and the verification transcript together. A different existing key
under the same Contact name is refused; inspect and remove the old Contact
explicitly before saving a verified replacement.

{% hint style="warning" %}
Do not compare using the invitation message or a new contact channel supplied
with it. An attacker controlling that channel could replace both the invitation
and comparison. The relay never marks a Contact trusted.
{% endhint %}

Where the consequences are serious, exchange and compare the key in person.

## Grant access

Once the Contact is trusted, grant access to an existing Lockbox:

```bash
lbx shared.lbox access grant alice
lbx shared.lbox access list
```

You can also create a Lockbox for a Contact from the outset:

```bash
lbx shared.lbox create --for alice
```

The Lockbox file itself can then travel through an untrusted channel. Its encrypted contents are accessible only to Profiles named in its access records.

Revoke future access to copies you control with:

```bash
lbx shared.lbox access revoke alice
```

Revocation cannot erase a copy or key the Contact already possesses. If previously shared material must no longer be trusted, create new keys and redistribute a new Lockbox to the remaining recipients.

## Authenticate the sender

After opening a received Lockbox, check its commit signatures against the
Contact's verified signing key:

```bash
lbx vault contact verify-author alice ./shared.lbox
```

This command requires a local Profile that can decrypt the Lockbox. It fails
for an unsigned archive or a different signer, even if decryption succeeds.

## Manage invitations

```bash
lbx vault contact exchanges
lbx vault contact exchanges --offline
lbx vault contact cancel-exchange <exchange-id>
lbx vault contact forget-exchange <exchange-id>
```

Only the inviter can cancel, and only before acceptance. Forgetting local
exchange state does not delete a verified Contact or revoke previously granted
Lockbox access. There is no receive-only invitation mode.
