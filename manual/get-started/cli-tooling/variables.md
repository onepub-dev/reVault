# Variables

A Lockbox variable is a name/value pair. It is useful for configuration that needs to travel with a project without becoming another loose file.

```bash
lbx secrets.lbox variable set DB_HOST 127.0.0.1
lbx secrets.lbox variable set DB_PORT 5432
lbx secrets.lbox variable get DB_HOST
lbx secrets.lbox variable list
```

Variables may live under paths, which lets one Lockbox hold separate environments:

```bash
lbx secrets.lbox variable set /accounting/production/DB_PORT 5432
lbx secrets.lbox variable set /accounting/staging/DB_PORT 5433
```

## Secret variables

Mark passwords, tokens and private credentials as secret:

```bash
lbx secrets.lbox variable set API_TOKEN --secret --interactive
lbx secrets.lbox variable get --secret API_TOKEN
```

Secret values cannot be supplied as a command-line value, because arguments may be exposed through process listings and shell history. Use one of these sources instead:

```bash
lbx secrets.lbox variable set API_TOKEN --secret --stdin
lbx secrets.lbox variable set API_TOKEN --secret --file ./token.txt
lbx secrets.lbox variable set API_TOKEN --secret --from-env API_TOKEN
```

Normal variables may use the same sources, or a positional value as shown in the first examples.

## Export normal variables

The export command intentionally exports only non-secret variables:

```bash
lbx secrets.lbox variable export --format posix
lbx secrets.lbox variable export --format json
```

Other supported formats are `powershell` and `cmd`. Review generated shell output before evaluating it, particularly when variable names or values came from someone else.

Use `variable move` and `variable remove` to reorganise or delete entries. Run `lbx secrets.lbox variable --help` for the complete command surface.

## Read values in scripts

See [Lockbox aliases and script helpers](../../protect-and-share/the-vault/lockbox-aliases.md) for alias management, installation and examples passing several values to one command.

The `lbxv` and `lbxx` helpers accept either a Lockbox file path or a Vault alias such as `a@dev`. Open the Vault and Lockbox with `lbx` first; the helpers do not prompt for credentials.

```bash
lbx vault lockbox alias set dev ./secrets.lbox
lbx a@dev open
TOKEN=$(lbxv a@dev ONEPUB_TOKEN)
lbxx a@dev ONEPUB_TOKEN -- dart pub get
lbxx a@dev ONEPUB_AUTH_TOKEN=ONEPUB_TOKEN -- dart pub get
lbxx a@dev /work/github@token -- your-command
lbxx a@dev GITHUB_TOKEN=/work/github@token -- your-command
```

`lbxv` writes one normal or secret value exactly, without adding a newline. It does not emit shell assignments. Bash command substitution removes trailing newlines; use `lbxx` when those bytes must be retained in an environment value. Never pass `lbxv` output to `eval`.

`lbxx` supports multiple selections before `--`. The default environment name is the variable basename or form field name. Use `NAME=selector` to override it. Names must use ASCII letters, digits and underscore and cannot start with a digit. Duplicate destinations are rejected, case-insensitively on Windows. Every selection must resolve before the child starts. The helper preserves the child's exit code (128 plus the signal number for a child terminated by a Unix signal).

Both helpers deliberately reveal secret values to their consumer. `lbxx` supplies them through the child's environment, without a temporary credentials file or secret command-line arguments. Environment values cannot contain NUL. Avoid shell tracing around secret command substitutions.

`@` is reserved in new internal entry and variable names for form selectors. Existing archives remain readable.
