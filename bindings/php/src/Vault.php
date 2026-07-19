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

    /** Returns the construct. */
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

    /** Returns the agent. */
    public function agent(): Agent { return $this->agent; }
    /** Returns the platform. */
    public function platform(): Platform { return $this->platform; }
    /** Returns the last error. */
    public function lastError(): string { return $this->operations->lastErrorMessage(); }
    /** Returns the last error details. */
    public function lastErrorDetails(): object { return $this->operations->bufferLastErrorDetails(); }

    /** Returns the lockbox format version. */
    public function lockboxFormatVersion(): int
    {
        return $this->operations->lockboxFormatVersion();
    }

    /** Returns the lockbox probe format version. */
    public function lockboxProbeFormatVersion(string $bytes): int
    {
        return $this->operations->lockboxProbeFormatVersion($bytes);
    }

    /** Returns the lockbox create. */
    public function lockboxCreate(string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreate($key));
    }

    /** Creates a lockbox with explicit cache capacity, workload, worker policy, and job count. */
    public function lockboxCreateWithOptions(string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithOptions($key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    /** Returns the lockbox create password. */
    public function lockboxCreatePassword(string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreatePassword($password));
    }

    /** Returns the lockbox create contact. */
    public function lockboxCreateContact(OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateContact($contact->nativeHandle()));
    }

    /** Returns the lockbox create with signing key. */
    public function lockboxCreateWithSigningKey(string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithSigningKey($contentKey, $signingKey->nativeHandle()));
    }

    /** Returns the lockbox open. */
    public function lockboxOpen(string $archive, string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpen($archive, $key));
    }

    /** Opens a lockbox with explicit cache capacity, workload, worker policy, and job count. */
    public function lockboxOpenWithOptions(string $archive, string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenWithOptions($archive, $key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    /** Returns the lockbox open password. */
    public function lockboxOpenPassword(string $archive, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenPassword($archive, $password));
    }

    /** Returns the lockbox open contact. */
    public function lockboxOpenContact(string $archive, OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenContact($archive, $contact->nativeHandle()));
    }

    /** Returns the lockbox inspect file. */
    public function lockboxInspectFile(string $path): \Revault\Bindings\FileInspection
    {
        return $this->operations->lockboxInspectFile($path);
    }

    /** Returns the lockbox recovery scan path. */
    public function lockboxRecoveryScanPath(string $path, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScanPath($path, $key);
    }

    /** Returns the lockbox recovery scan. */
    public function lockboxRecoveryScan(string $bytes, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScan($bytes, $key);
    }

    /** Returns the lockbox recovery salvage. */
    public function lockboxRecoverySalvage(string $bytes, string $key, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxRecoverySalvage($bytes, $key, $signingKey->nativeHandle()));
    }

    /** Returns the key contact generate. */
    public function keyContactGenerate(): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactGenerate());
    }

    /** Returns the key contact from private. */
    public function keyContactFromPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactFromPrivate($bytes));
    }

    /** Returns the key contact public from bytes. */
    public function keyContactPublicFromBytes(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->keyContactPublicFromBytes($bytes));
    }

    /** Returns the key signing generate. */
    public function keySigningGenerate(): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->keySigningGenerate());
    }

    /** Returns the key signing from private. */
    public function keySigningFromPrivate(string $bytes): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->keySigningFromPrivate($bytes));
    }

    /** Returns the key signing public from bytes. */
    public function keySigningPublicFromBytes(string $bytes): SigningPublicKey
    {
        return new SigningPublicKey($this->operations, $this->operations->keySigningPublicFromBytes($bytes));
    }

    /** Returns the vault key export private. */
    public function vaultKeyExportPrivate(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPrivate($key->nativeHandle(), $format);
    }

    /** Returns the vault key export public. */
    public function vaultKeyExportPublic(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPublic($key->nativeHandle(), $format);
    }

    /** Returns the vault key import private. */
    public function vaultKeyImportPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultKeyImportPrivate($bytes));
    }

    /** Returns the vault key import public. */
    public function vaultKeyImportPublic(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->vaultKeyImportPublic($bytes));
    }

    /** Returns the vault key fingerprint. */
    public function vaultKeyFingerprint(OwnedHandle $key): string
    {
        return $this->operations->vaultKeyFingerprint($key->nativeHandle());
    }

    /** Returns the vault key format hex. */
    public function vaultKeyFormatHex(string $bytes): string
    {
        return $this->operations->vaultKeyFormatHex($bytes);
    }

    /** Returns the vault key decode hex. */
    public function vaultKeyDecodeHex(string $text): string
    {
        return $this->operations->vaultKeyDecodeHex($text);
    }

    /** Returns the vault key format crockford. */
    public function vaultKeyFormatCrockford(string $bytes): string
    {
        return $this->operations->vaultKeyFormatCrockford($bytes);
    }

    /** Returns the vault key format crockford reading. */
    public function vaultKeyFormatCrockfordReading(string $code): string
    {
        return $this->operations->vaultKeyFormatCrockfordReading($code);
    }

    /** Returns the vault key decode crockford. */
    public function vaultKeyDecodeCrockford(string $code): string
    {
        return $this->operations->vaultKeyDecodeCrockford($code);
    }

    /** Returns the vault key hex encode. */
    public function vaultKeyHexEncode(string $bytes): string
    {
        return $this->operations->vaultKeyHexEncode($bytes);
    }

    /** Returns the vault key hex decode. */
    public function vaultKeyHexDecode(string $text): string
    {
        return $this->operations->vaultKeyHexDecode($text);
    }

    /** Returns the vault directory open. */
    public function vaultDirectoryOpen(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpen($root, $password));
    }

    /** Returns the vault structure version current. */
    public function vaultStructureVersionCurrent(): int
    {
        return $this->operations->vaultStructureVersionCurrent();
    }

    /** Returns the vault directory probe structure version. */
    public function vaultDirectoryProbeStructureVersion(string $root, string $password): int
    {
        return $this->operations->vaultDirectoryProbeStructureVersion($root, $password);
    }

    /** Returns the vault directory open or create default. */
    public function vaultDirectoryOpenOrCreateDefault(string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpenOrCreateDefault($password));
    }

    /** Returns the vault directory replace default. */
    public function vaultDirectoryReplaceDefault(string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryReplaceDefault($password));
    }

    /** Returns the vault directory change password. */
    public function vaultDirectoryChangePassword(string $root, string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangePassword($root, $oldPassword, $newPassword);
    }

    /** Returns the vault directory change default password. */
    public function vaultDirectoryChangeDefaultPassword(string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangeDefaultPassword($oldPassword, $newPassword);
    }

    /** Returns the vault directory replace. */
    public function vaultDirectoryReplace(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryReplace($root, $password));
    }

    /** Returns the vault directory open or create. */
    public function vaultDirectoryOpenOrCreate(string $root, string $password): VaultDirectory
    {
        return new VaultDirectory($this->operations, $this->operations->vaultDirectoryOpenOrCreate($root, $password));
    }

    /** Returns the vault backup default. */
    public function vaultBackupDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->operations->vaultBackupDefault($path, $overwrite);
    }

    /** Returns the vault restore default. */
    public function vaultRestoreDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->operations->vaultRestoreDefault($path, $overwrite);
    }

    /** Returns the vault read only open. */
    public function vaultReadOnlyOpen(string $root, string $password): ReadOnlyVaultDirectory
    {
        return new ReadOnlyVaultDirectory($this->operations, $this->operations->vaultReadOnlyOpen($root, $password));
    }

    /** Returns the vault read only open default. */
    public function vaultReadOnlyOpenDefault(string $password): ReadOnlyVaultDirectory
    {
        return new ReadOnlyVaultDirectory($this->operations, $this->operations->vaultReadOnlyOpenDefault($password));
    }

    /** Returns the vault default directory. */
    public function vaultDefaultDirectory(): string
    {
        return $this->operations->vaultDefaultDirectory();
    }

    /** Returns the vault default path. */
    public function vaultDefaultPath(): string
    {
        return $this->operations->vaultDefaultPath();
    }

    /** Returns the vault agent log path. */
    public function vaultAgentLogPath(): string
    {
        return $this->operations->vaultAgentLogPath();
    }

    /** Returns the vault agent log destination. */
    public function vaultAgentLogDestination(): string
    {
        return $this->operations->vaultAgentLogDestination();
    }

    /** Returns the vault local. */
    public function vaultLocal(): LocalVault
    {
        return new LocalVault($this->operations, $this->operations->vaultLocal());
    }

}

