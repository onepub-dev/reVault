<?php
declare(strict_types=1);

namespace Revault;

use FFI\CData;

/**
 * Entry point for encrypted lockboxes, cryptographic keys, local vault
 * metadata, the session agent, and the platform secret store.
 *
 * Structured values are concrete protobuf messages. Release owned handles
 * promptly and use callback-scoped secret accessors to avoid retaining
 * plaintext. See the repository README for installation and examples:
 * https://github.com/onepub-dev/reVault#readme
 */
final class Vault
{
    private readonly BindingOperations $operations;
    private readonly Agent $agent;
    private readonly Platform $platform;

    public function __construct(?string $library = null)
    {
        $this->operations = BindingOperations::load($library ?? self::nativeLibrary());
        $this->agent = new Agent($this->operations); $this->platform = new Platform($this->operations);
    }

    private static function nativeLibrary(): string
    {
        $override = getenv('REVAULT_LIBRARY');
        if ($override !== false && $override !== '') return $override;
        $arch = strtolower(php_uname('m'));
        $cpu = match ($arch) {
            'x86_64', 'amd64' => 'x86_64',
            'aarch64', 'arm64' => 'aarch64',
            default => throw new \RuntimeException("unsupported reVault architecture: $arch"),
        };
        [$target, $file] = match (PHP_OS_FAMILY) {
            'Windows' => ["windows-$cpu-msvc", 'revault_api.dll'],
            'Darwin' => ["macos-$cpu", 'librevault_api.dylib'],
            'Linux' => ["linux-$cpu-gnu", 'librevault_api.so'],
            default => throw new \RuntimeException('unsupported reVault operating system: '.PHP_OS_FAMILY),
        };
        $bundled = dirname(__DIR__)."/native/$target/$file";
        if (!is_file($bundled)) { throw new \RuntimeException("revault-api native carrier is missing for $target; set REVAULT_LIBRARY for development"); }
        return $bundled;
    }

    public function agent(): Agent { return $this->agent; }
    public function platform(): Platform { return $this->platform; }
    public function lastError(): string { return $this->operations->lastErrorMessage(); }
    public function lastErrorDetails(): object { return $this->operations->bufferLastErrorDetails(); }

    public function lockboxFormatVersion(): int
    {
        return $this->operations->lockboxFormatVersion();
    }

    public function lockboxProbeFormatVersion(string $bytes): int
    {
        return $this->operations->lockboxProbeFormatVersion($bytes);
    }

