---
description: Use Profiles to separate Lockbox ownership and access.
---

# Profiles

A Profile is a named credential stored encrypted in your Vault. It can contain a key pair or a generated password. Initialising a Vault creates the `default` key-pair Profile, which is all many people need.

Key-pair Profiles let you share public keys while keeping private keys in your Vault. Password Profiles let the Vault manage a password that you can provision to a server without giving the server a Vault or access to platform secure storage.

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

Both Profile types support backup and restore.

```bash
lbx vault profile backup ./personal.profile-backup --name personal
```

{% hint style="danger" %}
A Profile backup contains private keys or the Profile's password in plaintext. Anyone who obtains it may be able to open Lockboxes accessible to that Profile. Store it as carefully as the data itself. Whole-Vault backups also preserve password Profiles and remain encrypted.
{% endhint %}

Restore a backup with:

```bash
lbx vault profile restore ./personal.profile-backup
```

Use `--name` to restore it under a different name. If that name already exists, `--overwrite` replaces the Profile after reVault backs up the current Vault.

Replacement must use the same Profile type. Remove a Profile explicitly before reusing its name for a different type.

## Give a server password access

Create a password Profile on your own machine, then grant it access alongside your existing identity:

```bash
lbx vault profile create production-server --password
lbx shared.lbox access grant production-server
lbx shared.lbox access list
```

The Vault generates and stores a random 256-bit password, represented as 64 hexadecimal characters. Creating the Profile does not display the password or grant access. The grant command adds a password slot that protects the existing Lockbox content key; your existing identity retains access. Repeating the same grant leaves the existing entry unchanged.

Use `profile:production-server` to disambiguate a Profile from a Contact with the same name. Profile names are kept locally in the Vault; they are not written into the Lockbox's key directory.

To create a new Lockbox for a password Profile:

```bash
lbx server.lbox create --for production-server
```

### Retrieve the password

Retrieve the same password whenever you need it:

```bash
lbx vault profile password production-server
```

The command unlocks your Vault normally and prints the password. Profile listings show its type and keep the secret hidden. To write the exact password bytes to a file:

```bash
lbx vault profile password production-server --output ./server-credential
```

On Unix, the file has owner-only permissions. Existing files are refused unless you pass `--overwrite`.

Provision the password through your server's secret-delivery mechanism. The Vault remains your authoritative copy; the server needs the password at runtime.

### Open on the server

With the updated Lockbox and provisioned credential file on the server:

```bash
lbx shared.lbox open --password-file /run/secrets/lockbox-password
lbx shared.lbox list
lbx shared.lbox extract --to ./restored
lbx shared.lbox close
```

You can also use `open --password-stdin` or `open --password-env VARIABLE_NAME`. Explicit password opening caches access in the Session Agent without creating or unlocking a local Vault. It supports reading and extraction without platform secure storage. A password Profile does not supply an owner signing key; writing still requires the appropriate signing material.

If the server loses its password, retrieve it again from your Vault. If the Profile is lost, restore its backup. If both are lost, another identity that can manage the Lockbox can grant a replacement password; the original password cannot be recovered from the Lockbox itself.

### Replace or revoke the credential

Create a replacement Profile, grant it access and verify the server can read with its new credential before revoking the old entry:

```bash
lbx vault profile create production-server-next --password
lbx shared.lbox access grant production-server-next
# Provision and verify the replacement password on the server.
lbx shared.lbox access revoke production-server
```

Revocation rewrites the Lockbox with a fresh content key. Retained password entries must have their original passwords available through locally labelled Profiles; otherwise the operation refuses to proceed. Previously retained copies remain readable with their old credentials.

Removing a Profile from your Vault does not revoke the corresponding Lockbox access. Revoke access first, then remove the Profile when it is no longer needed. Replacing a stored password through Profile restore also leaves existing Lockbox slots unchanged.

## Rotate or remove a Profile

For key-pair Profiles, `lbx vault profile rotate personal` creates a new key generation while retaining the history needed to work with earlier Lockboxes. Use `lbx access refresh` on Lockboxes whose access entry needs the newer generation. Password Profiles use the replacement workflow above; key history, fingerprints, public export, publishing and key rotation apply to key-pair Profiles.

Removing a Profile can remove your ability to open its Lockboxes. Check its use and make a secure backup first:

```bash
lbx vault profile remove personal
```

Publishing a Profile shares only its public key. Read [Sharing](sharing.md) before exchanging keys or granting access.