abstract class OwnedHandle
{
    /** Returns the construct. */
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}
    final public function nativeHandle(): CData { return $this->handle; }
}

/** Owned, mutable view of one encrypted lockbox archive. */
class Lockbox extends OwnedHandle
{

    /** Adds file. */
    public function addFile(string $path, string $data, bool $replace): bool
    {
        return $this->operations->lockboxAddFile($this->handle, $path, $data, $replace);
    }

    /** Adds file with permissions. */
    public function addFileWithPermissions(string $path, string $data, int $permissions, bool $replace): bool
    {
        return $this->operations->lockboxAddFileWithPermissions($this->handle, $path, $data, $permissions, $replace);
    }

    /** Returns file. */
    public function getFile(string $path): string
    {
        return $this->operations->lockboxGetFile($this->handle, $path);
    }

    /** Extracts file. */
    public function extractFile(string $source, string $destination, bool $replace): bool
    {
        return $this->operations->lockboxExtractFile($this->handle, $source, $destination, $replace);
    }

    /** Extracts directory. */
    public function extractDirectory(string $destination, int $maxFileBytes, int $maxTotalBytes, int $maxFiles, bool $restoreSymlinks, bool $restorePermissions, bool $overwrite): bool
    {
        return $this->operations->lockboxExtractDirectory($this->handle, $destination, $maxFileBytes, $maxTotalBytes, $maxFiles, $restoreSymlinks, $restorePermissions, $overwrite);
    }

