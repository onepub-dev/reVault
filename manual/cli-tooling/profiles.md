---
description: Use Profiles to separate Lockbox ownership and access.
---

# Profiles

A Profile is a named public/private key pair used to open Lockboxes. Initialising a Vault creates the `default` Profile, which is all many people need.

Additional Profiles are useful when you want distinct keys for work, personal material, devices, clients or key-rotation policies. Choose the Profile when creating a Lockbox:

```bash
lbx personal.lbox create --for personal
```

## Create and inspect Profiles

```bash
lbx vault profile create personal
lbx vault profile list
lbx vault profile history personal
lbx vault profile fingerprint personal
```

Associate an email address before publishing a Profile through the key-sharing service:

```bash
lbx vault profile email personal alice@example.com
```

## Back up a Profile

```bash
lbx vault profile backup ./personal.profile-backup --name personal
```

{% hint style="danger" %}
A Profile backup contains private key material. Anyone who obtains it may be able to open Lockboxes accessible to that Profile. Store it as carefully as the data itself.
{% endhint %}

Restore a backup with:

```bash
lbx vault profile restore ./personal.profile-backup
```

Use `--name` to restore it under a different name. If that name already exists, `--overwrite` replaces the Profile after reVault backs up the current Vault.

## Rotate or remove a Profile

`lbx vault profile rotate personal` creates a new key generation while retaining the history needed to work with earlier Lockboxes. Use `lbx access refresh` on Lockboxes whose access entry needs the newer generation.

Removing a Profile can remove your ability to open its Lockboxes. Check its use and make a secure backup first:

```bash
lbx vault profile remove personal
```

Publishing a Profile shares only its public key. Read [Sharing](sharing.md) before exchanging keys or granting access.
