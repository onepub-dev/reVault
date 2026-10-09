---
description: Store related typed values as a Form record.
---

# Forms

A Form groups related fields into a typed record. A login, for example, can keep a username and secret password together instead of relying on a naming convention for separate variables.

## Define a reusable Form

Store a Form definition in the Vault when you want to reuse it across Lockboxes:

```bash
lbx vault form define login \
  --field username:text \
  --field password:secret
```

Copy that definition into a Lockbox:

```bash
lbx secrets.lbox form use login
```

For a definition needed by only one Lockbox, define it there directly:

```bash
lbx secrets.lbox form define login \
  --field username:text \
  --field password:secret
```

## Add and update records

Create a record at a meaningful path:

```bash
lbx secrets.lbox form add /work/github \
  --type login \
  --name GitHub \
  --interactive
```

Update an ordinary field directly:

```bash
lbx secrets.lbox form set /work/github@username alice
```

Supply a secret field interactively or through standard input rather than placing it in the command line:

```bash
lbx secrets.lbox form set /work/github@password --secret --stdin
```

Use `form list`, `form get`, `form move` and `form remove` to manage records. Use `form definitions` to inspect definitions in a Lockbox, or `lbx vault form list` to inspect reusable Vault definitions.

`form get` and `form set` take a single `/path@field` selector. This replaces the separate path and field arguments used by earlier alpha CLIs:

```bash
lbx secrets.lbox form get /work/github@username
lbx secrets.lbox form get --secret /work/github@password
lbxv a@dev /work/github@password
lbxx a@dev PASSWORD=/work/github@password -- your-command
```

With `lbxx`, omitting `PASSWORD=` uses the field name `password` as the environment name. Form record paths and field identifiers created now cannot contain `@`.