    /** Returns the stream content. */
    public function streamContent(bool $physical): \Revault\Bindings\StreamChunkList
    {
        return $this->operations->lockboxStreamContent($this->handle, $physical);
    }

    /** Returns cache statistics for this lockbox. */
    public function cacheStats(): \Revault\Bindings\CacheStats
    {
        return $this->operations->lockboxCacheStats($this->handle);
    }

    /** Returns import statistics for this lockbox. */
    public function importStats(): \Revault\Bindings\ImportStats
    {
        return $this->operations->lockboxImportStats($this->handle);
    }

    /** Updates import stats. */
    public function resetImportStats(): bool
    {
        return $this->operations->lockboxResetImportStats($this->handle);
    }

    /** Returns the page inspection. */
    public function pageInspection(): \Revault\Bindings\PageInspectionList
    {
        return $this->operations->lockboxPageInspection($this->handle);
    }

    /** Returns the recovery report. */
    public function recoveryReport(): \Revault\Bindings\RecoveryReport
    {
        return $this->operations->lockboxRecoveryReport($this->handle);
    }

    /** Returns the recovery report render. */
    public function recoveryReportRender(bool $verbose, int $maxEntries): string
    {
        return $this->operations->lockboxRecoveryReportRender($this->handle, $verbose, $maxEntries);
    }

    /** Returns the storage len. */
    public function storageLen(): int
    {
        return $this->operations->lockboxStorageLen($this->handle);
    }

    /** Sets workload profile. */
    public function setWorkloadProfile(string $profile): bool
    {
        return $this->operations->lockboxSetWorkloadProfile($this->handle, $profile);
    }

    /** Sets worker policy. */
    public function setWorkerPolicy(string $mode, int $jobs): bool
    {
        return $this->operations->lockboxSetWorkerPolicy($this->handle, $mode, $jobs);
    }

    /** Returns the runtime options. */
    public function runtimeOptions(): \Revault\Bindings\RuntimeOptions
    {
        return $this->operations->lockboxRuntimeOptions($this->handle);
    }

    /** Authenticates and publishes the staged changes. */
    public function commit(): bool
    {
        return $this->operations->lockboxCommit($this->handle);
    }

    /** Creates dir. */
    public function createDir(string $path, bool $createParents): bool
    {
        return $this->operations->lockboxCreateDir($this->handle, $path, $createParents);
    }

    /** Removes delete. */
    public function delete(string $path): bool
    {
        return $this->operations->lockboxDelete($this->handle, $path);
    }

    /** Removes dir. */
    public function removeDir(string $path, bool $recursive): bool
    {
        return $this->operations->lockboxRemoveDir($this->handle, $path, $recursive);
    }

    /** Creates parent dirs. */
    public function createParentDirs(string $path): bool
    {
        return $this->operations->lockboxCreateParentDirs($this->handle, $path);
    }

    /** Updates rename. */
    public function rename(string $from, string $to): bool
    {
        return $this->operations->lockboxRename($this->handle, $from, $to);
    }

    /** Lists list. */
    public function list(string $path, bool $recursive): \Revault\Bindings\LockboxEntryList
    {
        return $this->operations->lockboxList($this->handle, $path, $recursive);
    }

