# Quick start guide

No cruft, just the juicy bits.

This guide focuses on the CLI. You can perform the same work through the
[language bindings](../apis/revault-api.md), but the CLI is a good place to
learn the core reVault concepts.

## Install reVault

If you don't have Cargo, follow the
[Rust installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Install reVault:

```bash
cargo install revault_cli
```

This installs `lockbox` and its short alias, `lbx`. The examples below use
`lbx`; you can substitute `lockbox` anywhere.

## Initialise and back up your Vault

Initialise your Vault once:

```bash
lbx vault init
```

Choose a strong Vault passphrase and store it somewhere safe. Then create an
encrypted backup:

```bash
lbx vault backup ./vault-backup.lockbox-backup
```

{% hint style="danger" %}
Keep the backup and Vault passphrase on separate, secure storage. If you lose
the Vault, its backup and the credentials needed by a Lockbox, nobody can
recover that Lockbox for you.
{% endhint %}

Run the doctor whenever you want to check the installation, Vault, Auto Open
and Session Agent:

```bash
lbx doctor
```

## Create a Lockbox

The Lockbox path comes before the command:

```bash
lbx mystuff.lbox create
```

You can keep writing the path, or make this your default Lockbox:

```bash
lbx session default mystuff.lbox
```

Open it for an hour:

```bash
lbx open --duration 1h
```

Without `--duration`, the default session duration is 15 minutes. Using a
Lockbox extends its sliding expiry.

## Add, list and extract files

Add a file:

```bash
lbx add ./readme.md
```

Store it at a different path inside the Lockbox. Missing parent directories are
created automatically:

```bash
lbx add ./readme.md --to docs/readme.md
```

Add a complete directory tree:

```bash
lbx add --recursive ./project --to archive/project/
```

List the contents:

```bash
lbx list
lbx list /archive --recursive
lbx list '/archive/**/*.md'
```

Extract one file or the complete Lockbox:

```bash
lbx extract /docs/readme.md ./restored-readme.md
lbx extract --to ./restored
```

Remove an entry:

```bash
lbx remove /docs/readme.md
```

## Store variables

Store and retrieve a normal variable:

```bash
lbx variable set DB_PORT 5432
lbx variable get DB_PORT
```

Store a secret without putting its value in shell history or the process list:

```bash
lbx variable set --secret API_TOKEN --interactive
lbx variable get --secret API_TOKEN
```

You can also supply a secret through `--stdin`, `--file` or `--from-env`.

## Store a form record

Create a reusable form definition in your Vault:

```bash
lbx vault form define login \
  --field username:text:required:Username \
  --field password:secret:required:Password
```

Copy the definition into the default Lockbox and add a record:

```bash
lbx form use login
lbx form add /work/github --type login --name GitHub --interactive
lbx form show /work/github
```

See [Forms](forms.md) for field types and non-interactive examples.

## Share a Lockbox

Ask your associate to publish their default Profile:

```bash
lbx vault profile publish
```

They give you the resulting publish code. Receive it under a local Contact
name:

```bash
lbx vault contact receive <publish-code> alice
```

The command asks you to verify the fingerprint through a second, trusted
channel. You should initiate that second contact using details you already
trust.

Grant the Contact access:

```bash
lbx mystuff.lbox access grant alice
```

You can now send `mystuff.lbox` to Alice. Read [Sharing](sharing.md) before
using this workflow for sensitive information.

## Finish the session

Close the default Lockbox when you are done:

```bash
lbx close
```

This clears its temporary content key from the Session Agent. If Auto Open is
enabled, the Vault may still hold a persistent credential that can open the
Lockbox again. Read [Session Management](session-management.md) to choose the
right setting for your environment.
