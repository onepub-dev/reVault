<?php
/** Public file lifecycle and native archive lock conformance. */
declare(strict_types=1);
require_once __DIR__ . '/../../php/src/DomainModels.php';
require_once __DIR__ . '/../../php/src/BindingOperations.php';
require_once __DIR__ . '/../../php/src/Vault.php';
use Revault\Revault;
use Revault\Lockbox;
use Revault\RevaultError;
putenv('LOCKBOX_LOCK_TIMEOUT_MS=60');
$api = Revault::load();
$signer = $api->generateProfileSigningKeyPair();
$key = str_repeat('K', 32);
$check = static function(bool $condition): void { if (!$condition) throw new RuntimeException('file facade assertion failed'); };
$refuses = static function(callable $action): void { try { $action(); } catch (RevaultError $error) { return; } throw new RuntimeException('operation should have been refused'); };
if (($archive = getenv('REVAULT_READONLY_ARCHIVE')) !== false) {
    $box = Lockbox::open($archive, contentKey: $key);
    try { $check($box->getFile('/hello') === 'read only'); } finally { $box->close(); }
} else {
    $root = sys_get_temp_dir() . '/revault-php-' . bin2hex(random_bytes(8));
    mkdir($root, 0700);
    $archive = $root . '/résumé.lbox';
    $create = static fn(bool $overwrite = false) => Lockbox::create($archive, contentKey: $key, signingKey: $signer, overwrite: $overwrite);
    $open = static fn(bool $write = false) => Lockbox::open($archive, contentKey: $key, signingKey: $write ? $signer : null);
    $payload = "hello\0\xff";
    try {
        $box = $create();
        try { $box->addFile('/hello', $payload, false); $box->commit(); $box->commit(); } finally { $box->close(); }
        $reader = $open();
        try {
            $box = $open();
            try { $check($box->getFile('/hello') === $payload); } finally { $box->close(); }
            $refuses(fn() => $reader->setOwnerSigningKey($signer));
            $refuses(fn() => $reader->addFile('/bad', $payload, false));
            $refuses(fn() => $open(true));
            $refuses(fn() => $create(true));
        } finally { $reader->close(); }
        $box = $open(true);
        try { $box->addFile('/hello', 'replacement', true); $box->addFile('/added', $payload, false); $box->commit(); } finally { $box->close(); }
        $box = $open();
        try { $check($box->getFile('/hello') === 'replacement'); $check($box->getFile('/added') === $payload); } finally { $box->close(); }
        $box = $open(true);
        try { $box->delete('/hello'); $box->commit(); } finally { $box->close(); }
        $box = $open();
        try { $check(!$box->exists('/hello')); $check($box->getFile('/added') === $payload); } finally { $box->close(); }
        $refuses(fn() => $create());
        $box = $create(true);
        try { $box->addFile('/new', $payload, false); $box->commit(); } finally { $box->close(); }
        $box = $open();
        try { $check(!$box->exists('/added')); $check($box->getFile('/new') === $payload); } finally { $box->close(); }
        $check(count(scandir($root)) === 3);
        $box = Lockbox::create($archive, password: 'test password', signingKey: $signer, overwrite: true);
        try { $box->addFile('/hello', $payload, false); $box->commit(); } finally { $box->close(); }
        $box = Lockbox::open($archive, password: 'test password');
        try { $check($box->getFile('/hello') === $payload); } finally { $box->close(); }
    } finally { if (file_exists($archive)) unlink($archive); rmdir($root); }
}
$signer->free();
$assertions = getenv('REVAULT_READONLY_ARCHIVE') !== false ? 1 : 18;
echo "PASS\tphp\tlockbox_file\t$assertions\n";
