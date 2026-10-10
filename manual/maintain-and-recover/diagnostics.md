---
description: Use reVault diagnostics and collect useful troubleshooting evidence.
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
format version separately from its internal Vault structure version. An
unsupported container prevents reading the structure; the report says so and
provides Vault-specific migration or upgrade guidance.

`doctor` does not print decrypted file paths, variable values or secret contents. Even so, review diagnostic output before sharing it because local paths and platform details may identify your environment.

For a command failure, record:

* `lbx --version`;
* the exact command with secret values removed;
* its numeric [exit code](../get-started/cli-tooling/exit-codes.md);
* `lbx doctor` output; and
* whether the problem changes after `lbx session stop` and a fresh open.

Do not paste Vault passphrases, Profile backups, fingerprints that have not yet been verified, secret variables or complete Lockbox files into a public issue.

See [Troubleshooting](troubleshooting.md) for symptom-based recovery steps.

## Recovery and deferred maintenance

If cleanup is required, preview `lbx secrets.lbox doctor recover --dry-run`
before proceeding. The format-3 line retains its existing cleanup and salvage
commands. The v4-only `doctor --deep` and `doctor compact` commands remain
on the performance branch and are not available in this line.