    /** Lists with options. */
    public function listWithOptions(string $path, string $glob, bool $recursive, bool $includeFiles, bool $includeSymlinks, bool $includeDirectories, int $limit): \Revault\Bindings\LockboxEntryList
    {
        return $this->operations->lockboxListWithOptions($this->handle, $path, $glob, $recursive, $includeFiles, $includeSymlinks, $includeDirectories, $limit);
    }

    /** Returns metadata for the selected lockbox entry. */
    public function stat(string $path): \Revault\Bindings\OptionalLockboxEntry
    {
        return $this->operations->lockboxStat($this->handle, $path);
    }

    /** Sets variable. */
    public function setVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetVariable($this->handle, $name, $value);
    }

    /** Stores a secret variable from binary-safe PHP string bytes. */
    public function setSecretVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetSecretVariable($this->handle, $name, $value);
    }

    /** Returns variable. */
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

    /** Removes variable. */
    public function deleteVariable(string $name): bool
    {
        return $this->operations->lockboxDeleteVariable($this->handle, $name);
    }

    /** Updates variables. */
    public function moveVariables(string $movesProto): bool
    {
        return $this->operations->lockboxMoveVariables($this->handle, $movesProto);
    }

    /** Lists variables. */
    public function listVariables(): \Revault\Bindings\VariableList
    {
        return $this->operations->lockboxListVariables($this->handle);
    }

    /** Returns the variable sensitivity. */
    public function variableSensitivity(string $name): \Revault\Bindings\OptionalString
    {
        return $this->operations->lockboxVariableSensitivity($this->handle, $name);
    }

    /** Adds symlink. */
    public function addSymlink(string $path, string $target, bool $replace): bool
    {
        return $this->operations->lockboxAddSymlink($this->handle, $path, $target, $replace);
    }

    /** Returns symlink target. */
    public function getSymlinkTarget(string $path): string
    {
        return $this->operations->lockboxGetSymlinkTarget($this->handle, $path);
    }

    /** Returns the id. */
    public function id(): string
    {
        return $this->operations->lockboxId($this->handle);
    }

    /** Reports whether exists. */
    public function exists(string $path): bool
    {
        return $this->operations->lockboxExists($this->handle, $path);
    }

    /** Reports whether dir. */
    public function isDir(string $path): bool
    {
        return $this->operations->lockboxIsDir($this->handle, $path);
    }

    /** Returns the permissions. */
    public function permissions(string $path): int
    {
        return $this->operations->lockboxPermissions($this->handle, $path);
    }

    /** Sets permissions. */
    public function setPermissions(string $path, int $permissions): bool
    {
        return $this->operations->lockboxSetPermissions($this->handle, $path, $permissions);
    }

    /** Returns range. */
    public function readRange(string $path, int $offset, int $len): string
    {
        return $this->operations->lockboxReadRange($this->handle, $path, $offset, $len);
    }

    /** Adds password. */
    public function addPassword(string $password): int
    {
        return $this->operations->lockboxAddPassword($this->handle, $password);
    }

    /** Adds contact. */
    public function addContact(OwnedHandle $contact, string $name): int
    {
        return $this->operations->lockboxAddContact($this->handle, $contact->nativeHandle(), $name);
    }

    /** Removes key. */
    public function deleteKey(int $id): bool
    {
        return $this->operations->lockboxDeleteKey($this->handle, $id);
    }

    /** Lists key slots. */
    public function listKeySlots(): \Revault\Bindings\KeySlotList
    {
        return $this->operations->lockboxListKeySlots($this->handle);
    }

    /** Sets owner signing key. */
    public function setOwnerSigningKey(OwnedHandle $key): bool
    {
        return $this->operations->lockboxSetOwnerSigningKey($this->handle, $key->nativeHandle());
    }

    /** Returns the owner inspection. */
    public function ownerInspection(): \Revault\Bindings\OwnerInspection
    {
        return $this->operations->lockboxOwnerInspection($this->handle);
    }

    /** Returns the define form. */
    public function defineForm(string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->operations->lockboxDefineForm($this->handle, $alias, $name, $description, $fieldsProto);
    }

    /** Lists form definitions. */
    public function listFormDefinitions(): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->lockboxListFormDefinitions($this->handle);
    }

    /** Returns the resolve form. */
    public function resolveForm(string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->operations->lockboxResolveForm($this->handle, $reference);
    }

    /** Lists form revisions. */
    public function listFormRevisions(string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->lockboxListFormRevisions($this->handle, $typeId);
    }

    /** Creates form record. */
    public function createFormRecord(string $path, string $typeReference, string $name): \Revault\Bindings\FormRecord
    {
        return $this->operations->lockboxCreateFormRecord($this->handle, $path, $typeReference, $name);
    }

    /** Sets form field. */
    public function setFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetFormField($this->handle, $path, $field, $value);
    }

    /** Stores a secret form field from binary-safe PHP string bytes. */
    public function setSecretFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetSecretFormField($this->handle, $path, $field, $value);
    }

    /** Lists form records. */
    public function listFormRecords(): \Revault\Bindings\FormRecordList
    {
        return $this->operations->lockboxListFormRecords($this->handle);
    }

    /** Returns form record. */
    public function getFormRecord(string $path): \Revault\Bindings\OptionalFormRecord
    {
        return $this->operations->lockboxGetFormRecord($this->handle, $path);
    }

    /** Removes form record. */
    public function deleteFormRecord(string $path): bool
    {
        return $this->operations->lockboxDeleteFormRecord($this->handle, $path);
    }

    /** Updates form records. */
    public function moveFormRecords(string $movesProto): bool
    {
        return $this->operations->lockboxMoveFormRecords($this->handle, $movesProto);
    }

    /** Returns form field. */
    public function getFormField(string $path, string $field): \Revault\Bindings\OptionalFormValue
    {
        return $this->operations->lockboxGetFormField($this->handle, $path, $field);
    }

    /** Invokes the callback with temporary field bytes, then wipes the native transfer. */
    public function withSecretFormField(string $path, string $field, callable $callback): mixed
    {
        return $this->operations->lockboxWithSecretFormField($this->handle, $path, $field, $callback);
    }

    /** Returns the to bytes. */
    public function toBytes(): string
    {
        return $this->operations->lockboxToBytes($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->lockboxFree($this->handle);
    }

}