    public function lockboxCreate(string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreate($key));
    }

    public function lockboxCreateWithOptions(string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithOptions($key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    public function lockboxCreatePassword(string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreatePassword($password));
    }

    public function lockboxCreateContact(OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateContact($contact->nativeHandle()));
    }

    public function lockboxCreateWithSigningKey(string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithSigningKey($contentKey, $signingKey->nativeHandle()));
    }

    public function lockboxOpen(string $archive, string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpen($archive, $key));
    }

    public function lockboxOpenWithOptions(string $archive, string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenWithOptions($archive, $key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    public function lockboxOpenPassword(string $archive, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenPassword($archive, $password));
    }

    public function lockboxOpenContact(string $archive, OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenContact($archive, $contact->nativeHandle()));
    }

    public function lockboxInspectFile(string $path): \Revault\Bindings\FileInspection
    {
        return $this->operations->lockboxInspectFile($path);
    }

    public function lockboxRecoveryScanPath(string $path, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScanPath($path, $key);
    }

    public function lockboxRecoveryScan(string $bytes, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScan($bytes, $key);
    }

    public function lockboxRecoverySalvage(string $bytes, string $key, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxRecoverySalvage($bytes, $key, $signingKey->nativeHandle()));
    }

    public function keyContactGenerate(): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactGenerate());
    }

    public function keyContactFromPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactFromPrivate($bytes));
    }

    public function keyContactPublicFromBytes(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->keyContactPublicFromBytes($bytes));
    }

    public function keySigningGenerate(): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->keySigningGenerate());
    }

    public function keySigningFromPrivate(string $bytes): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->keySigningFromPrivate($bytes));
    }

    public function keySigningPublicFromBytes(string $bytes): SigningPublicKey
    {
        return new SigningPublicKey($this->operations, $this->operations->keySigningPublicFromBytes($bytes));
    }

    public function vaultKeyExportPrivate(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPrivate($key->nativeHandle(), $format);
    }

    public function vaultKeyExportPublic(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPublic($key->nativeHandle(), $format);
    }

    public function vaultKeyImportPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultKeyImportPrivate($bytes));
    }

    public function vaultKeyImportPublic(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->vaultKeyImportPublic($bytes));
    }

    public function vaultKeyFingerprint(OwnedHandle $key): string
    {
        return $this->operations->vaultKeyFingerprint($key->nativeHandle());
    }

    public function vaultKeyFormatHex(string $bytes): string
    {
        return $this->operations->vaultKeyFormatHex($bytes);
    }

    public function vaultKeyDecodeHex(string $text): string
    {
        return $this->operations->vaultKeyDecodeHex($text);
    }

    public function vaultKeyFormatCrockford(string $bytes): string
    {
        return $this->operations->vaultKeyFormatCrockford($bytes);
    }

    public function vaultKeyFormatCrockfordReading(string $code): string
    {
        return $this->operations->vaultKeyFormatCrockfordReading($code);
    }

    public function vaultKeyDecodeCrockford(string $code): string
    {
        return $this->operations->vaultKeyDecodeCrockford($code);
    }

    public function vaultKeyHexEncode(string $bytes): string
    {
        return $this->operations->vaultKeyHexEncode($bytes);
    }

    public function vaultKeyHexDecode(string $text): string
    {
        return $this->operations->vaultKeyHexDecode($text);
    }

    public function vaultDirectoryOpen(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpen($root, $password));
    }

    public function vaultStructureVersionCurrent(): int
    {
        return $this->operations->vaultStructureVersionCurrent();
    }

    public function vaultDirectoryProbeStructureVersion(string $root, string $password): int
    {
        return $this->operations->vaultDirectoryProbeStructureVersion($root, $password);
    }

    public function vaultDirectoryOpenOrCreateDefault(string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpenOrCreateDefault($password));
    }

    public function vaultDirectoryReplaceDefault(string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryReplaceDefault($password));
    }

    public function vaultDirectoryChangePassword(string $root, string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangePassword($root, $oldPassword, $newPassword);
    }

    public function vaultDirectoryChangeDefaultPassword(string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangeDefaultPassword($oldPassword, $newPassword);
    }

    public function vaultDirectoryReplace(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryReplace($root, $password));
    }

    public function vaultDirectoryOpenOrCreate(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpenOrCreate($root, $password));
    }

    public function vaultBackupDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->operations->vaultBackupDefault($path, $overwrite);
    }

    public function vaultRestoreDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->operations->vaultRestoreDefault($path, $overwrite);
    }

    public function vaultReadOnlyOpen(string $root, string $password): ReadOnlyVaultDirectory
    {
        return new ReadOnlyVaultDirectory($this->operations, $this->operations->vaultReadOnlyOpen($root, $password));
    }

    public function vaultReadOnlyOpenDefault(string $password): ReadOnlyVaultDirectory
    {
        return new ReadOnlyVaultDirectory($this->operations, $this->operations->vaultReadOnlyOpenDefault($password));
    }

    public function vaultDefaultDirectory(): string
    {
        return $this->operations->vaultDefaultDirectory();
    }

    public function vaultDefaultPath(): string
    {
        return $this->operations->vaultDefaultPath();
    }

    public function vaultAgentLogPath(): string
    {
        return $this->operations->vaultAgentLogPath();
    }

    public function vaultAgentLogDestination(): string
    {
        return $this->operations->vaultAgentLogDestination();
    }

    public function vaultLocal(): LocalVault
    {
        return new LocalVault($this->operations, $this->operations->vaultLocal());
    }

}

