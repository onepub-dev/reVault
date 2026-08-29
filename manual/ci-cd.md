# CI/CD

reVault can keep deployment configuration and credentials together in an encrypted Lockbox. The important design choice is separation: development, CI, staging and production should not share one broad credential set.

A practical layout is one Lockbox and one Profile per environment. Give the CI runner access only to the CI Profile and secrets it actually needs. Do not copy a developer's complete Vault onto the runner.

## Prepare access

On the runner, initialise its own Vault and Profile:

```bash
lbx vault init
lbx vault profile create ci
lbx vault profile export ./ci.pub --name ci
lbx vault profile fingerprint ci
```

On an administration machine, import that public Profile as a Contact after independently checking its fingerprint, then grant it access:

```bash
lbx vault contact import ci-runner ./ci.pub \
  --fingerprint <fingerprint-code> \
  --fingerprint-channel deployment-console
lbx ci.lbox access grant ci-runner
```

Transfer the updated Lockbox to the runner through your normal artefact channel.

## Use secrets in a job

Prefer retrieving only the value needed by the current step:

```bash
lbx ci.lbox variable get --secret DEPLOY_TOKEN
```

Avoid command tracing around secret-handling steps, and do not print the result. reVault deliberately excludes secret variables from `variable export`.

The runner still needs a way to unlock its Vault. Supply that through the CI platform's protected secret mechanism or provision the machine's secure credential store. Treat Auto Open on a shared runner with care: any process operating as that account may be able to use it.

Rotate runner credentials regularly and immediately after suspected exposure. Revoking a Contact from a Lockbox cannot invalidate a secret the runner has already read.

