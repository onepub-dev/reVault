# Lockbox management

A Lockbox is a single encrypted archive file. You can copy, rename or move it like any other file, but the Vault and Session Agent may remember its path.

The safest way to move a remembered Lockbox is:

```bash
lbx vault lockbox move ./system_api_keys.lbox ./archive/system_api_keys.lbox
```

This moves the file and updates the paths known to the Vault and Session Agent. The destination's parent directories are created when required.

Inspect remembered paths with:

```bash
lbx vault lockbox list
```

If you moved or deleted a Lockbox outside reVault, remove its stale record:

```bash
lbx vault lockbox forget ./old-project.lbox
```

Forgetting a record does not delete a Lockbox. Likewise, copying a Lockbox does not automatically create a new key or identity: both copies contain the same encrypted material at the moment of copying.

Commands that extract or move data create missing destination parent directories. An existing destination is not silently replaced unless the command explicitly offers and receives an overwrite option.

