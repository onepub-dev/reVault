# Lockbox CLI How-To

This guide describes the intended Lockbox CLI user experience for the first
format release. Some commands are still being hardened, but the examples define
the target behavior.

## Path Model

Lockbox stores logical archive paths, not host filesystem paths. CLI commands
accept either `/docs/readme.md` or the more convenient relative spelling
`docs/readme.md`; relative logical paths are rooted before they are stored.

Good archive paths:

```text
/docs/readme.md
/project/src/main.rs
/myapp/config.yaml
```

Host absolute paths are not stored in the lockbox. Paths such as `/etc/passwd`,
`C:\Users\bob\file.txt`, `\\server\share\file.txt`, and paths containing `..`
are rejected as unsafe archive entries.

## Add Files

The preferred grammar selects the lockbox before the command:

```bash
lockbox secrets.lbox open
```

The open is cached in a per-user in-memory agent for a short sliding TTL.
Clear the cached open explicitly when done:

```bash
lockbox secrets.lbox close
```

## Exit Codes

Scripts should use the CLI exit status instead of parsing human-readable error
messages. The stable exit codes are `0` for success, `1` for an unclassified
failure, `2` for invalid command usage or input, `10` when the lockbox session
is closed, `11` when authentication fails, `12` when an entry is not found,
`13` when the local vault is unavailable, `14` for an unsupported lockbox or
vault format, and `15` for corrupt or truncated data. In particular, exit code
`10` means the caller should ask the user to run `lbx open <lockbox>` and retry.
Error descriptions and recovery guidance are written to standard error and may
be improved without changing these numeric codes.

Create a lockbox and cache open access:

```bash
lockbox secrets.lbox create
```

Add a directory at a logical destination:

```bash
lockbox secrets.lbox add --recursive ./project --to project/
```

Example stored paths:

```text
/project/README.md
/project/src/main.rs
```

Add only the directory contents:

```bash
lockbox secrets.lbox add --recursive ./project
```

Example stored paths:

```text
/README.md
/src/main.rs
```

Add a directory under a logical destination prefix:

```bash
lockbox secrets.lbox add --recursive ./project \
  --to backups/project-2026/
```

Example stored paths:

```text
/backups/project-2026/README.md
/backups/project-2026/src/main.rs
```

## Add Multiple Files Or Rename One File

Every positional argument after `add` is a host source path. This means shell
expansion works without the CLI mistaking the final file for a destination:

```bash
lockbox secrets.lbox add ./certificates/*.pem --to certificates/
```

When adding multiple sources, `--to` must end in `/`; each source keeps its
basename. Use an exact `--to` path to rename one file:

```bash
lockbox secrets.lbox add ./generated.env --to secrets/prod.env
```

By default, `add` fails before committing when a mapped file already exists.
Make replacement explicit when refreshing stored content:

```bash
lockbox secrets.lbox add ./generated.env \
  --to secrets/prod.env \
  --overwrite
```

`--overwrite` replaces mapped files regardless of their timestamps. It still
adds mapped files that do not yet exist.

## Remove Files

`remove` accepts multiple stored paths, including an unquoted shell expansion:

```bash
lockbox secrets.lbox remove package.json package-lock.json
```

Quote patterns that reVault should match against paths inside the lockbox.
`*` stays within one directory; `**` crosses directory boundaries:

```bash
lockbox secrets.lbox rm '*.json'
lockbox secrets.lbox rm '**/*.json'
```

The complete match set is validated and shown before one confirmation prompt.
Use `--force` to skip the prompt. Removing a directory and its contents
requires `--recursive` (`-r` or `-R`).

## Command Aliases

The CLI uses descriptive command names in documentation and offers conventional
short aliases for frequent operations:

- `list` has the alias `ls`.
- `remove` has the alias `rm`.
- `move` has the alias `mv`.

These aliases are available consistently on command groups that provide the
corresponding operation. `rename` is also a full, descriptive synonym for the
top-level `move` command. Specialized operations such as `extract`, `revoke`,
`forget`, and `recover` are not given potentially ambiguous abbreviations.

