# The Vault

When you install reVault the first action you need to under take is to create a Vault.

Note: you can create lockboxes without a vault, but a Vault (along with the sesion agent) eliminates having to pass keys to every action on a lockbox.

{% hint style="danger" %}
**A vault is used to store all the keys to your lockboxes, so keeping a vault secure is CRITICAL.**&#x20;
{% endhint %}

You will (normally) only have a single vault per device; you can share a vault between devices but we don't recommend this as it can be hard to sync the vaults and you risk overwriting lockbox keys ( and losing access to those lockboxes) if you screw up the sync process.&#x20;

reVault does allow you to use external passwords and external keys to access lockboxes but the Vault is the default way to store your passwords and keys and provides significant ergonomics over managing a gaggle of passwords and keys.

The Vault stores its own contents in a standard  lockbox to ensure that its contents are secured.

To access the Vault you must have the pass phrase originally entered or generated when creating the Vault. It is CRITICAL that you backup the vault pass phrase. Losing the vault pass phrase means that you lose access to the vault and ALL of your lockboxes.

### Create the Vault

To create the vault run:

```
lbx vault init
```

You will be prompted to enter or generate a passphrase for the vault. &#x20;

**DANGER**:&#x20;

{% hint style="danger" %}
It is CRITICAL that you backup the pass phrase. If you lose the passphrase you will lose access to all of you lockboxes.
{% endhint %}

### Verify the Vault

You can verify the vault is intact by running:

```
lbx vault verify
```

### Overwrite the vault

You can create a new vault by overwriting the existing vault. This EXTREMELY DANGEROUS.&#x20;

{% hint style="danger" %}
Overwriting the existing vault will cause you to lose access to all existing lockboxes.
{% endhint %}

To overwrite the existing vault run:

```
lbx vault init --overwrite
```

The overwrite will backup the existing vault before creating the new vault.&#x20;

You will be requested to enter a new password for the new vault.  It is CRITICAL that you backup the password. If you lose the password you will lose access to all of you lockboxes.

## Backup

When you created you vault via 'init' you will have also backed up your vault passphrase, however you should also backup your vault from time to time is it contains important information such as your list of contacts and the public/private keys for each 'profile' used to open/close your Lockboxes.

A vault is simply a Lockbox file, so whilst you can backup the lockbox containing your vault by simply copying the lockbox, we recommend using the backup command as:

Compared with manually copying it, the command:

* Locks the vault while reading, ensuring a consistent snapshot.
* Stores the already-encrypted vault bytes in a recognised backup format.
* Adds a manifest containing creation time, size, format version, and SHA-256 checksum.
* Verifies size and checksum during lbx vault restore.
* Prevents accidental replacement unless --overwrite is specified.
* Flushes the completed backup to storage before reporting success.

To backup your vault run:

\`lbx vault backup \<backup path>

NOTE:

The backup file is encrypted using the original pass phrase you used to create the vault.

As such if you loose your pass phrase you can not restore you vault.

### Restore

To restore a vault from backup run:

`lbx vault restore <backup path>`

## Change the passphrase

You can change the pass phrase protecting your vault (providing you know the existing passphrase).



## Technical Details

You don't need to read the following to understand how to use your vault.

TODO: describe the vault data including the contents of its lockbox.
