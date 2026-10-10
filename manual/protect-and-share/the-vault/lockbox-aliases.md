---
description: "Name a Lockbox once and use its variables and form fields in scripts."
---

# Lockbox aliases and script helpers

Give a Lockbox a short name in your Vault, then use that name wherever a command accepts a Lockbox selector. Use `a@name` to select the alias explicitly. The `lbxv` helper reads one value; `lbxx` runs a command with selected values in its environment.

These commands are available in the current development branch. The CLI package installs `lbxv` and `lbxx` alongside `lockbox` and `lbx`; older releases may not include them.

## Create and use an alias

For an existing Lockbox:

```bash
lbx vault lockboxes aliases set dev ./developer-secrets.lbox
lbx vault lockboxes aliases list
lbx dev open
lbx dev variables list
```

Names are case-sensitive and contain up to 128 ASCII letters, digits, underscores or hyphens. Setting an existing alias replaces its mapping.

A simple selector such as `dev`, with no path or extension, checks both the local Lockbox filename and the Vault alias. If only one exists, it selects that Lockbox. If both identify the same Lockbox, the selection is also accepted. If they identify different Lockboxes, the command stops and asks you to choose explicitly:

```bash
lbx a@dev open    # Select the Vault alias
lbx ./dev.lbox open # Select a local filename
lbx dev.lbox open # Select a local filename
```

Paths and names with an extension select files. They do not fall back to aliases.

Aliases are encrypted Vault records and are included in [Vault backups](backup-and-restore.md). They identify a Lockbox by its stable identity. Move the file through reVault to keep its remembered location current. The destination directory must already exist:

```bash
lbx vault lockboxes move ./developer-secrets.lbox ./secrets/developer.lbox
```

If you already moved the file through a shell or file manager, record its new location instead:

```bash
lbx vault lockboxes remember ./secrets/developer.lbox
```

A missing target or a different Lockbox at the remembered path causes an error. Restoring a Vault onto another machine may require remembering the target's new path.

`a@dev` always selects an alias. To select an actual host file named `a@dev`, use `./a@dev` (or `.\a@dev` on Windows).

## Commands that accept aliases

Use a bare name or `a@name` wherever a command selects an existing Lockbox: files, variables,
forms, access, mirrors, open/close, diagnostics, recovery and compaction, as well
as the `lbxv` and `lbxx` helpers. Explicit Lockbox arguments also accept aliases:

```bash
lbx session default a@dev
lbx vault lockboxes remember a@dev
lbx vault lockboxes move a@dev ./secrets/renamed.lbox
lbx doctor migrate lockbox a@dev --replace
lbx a@dev doctor migrate lockbox --replace
lbx vault lockboxes aliases set work a@dev
lbx vault lockboxes forget a@work
```

The two migration forms are alternatives; supply the source in one place only.
`session default` stores the resolved path, and `vault lockboxes move` updates it.
`forget` removes the remembered record even if its file is missing. It leaves
the alias record in place, but that alias cannot select a Lockbox until its
target is remembered again.

Recovery output, migration import output, direct migration output and Vault move
destinations also resolve aliases. Existing-file safeguards still apply: creating
a new archive or moving onto another alias refuses to overwrite its existing
target. Only `create` creates a new Lockbox; it can recreate a missing target
selected through an alias or the session default. Other operations report a
missing target instead of recreating it. Recovery requires `--overwrite` to
replace an existing output. To name a new destination explicitly, use a
filesystem path.

Aliases do not substitute for paths *inside* an archive, ordinary input files,
exported value files, migration artifacts, key files or Vault backup files.
Use `./a@name` for a literal host filename beginning with `a@`.

## Read one value with lbxv

Open the Vault and Lockbox with `lbx` first. The helpers never prompt for credentials.

In Bash, capture a variable or form field with command substitution:

```bash
TOKEN=$(lbxv a@dev ONEPUB_TOKEN)
PASSWORD=$(lbxv a@dev /database@password)
```

You can also use `lbxv dev ONEPUB_TOKEN`, or omit the Lockbox argument to use
the session default: `lbxv ONEPUB_TOKEN`.

`lbxv` accepts exactly one selector and writes its value without an added newline. It reads both normal and secret values, without a `--secret` flag. It does not accept `NAME=selector` or emit shell assignments.

Bash removes trailing newlines in command substitutions. Use `lbxx` when those bytes must be preserved in an environment value. Avoid shell tracing around secret substitutions, and never pass the output to `eval`.

## Run a command with lbxx

Pass one or more selections before `--`, followed by the command and its arguments:

```bash
lbxx a@dev ONEPUB_TOKEN -- dart pub get
lbxx a@dev ONEPUB_AUTH_TOKEN=ONEPUB_TOKEN -- dart pub get
lbxx a@dev ONEPUB_TOKEN /database@username DB_PASSWORD=/database@password -- dart run
```

Use `lbxx --lockbox dev ONEPUB_TOKEN -- dart pub get` to select a bare name
explicitly. With multiple positional arguments, `lbxx` recognizes a leading
local Lockbox or an alias in the unlocked Vault. Otherwise, positional arguments
are value selections from the session default. A single value selection always
uses the default; `--lockbox` removes any uncertainty about which argument names
the Lockbox.

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

Install completion for all four commands in one step. For Bash:

```bash
lbx completion install --shell bash
```

Restart the shell. Completion suggests aliases, variable names and `/form/path@field`, including selectors after `NAME=` in `lbxx`. It suggests names only, never stored values, and does not prompt to unlock a Vault or Lockbox.

See [Command-line completion](../../get-started/cli-tooling/command-line-completion.md) for Zsh, Fish, PowerShell and Elvish setup.

## Remove an alias

```bash
lbx vault lockboxes aliases remove dev
```

This removes the alias while retaining the Lockbox file and its remembered location.
