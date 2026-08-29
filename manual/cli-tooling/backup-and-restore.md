---
description: "Back up a Vault and Profiles, then restore them on a replacement machine."
---

# Back up and restore

A useful recovery plan contains the encrypted Vault backup, the Vault passphrase, important Profile recovery files, and independent copies of the Lockboxes themselves. Do not keep every part only on the machine being protected.

## Back up the Vault

```bash
lbx vault backup ./vault-backup.lockbox-backup
```

The backup remains encrypted with the Vault passphrase. Store them separately. Test that the backup can be read before depending on it.

Profile recovery files provide an additional route to the keys belonging to one Profile:

```bash
lbx vault profile backup ./default.profile-backup
lbx vault profile backup ./production.profile-backup --name production
```

{% hint style="danger" %}
Profile recovery files contain private key material. Treat them as plaintext master credentials and keep them offline or inside another appropriately protected system.
{% endhint %}

## Restore a replacement machine

1. Install the same or a compatible reVault version.
2. Restore the complete Vault backup:

   ```bash
   lbx vault restore ./vault-backup.lockbox-backup
   ```

3. If a Vault backup is unavailable, initialise a new Vault and restore the required Profiles:

   ```bash
   lbx vault init
   lbx vault profile restore ./default.profile-backup
   ```

4. Copy the Lockbox files to the machine.
5. Open each important Lockbox and extract a sample to a temporary location.
6. Reconfigure Auto Open only after confirming the new machine's security posture.
7. Create a new backup set and retire obsolete copies securely.

Use `--overwrite` only when you have deliberately chosen to replace an existing Vault or Profile. reVault backs up the existing Vault before a Profile overwrite, but you should still retain your independent recovery copy.

## Practise recovery

A backup is an assertion until restored. Periodically test the process on an isolated account or machine, verify the extracted bytes, and record which Profile opens each critical Lockbox.
