---
description: "Use reVault diagnostics and collect useful troubleshooting evidence."
---

# Diagnostics

Run the general diagnostic first:

```bash
lbx doctor
```

Without a session default, it reports the CLI version, Vault state, Session Agent, Auto Open support and relevant platform capabilities. With a default, it inspects that Lockbox. Supply a path or alias to select a different archive:

```bash
lbx secrets.lbox doctor
```

The global report identifies the local Vault file and its Lockbox container
format version separately from its internal Vault structure version. For
known container versions 2 and 3, `doctor` can probe the structure version
read-only when the Vault is available to unlock. It does not probe the
structure of an unsupported container version; the report says so and provides
Vault-specific migration or upgrade guidance.

For example, a Vault whose container still uses version 2 but whose internal
structure is version 3 reports both values:

```text
Local vault
  container format version: 2
  structure version: 3
```

This container-2/structure-3 state needs a container migration. Run
`lbx doctor migrate vault --replace` and then run `lbx doctor` again to confirm
the container format has advanced to version 3.

To migrate the Vault and all Lockboxes remembered by it together, run
`lbx doctor migrate all --replace`. The Vault is upgraded first, and each
successful replacement retains a backup. The command reports missing or
unreadable Lockboxes individually; it does not recreate them.

`doctor` does not print decrypted file paths, variable values or secret contents. Even so, review diagnostic output before sharing it because local paths and platform details may identify your environment.

For a command failure, record:

* `lbx --version`;
* the exact command with secret values removed;
* its numeric [exit code](exit-codes.md);
* `lbx doctor` output; and
* whether the problem changes after `lbx session stop` and a fresh open.

Do not paste Vault passphrases, Profile backups, fingerprints that have not yet been verified, secret variables or complete Lockbox files into a public issue.

See [Troubleshooting](../troubleshooting.md) for symptom-based recovery steps.

## Recovery and deferred maintenance

If cleanup is required, preview `lbx secrets.lbox doctor recover --dry-run`
before proceeding. The format-3 line retains its existing cleanup and salvage
commands. The v4-only `doctor --deep` and `doctor compact` commands remain
on the performance branch and are not available in this line.