abstract class OwnedHandle
{
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}
    final public function nativeHandle(): CData { return $this->handle; }
}

/** Owned, mutable view of one encrypted lockbox archive. */
class Lockbox extends OwnedHandle
{

    public function addFile(string $path, string $data, bool $replace): bool
    {
        return $this->operations->lockboxAddFile($this->handle, $path, $data, $replace);
    }

    public function addFileWithPermissions(string $path, string $data, int $permissions, bool $replace): bool
    {
        return $this->operations->lockboxAddFileWithPermissions($this->handle, $path, $data, $permissions, $replace);
    }

    public function getFile(string $path): string
    {
        return $this->operations->lockboxGetFile($this->handle, $path);
    }

    public function extractFile(string $source, string $destination, bool $replace): bool
    {
        return $this->operations->lockboxExtractFile($this->handle, $source, $destination, $replace);
    }

    public function extractDirectory(string $destination, int $maxFileBytes, int $maxTotalBytes, int $maxFiles, bool $restoreSymlinks, bool $restorePermissions, bool $overwrite): bool
    {
        return $this->operations->lockboxExtractDirectory($this->handle, $destination, $maxFileBytes, $maxTotalBytes, $maxFiles, $restoreSymlinks, $restorePermissions, $overwrite);
    }

    public function streamContent(bool $physical): \Revault\Bindings\StreamChunkList
    {
        return $this->operations->lockboxStreamContent($this->handle, $physical);
    }

    public function cacheStats(): \Revault\Bindings\CacheStats
    {
        return $this->operations->lockboxCacheStats($this->handle);
    }

    public function importStats(): \Revault\Bindings\ImportStats
    {
        return $this->operations->lockboxImportStats($this->handle);
    }

    public function resetImportStats(): bool
    {
        return $this->operations->lockboxResetImportStats($this->handle);
    }

    public function pageInspection(): \Revault\Bindings\PageInspectionList
    {
        return $this->operations->lockboxPageInspection($this->handle);
    }