/** Owned contact key pair used to decrypt content keys sent by contacts. */
class ContactKeyPair extends OwnedHandle
{

    /** Returns the public. */
    public function public(): string
    {
        return $this->operations->keyContactPublic($this->handle);
    }

    /** Returns the private. */
    public function private(): string
    {
        return $this->operations->keyContactPrivate($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->keyContactFree($this->handle);
    }

    /** Decrypts a wrapped content key for this contact. */
    public function decrypt(OwnedHandle $wrapped): string
    {
        return $this->operations->keyContactDecrypt($this->handle, $wrapped->nativeHandle());
    }

}

/** Shareable contact public key used to encrypt a recipient content key. */
class ContactPublicKey extends OwnedHandle
{

    /** Returns the public free. */
    public function publicFree(): void
    {
        $this->operations->keyContactPublicFree($this->handle);
    }

    /** Encrypts a content key for the selected contact. */
    public function encrypt(string $contentKey): WrappedContactKey
    {
        return new WrappedContactKey($this->operations, $this->operations->keyContactEncrypt($this->handle, $contentKey));
    }

}

/** Owned encrypted content-key envelope for one contact recipient. */
class WrappedContactKey extends OwnedHandle
{

    /** Returns the public. */
    public function public(): string
    {
        return $this->operations->keyContactWrappedPublic($this->handle);
    }

    /** Returns the ciphertext. */
    public function ciphertext(): string
    {
        return $this->operations->keyContactWrappedCiphertext($this->handle);
    }

    /** Returns the encrypted. */
    public function encrypted(): string
    {
        return $this->operations->keyContactWrappedEncrypted($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->keyContactWrappedFree($this->handle);
    }

}

/** Owned signing key pair used to authorize mutable lockbox commits. */
class SigningKeyPair extends OwnedHandle
{

    /** Returns the public. */
    public function public(): string
    {
        return $this->operations->keySigningPublic($this->handle);
    }

    /** Returns the private. */
    public function private(): string
    {
        return $this->operations->keySigningPrivate($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->keySigningFree($this->handle);
    }

}

/** Public key used to verify owner-authorized lockbox commits. */
class SigningPublicKey extends OwnedHandle
{

    /** Returns the public free. */
    public function publicFree(): void
    {
        $this->operations->keySigningPublicFree($this->handle);
    }

}

/** Writable, password-protected local metadata vault. */
class VaultDirectory extends OwnedHandle
{

    /** Returns the root. */
    public function root(): string
    {
        return $this->operations->vaultDirectoryRoot($this->handle);
    }

    /** Returns the structure version. */
    public function structureVersion(): int
    {
        return $this->operations->vaultDirectoryStructureVersion($this->handle);
    }

