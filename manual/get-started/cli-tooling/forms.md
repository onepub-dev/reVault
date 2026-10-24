---
description: Store related typed values as a Form record.
---

# Forms

A Form groups related fields into a typed record. A login, for example, can keep a username and secret password together instead of relying on a naming convention for separate variables.

## Define a reusable Form

Store a Form definition in the Vault when you want to reuse it across Lockboxes:

```bash
lbx vault forms define login \
  --field username:text \
  --field password:secret
```

Copy that definition into a Lockbox:

```bash
lbx secrets.lbox forms use login
```

For a definition needed by only one Lockbox, define it there directly:

```bash
lbx secrets.lbox forms define login \
  --field username:text \
  --field password:secret
```

## Add and update records

Create a record at a meaningful path:

```bash
lbx secrets.lbox forms add /work/github \
  --type login \
  --name GitHub \
  --interactive
```

Update an ordinary field directly:

```bash
lbx secrets.lbox forms set /work/github@username alice
```

Supply a secret field interactively or through standard input rather than placing it in the command line:

```bash
lbx secrets.lbox forms set /work/github@password --secret --stdin
```

Use `forms list`, `forms get`, `forms move` and `forms remove` to manage records. Use `forms definitions` to inspect definitions in a Lockbox, or `lbx vault forms list` to inspect reusable Vault definitions.

`forms get` and `forms set` take a single `/path@field` selector. This replaces the separate path and field arguments used by earlier alpha CLIs:

```bash
lbx secrets.lbox forms get /work/github@username
lbx secrets.lbox forms get --secret /work/github@password
lbxv a@dev /work/github@password
lbxx a@dev PASSWORD=/work/github@password -- your-command
```

With `lbxx`, omitting `PASSWORD=` uses the field name `password` as the environment name. Form record paths and field identifiers created now cannot contain `@`.

## Describe forms and fields

A definition description explains what a form is for. Each field definition can
also have guidance explaining what to enter. A record has its own optional
description for context about that particular record. Field values do not have
separate descriptions.

```bash
lbx secrets.lbox forms definition-description set login "Credentials for a service"
lbx secrets.lbox forms field-description set login username "Use your full email address"
lbx secrets.lbox forms description set /work/github "Company GitHub account"
lbx secrets.lbox forms show /work/github
```

`forms show` displays the latest definition and field guidance, the record's own
description, and its values with secret values masked. `revision` identifies the
record's stored definition revision; `guidance_revision` identifies the latest
definition used for the displayed guidance. Updating guidance does not rewrite
record values. A changed definition or field description creates a definition
revision; repeating the same change does not.

For reusable definitions, use `lbx vault forms description` and
`lbx vault forms field-description`. `forms use` copies field guidance along with
the selected definition revision.

All description commands support `get`, `set`, and `clear`:

```bash
lbx secrets.lbox forms field-description set login username --file ./username-help.txt
lbx secrets.lbox forms field-description get login username
lbx secrets.lbox forms field-description clear login username
lbx secrets.lbox forms description clear /work/github
```

Descriptions support multiline UTF-8 text. `set` accepts a positional value,
`--file`, `--stdin`, or `--interactive`. `get` writes the exact text without an
added newline. `forms show` displays the full text. List tables show a first-line
preview of up to 100 characters; JSON retains the full text; TSV escapes
backslashes, tabs, carriage returns, and newlines.

Clearing removes optional record or field guidance; setting an empty string
stores an explicitly empty description. Definition descriptions retain their
existing convention: clearing sets the description to an empty string.
Descriptions are encrypted with the Lockbox but appear in ordinary descriptive
output, so do not use them to store secret field values.

Descriptions are native fields in the encrypted format-5 representation,
introduced by the 0.6.x compatibility line. Migrate older Vaults and Lockboxes
before opening them with this version. Format-4 clients cannot read these files.
Each optional description accepts up to one mebibyte of UTF-8 text, subject to
the containing definition's storage limits.