    public function recoveryReport(): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryReport($this->handle);
    }

    public function recoveryReportRender(bool $verbose, int $maxEntries): string
    {
        return $this->operations->lockboxRecoveryReportRender($this->handle, $verbose, $maxEntries);
    }

    public function storageLen(): int
    {
        return $this->operations->lockboxStorageLen($this->handle);
    }

    public function setWorkloadProfile(string $profile): bool
    {
        return $this->operations->lockboxSetWorkloadProfile($this->handle, $profile);
    }

    public function setWorkerPolicy(string $mode, int $jobs): bool
    {
        return $this->operations->lockboxSetWorkerPolicy($this->handle, $mode, $jobs);
    }

    public function runtimeOptions(): \Revault\Bindings\RuntimeOptions
    {
        return $this->operations->lockboxRuntimeOptions($this->handle);
    }

    public function commit(): bool
    {
        return $this->operations->lockboxCommit($this->handle);
    }

    public function createDir(string $path, bool $createParents): bool
    {
        return $this->operations->lockboxCreateDir($this->handle, $path, $createParents);
    }

    public function delete(string $path): bool
    {
        return $this->operations->lockboxDelete($this->handle, $path);
    }

    public function removeDir(string $path, bool $recursive): bool
    {
        return $this->operations->lockboxRemoveDir($this->handle, $path, $recursive);
    }

    public function createParentDirs(string $path): bool
    {
        return $this->operations->lockboxCreateParentDirs($this->handle, $path);
    }

    public function rename(string $from, string $to): bool
    {
        return $this->operations->lockboxRename($this->handle, $from, $to);
    }

    public function list(string $path, bool $recursive): \Revault\Bindings\LockboxEntryList
    {
        return $this->operations->lockboxList($this->handle, $path, $recursive);
    }

    public function listWithOptions(string $path, string $glob, bool $recursive, bool $includeFiles, bool $includeSymlinks, bool $includeDirectories, int $limit): \Revault\Bindings\LockboxEntryList
    {
        return $this->operations->lockboxListWithOptions($this->handle, $path, $glob, $recursive, $includeFiles, $includeSymlinks, $includeDirectories, $limit);
    }

    public function stat(string $path): \Revault\Bindings\OptionalLockboxEntry
    {
        return $this->operations->lockboxStat($this->handle, $path);
    }

    public function setVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetVariable($this->handle, $name, $value);
    }

    /** Stores a secret variable from binary-safe PHP string bytes. */
    public function setSecretVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetSecretVariable($this->handle, $name, $value);
    }

    public function getVariable(string $name): ?string
    {
        $value = $this->operations->lockboxGetVariable($this->handle, $name);
        return $value->getPresent() ? $value->getValue() : null;
    }

    /** Invokes the callback with temporary secret bytes, then wipes the native transfer. */
    public function withSecretVariable(string $name, callable $callback): mixed
    {
        return $this->operations->lockboxWithSecretVariable($this->handle, $name, $callback);
    }

    public function deleteVariable(string $name): bool
    {
        return $this->operations->lockboxDeleteVariable($this->handle, $name);
    }

    public function moveVariables(string $movesProto): bool
    {
        return $this->operations->lockboxMoveVariables($this->handle, $movesProto);
    }

    public function listVariables(): \Revault\Bindings\VariableList
    {
        return $this->operations->lockboxListVariables($this->handle);
    }

    public function variableSensitivity(string $name): \Revault\Bindings\OptionalString
    {
        return $this->operations->lockboxVariableSensitivity($this->handle, $name);
    }

    public function addSymlink(string $path, string $target, bool $replace): bool
    {
        return $this->operations->lockboxAddSymlink($this->handle, $path, $target, $replace);
    }

    public function getSymlinkTarget(string $path): string
    {
        return $this->operations->lockboxGetSymlinkTarget($this->handle, $path);
    }

    public function id(): string
    {
        return $this->operations->lockboxId($this->handle);
    }

    public function exists(string $path): bool
    {
        return $this->operations->lockboxExists($this->handle, $path);
    }

    public function isDir(string $path): bool
    {
        return $this->operations->lockboxIsDir($this->handle, $path);
    }

    public function permissions(string $path): int
    {
        return $this->operations->lockboxPermissions($this->handle, $path);
    }

    public function setPermissions(string $path, int $permissions): bool
    {
        return $this->operations->lockboxSetPermissions($this->handle, $path, $permissions);
    }

    public function readRange(string $path, int $offset, int $len): string
    {
        return $this->operations->lockboxReadRange($this->handle, $path, $offset, $len);
    }

    public function addPassword(string $password): int
    {
        return $this->operations->lockboxAddPassword($this->handle, $password);
    }

    public function addContact(OwnedHandle $contact, string $name): int
    {
        return $this->operations->lockboxAddContact($this->handle, $contact->nativeHandle(), $name);
    }

    public function deleteKey(int $id): bool
    {
        return $this->operations->lockboxDeleteKey($this->handle, $id);
    }

    public function listKeySlots(): \Revault\Bindings\KeySlotList
    {
        return $this->operations->lockboxListKeySlots($this->handle);
    }

    public function setOwnerSigningKey(OwnedHandle $key): bool
    {
        return $this->operations->lockboxSetOwnerSigningKey($this->handle, $key->nativeHandle());
    }

    public function ownerInspection(): \Revault\Bindings\OwnerInspection
    {
        return $this->operations->lockboxOwnerInspection($this->handle);
    }

    public function defineForm(string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->operations->lockboxDefineForm($this->handle, $alias, $name, $description, $fieldsProto);
    }

    public function listFormDefinitions(): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->lockboxListFormDefinitions($this->handle);
    }

    public function resolveForm(string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->operations->lockboxResolveForm($this->handle, $reference);
    }

    public function listFormRevisions(string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->lockboxListFormRevisions($this->handle, $typeId);
    }

    public function createFormRecord(string $path, string $typeReference, string $name): \Revault\Bindings\FormRecord
    {
        return $this->operations->lockboxCreateFormRecord($this->handle, $path, $typeReference, $name);
    }

    public function setFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetFormField($this->handle, $path, $field, $value);
    }

    /** Stores a secret form field from binary-safe PHP string bytes. */
    public function setSecretFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetSecretFormField($this->handle, $path, $field, $value);
    }

    public function listFormRecords(): \Revault\Bindings\FormRecordList
    {
        return $this->operations->lockboxListFormRecords($this->handle);
    }

    public function getFormRecord(string $path): \Revault\Bindings\OptionalFormRecord
    {
        return $this->operations->lockboxGetFormRecord($this->handle, $path);
    }

    public function deleteFormRecord(string $path): bool
    {
        return $this->operations->lockboxDeleteFormRecord($this->handle, $path);
    }

    public function moveFormRecords(string $movesProto): bool
    {
        return $this->operations->lockboxMoveFormRecords($this->handle, $movesProto);
    }

    public function getFormField(string $path, string $field): \Revault\Bindings\OptionalFormValue
    {
        return $this->operations->lockboxGetFormField($this->handle, $path, $field);
    }

    /** Invokes the callback with temporary field bytes, then wipes the native transfer. */
    public function withSecretFormField(string $path, string $field, callable $callback): mixed
    {
        return $this->operations->lockboxWithSecretFormField($this->handle, $path, $field, $callback);
    }

    public function toBytes(): string
    {
        return $this->operations->lockboxToBytes($this->handle);
    }

    public function free(): void
    {
        $this->operations->lockboxFree($this->handle);
    }

}

