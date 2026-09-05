---
description: Exchange trusted public keys and share a Lockbox.
---

# Sharing

A Lockbox may grant access to more than one Profile. To share with another person's key-pair Profile, first save its public key as a Contact, then grant that Contact access. For a server that needs a password managed by your Vault, use a [password Profile](profiles.md#give-a-server-password-access).

Public keys are safe to share. Private keys and Profile backups are not.

## Establish trust

The difficult part of exchanging a public key is proving who owns it. reVault gives the key a fingerprint: a short representation you can compare through a second, independent channel.

For example, Alice can publish her `default` Profile:

```bash
lbx vault profile email default alice@example.com
lbx vault profile publish default
```

The service verifies control of the email address and returns a publish code. Alice sends that code to Bob. Bob receives the Profile and chooses a local Contact name:

```bash
lbx vault contact receive <publish-code> alice
```

Bob must compare the fingerprint with Alice through a channel he already trusts. He should initiate that contact himself. An email address proves access to an inbox at one point in time; it does not prove a legal identity, employment, authority or continuing ownership.

{% hint style="warning" %}
Do not accept a fingerprint that arrives unsolicited alongside the public key. An attacker who replaced one may be able to replace both.
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

Revocation replaces the content key in the Lockbox you control and reconstructs retained access. It cannot erase a copy or key the Contact already possesses. Distribute the updated Lockbox to the remaining recipients.

## Exchange without the service

Export your public Profile:

```bash
lbx vault profile export ./default.pub
lbx vault profile fingerprint default
```

The recipient imports it after independently obtaining and checking the fingerprint:

```bash
lbx vault contact import alice ./default.pub \
  --fingerprint <fingerprint-code> \
  --fingerprint-channel phone-call-to-owner
```

The channel description records how the verification was performed.
