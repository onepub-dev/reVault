# revault_cli

reVault is a way to keep files and structured secrets in portable, encrypted
archives called **lockboxes**. A lockbox is a `.lbox` file: you can keep it with
your project, move it between machines, or share it with people you trust.

`revault_cli` is the everyday command-line application for working with those
lockboxes. It is the best place to start if you want to use reVault rather than
embed it in another program.

Before using the commands, these terms are useful:

- A **lockbox** is the encrypted archive that holds files, variables, and form
  records.
- A **vault** is your private local store for profiles, trusted contacts, and
  the keys that let you create and open lockboxes.
- An **profile** is your public/private key pair. You can use it to give
  trusted contacts access to a lockbox without sharing a password.

Installing this crate provides the `lockbox` and `lbx` commands. It is an
application package, not a Rust library: install it with `cargo install`,
rather than adding it as a dependency.

End users can install prebuilt Linux, macOS, and Windows releases without a
Rust toolchain. See the [CLI distribution guide](https://github.com/onepub-dev/reVault/blob/master/docs/cli_distribution.md)
for installers, supported targets, and package-manager plans.

The CLI creates and opens lockboxes, manages their files, variables and forms,
and manages the vault, profiles, contacts and recipient access that make
sharing possible. It also maintains a local open session so you do not need to
re-enter keys for every command.

It can also maintain named, one-way mirror projects between host directories
and exclusively managed lockbox subtrees.

```bash
cargo install revault_cli
lbx vault init
```

Create a separate native vault and introduce its new profile to the current
vault as a public contact:

```bash
lbx vault beget production
lbx access grant deploy.lbox contact:production
```

This creates `production.vault.lbx`; it does not copy the current vault or
grant access by itself. Use `--contact-name` or `--no-contact` to control local
contact creation.

Begin with the [reVault project overview](https://github.com/onepub-dev/reVault#readme),
then follow its CLI quick start and command guide.

## A complete first lockbox

This example creates a vault-backed lockbox for a small application. It imports
files, stores ordinary configuration and a secret as variables, and stores a
website login as a typed form record. Run `lbx vault init` once before this
example; it creates your local vault and default profile.

```bash
# Create and open an encrypted lockbox for the default vault profile.
lbx project-secrets.lbox create

# Add one host file at a chosen path inside the lockbox.
lbx project-secrets.lbox add ./README.md --to project/README.md

# Replace that stored file after the host copy changes.
lbx project-secrets.lbox add ./README.md --to project/README.md --overwrite

# Add every file below a directory. The final path is the destination inside
# the lockbox, not a path on the host machine.
lbx project-secrets.lbox add --recursive ./deploy --to project/deploy/

# Store a normal configuration value. Variables are encrypted metadata, not
# files, so they do not appear in ordinary file listings.
lbx variable set project-secrets.lbox APP_ENV production

# Store a secret without putting its value in shell history or the process list.
# This prompts without echoing the value.
lbx variable set --secret project-secrets.lbox API_TOKEN --interactive

# Define a reusable structured record type in this lockbox. A `secret` field is
# hidden and must be supplied interactively or via an explicit secret source.
lbx form define project-secrets.lbox login \
  --name 'Website login' \
  --description 'Credentials for an external service' \
  --field username:text:required:Username \
  --field password:secret:required:Password \
  --field site:url:required:Website

# Add a login record. --set supplies the non-secret fields; --interactive
# securely prompts for the password field.
lbx form add project-secrets.lbox /services/github \
  --type login \
  --name GitHub \
  --set username=octavia \
  --set site=https://github.com \
  --interactive

# Inspect the non-secret structure and values.
lbx list project-secrets.lbox /
lbx variable get project-secrets.lbox APP_ENV
lbx form show project-secrets.lbox /services/github

# Frequent commands also have familiar aliases: ls, rm, and mv.
lbx project-secrets.lbox ls /

# Close the local session when you no longer need it. The encrypted .lbox file
# remains; close only removes the temporary local open session.
lbx close project-secrets.lbox
```

Use secret variables for tokens, passwords, and private keys rather than files
or command-line values. `--interactive` is the safest convenient default;
`--stdin`, `--file`, and `--from-env` are available for automated workflows.
To see a secret form field, make that choice explicit with `--secret`, for
example `lbx form get --secret project-secrets.lbox /services/github password`.

## One-way directory mirrors

Create a named project, inspect its first update, then apply it:

```bash
lbx backup.lbox mirror project create --from ./project --to /projects/project
lbx backup.lbox mirror project status
lbx backup.lbox mirror project update
```

Creation stores configuration but copies no files. The host is authoritative:
an update adds and replaces selected files and, by default, removes managed
files missing from the host. Set `--missing-files retain` when that project
should preserve archive-only files:

```bash
lbx backup.lbox mirror project configure --missing-files retain
```

The archive TOC is the archive-side manifest. The encrypted project record
stores the canonical host path, lockbox destination, filesystem identity where
available, rules, and missing-file policy. Multiple projects may coexist, but
their destinations cannot overlap. Ordinary file commands cannot mutate a
managed subtree; use the corresponding `mirror NAME add`, `extract`, `cat`,
`list`, `remove`, or `move` command.

Configure persistent source-relative rules explicitly:

```bash
lbx backup.lbox mirror project rule add include 'src/**' README.md
lbx backup.lbox mirror project rule add exclude target/** '*.tmp'
lbx backup.lbox mirror project rule list
lbx backup.lbox mirror project rule remove exclude '*.tmp'
```

The project is a normal encrypted variable under
`/.revault/mirrors/PROJECT`. Variable listings hide dot-prefixed variables
unless `variable list --all` is supplied, and exports always omit them. An
exact `variable get` can inspect one.

Deletion is guarded separately: an empty selected source needs `--allow-empty`,
and a plan deleting more than half the managed files needs
`--allow-large-delete`. These checks reduce accidental wrong-source deletion;
they cannot make a compromised source directory trustworthy.

## License

See the repository license for licensing terms.