/** Owned contact key pair used to decrypt content keys sent by contacts. */
class ContactKeyPair extends OwnedHandle
{

    public function public(): string
    {
        return $this->operations->keyContactPublic($this->handle);
    }

    public function private(): string
    {
        return $this->operations->keyContactPrivate($this->handle);
    }

    public function free(): void
    {
        $this->operations->keyContactFree($this->handle);
    }

    public function decrypt(OwnedHandle $wrapped): string
    {
        return $this->operations->keyContactDecrypt($this->handle, $wrapped->nativeHandle());
    }

}

/** Shareable contact public key used to encrypt a recipient content key. */
class ContactPublicKey extends OwnedHandle
{

    public function publicFree(): void
    {
        $this->operations->keyContactPublicFree($this->handle);
    }

    public function encrypt(string $contentKey): WrappedContactKey
    {
        return new WrappedContactKey($this->operations, $this->operations->keyContactEncrypt($this->handle, $contentKey));
    }

}

/** Owned encrypted content-key envelope for one contact recipient. */
class WrappedContactKey extends OwnedHandle
{

    public function public(): string
    {
        return $this->operations->keyContactWrappedPublic($this->handle);
    }

    public function ciphertext(): string
    {
        return $this->operations->keyContactWrappedCiphertext($this->handle);
    }

    public function encrypted(): string
    {
        return $this->operations->keyContactWrappedEncrypted($this->handle);
    }

    public function free(): void
    {
        $this->operations->keyContactWrappedFree($this->handle);
    }

}

/** Owned signing key pair used to authorize mutable lockbox commits. */
class SigningKeyPair extends OwnedHandle
{

    public function public(): string
    {
        return $this->operations->keySigningPublic($this->handle);
    }

    public function private(): string
    {
        return $this->operations->keySigningPrivate($this->handle);
    }