    /** Lists private keys. */
    public function listPrivateKeys(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeys($this->handle);
    }

    /** Lists private key names. */
    public function listPrivateKeyNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeyNames($this->handle);
    }

    /** Lists contact names. */
    public function listContactNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListContactNames($this->handle);
    }

    /** Lists form aliases. */
    public function listFormAliases(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultDirectoryListFormAliases($this->handle);
    }

    /** Returns the private key exists. */
    public function privateKeyExists(string $name): bool
    {
        return $this->operations->vaultDirectoryPrivateKeyExists($this->handle, $name);
    }

    /** Removes private key. */
    public function deletePrivateKey(string $name): bool
    {
        return $this->operations->vaultDirectoryDeletePrivateKey($this->handle, $name);
    }

    /** Stores private key. */
    public function storePrivateKey(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStorePrivateKey($this->handle, $name, $key->nativeHandle());
    }

    /** Loads private key. */
    public function loadPrivateKey(string $name): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultDirectoryLoadPrivateKey($this->handle, $name));
    }

    /** Loads private key generation. */
    public function loadPrivateKeyGeneration(string $name, int $index): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultDirectoryLoadPrivateKeyGeneration($this->handle, $name, $index));
    }

    /** Stores contact. */
    public function storeContact(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStoreContact($this->handle, $name, $key->nativeHandle());
    }

    /** Loads contact. */
    public function loadContact(string $name): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->vaultDirectoryLoadContact($this->handle, $name));
    }

    /** Returns the contact exists. */
    public function contactExists(string $name): bool
    {
        return $this->operations->vaultDirectoryContactExists($this->handle, $name);
    }

    /** Removes contact. */
    public function deleteContact(string $name): bool
    {
        return $this->operations->vaultDirectoryDeleteContact($this->handle, $name);
    }

    /** Lists contacts. */
    public function listContacts(): \Revault\Bindings\ContactList
    {
        return $this->operations->vaultDirectoryListContacts($this->handle);
    }

    /** Stores profile email. */
    public function storeProfileEmail(string $name, string $email): bool
    {
        return $this->operations->vaultDirectoryStoreProfileEmail($this->handle, $name, $email);
    }

    /** Returns the profile email. */
    public function profileEmail(string $name): \Revault\Bindings\OptionalString
    {
        return $this->operations->vaultDirectoryProfileEmail($this->handle, $name);
    }

    /** Stores backup. */
    public function storeBackup(string $id, string $bytes): bool
    {
        return $this->operations->vaultDirectoryStoreBackup($this->handle, $id, $bytes);
    }

    /** Loads backup. */
    public function loadBackup(string $id): string
    {
        return $this->operations->vaultDirectoryLoadBackup($this->handle, $id);
    }

    /** Returns the backup count. */
    public function backupCount(): int
    {
        return $this->operations->vaultDirectoryBackupCount($this->handle);
    }

    /** Returns the restore private key. */
    public function restorePrivateKey(string $name, OwnedHandle $key, OwnedHandle $signingKey, bool $overwrite): bool
    {
        return $this->operations->vaultDirectoryRestorePrivateKey($this->handle, $name, $key->nativeHandle(), $signingKey->nativeHandle(), $overwrite);
    }

    /** Loads owner signing key. */
    public function loadOwnerSigningKey(string $name): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKey($this->handle, $name));
    }

    /** Loads owner signing key generation. */
    public function loadOwnerSigningKeyGeneration(string $name, int $index): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKeyGeneration($this->handle, $name, $index));
    }

    /** Stores contact signing key. */
    public function storeContactSigningKey(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStoreContactSigningKey($this->handle, $name, $key->nativeHandle());
    }

    /** Loads contact signing key. */
    public function loadContactSigningKey(string $name): SigningPublicKey
    {
        return new SigningPublicKey($this->operations, $this->operations->vaultDirectoryLoadContactSigningKey($this->handle, $name));
    }

    /** Lists profile generations. */
    public function listProfileGenerations(string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->operations->vaultDirectoryListProfileGenerations($this->handle, $name);
    }

    /** Updates private key. */
    public function rotatePrivateKey(string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->operations->vaultDirectoryRotatePrivateKey($this->handle, $name);
    }

    /** Stores lockbox. */
    public function rememberLockbox(string $id, string $path): bool
    {
        return $this->operations->vaultDirectoryRememberLockbox($this->handle, $id, $path);
    }

    /** Lists known lockboxes. */
    public function listKnownLockboxes(): \Revault\Bindings\KnownLockboxList
    {
        return $this->operations->vaultDirectoryListKnownLockboxes($this->handle);
    }

    /** Removes lockbox. */
    public function forgetLockbox(string $path): bool
    {
        return $this->operations->vaultDirectoryForgetLockbox($this->handle, $path);
    }

    /** Stores access slot label. */
    public function rememberAccessSlotLabel(string $id, int $slotId, string $name): bool
    {
        return $this->operations->vaultDirectoryRememberAccessSlotLabel($this->handle, $id, $slotId, $name);
    }

    /** Lists access slot labels. */
    public function listAccessSlotLabels(string $id): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryListAccessSlotLabels($this->handle, $id);
    }

    /** Returns the find access slot labels. */
    public function findAccessSlotLabels(string $id, string $name): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryFindAccessSlotLabels($this->handle, $id, $name);
    }

    /** Removes access slot label. */
    public function forgetAccessSlotLabel(string $id, int $slotId): bool
    {
        return $this->operations->vaultDirectoryForgetAccessSlotLabel($this->handle, $id, $slotId);
    }

    /** Returns the define form. */
    public function defineForm(string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->operations->vaultDirectoryDefineForm($this->handle, $alias, $name, $description, $fieldsProto);
    }

    /** Returns the resolve form. */
    public function resolveForm(string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->operations->vaultDirectoryResolveForm($this->handle, $reference);
    }

    /** Lists forms. */
    public function listForms(): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListForms($this->handle);
    }

    /** Lists form revisions. */
    public function listFormRevisions(string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListFormRevisions($this->handle, $typeId);
    }

    /** Returns the seed forms. */
    public function seedForms(): int
    {
        return $this->operations->vaultDirectorySeedForms($this->handle);
    }

    /** Stores password. */
    public function rememberPassword(string $id, string $password): bool
    {
        return $this->operations->vaultDirectoryRememberPassword($this->handle, $id, $password);
    }

    /** Returns the remembered password. */
    public function rememberedPassword(string $id): string
    {
        return $this->operations->vaultDirectoryRememberedPassword($this->handle, $id);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->vaultDirectoryFree($this->handle);
    }

}

