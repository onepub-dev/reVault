# command-line completion

reVault provides command-line completion for:

* Bash
* Zsh
* Fish
* PowerShell
* Elvish

Completion works with both executable names:

lockbox  or lbx

It can complete:

* Commands and subcommands
* Command options
* Vault identity names
* Saved contact names
* Reusable form names
* Paths, variables and forms stored inside open lockboxes

### Install completion

In most environments, reVault can detect the current shell:

`lockbox completion install`

Alternatively:

`lbx completion install`

Restart the shell after installation.

If reVault cannot identify the shell, specify it explicitly:

`lockbox completion install --shell bash`

`lockbox completion install --shell zsh`

`lockbox completion install --shell fish`

`lockbox completion install --shell powershell`

`lockbox completion install --shell elvish`

PowerShell completion is installed as a managed block in the current user’s PowerShell profile. Uninstalling completion removes only the block managed by reVault.

### Completing commands

Type part of a command and press Tab:

`lockbox va`

This can complete to:

`lockbox vault`

Continue typing and press Tab again:

`lockbox vault id`

This can complete to:

`lockbox vault identity`

To see the available subcommands:

`lockbox vault identity`

Suggestions include commands such as:

list create history email fingerprint publish backup restore export remove rotate

The exact display and whether Tab must be pressed once or twice depends on the shell’s completion configuration.

### Completing options

Options are completed in the same way:

`lockbox completion install --sh`

This completes to:

`lockbox completion install --shell`

Shell names can then be completed:

`lockbox completion install --shell po`

This completes to:

`lockbox completion install --shell powershell`

Other examples include:

`lockbox list --rec`

`lockbox vault identity create --over`

`lockbox completion generate --out`

### Completing vault identities

When vault is automatically open (by the user session), reVault can suggest identity names.

Suppose the vault contains these identities:

default laptop production

Enter:

`lockbox vault identity history lap`

This completes to:

`lockbox vault identity history laptop`

It also works with identity options:

`lockbox vault identity export ./laptop.pub --name lap`

This completes to:

`lockbox vault identity export ./laptop.pub --name laptop`

### Completing identities and contacts

When creating a lockbox, the --for option can suggest both your identities and saved contacts.

Suppose the vault contains:

* Identity: laptop
* Identity: production
* Contact: alice
* Contact: accounts-team

Enter:

`lockbox create --for al`

This completes to:

`lockbox create --for alice`

Saved contact names can also be completed for contact-management commands:

`lockbox vault contact remove acc`

This completes to:

\`lockbox vault contact remove accounts-team\`

### Completing reusable forms

Suppose the vault contains these reusable forms:

login credit-card server-access

Enter:

`lockbox form use log`

This completes to:

`lockbox form use login`

You can then provide the lockbox:

`lockbox form use login secrets.lbox`

With a default lockbox configured, the lockbox argument can be omitted:

`lockbox form use login`

### Completing paths inside a lockbox

For open lockboxes, reVault can complete stored paths.

First open the lockbox:

`lockbox open secrets.lbox`

Suppose it contains:

/documents/ /documents/report.pdf /documents/notes.txt /projects/ /projects/revault/design.md /secrets/api-token.txt

#### Read a stored file

Complete a stored path for cat:

`lockbox cat secrets.lbox /doc`

Possible suggestions include:

`/documents/ /documents/report.pdf /documents/notes.txt`

Continue typing to narrow the result:

`lockbox cat secrets.lbox /documents/rep`

This completes to:

`lockbox cat secrets.lbox /documents/report.pdf`

#### List a stored directory

Enter:

`lockbox list secrets.lbox /pro`

This completes to:

`lockbox list secrets.lbox /projects/`

#### Extract a stored file

Enter:

`lockbox extract secrets.lbox /documents/not`

This completes the stored source path:

`lockbox extract secrets.lbox /documents/notes.txt`

#### Remove a stored entry

Enter:

`lockbox rm secrets.lbox /secrets/api`

This completes to:

`lockbox rm secrets.lbox /secrets/api-token.txt`

### Using completion with a default lockbox

If a default lockbox has been configured, commands that support the default lockbox can omit its path.

Set the default lockbox:

`lockbox session default secrets.lbox`

You can then complete a stored path directly:

`lockbox cat /documents/rep`

This completes to:

`lockbox cat /documents/report.pdf`

Other examples include:

lockbox list /projects/

lockbox rm /documents/not

lockbox extract /documents/report

### Closing a lockbox

Lockbox path completion depends on the lockbox being open.

Close a lockbox when it is no longer needed:

`lockbox close secrets.lbox`

After it has been closed, its stored paths are no longer offered as completion suggestions.

To inspect the current sessions:

`lockbox session`

### When encrypted suggestions are unavailable

Commands, subcommands, and options can still be completed without access to encrypted data.

Vault identities, contacts, forms, and stored lockbox paths are suggested only when the vault is auto-opened.

Vault suggestions can use:

* An enabled Lockbox Session Agent containing the vault unlock secret
* The LOCKBOX\_VAULT\_PASSWORD environment variable

When encrypted information is unavailable, reVault omits those suggestions. Command and option completion continues to work.

### Generate a completion script manually

If you manage shell configuration yourself, you can generate a registration script without installing it.

#### Bash

Write the script to standard output:

lockbox completion generate --shell bash

Write it to a file:

lockbox completion generate\
\--shell bash\
\--output ./lockbox-completion.bash

#### Zsh

lockbox completion generate\
\--shell zsh\
\--output ./\_lockbox

#### Fish

lockbox completion generate\
\--shell fish\
\--output ./lockbox.fish

#### PowerShell

lockbox completion generate `--shell powershell` --output ./lockbox-completion.ps1

#### Elvish

lockbox completion generate\
\--shell elvish\
\--output ./lockbox-completion.elv

Manual generation is useful when:

* Shell configuration is managed centrally
* Completion is installed as part of a package
* Building a container image
* Installing completion for multiple users
* Using a non-standard completion directory

### Install to an explicit path

Override the standard installation path with --path.

e.g.

`lockbox completion install`\
`--shell bash`\
`--path ~/.local/share/bash-completion/completions/lockbox`

When --path is supplied, reVault writes the completion script to that location. Ensure the selected location is loaded by the shell.

### Remove completion

Remove completion using automatic shell detection:

`lockbox completion uninstall`

Alternatively, specify the shell:

`lockbox completion uninstall --shell bash`

`lockbox completion uninstall --shell zsh`

`lockbox completion uninstall --shell fish`

`lockbox completion uninstall --shell powershell`

`lockbox completion uninstall --shell elvish`

If completion was installed to an explicit path, provide the same path when uninstalling.

For Bash:

`lockbox completion uninstall`\
`--shell bash`\
`--path ~/.local/share/bash-completion/completions/lockbox`

For PowerShell:

`lockbox completion uninstall --shell powershell --path "$HOME\lockbox-completion.ps1"`

Restart the shell after uninstalling.