    public function free(): void
    {
        $this->operations->keySigningFree($this->handle);
    }

}

/** Public key used to verify owner-authorized lockbox commits. */
class SigningPublicKey extends OwnedHandle
{

    public function publicFree(): void
    {
        $this->operations->keySigningPublicFree($this->handle);
    }

}

/** Writable, password-protected local metadata vault. */
class VaultDirectory extends OwnedHandle
{

    public function root(): string
    {
        return $this->operations->vaultDirectoryRoot($this->handle);
    }

    public function structureVersion(): int
    {
        return $this->operations->vaultDirectoryStructureVersion($this->handle);
    }

    public function listPrivateKeys(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeys($this->handle);
    }

    public function listPrivateKeyNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeyNames($this->handle);
    }

    public function listContactNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListContactNames($this->handle);
    }

    public function listFormAliases(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListFormAliases($this->handle);
    }

    public function privateKeyExists(string $name): bool
    {
        return $this->operations->vaultDirectoryPrivateKeyExists($this->handle, $name);
    }

    public function deletePrivateKey(string $name): bool
    {
        return $this->operations->vaultDirectoryDeletePrivateKey($this->handle, $name);
    }

    public function storePrivateKey(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStorePrivateKey($this->handle, $name, $key->nativeHandle());
    }

    public function loadPrivateKey(string $name): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultDirectoryLoadPrivateKey($this->handle, $name));
    }

    public function loadPrivateKeyGeneration(string $name, int $index): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultDirectoryLoadPrivateKeyGeneration($this->handle, $name, $index));
    }

    public function storeContact(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStoreContact($this->handle, $name, $key->nativeHandle());
    }

    public function loadContact(string $name): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->vaultDirectoryLoadContact($this->handle, $name));
    }

    public function contactExists(string $name): bool
    {
        return $this->operations->vaultDirectoryContactExists($this->handle, $name);
    }

    public function deleteContact(string $name): bool
    {
        return $this->operations->vaultDirectoryDeleteContact($this->handle, $name);
    }

    public function listContacts(): \Revault\Bindings\ContactList
    {
        return $this->operations->vaultDirectoryListContacts($this->handle);
    }

    public function storeProfileEmail(string $name, string $email): bool
    {
        return $this->operations->vaultDirectoryStoreProfileEmail($this->handle, $name, $email);
    }

    public function profileEmail(string $name): \Revault\Bindings\OptionalString
    {
        return $this->operations->vaultDirectoryProfileEmail($this->handle, $name);
    }

    public function storeBackup(string $id, string $bytes): bool
    {
        return $this->operations->vaultDirectoryStoreBackup($this->handle, $id, $bytes);
    }

    public function loadBackup(string $id): string
    {
        return $this->operations->vaultDirectoryLoadBackup($this->handle, $id);
    }

    public function backupCount(): int
    {
        return $this->operations->vaultDirectoryBackupCount($this->handle);
    }

    public function restorePrivateKey(string $name, OwnedHandle $key, OwnedHandle $signingKey, bool $overwrite): bool
    {
        return $this->operations->vaultDirectoryRestorePrivateKey($this->handle, $name, $key->nativeHandle(), $signingKey->nativeHandle(), $overwrite);
    }

    public function loadOwnerSigningKey(string $name): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKey($this->handle, $name));
    }

    public function loadOwnerSigningKeyGeneration(string $name, int $index): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKeyGeneration($this->handle, $name, $index));
    }

    public function storeContactSigningKey(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStoreContactSigningKey($this->handle, $name, $key->nativeHandle());
    }

    public function loadContactSigningKey(string $name): SigningPublicKey
    {
        return new SigningPublicKey($this->operations, $this->operations->vaultDirectoryLoadContactSigningKey($this->handle, $name));
    }

    public function listProfileGenerations(string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->operations->vaultDirectoryListProfileGenerations($this->handle, $name);
    }

    public function rotatePrivateKey(string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->operations->vaultDirectoryRotatePrivateKey($this->handle, $name);
    }

    public function rememberLockbox(string $id, string $path): bool
    {
        return $this->operations->vaultDirectoryRememberLockbox($this->handle, $id, $path);
    }

    public function listKnownLockboxes(): \Revault\Bindings\KnownLockboxList
    {
        return $this->operations->vaultDirectoryListKnownLockboxes($this->handle);
    }

    public function forgetLockbox(string $path): bool
    {
        return $this->operations->vaultDirectoryForgetLockbox($this->handle, $path);
    }

    public function rememberAccessSlotLabel(string $id, int $slotId, string $name): bool
    {
        return $this->operations->vaultDirectoryRememberAccessSlotLabel($this->handle, $id, $slotId, $name);
    }

    public function listAccessSlotLabels(string $id): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryListAccessSlotLabels($this->handle, $id);
    }

    public function findAccessSlotLabels(string $id, string $name): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryFindAccessSlotLabels($this->handle, $id, $name);
    }

    public function forgetAccessSlotLabel(string $id, int $slotId): bool
    {
        return $this->operations->vaultDirectoryForgetAccessSlotLabel($this->handle, $id, $slotId);
    }

    public function defineForm(string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->operations->vaultDirectoryDefineForm($this->handle, $alias, $name, $description, $fieldsProto);
    }

    public function resolveForm(string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->operations->vaultDirectoryResolveForm($this->handle, $reference);
    }

    public function listForms(): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListForms($this->handle);
    }

    public function listFormRevisions(string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListFormRevisions($this->handle, $typeId);
    }

    public function seedForms(): int
    {
        return $this->operations->vaultDirectorySeedForms($this->handle);
    }

    public function rememberPassword(string $id, string $password): bool
    {
        return $this->operations->vaultDirectoryRememberPassword($this->handle, $id, $password);
    }

    public function rememberedPassword(string $id): string
    {
        return $this->operations->vaultDirectoryRememberedPassword($this->handle, $id);
    }

    public function free(): void
    {
        $this->operations->vaultDirectoryFree($this->handle);
    }

}

