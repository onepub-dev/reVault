---
description: "Resolve common Vault, Lockbox, Session Agent, sharing and migration failures."
---

# Troubleshooting

Start with `lbx doctor`, then use the symptom below. The [exit-code reference](cli-tooling/exit-codes.md) is more stable for scripts than error text.

## The Vault is unavailable

Run `lbx vault init --verify`. Confirm that you are operating as the expected desktop user and that `LOCKBOX_CONFIG` or platform paths do not select another environment. When using `sudo`, follow the [sudo guide](cli-tooling/sudo.md) rather than copying a passphrase into root's credential store.

## The Lockbox is not open

```bash
lbx secrets.lbox open
lbx session
```

If Auto Open is intentionally disabled, a fresh process may need the Vault passphrase. Exit code `10` means the caller can ask the user to open the Lockbox and retry.

## Auto Open is unavailable

Run `lbx doctor` and `lbx session auto-open status`. Linux requires an accessible Secret Service provider in the same desktop/D-Bus session. Headless and container environments often do not provide one; use an explicit protected credential rather than weakening permissions around another user's keyring.

## The Session Agent behaves unexpectedly

```bash
lbx session stop
lbx session
```

A later operation starts a fresh agent. Set `LOCKBOX_SESSION_AGENT_LOG` to a private file path for targeted logging. Remember that Auto Open can reopen a Lockbox after its cached content key is cleared.

## A Contact fingerprint does not match

Stop. Remove the unverified Contact and restart the exchange using contact details you already trust. Do not ask the sender to resend both key and fingerprint through the same channel.

## A Lockbox is corrupt or an operation was interrupted

Keep the original and run:

```bash
lbx damaged.lbox recover --dry-run
```

Follow [Recover a damaged Lockbox](cli-tooling/recovery.md). Use `recover --transaction` only when the error specifically requests transaction redaction cleanup.

## Migration fails

Confirm the Vault is migrated first, the source is unchanged, enough free space exists, and the historical exporter can be installed. Repeat the same command to resume its journal. See [Migrating between versions](cli-tooling/migrating-between-versions.md).

## An output path already exists

reVault avoids replacing files by default. Inspect the existing output and choose a new path, or use the operation's explicit `--overwrite` option when replacement is intentional.

## Ask for help safely

Include the reVault version, operating system, redacted command, exit code and `lbx doctor` output. Never publish passwords, Vault/Profile backups, secret values, private keys or a sensitive Lockbox.
