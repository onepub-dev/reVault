# CI/CD with reVault

> **Draft command design:** `lockbox vault ci bootstrap` and
> `lockbox vault ci open` are proposed commands. They are not implemented yet.
> This document defines the intended user experience for review.

reVault can keep a versioned bundle of deployment secrets beside a project
without placing the decrypted values in the repository. A CI job receives one
passphrase from its native secret store, opens the encrypted bundle for the
duration of a command, and then discards the temporary vault and session.

The intended setup has two commands:

- `lockbox vault ci bootstrap` creates a dedicated CI identity and optionally
  grants it access to one or more lockboxes.
- `lockbox vault ci open` reconstructs that identity temporarily, opens the
  requested lockbox, runs a command, and cleans up.

The CI profile is stable so its access can be granted and revoked. The local
vault used by an individual job is temporary.

## Five-Minute Setup

Assume `deploy.lbox` contains the variables needed by `./ci/deploy.sh`.

Open the lockbox locally as an owner, then bootstrap a production CI identity:

```bash
lockbox open deploy.lbox

lockbox vault ci bootstrap production \
  --output .revault/ci/production.lockbox-ci \
  --grant deploy.lbox
```

The command creates:

```text
.revault/ci/production.lockbox-ci
```

The bundle contains an encrypted, minimal vault with one profile. It does not
contain the vault passphrase. The command prints the generated passphrase once
and instructs the operator to save it as this CI secret:

```text
LOCKBOX_VAULT_PASSWORD
```

Commit the encrypted CI bundle and the lockbox:

```bash
git add .revault/ci/production.lockbox-ci deploy.lbox
git commit -m "Configure the production CI identity"
```

This is appropriate only when bootstrap generated a high-entropy passphrase
and repository policy permits encrypted key material. If the bundle must not be
committed, store it as a protected CI file or deployment artifact and pass its
downloaded path to `ci open`. Keep the bundle separate from its passphrase in
either case.

Configure `LOCKBOX_VAULT_PASSWORD` as a protected secret for the production
environment in the CI provider. The deployment job can then run:

```bash
lockbox vault ci open .revault/ci/production.lockbox-ci \
  --lockbox deploy.lbox \
  -- ./ci/deploy.sh
```

`./ci/deploy.sh` and its child processes inherit the temporary reVault runtime.
They can use normal commands without knowing where the temporary vault or agent
is stored:

```bash
#!/usr/bin/env bash
set -euo pipefail

token_file="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/deployment-token"

lockbox variable get --secret \
  --output "$token_file" \
  deploy.lbox DEPLOYMENT_TOKEN

deploy-tool --token-file "$token_file"
```

The secret is written to a temporary file instead of the build log or command
line. The CI runner should be ephemeral, or the script should remove any output
files containing decrypted values.

## Bootstrap Command

The proposed command shape is:

```text
lockbox vault ci bootstrap <name>
    [--output <bundle>]
    [--grant <lockbox>]...
    [--public-key <file>]
    [--password-stdin]
```

For example:

```bash
lockbox vault ci bootstrap production \
  --output .revault/ci/production.lockbox-ci \
  --grant deploy.lbox \
  --grant release-signing.lbox
```

The command should:

1. Create an isolated temporary vault without changing the user's normal
   vault.
2. Create exactly one profile named after the CI identity.
3. Generate a strong random vault passphrase when one is not supplied through
   `LOCKBOX_VAULT_PASSWORD` or `--password-stdin`.
4. Create one encrypted CI bundle containing the minimal vault plus public
   metadata such as its name, format version, and fingerprint.
5. Grant the profile to every `--grant` lockbox using the existing owner
   session. Each grant changes the lockbox and must be committed by the user.
6. Optionally write the public key to `--public-key` when access will be
   granted on another machine.
7. Print the fingerprint, output path, CI secret name, and next commands.
8. Never print private profile recovery material.

If `--grant` is omitted, bootstrap should explain how to grant access manually:

```bash
lockbox vault ci bootstrap production \
  --output production.lockbox-ci \
  --public-key production.pub

lockbox access grant deploy.lbox ci-production production.pub
```

The generated passphrase may be shown once on an interactive terminal. It must
not be shown when stdout is redirected or the command is running in CI. In
those cases the caller must provide it through `LOCKBOX_VAULT_PASSWORD` or
`--password-stdin`.

### Suggested output

```text
CI identity created: production
Fingerprint: 4a:7f:...
Encrypted bundle: .revault/ci/production.lockbox-ci
Granted access: deploy.lbox

Create a protected CI secret named LOCKBOX_VAULT_PASSWORD with this value:
  <generated passphrase shown once>

Run in CI:
  lockbox vault ci open .revault/ci/production.lockbox-ci \
    --lockbox deploy.lbox -- ./ci/deploy.sh
```

## Open Command

The proposed command shape is:

```text
lockbox vault ci open <bundle>
    --lockbox <lockbox>...
    [--duration <duration>]
    [--env <name>[=<variable>]]...
    -- <command> [arguments...]
```

The child-command form is deliberate. A process cannot safely change its
parent shell's environment, and CI steps frequently run in separate shells.
Running the deployment as a child gives `ci open` a clear lifetime in which it
can guarantee setup and cleanup.

`ci open` should:

1. Require `LOCKBOX_VAULT_PASSWORD`, unless a secure password input option was
   selected explicitly.
2. Create private temporary vault and session-agent directories.
3. Restore and verify the encrypted CI bundle.
4. Disable platform credential-store integration for the temporary vault.
5. Open every requested lockbox using the CI profile.
6. Run the child command with the temporary vault and agent settings inherited.
7. Forward termination signals and return the child command's exit status.
8. Close all opened lockboxes, stop the session agent, zeroize in-memory secret
   material, and remove temporary state on success or failure.