Stored path:

```text
/secrets/prod.env
```

## Variables

Lockbox can store environment variables in encrypted variable pages. They are not
file entries, do not appear in `ls`, and should only be loaded when variable commands
or APIs request them.

Set a variable:

```bash
lockbox secrets.lbox variable set DATABASE_URL 'postgres://localhost/app'
lockbox secrets.lbox variable set DATABASE_URL='postgres://localhost/app'
lockbox secrets.lbox variable set DATABASE_URL --value 'postgres://localhost/app'
```

Set a secret variable with an explicit value source:

```bash
lockbox secrets.lbox variable set --secret API_TOKEN --interactive
lockbox secrets.lbox variable set --secret API_TOKEN --file ./api-token.txt
lockbox secrets.lbox variable set --secret API_TOKEN --stdin
lockbox secrets.lbox variable set --secret API_TOKEN --from-env API_TOKEN
lockbox secrets.lbox variable set --secret API_TOKEN --value "$API_TOKEN"
```

Short forms are also supported:

```bash
lockbox secrets.lbox variable set -s API_TOKEN -i
lockbox secrets.lbox variable set -s API_TOKEN -f ./api-token.txt
lockbox secrets.lbox variable set -s API_TOKEN -t
lockbox secrets.lbox variable set -s API_TOKEN -e API_TOKEN
lockbox secrets.lbox variable set -s API_TOKEN -v "$API_TOKEN"
```

Sensitivity is declared when a variable is created. Updating the value preserves
that sensitivity. To change a variable from secret to non-secret, or the other
way around, delete it and recreate it.

Get a variable:

```bash
lockbox secrets.lbox variable get DATABASE_URL
lockbox secrets.lbox variable get --secret API_TOKEN
```

List variable names:

```bash
lockbox secrets.lbox variable list
```

Export variables for shell use:

```bash
lockbox secrets.lbox variable export
```

Remove a variable:

```bash
lockbox secrets.lbox variable rm DATABASE_URL
```

Environment variable names should use portable shell-style names:

```text
DATABASE_URL
FEATURE_FLAG
_PRIVATE_TOKEN
```

Variable names, grouped paths, and patterns are case-sensitive on every
platform. For example, `API_TOKEN` and `api_token` are distinct lockbox
variables, even on Windows where process environment variables are commonly
case-insensitive. Avoid case-only names when values will be exported into a
Windows process environment.

Names that start with a number or contain spaces, dashes, dots, NUL bytes, or
other unsafe characters are rejected. Values are encrypted and bounded; NUL and
control characters are rejected.

Form aliases and field identifiers are also case-sensitive. Display names and
labels remain user-facing text and are not identifiers.

## Visualize A Lockbox

Use `visualize` when developing, diagnosing corruption, or checking what a
lockbox contains without extracting it:

```bash
lockbox secrets.lbox visualize
```

The command is intentionally hidden from normal help; use
`lockbox --help --verbose` to show it.

The command prints public lockbox profile, summary counts for files, symlinks,
environment variables, key slots, logical file bytes, per-page metadata, page
object kinds, and a recovery-scan summary. It does not print file paths, file
contents, environment variable names, or environment variable values. Use
`lockbox list` and `lockbox variable list` for those details.

## List Files

List a directory:

```bash
lockbox secrets.lbox ls /
lockbox secrets.lbox ls /docs
```

Filter with a glob:

```bash
lockbox secrets.lbox ls /docs --glob '*.pdf'
lockbox secrets.lbox ls /docs --glob '**/*.pdf'
```

The glob is applied to logical Lockbox paths. It does not access the host
filesystem.

## Extract Files

Extract one file:

```bash
lockbox secrets.lbox extract /docs/a.txt ./out/a.txt
```

Extract into a selected directory:

```bash
lockbox secrets.lbox extract --to ./out --restore-permissions
```

Example mapping:

```text
/docs/a.txt -> ./out/docs/a.txt
```

