# reVault for PHP

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Composer package uses PHP FFI and includes the
matching native runtime. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
composer require onepub/revault-api
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```php
$runtime = Revault\Revault::load(); // loading does not open a Vault
$signing = $runtime->generateProfileSigningKeyPair();
$publicSigningKey = $signing->publicKey();
$vault = $runtime;
$box = $vault->lockboxCreate(str_repeat("\0", 32));
$box->setOwnerSigningKey($signing); // Profile becomes this Lockbox's owner
$box->addFile('/hello.txt', "hello\n", false);
$box->setVariable('owner', 'alice');
$box->setSecretVariable('token', 'secret');
$box->withSecretVariable('token', function (FFI\CData $token, int $length) {
    // Consume the bytes only inside this callback.
});
$box->commit();
$box->free();
$publicSigningKey->free(); $signing->free();

$persistent = Revault\Vault::openOrCreate('/tmp/revault-vault', 'Vault passphrase');
$persistent->close();
```

Pass a carrier path to `Revault::load($nativeLibraryPath)` for an
application-owned installation. Otherwise a non-empty inherited
`REVAULT_LIBRARY` is used before the Composer package carrier. A bare library
name delegates to the operating-system search path.

Enable `ext-ffi` in production. Callback memory is cleared after return; PHP
strings are immutable, so do not retain plaintext secret strings.
Use `AgentSession` explicitly for temporary delegated content keys; ordinary
Lockbox operations never contact the agent. `SecretString` and `SecretBytes`
must be closed promptly.
