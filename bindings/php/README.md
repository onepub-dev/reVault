# reVault for PHP

reVault is a fast, local toolkit for creating secure portable archives called
Lockboxes. Each Lockbox is encrypted, compressed, and signed. It can store
files and directory trees, variables such as API keys, and forms such as login
details.

Lockboxes are easy to copy, share, and back up, and they do not require a
hosted service. The engine is designed for speed and effective compression.
Applications can read, write, and seek within stored files without extracting
the archive, and recover data from partial corruption. reVault provides a
command line tool for everyday work and APIs for application code.

Read the [reVault manual](https://docs.revault.onepub.dev/) for the quick start,
core concepts, and security model.

Your Vault holds your profile and contacts. The CLI protects a new Lockbox for
your profile by default, and you can grant access to contacts using their
public keys. Use password access when you do not have a recipient's contact
(public key) details.

`onepub/revault-api` provides typed PHP classes for Lockboxes, the Vault, and
the optional session agent. It uses PHP FFI to load the matching reVault engine,
while callers work with ordinary PHP objects.

The package supports PHP 8.1 or later on Linux, macOS, and Windows, on x86-64
and ARM64. The FFI extension must be installed and enabled for the PHP process
that uses reVault.

## Installation

Install the current package from Packagist:

```console
composer require onepub/revault-api
```

Load the bundled runtime once during application startup:

```php
<?php

require __DIR__ . '/vendor/autoload.php';

use Revault\Revault;

$runtime = Revault::load();
// Vault, Lockbox, and AgentSession are now ready to use.
```

Loading the runtime does not open a Vault or Lockbox and does not start the
session agent.

## Recommended: protect Lockboxes with a Vault profile

Vault initialization creates your default profile. The profile identifies you
and holds the private keys used to open and sign Lockboxes. Add contacts to
grant other people access using their public keys:

```console
cargo install revault_cli
lbx vault init
lbx team-secrets.lbox create \
  --description 'Team deployment credentials'
```

PHP can open the same Lockbox with the default profile's private key stored in
that Vault. The Vault passphrase and key objects remain owned by the caller:

```php
<?php

require __DIR__ . '/vendor/autoload.php';

use Revault\Lockbox;
use Revault\Revault;
use Revault\SecretString;
use Revault\Vault;

$runtime = Revault::load();
$vaultPassphrase = new SecretString($vaultPassphraseFromPrompt);
$vault = Vault::open(
    $runtime->vaultDefaultDirectory(),
    $vaultPassphrase->bytes(),
);
$profileKey = $vault->loadPrivateKey('default');
$lockbox = Lockbox::open('team-secrets.lbox', contact: $profileKey);

try {
    echo $lockbox->description(), PHP_EOL;
    foreach ($lockbox->list('/', true)->entries as $entry) {
        echo $entry->path, PHP_EOL;
    }
} finally {
    $lockbox->close();
    $profileKey->close();
    $vault->close();
    $vaultPassphrase->close();
}
```

Use `Lockbox::create(contact: ..., signingKey: ...)` when creating a
profile protected Lockbox directly from PHP. Obtain the contact public key and
profile signing key from the Vault, then close both after the Lockbox is
created. The [PHP conformance program](../e2e/php/conformance.php) contains the
complete key creation and cleanup sequence.

## Password access when needed

Use password access when you do not have a recipient's contact (public key)
details. Ask the CLI to create a password protected Lockbox explicitly:

```console
lbx shared-secrets.lbox create --password
```

The command prompts for the Lockbox password. Supply that same password to PHP:

```php
$password = new SecretString($passwordFromPrompt);
$lockbox = Lockbox::open('shared-secrets.lbox', password: $password->bytes());

try {
    echo $lockbox->description(), PHP_EOL;
} finally {
    $lockbox->close();
    $password->close();
}
```

`Lockbox::open()` never creates a missing archive. Use `Lockbox::create()` for
a new archive and pass `overwrite: true` only when replacement is intentional.
Likewise, `Vault::open()` opens an existing Vault, `Vault::openOrCreate()` may
create one, and `Vault::replace()` deliberately replaces persistent state.

## Core API concepts

- `Revault` loads and verifies the native runtime shared by the process. It does not
  represent persistent Vault state.
- `Vault` is the encrypted local store for Profiles, private keys, Contacts,
  signing keys, remembered Lockbox credentials, and Lockbox metadata.
- `Lockbox` is a portable encrypted `.lbox` archive containing files,
  variables, secrets, and structured forms.
- `AgentSession` controls the optional session agent process and its temporary
  cache of selected Lockbox content keys.

A vault passphrase, Lockbox password, and Lockbox content key are different
secrets. Method names and application variables should preserve that
distinction.

## Secret values and ownership

`SecretString` and `SecretBytes` own buffers that are overwritten by `close()`.
The PHP string passed to their constructor cannot be erased by the binding, so
do not retain the source string or create unnecessary copies.

Secret values stored in a Lockbox are exposed only during a callback:

```php
$length = $lockbox->withSecretVariable(
    'api-token',
    function (FFI\CData $token, int $length): int {
        // Read token[0] through token[$length - 1] only in this callback.
        return $length;
    },
);
```

The callback memory is valid only during the call and is cleared immediately
after the callback returns. Do not return the `FFI\CData`, retain it in an
object, or convert it to a retained PHP string.

Call `close()` on `Vault`, `Lockbox`, key, and secret objects in `finally`
blocks. Closing a Lockbox releases the content key it holds in this process. It does not
delete the archive and does not remove an independent key cached by the agent.

## Use the optional session agent

Ordinary `Lockbox::open()` calls keep their state in this process and never
start or consult the agent. Use `AgentSession` when Lockbox keys need to be
shared across processes or remain available after the process that opened the
Lockbox exits:

```php
$agent = Revault::agentSession();
$agent->start();
$agent->cacheLockboxPassword('team-secrets.lbox', $password->bytes(), 30 * 60);

$lockbox = $agent->openLockboxPassword(
    'team-secrets.lbox',
    $password->bytes(),
);
try {
    // Work with the Lockbox handle held by this process.
} finally {
    $lockbox->close();
    $agent->closeLockbox('team-secrets.lbox');
    $agent->close();
}
```

The agent caches a temporary content key, not an open file handle. Closing an
agent entry forgets that cached key; it does not delete the Lockbox or a
persistent credential stored in the Vault.

## Platform credential store

The platform credential store can hold the Vault passphrase. The user's
operating system login normally unlocks that store. After login, another
process running as that user may be able to retrieve the passphrase if the
access policy applied to the saved Vault passphrase does not require approval
for each retrieval. Exact access depends on the operating system, the
credential store configuration, and that access policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault can
then provide access to Lockboxes through profile keys or remembered Lockbox
passwords. Both remain encrypted inside the Vault; they are not copied to the
operating system credential store.

Agent expiry and `closeAll()` improve memory hygiene. They are not
authentication boundaries after login if the saved Vault passphrase can be
retrieved without approval.

## Native runtime distribution

The Composer package contains the libraries for the supported operating
systems and architectures under `native/<target>`. PHP selects the library
matching the running PHP process and loads it through the FFI extension.
`Revault::load()` performs this selection once during application startup. An
application that maintains its own copy may provide its path:

```php
$runtime = Revault::load('/opt/my-app/lib/librevault_api.so');
```

Resolution order is an explicit path, a nonempty inherited
`REVAULT_LIBRARY`, and then the matching library in the Composer package. A
bare library name delegates lookup to the operating system. Library selection
is shared by the process and must happen before the first reVault operation.

## API documentation and support

- The public PHP classes and method contracts are in [`src/Vault.php`](src/Vault.php).
- The [PHP conformance program](../e2e/php/conformance.php) demonstrates the
  complete operation inventory, including ownership and failure handling.
- The [cross language example index](../API_EXAMPLES.md) maps each binding to
  its executable examples.
- The [reVault manual](https://docs.revault.onepub.dev/) covers the archive
  format, Vault lifecycle, sharing, recovery, and security model.
- Use the [Packagist package](https://packagist.org/packages/onepub/revault-api)
  for installation metadata and report defects in the
  [reVault issue tracker](https://github.com/onepub-dev/reVault/issues).