/** Read-only metadata view that never loads an owner signing key. */
class ReadOnlyVaultDirectory
{
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}

    public function listProfileNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListProfileNames($this->handle);
    }

    public function listContactNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListContactNames($this->handle);
    }

    public function listFormAliases(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListFormAliases($this->handle);
    }

    public function listKnownLockboxes(): \Revault\Bindings\KnownLockboxList
    {
        return $this->operations->vaultReadOnlyListKnownLockboxes($this->handle);
    }

    public function free(): void
    {
        $this->operations->vaultReadOnlyFree($this->handle);
    }

}

/** Client for the local session agent's time-limited secret cache. */
class Agent
{
    public function __construct(protected readonly BindingOperations $operations) {}

    public function isRunning(): bool
    {
        return $this->operations->vaultIsRunning();
    }

    public function forgetAll(): bool
    {
        return $this->operations->vaultForgetAll();
    }

    public function serve(): bool
    {
        return $this->operations->vaultAgentServe();
    }

    public function verifyTransport(): bool
    {
        return $this->operations->vaultAgentVerifyTransport();
    }

    public function get(string $id): string
    {
        return $this->operations->vaultAgentGet($id);
    }

    public function put(string $id, string $key): bool
    {
        return $this->operations->vaultAgentPut($id, $key);
    }

    public function forget(string $id): bool
    {
        return $this->operations->vaultAgentForget($id);
    }

    public function stop(): bool
    {
        return $this->operations->vaultAgentStop();
    }

    public function start(): bool
    {
        return $this->operations->vaultAgentStart();
    }

    public function list(): \Revault\Bindings\AgentEntryList
    {
        return $this->operations->vaultAgentList();
    }

