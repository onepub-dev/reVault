# Command-line completion

reVault can install dynamic completion for Bash, Zsh, Fish, PowerShell and Elvish. It works with both `lockbox` and `lbx`.

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