Extraction must verify that every destination remains inside the chosen output
directory. Existing files should not be overwritten unless the user explicitly
passes `--overwrite`.

## Logical Destination Prefixes

Use destination prefixes to keep different host trees separate inside a
lockbox:

```bash
lockbox app.lbox add --recursive ./etc/myapp --to config/myapp/
lockbox app.lbox add --recursive ./var/lib/myapp --to data/myapp/
```

The stored paths remain absolute and portable:

```text
/config/myapp/config.yaml
/data/myapp/state.db
```

Relative paths are accepted for convenience when listing or otherwise
addressing entries:

```bash
lockbox app.lbox list config
lockbox app.lbox extract --to ./restore
```

## Symlinks

Symlinks are not restored by default:

```bash
lockbox secrets.lbox extract --to ./out
```

To restore symlinks explicitly:

```bash
lockbox secrets.lbox extract --to ./out --restore-symlinks
```

Lockbox still validates both the symlink path and target as safe logical paths.
Symlinks with `..`, host absolute paths, Windows drive paths, UNC paths,
backslashes, NUL bytes, or control characters are rejected.

## Permissions

Ignore archive permissions and use safe defaults:

```bash
lockbox secrets.lbox extract --to ./out --no-restore-permissions
```

Restore stored permissions where supported:

```bash
lockbox secrets.lbox extract --to ./out --restore-permissions
```

The format stores only basic permission bits. Special bits and platform-specific
metadata should be rejected or ignored unless explicitly supported later.

## Recovery

Inspect a damaged lockbox:

```bash
lockbox damaged.lbox recover --report
```

`recover --report` is read-only. It scans the lockbox and reports what can be
read without writing a new file. Use `--format table` or `--format json` when
you need machine-readable output.

```text
field                value
intact_files         128442
partial_files        3
corrupt_records      7
toc_recovered        false
env_recovered        true
env_count            12
forms_recovered      true
form_definitions     4
form_records         38
```

Write a clean lockbox containing recovered entries:

```bash
lockbox damaged.lbox recover --output recovered.lbox
```

Use `--overwrite` only when replacing an existing recovery output:

```bash
lockbox damaged.lbox recover --output recovered.lbox --overwrite
```

The recovered lockbox is a new valid lockbox with the same content key. It
contains only path-bearing entries whose payloads can be fully read: complete
files, symlinks whose targets can be decoded, plus variable values and form metadata
when the latest commit root is recoverable. Partial files are reported by count
and are skipped rather than written as shortened files.

Recovery can scan fixed-size encrypted pages and encrypted metadata even when
the fixed header or latest TOC is damaged. File content without recoverable path
metadata is not written to the output lockbox, because the current format does
not create unnamed placeholder files during recovery.

## Recipient Keys

Initialize the local vault lockbox:

```bash
lockbox vault init
```

This creates `local-vault.lbox` in the platform-specific vault directory and
prompts for the vault password. For automation, `LOCKBOX_VAULT_PASSWORD` can
supply that password.

Generate the default local recipient keypair and export its public key:

```bash
lockbox vault keygen default alice.pub
```

The default key file format is native Lockbox PEM:

```text
-----BEGIN LOCKBOX PUBLIC KEY-----
...
-----END LOCKBOX PUBLIC KEY-----
```

Private keys are stored inside the encrypted local vault lockbox. There is no
separate private-key password layer.

Import an existing private key file into the vault:

```bash
lockbox vault import-key legacy alice.key alice.pub
```

Export a vault-managed private key:

```bash
lockbox vault profile export-private legacy legacy.key
```

Supported key file formats:

- `lockbox-pem`: default text format with `BEGIN LOCKBOX PRIVATE KEY` or
  `BEGIN LOCKBOX PUBLIC KEY` armor.
- `jwk`: JSON Web Key using the Lockbox ML-KEM-1024 profile.
- `jwks`: JSON Web Key Set containing one key.
- `raw-hex`: legacy/developer hex encoding of the raw ML-KEM seed or recipient
  key.

Select an export format with `--format`:

