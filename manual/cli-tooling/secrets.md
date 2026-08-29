---
description: How reVault handles values marked as secret.
---

# Secrets

Files in a Lockbox are encrypted at rest. Variables and Form fields marked as `secret` receive extra handling while a command is running:

* the CLI will not accept their value as a command-line argument;
* secret buffers use locked memory where the operating system supports it;
* buffers are cleared when no longer needed; and
* secret variables are excluded from `variable export`.

Enter a secret interactively or use standard input, a file, or an existing environment variable:

```bash
lbx secrets.lbox variable set API_TOKEN --secret --interactive
lbx secrets.lbox variable set API_TOKEN --secret --stdin
lbx secrets.lbox variable set API_TOKEN --secret --file ./token.txt
lbx secrets.lbox variable set API_TOKEN --secret --from-env API_TOKEN
```

Each alternative has a trade-off. A source file can remain on disk; an environment variable can be inherited by child processes; a pipeline can expose the value to another command. Interactive entry is the safest default for a person at a terminal.

Secret handling reduces accidental exposure. It cannot protect a value from a compromised account, privileged debugger, malicious dependency or the program to which you deliberately supply it.

See [Keeping secrets a secret](../keeping-secrets-a-secret.md) for practical guidance.

