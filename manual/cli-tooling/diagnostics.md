---
description: "Use reVault diagnostics and collect useful troubleshooting evidence."
---

# Diagnostics

Run the general diagnostic first:

```bash
lbx doctor
```

It reports the CLI version, Vault state, Session Agent, Auto Open support and relevant platform capabilities. Add a Lockbox to inspect its accessible public and authenticated state:

```bash
lbx secrets.lbox doctor
```

`doctor` does not print decrypted file paths, variable values or secret contents. Even so, review diagnostic output before sharing it because local paths and platform details may identify your environment.

For a command failure, record:

* `lbx --version`;
* the exact command with secret values removed;
* its numeric [exit code](exit-codes.md);
* `lbx doctor` output; and
* whether the problem changes after `lbx session stop` and a fresh open.

Do not paste Vault passphrases, Profile backups, fingerprints that have not yet been verified, secret variables or complete Lockbox files into a public issue.

See [Troubleshooting](../troubleshooting.md) for symptom-based recovery steps.
