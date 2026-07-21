# ``RevaultAPI``

Encrypt files, variables, and typed form records in portable reVault lockboxes,
and manage cryptographic keys and local vault metadata.

## Overview

Create a ``Vault`` to access the API, then create or open a ``Lockbox``. Values
that retain sensitive state release it when they are closed or deallocated.
Secret variables and secret form fields are available only through
callback-scoped accessors, reducing the chance that plaintext remains in Swift
memory longer than intended.

The [repository README](https://github.com/onepub-dev/reVault#readme) explains
installation, the security model, and complete workflows.

## Topics

### Entry point

- ``Vault``

### Encrypted content

- ``Lockbox``

### Keys

- ``ContactKeyPair``
- ``ContactPublicKey``
- ``WrappedContactKey``
- ``SigningKeyPair``
- ``SigningPublicKey``

### Local services

- ``VaultDirectory``
- ``ReadOnlyVaultDirectory``
- ``Agent``
- ``AgentActivity``
- ``Platform``
- ``LocalVault``