/** Read-only metadata view that never loads an owner signing key. */
class ReadOnlyVaultDirectory
{
    /** Returns the construct. */
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}

    /** Lists profile names. */
    public function listProfileNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListProfileNames($this->handle);
    }

    /** Lists contact names. */
    public function listContactNames(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListContactNames($this->handle);
    }

    /** Lists form aliases. */
    public function listFormAliases(): \Revault\Bindings\StringList
    {
        return $this->operations->vaultReadOnlyListFormAliases($this->handle);
    }

    /** Lists known lockboxes. */
    public function listKnownLockboxes(): \Revault\Bindings\KnownLockboxList
    {
        return $this->operations->vaultReadOnlyListKnownLockboxes($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->vaultReadOnlyFree($this->handle);
    }

}

/** Client for the local session agent's time-limited secret cache. */
class Agent
{
    /** Returns the construct. */
    public function __construct(protected readonly BindingOperations $operations) {}

    /** Reports whether running. */
    public function isRunning(): bool
    {
        return $this->operations->vaultIsRunning();
    }

    /** Removes all. */
    public function forgetAll(): bool
    {
        return $this->operations->vaultForgetAll();
    }

    /** Returns the serve. */
    public function serve(): bool
    {
        return $this->operations->vaultAgentServe();
    }

    /** Verifies transport. */
    public function verifyTransport(): bool
    {
        return $this->operations->vaultAgentVerifyTransport();
    }

    /** Returns get. */
    public function get(string $id): string
    {
        return $this->operations->vaultAgentGet($id);
    }

    /** Stores put. */
    public function put(string $id, string $key): bool
    {
        return $this->operations->vaultAgentPut($id, $key);
    }

    /** Removes forget. */
    public function forget(string $id): bool
    {
        return $this->operations->vaultAgentForget($id);
    }

    /** Stops stop. */
    public function stop(): bool
    {
        return $this->operations->vaultAgentStop();
    }

    /** Starts start. */
    public function start(): bool
    {
        return $this->operations->vaultAgentStart();
    }

    /** Lists list. */
    public function list(): \Revault\Bindings\AgentEntryList
    {
        return $this->operations->vaultAgentList();
    }