    public function sleepSupport(): \Revault\Bindings\SleepSupport
    {
        return $this->operations->vaultAgentSleepSupport();
    }

    public function getVaultUnlockKey(string $vaultId): string
    {
        return $this->operations->vaultAgentGetVaultUnlockKey($vaultId);
    }

    public function putVaultUnlockKey(string $vaultId, string $key, int $ttlSeconds): bool
    {
        return $this->operations->vaultAgentPutVaultUnlockKey($vaultId, $key, $ttlSeconds);
    }

    public function forgetVaultUnlockKey(string $vaultId): bool
    {
        return $this->operations->vaultAgentForgetVaultUnlockKey($vaultId);
    }

    public function getOwnerSigningKey(string $vaultId, string $profile): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultAgentGetOwnerSigningKey($vaultId, $profile));
    }

    public function putOwnerSigningKey(string $vaultId, string $profile, OwnedHandle $key, int $ttlSeconds): bool
    {
        return $this->operations->vaultAgentPutOwnerSigningKey($vaultId, $profile, $key->nativeHandle(), $ttlSeconds);
    }

    public function forgetOwnerSigningKey(string $vaultId, string $profile): bool
    {
        return $this->operations->vaultAgentForgetOwnerSigningKey($vaultId, $profile);
    }

    public function beginActivity(string $kind): AgentActivity
    {
        return new AgentActivity($this->operations, $this->operations->vaultAgentBeginActivity($kind));
    }

    public function endActivity(OwnedHandle $handle): void
    {
        $this->operations->vaultAgentEndActivity($handle->nativeHandle());
    }

}

/** Owned registration for an operation that currently requires secret access. */
class AgentActivity extends OwnedHandle
{

}

/** Controls integration with the operating system's secret store. */
class Platform
{
    public function __construct(protected readonly BindingOperations $operations) {}

    public function status(): \Revault\Bindings\PlatformStatus
    {
        return $this->operations->vaultPlatformStatus();
    }

    public function setScope(string $scope): bool
    {
        return $this->operations->vaultPlatformSetScope($scope);
    }

    public function forgetPassword(): bool
    {
        return $this->operations->vaultPlatformForgetPassword();
    }

    public function putPassword(string $password): bool
    {
        return $this->operations->vaultPlatformPutPassword($password);
    }

    public function enable(): bool
    {
        return $this->operations->vaultPlatformEnable();
    }

    public function disable(): bool
    {
        return $this->operations->vaultPlatformDisable();
    }

    public function disabled(): bool
    {
        return $this->operations->vaultPlatformDisabled();
    }

    public function getPassword(): string
    {
        return $this->operations->vaultPlatformGetPassword();
    }

}

/** High-level workflow for local metadata and remembered lockboxes. */
class LocalVault extends OwnedHandle
{

    public function createLockboxPassword(string $path, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxPassword($this->handle, $path, $password));
    }

    public function openLockboxPassword(string $path, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultOpenLockboxPassword($this->handle, $path, $password));
    }

    public function createLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContentKey($this->handle, $path, $contentKey, $signingKey->nativeHandle()));
    }

    public function createLockboxContact(string $path, OwnedHandle $contact, string $name, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContact($this->handle, $path, $contact->nativeHandle(), $name, $signingKey->nativeHandle()));
    }

    public function openLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultOpenLockboxContentKey($this->handle, $path, $contentKey, $signingKey->nativeHandle()));
    }

    public function cacheLockboxPassword(string $path, string $password, int $ttlSeconds): bool
    {
        return $this->operations->vaultCacheLockboxPassword($this->handle, $path, $password, $ttlSeconds);
    }

    public function closeLockbox(string $path): bool
    {
        return $this->operations->vaultCloseLockbox($this->handle, $path);
    }

    public function closeAll(): bool
    {
        return $this->operations->vaultCloseAll($this->handle);
    }

    public function free(): void
    {
        $this->operations->vaultFree($this->handle);
    }

}
