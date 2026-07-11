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
- A **vault** is your private local store for identities, trusted contacts, and
  the keys that let you create and open lockboxes.
- An **identity** is your public/private key pair. You can use it to give
  trusted contacts access to a lockbox without sharing a password.

Installing this crate provides the `lockbox` and `lbx` commands. It is an
application package, not a Rust library: install it with `cargo install`,
rather than adding it as a dependency.

The CLI creates and opens lockboxes, manages their files, variables and forms,
and manages the vault, identities, contacts and recipient access that make
sharing possible. It also maintains a local open session so you do not need to
re-enter keys for every command.

```bash
cargo install revault_cli
lbx vault init
```

Begin with the [reVault project overview](https://github.com/onepub-dev/reVault#readme),
then follow its CLI quick start and command guide.

## License

See the repository license for licensing terms.
