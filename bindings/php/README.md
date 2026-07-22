# reVault for PHP

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Composer package uses PHP FFI and includes the
matching native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
composer require onepub/revault-api:0.2.0
```

```php
$vault = new Revault\Vault();
$box = $vault->lockboxCreate(str_repeat("\0", 32));
$box->addFile('/hello.txt', "hello\n", false);
$box->setVariable('owner', 'alice');
$box->setSecretVariable('token', 'secret');
$box->withSecretVariable('token', function (FFI\CData $token, int $length) {
    // Consume the bytes only inside this callback.
});
$box->commit();
$box->free();
```

Enable `ext-ffi` in production. Callback memory is cleared after return; PHP
strings are immutable, so do not retain plaintext secret strings.
