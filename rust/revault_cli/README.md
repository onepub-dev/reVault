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

The CLI creates and opens lockboxes, manages their files, variables and forms,
and manages the vault, profiles, contacts and recipient access that make
sharing possible. It also maintains a local open session so you do not need to
re-enter keys for every command.

```bash
cargo install revault_cli
lbx vault init
```

Begin with the [reVault project overview](https://github.com/onepub-dev/reVault#readme),
then follow its CLI quick start and command guide.

## A complete first lockbox

This example creates a vault-backed lockbox for a small application. It imports
files, stores ordinary configuration and a secret as variables, and stores a
website login as a typed form record. Run `lbx vault init` once before this
example; it creates your local vault and default profile.

```bash
# Create and open an encrypted lockbox for the default vault profile.
lbx create project-secrets.lbox

# Add one host file at a chosen path inside the lockbox.
lbx add project-secrets.lbox ./README.md /project/README.md

# Add every file below a directory. The final path is the destination inside
# the lockbox, not a path on the host machine.
lbx add --recursive project-secrets.lbox ./deploy /project/deploy

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

# Close the local session when you no longer need it. The encrypted .lbox file
# remains; close only removes the temporary local open session.
lbx close project-secrets.lbox
```

Use secret variables for tokens, passwords, and private keys rather than files
or command-line values. `--interactive` is the safest convenient default;
`--stdin`, `--file`, and `--from-env` are available for automated workflows.
To see a secret form field, make that choice explicit with `--secret`, for
example `lbx form get --secret project-secrets.lbox /services/github password`.

## License

See the repository license for licensing terms.
