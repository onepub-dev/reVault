# The Vault

Your Vault stores the Profiles, Contacts and Lockbox records that make reVault practical to use. Its contents are encrypted, and its passphrase protects the keys that open your Lockboxes.

You will normally have one Vault on each device. Avoid synchronising a live Vault file between devices: two copies can diverge and overwrite key information. Use Vault and Profile backup commands instead.

{% hint style="danger" %}
Keep a recoverable copy of the Vault passphrase. If you lose both the passphrase and every usable Profile backup, reVault cannot recover your Lockboxes.
{% endhint %}

## Create or verify the Vault

```bash
lbx vault init
```

If the Vault already exists, this reports its path and changes nothing. Ask reVault to verify that the existing Vault opens with:

```bash
lbx vault init --verify
```

`lbx vault init --overwrite` replaces the existing Vault. Use it only when you intend to discard records stored solely in that Vault; reVault makes a backup before replacing it.

## Back up and restore

Create a checked, consistent backup with:

```bash
lbx vault backup ./vault-backup.rvlt
```

The backup remains encrypted with the Vault passphrase. The command adds integrity metadata and flushes the completed backup before reporting success.

Restore it with:

```bash
lbx vault restore ./vault-backup.rvlt
```

Do not rely on a backup you have never tested. Keep at least one copy away from the device that holds the working Vault.

## Change the passphrase

```bash
lbx Vault passphrase
```

Changing the passphrase does not change the keys belonging to your Profiles or Lockboxes. Update any separately recorded recovery information and review your Auto Open setting afterwards.

## Remembered Lockboxes

The Vault remembers Lockbox paths. If you move a Lockbox through the shell or a file manager, tell reVault where it went:

```bash
lbx vault lockbox move ./old.lbox ./archive/new.lbox
```

You can inspect or remove remembered paths with:

```bash
lbx vault lockbox list
lbx vault lockbox forget ./old-project.lbox
```

Forgetting a path does not delete the Lockbox.