The command must not echo the vault passphrase, decrypted variables, or the
child command's environment.

### Explicit environment injection

For tools that conventionally consume environment variables, repeated `--env`
options could provide a convenient, explicit shortcut:

```bash
lockbox vault ci open .revault/ci/production.lockbox-ci \
  --lockbox deploy.lbox \
  --env API_TOKEN \
  --env DATABASE_URL=/production/DATABASE_URL \
  -- ./ci/deploy.sh
```

`--env API_TOKEN` reads `API_TOKEN` and gives it the same name in the child
environment. `--env DATABASE_URL=/production/DATABASE_URL` maps a lockbox
variable path to `DATABASE_URL`.

Injection is explicit: `ci open` must not export every secret variable by
default. The values exist only in the child process and its descendants. For a
credential that can be consumed from a file, `lockbox variable get --secret
--output ...` remains preferable.

The initial implementation can omit `--env`; the managed child-command scope is
the essential behavior.

## GitHub Actions

Store `LOCKBOX_VAULT_PASSWORD` as a repository or environment secret. Use an
environment secret for deployments that require environment protection or
approval. GitHub documents how secrets are made available through the `secrets`
context in [Using secrets in GitHub Actions][github-secrets].

An example deployment job is:

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: production
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4

      - name: Install reVault
        run: ./ci/install-revault.sh

      - name: Deploy
        env:
          LOCKBOX_VAULT_PASSWORD: ${{ secrets.LOCKBOX_VAULT_PASSWORD }}
        run: |
          lockbox vault ci open .revault/ci/production.lockbox-ci \
            --lockbox deploy.lbox \
            -- ./ci/deploy.sh
```

Do not provide the vault password to a job that runs unreviewed pull-request
code. A workflow that receives the password can decrypt every lockbox granted
to that CI profile. GitHub notes that secret redaction is not guaranteed for
transformed values, so scripts must still avoid printing secrets.

## GitLab CI/CD

Create `LOCKBOX_VAULT_PASSWORD` under **Settings > CI/CD > Variables**. Mark it
masked, hidden, and protected when those options are available. Scope it to the
deployment environment when environment-scoped variables are available. See
GitLab's [CI/CD variable documentation](https://docs.gitlab.com/ci/variables/)
for the current options.

```yaml
deploy-production:
  stage: deploy
  environment:
    name: production
  script:
    - ./ci/install-revault.sh
    - >-
      lockbox vault ci open
      .revault/ci/production.lockbox-ci
      --lockbox deploy.lbox
      -- ./ci/deploy.sh
  rules:
    - if: '$CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH'
```

The protected variable is inherited as `LOCKBOX_VAULT_PASSWORD`; it does not
need to appear in `.gitlab-ci.yml`.

## One Identity per Trust Boundary

Create separate CI identities when access should be independently revocable.
Typical boundaries are:

- production and staging;
- unrelated repositories;
- build and deployment jobs;
- separate customers or tenants.

Do not share a developer's profile with CI. A dedicated profile makes it
possible to revoke a compromised pipeline without changing human access.

Creating one identity per job run is also undesirable: every run would have a
new public key and every lockbox would need to be modified before that run
could open it. The CI profile should be stable while the vault restored on the
runner remains ephemeral.

## Rotation and Revocation

Rotate a CI identity without interrupting deployments:

1. Bootstrap a new version, such as `production-v2`, and grant it access while
   `production` still works.
2. Add the new protected password to the CI provider under a versioned secret
   name.
3. Commit the new bundle and update the workflow to use the new secret.
4. Run a deployment and verify it succeeds.
5. Revoke the old profile's access slots from every lockbox.
6. Remove the old CI secret and bundle.

Versioned secret names avoid a transition where a new bundle is paired with an
old password, or the reverse.

Revocation prevents the old identity from opening updated lockboxes. It cannot
erase secrets or lockbox copies that were already decrypted or copied by a
compromised job.

## Password-Protected Lockboxes

A dedicated, randomly generated lockbox password is also a valid CI design:

```text
lockbox password -> lockbox access
```

The profile-based design uses two inputs:

```text
encrypted CI bundle + vault password -> CI profile -> lockbox access
```

Both inputs are present while the job is running, so control of the job means
control of the available secrets in either design. The profile workflow is most
useful when one CI identity needs several lockboxes, when access should be
granted without exchanging a secret, or when public-key-based rotation is more
convenient.

A future `ci open` may support a password-only mode for the smallest possible
setup. It should use a dedicated random password per trust boundary, not a
human-chosen or organization-wide shared password.

## What reVault Does Not Provide

reVault is a portable encrypted bundle, not an online secret-control plane. It
does not provide dynamic credentials, server-side access policy, or per-read
audit events. See [CI Secret Storage Comparison](ci_secret_storage_comparison.md)
for cases where a managed secret service is a better fit.

The CI provider remains responsible for deciding which jobs receive
`LOCKBOX_VAULT_PASSWORD`. reVault cannot protect secrets from malicious code
that runs inside an authorized job.

## Proposed Acceptance Criteria

The first implementation should be considered complete when:

- bootstrap creates a minimal encrypted bundle without modifying the normal
  user vault;
- bootstrap can grant the CI profile to one or more already-open lockboxes;
- no private key or generated passphrase is printed to redirected output;
- open uses private temporary vault and agent directories;
- open runs a child command and always performs cleanup;
- the child command receives the original exit status and termination signals;
- secret values never appear in normal command output;
- Linux, macOS, and Windows runners are covered by automated tests;
- the generic, GitHub Actions, GitLab CI/CD, rotation, and revocation workflows
  are documented.

[github-secrets]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets
