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
