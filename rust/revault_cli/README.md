# revault_cli

The reVault command-line application. Installing this crate provides the
`lockbox` and `lbx` commands for creating, opening, and managing encrypted
lockboxes and local vaults. It is an application package, not a library crate:
install it with `cargo install`, rather than adding it as a dependency.

The CLI manages files, variables, forms, recipient access, identities, contacts,
and the local vault-backed open session used to avoid repeatedly entering keys.

```bash
cargo install revault_cli
lbx vault init
```

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete command guide.

## License

See the repository license for licensing terms.