    /** Returns the sleep support. */
    public function sleepSupport(): \Revault\Bindings\SleepSupport
    {
        return $this->operations->vaultAgentSleepSupport();
    }

    /** Returns vault unlock key. */
    public function getVaultUnlockKey(string $vaultId): string
    {
        return $this->operations->vaultAgentGetVaultUnlockKey($vaultId);
    }

    /** Stores vault unlock key. */
    public function putVaultUnlockKey(string $vaultId, string $key, int $ttlSeconds): bool
    {
        return $this->operations->vaultAgentPutVaultUnlockKey($vaultId, $key, $ttlSeconds);
    }

    /** Removes vault unlock key. */
    public function forgetVaultUnlockKey(string $vaultId): bool
    {
        return $this->operations->vaultAgentForgetVaultUnlockKey($vaultId);
    }

    /** Returns owner signing key. */
    public function getOwnerSigningKey(string $vaultId, string $profile): SigningKeyPair
    {
        return new SigningKeyPair($this->operations, $this->operations->vaultAgentGetOwnerSigningKey($vaultId, $profile));
    }

    /** Stores owner signing key. */
    public function putOwnerSigningKey(string $vaultId, string $profile, OwnedHandle $key, int $ttlSeconds): bool
    {
        return $this->operations->vaultAgentPutOwnerSigningKey($vaultId, $profile, $key->nativeHandle(), $ttlSeconds);
    }

    /** Removes owner signing key. */
    public function forgetOwnerSigningKey(string $vaultId, string $profile): bool
    {
        return $this->operations->vaultAgentForgetOwnerSigningKey($vaultId, $profile);
    }

    /** Starts activity. */
    public function beginActivity(string $kind): AgentActivity
    {
        return new AgentActivity($this->operations, $this->operations->vaultAgentBeginActivity($kind));
    }

    /** Stops activity. */
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
    /** Returns the construct. */
    public function __construct(protected readonly BindingOperations $operations) {}

    /** Returns the status. */
    public function status(): \Revault\Bindings\PlatformStatus
    {
        return $this->operations->vaultPlatformStatus();
    }

    /** Sets scope. */
    public function setScope(string $scope): bool
    {
        return $this->operations->vaultPlatformSetScope($scope);
    }

    /** Removes password. */
    public function forgetPassword(): bool
    {
        return $this->operations->vaultPlatformForgetPassword();
    }

    /** Stores password. */
    public function putPassword(string $password): bool
    {
        return $this->operations->vaultPlatformPutPassword($password);
    }

    /** Returns the enable. */
    public function enable(): bool
    {
        return $this->operations->vaultPlatformEnable();
    }

    /** Returns the disable. */
    public function disable(): bool
    {
        return $this->operations->vaultPlatformDisable();
    }

    /** Returns the disabled. */
    public function disabled(): bool
    {
        return $this->operations->vaultPlatformDisabled();
    }

    /** Returns password. */
    public function getPassword(): string
    {
        return $this->operations->vaultPlatformGetPassword();
    }

}

/** High-level workflow for local metadata and remembered lockboxes. */
class LocalVault extends OwnedHandle
{

    /** Creates lockbox password. */
    public function createLockboxPassword(string $path, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxPassword($this->handle, $path, $password));
    }

    /** Opens lockbox password. */
    public function openLockboxPassword(string $path, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultOpenLockboxPassword($this->handle, $path, $password));
    }

    /** Creates lockbox content key. */
    public function createLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContentKey($this->handle, $path, $contentKey, $signingKey->nativeHandle()));
    }

    /** Creates lockbox contact. */
    public function createLockboxContact(string $path, OwnedHandle $contact, string $name, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContact($this->handle, $path, $contact->nativeHandle(), $name, $signingKey->nativeHandle()));
    }

    /** Opens lockbox content key. */
    public function openLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultOpenLockboxContentKey($this->handle, $path, $contentKey, $signingKey->nativeHandle()));
    }

    /** Stores lockbox password. */
    public function cacheLockboxPassword(string $path, string $password, int $ttlSeconds): bool
    {
        return $this->operations->vaultCacheLockboxPassword($this->handle, $path, $password, $ttlSeconds);
    }

    /** Releases the native resources held by lockbox. */
    public function closeLockbox(string $path): bool
    {
        return $this->operations->vaultCloseLockbox($this->handle, $path);
    }

    /** Releases the native resources held by all. */
    public function closeAll(): bool
    {
        return $this->operations->vaultCloseAll($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->vaultFree($this->handle);
    }

}
