---
description: "Name a Lockbox once and use its variables and form fields in scripts."
---

# Lockbox aliases and script helpers

Give a Lockbox a short name in your Vault, then use `a@name` wherever a command accepts a Lockbox selector. The `lbxv` helper reads one value; `lbxx` runs a command with selected values in its environment.

These commands are available in the current development branch. The CLI package installs `lbxv` and `lbxx` alongside `lockbox` and `lbx`; older releases may not include them.

## Create and use an alias

Creating a Lockbox registers an alias in your Vault. By default, the alias is
the filename without its final extension:

```bash
lbx project-secrets.lbox create
lbx a@project-secrets variable list
```

Choose a different name with `--alias`:

```bash
lbx ./developer.prod.lbox create --alias dev
lbx a@dev variable list
```

Directory names do not contribute to the alias. The filename stem is normalized
to NFC, then each unsupported character is replaced with `_`. Repeated
underscores are retained. Positional violations, such as a leading hyphen or
an unattached combining mark, are repaired by the same rule:

| Filename | Automatic alias |
| --- | --- |
| `résumé 2026.lbox` | `résumé_2026` |
| `项目资料.lbox` | `项目资料` |
| `dev.prod.lbox` | `dev.prod` |
| `-work.lbox` | `_work` |

An extensionless path such as `project-secrets` creates `project-secrets.lbox`
and derives `project-secrets`.

Vault-backed creation uses the Vault to register the alias. Standalone unsigned
or raw-key creation can still succeed without an unlocked Vault; it warns when
an alias cannot be registered. Supplying `--alias` explicitly requests Vault
access. Explicit invalid aliases fail before creation, with an explanation and
a suggested safe spelling. Automatic aliases are never silently truncated: if
the name cannot be derived or exceeds the limit, the Lockbox is
created and the warning explains why no alias was registered.
If either an explicit or derived alias already exists, the new Lockbox is still
created: a warning explains that no alias was created, and the existing mapping
is preserved. Use the new Lockbox's path or assign it another alias afterward.
A failed creation does not register an alias.

For an existing Lockbox:

```bash
lbx vault lockbox alias set dev ./developer-secrets.lbox
lbx vault lockbox alias list
lbx a@dev open
lbx a@dev variable list
```

Use the bare name `dev` when managing the alias and `a@dev` when selecting its
Lockbox. Unicode aliases follow the same syntax: `lbx a@项目资料 open`.
Setting an existing alias replaces its mapping; creating a new Lockbox never
overwrites an existing alias.

## Unicode names and compatibility

Unicode aliases are part of the format-4 development line on the performance
branch. They use the existing Vault alias records. Format-3 clients reject the
format-4 container before decoding these records; this does not change the Vault
structure version or introduce a second alias collection.

Names use Unicode 17.0.0 properties and NFC normalization. Creation, lookup,
removal, collision checks and completion use the normalized spelling. Composed
`café` and decomposed `café` therefore identify one alias. Case remains
significant, and visually similar characters are not merged or transliterated.
Fish completion preserves the spelling of the prefix already typed so Fish
does not discard an equivalent suggestion. The completed alias is normalized
when used; stored names remain NFC.

New aliases follow these rules:

- The first character is a Unicode letter (`Lu`, `Ll`, `Lt`, `Lm`, `Lo`),
  number (`Nd`, `Nl`, `No`), or ASCII `_`.
- Later characters may also include ASCII `.`, `-`, and combining marks. A mark
  (`Mn`, `Mc`, `Me`) must follow a letter, number, or another mark, not
  punctuation or `_`.
- Unicode default-ignorable characters are rejected, including zero-width and
  bidi controls, combining grapheme joiners, and variation selectors. Whitespace,
  controls, separators and shell metacharacters are also rejected.
- The normalized name must be between 1 and 128 UTF-8 bytes. The limit counts
  bytes rather than characters and is checked after normalization and automatic
  filename translation. Names are never truncated.

Leading `-` or `.` and standalone marks are not accepted for new aliases. In
particular, `.` and `..` are not aliases. An internal or trailing dot is allowed;
aliases are Vault names and do not acquire host filesystem naming restrictions.
Existing ASCII aliases remain readable, resolvable and removable, including
legacy leading-hyphen names. Use `a@-name` to select such an alias and `-- -name`
when removing it: `lbx vault lockbox alias remove -- -name`.

The documented shells are Bash, Zsh, Fish, PowerShell and Elvish. Tests exercise
unquoted alias arguments and completion with UTF-8 terminal/native argument
handling. Platform and shell-version results are recorded in the
[validation evidence](../../../docs/evidence/unicode-aliases-2026-10-10/README.md);
Linux PowerShell results do not establish Windows behavior.

## Alias records and remembered paths

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

## Commands that accept aliases

Use `a@name` wherever a command selects an existing Lockbox: files, variables,
forms, access, mirrors, open/close, diagnostics, recovery and compaction, as well
as the `lbxv` and `lbxx` helpers. Explicit Lockbox arguments also accept aliases:

```bash
lbx session default a@dev
lbx vault lockbox remember a@dev
lbx vault lockbox move a@dev ./secrets/renamed.lbox
lbx doctor migrate lockbox a@dev --replace
lbx a@dev doctor migrate lockbox --replace
lbx vault lockbox alias set work a@dev
lbx vault lockbox forget a@work
```

The two migration forms are alternatives; supply the source in one place only.
`session default` stores the resolved path, and `vault lockbox move` updates it.
`forget` removes the remembered record even if its file is missing. It leaves
the alias record in place, but that alias cannot select a Lockbox until its
target is remembered again.

Recovery output, migration import output, direct migration output and Vault move
destinations also resolve aliases. Existing-file safeguards still apply: aliases
always name existing archives, so creating a new archive or moving onto another
alias refuses to overwrite it. Recovery requires `--overwrite` to replace an
existing output. To create a new destination, use a new filesystem path.

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

Install completion for all four commands in one step. For Bash:

```bash
lbx completion install --shell bash
```

Restart the shell. Completion suggests aliases, variable names and `/form/path@field`, including selectors after `NAME=` in `lbxx`. It suggests names only, never stored values, and does not prompt to unlock a Vault or Lockbox.

See [Command-line completion](../../get-started/cli-tooling/command-line-completion.md) for Zsh, Fish, PowerShell and Elvish setup.

## Remove an alias

```bash
lbx vault lockbox alias remove dev
```

This removes the alias while retaining the Lockbox file and its remembered location.
