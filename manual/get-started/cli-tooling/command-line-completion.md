# Command-line completion

reVault can install dynamic completion for Bash, Zsh, Fish, PowerShell and Elvish. It supports `lockbox`, `lbx`, `lbxv` and `lbxx`.

The value helpers are installed alongside the main CLI, but have their own completion registrations. Installing completion for `lbx` does not register them:

```bash
lbxv completion install --shell bash
lbxx completion install --shell bash
```

Use the same supported shell names with either helper. Completion suggests `a@alias`, variable names and `/form/path@field` selectors, including the source after `NAME=`. Suggestions contain names only, never stored values. The Vault and target Lockbox must already be available to the session; completion does not prompt to unlock them.

See [Lockbox aliases and script helpers](../../protect-and-share/the-vault/lockbox-aliases.md) for setup and examples combining multiple selections.

In most environments reVault detects the current shell:

```bash
lbx completion install
```

Specify it when detection is not possible:

```bash
lbx completion install --shell bash
lbx completion install --shell zsh
lbx completion install --shell fish
lbx completion install --shell powershell
lbx completion install --shell elvish
```

Restart the shell after installation. PowerShell uses a managed block in the current user's profile; uninstalling removes only that block.

Remove the installed completion with:

```bash
lbx completion uninstall
```

To manage the script yourself, generate it on standard output or into a file:

```bash
lbx completion generate --shell bash
lbx completion generate --shell bash --output ./lbx-completion.bash
```

## Dynamic suggestions

As well as commands and options, completion can suggest:

* Profile and Contact names;
* reusable Form names;
* remembered Lockbox paths; and
* paths, variables and Forms inside an open Lockbox.

For example, after opening `secrets.lbox`:

```bash
lbx secrets.lbox open
lbx secrets.lbox cat /doc<Tab>
```

Encrypted suggestions are offered only when reVault can open the relevant Vault or Lockbox. Static command and option completion remains available when it cannot.

Set a default Lockbox if you want shorter completion and commands:

```bash
lbx session default secrets.lbox
lbx cat /doc<Tab>
```
