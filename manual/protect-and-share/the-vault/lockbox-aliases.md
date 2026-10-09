---
description: "Name a Lockbox once and use its variables and form fields in scripts."
---

# Lockbox aliases and script helpers

Give a Lockbox a short name in your Vault, then use `a@name` wherever a command accepts a Lockbox selector. The `lbxv` helper reads one value; `lbxx` runs a command with selected values in its environment.

These commands are available in the current development branch. The CLI package installs `lbxv` and `lbxx` alongside `lockbox` and `lbx`; older releases may not include them.

## Create and use an alias

For an existing Lockbox:

```bash
lbx vault lockbox alias set dev ./developer-secrets.lbox
lbx vault lockbox alias list
lbx a@dev open
lbx a@dev variable list
```

Use the bare name `dev` when managing the alias and `a@dev` when selecting its Lockbox. Names are case-sensitive and contain up to 128 ASCII letters, digits, underscores or hyphens. Setting an existing alias replaces its mapping.

Aliases are encrypted Vault records and are included in [Vault backups](backup-and-restore.md). They identify a Lockbox by its stable identity. Move the file through reVault to keep its remembered location current. The destination directory must already exist:

```bash
lbx vault lockbox move ./developer-secrets.lbox ./secrets/developer.lbox
```

If you already moved the file through a shell or file manager, record its new location instead:

```bash
lbx vault lockbox remember ./secrets/developer.lbox
```

A missing target or a different Lockbox at the remembered path causes an error. Restoring a Vault onto another machine may require remembering the target's new path.

`a@dev` always selects an alias. To select an actual host file named `a@dev`, use `./a@dev` (or `.\a@dev` on Windows).

## Read one value with lbxv

Open the Vault and Lockbox with `lbx` first. The helpers never prompt for credentials.

In Bash, capture a variable or form field with command substitution:

```bash
TOKEN=$(lbxv a@dev ONEPUB_TOKEN)
PASSWORD=$(lbxv a@dev /database@password)
```

`lbxv` accepts exactly one selector and writes its value without an added newline. It reads both normal and secret values, without a `--secret` flag. It does not accept `NAME=selector` or emit shell assignments.

Bash removes trailing newlines in command substitutions. Use `lbxx` when those bytes must be preserved in an environment value. Avoid shell tracing around secret substitutions, and never pass the output to `eval`.

## Run a command with lbxx

Pass one or more selections before `--`, followed by the command and its arguments:

```bash
lbxx a@dev ONEPUB_TOKEN -- dart pub get
lbxx a@dev ONEPUB_AUTH_TOKEN=ONEPUB_TOKEN -- dart pub get
lbxx a@dev ONEPUB_TOKEN /database@username DB_PASSWORD=/database@password -- dart run
```

All selections come from the same Lockbox. Assignments are optional: the default environment name is the variable's final path component or the form field name, preserving case.

| Selection | Child environment name |
| --- | --- |
| `ONEPUB_TOKEN` | `ONEPUB_TOKEN` |
| `/services/API_TOKEN` | `API_TOKEN` |
| `/database@username` | `username` |
| `DB_PASSWORD=/database@password` | `DB_PASSWORD` |

Environment names must contain ASCII letters, digits or underscores and cannot start with a digit. Use an explicit assignment when a stored name does not meet those rules. Duplicate destinations are rejected, case-insensitively on Windows. Every selection must resolve before the command starts.

Values are supplied only to the child environment; your parent shell and stored values are unchanged. `lbxx` starts the command directly, without evaluating a shell expression, and returns its exit status. Values may contain newlines but cannot contain NUL bytes.

Both helpers deliberately disclose secret values to their consumer. `lbxx` avoids putting those values in command arguments or temporary credential files.

## Complete aliases and selectors

Register completion for each command you use. For Bash:

```bash
lbx completion install --shell bash
lbxv completion install --shell bash
lbxx completion install --shell bash
```

Restart the shell. Completion suggests aliases, variable names and `/form/path@field`, including selectors after `NAME=` in `lbxx`. It suggests names only, never stored values, and does not prompt to unlock a Vault or Lockbox.

See [Command-line completion](../../get-started/cli-tooling/command-line-completion.md) for Zsh, Fish, PowerShell and Elvish setup.

## Remove an alias

```bash
lbx vault lockbox alias remove dev
```

This removes the alias while retaining the Lockbox file and its remembered location.