```bash
lockbox vault profile export --format jwk default alice.jwk
lockbox vault profile export-private --format lockbox-pem legacy legacy.key
```

Imports auto-detect native Lockbox PEM, JWK, JWKS, and raw hex.

Trust another recipient public key in the local vault:

```bash
lockbox vault trust bob bob.pub
```

List local vault records:

```bash
lockbox vault list
```

Remove local vault records:

```bash
lockbox vault remove-key default
lockbox vault remove-trusted bob
```

The default vault location is platform-specific and can be overridden:

```bash
LOCKBOX_VAULT_DIR=/secure/local/vault lockbox vault init
```

Create a lockbox for one of your vault keys:

```bash
lockbox secrets.lbox create --for default
```

Add a recipient public key or trusted recipient name to an opened lockbox:

```bash
lockbox add-recipient secrets.lbox alice.pub
lockbox add-recipient secrets.lbox bob
```

List key slots:

```bash
lockbox list-keys secrets.lbox
```

Remove a key slot:

```bash
lockbox remove-key secrets.lbox 2
```

Removing a key is a compaction operation. The CLI rewrites the current lockbox state
so stale key-directory history is not left behind as an easy way for the removed
credential to keep opening the lockbox.

Open with a private key:

```bash
lockbox secrets.lbox open-key default
```

If no key name is supplied, `open-key` uses the default private key in the local
vault:

```bash
lockbox secrets.lbox open-key
```

The CLI uses vault-managed private keys by name. External private key files
should be imported into the vault before use rather than loaded directly.

Commands that create, open, or change lockbox key slots mirror the current
key directory into the local vault as a recovery aid. The lockbox remains the
portable source of truth; the local mirror is user-local convenience state.
When the lockbox header is intact but embedded key-directory copies are damaged,
`open` and `open-key` can use the local mirror to unwrap the content key.

## Safety Summary

### Synchronize a host directory safely

`sync` maps the contents of a host directory directly below one logical
lockbox directory:

```bash
lbx backup.lbox sync ./project --to /project --dry-run
lbx backup.lbox sync ./project --to /project --force
```

New files are added, changed files are replaced using content-hash comparison,
and unchanged files are skipped. Lockbox-only files are preserved unless
`--delete` is supplied:

```bash
lbx backup.lbox sync ./project --to /project --delete --dry-run
lbx backup.lbox sync ./project --to /project --delete --force
```

Use repeatable source-relative inclusion and exclusion rules such as
`--include 'src/**'`, `--exclude .git/`, and `--exclude '*.tmp'`. Ordinary
deletion protects filtered entries;
`--delete-excluded` explicitly removes them.

The first successful run stores an encrypted profile containing the canonical
source path, destination, rules, symlink policy, stable profile id, and a
filesystem directory identity when available. On Unix this is the device and
inode pair. A moved or replaced source needs `--rebind-host-path`; changed
include/exclude rules need `--update-rules`. Empty sources and
plans deleting more than half of the destination require `--allow-empty` and
`--allow-large-delete` respectively.

When a later sync omits `--include` and `--exclude`, it reuses the stored
rules. Supplying different rules is rejected unless `--update-rules` is also
given. Inspect them with `sync --show-rules --to /destination`. An
`--update-rules` invocation without rule arguments is rejected; use the
explicit `--clear-rules` option to remove every stored rule.

The profile is stored as a normal encrypted variable under
`/.revault/sync/`. Dot-prefixed variables are omitted from ordinary
`variable list` output and from every export. Use `variable list --all` to
show their names and exact `variable get` to inspect a selected value.

The CLI should reject or fail closed on:

- `..` path components,
- host absolute paths in archive entries,
- Windows drive paths,
- UNC paths,
- backslashes in archive paths,
- unsafe symlinks,
- extraction outside the selected output directory or mapped root,
- overwriting existing files unless `--overwrite` is explicit,
- extraction that exceeds configured file count or byte limits.

The mental model is:

```text
Archive stores logical paths and logical roots.
The CLI maps logical roots to real disk locations only at extraction time.
```
