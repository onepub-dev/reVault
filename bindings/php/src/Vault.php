<?php
declare(strict_types=1);

namespace Revault;

use FFI\CData;

/**
 * Entry point for encrypted lockboxes, cryptographic keys, local vault
 * metadata, the Session Agent, and the platform credential store.
 *
 * Create one when the application starts, then use it to open lockboxes and
 * manage keys and local services. Release disposable values promptly and use
 * callback-scoped secret accessors to avoid retaining plaintext. See the
 * repository README for installation and examples:
 * https://github.com/onepub-dev/reVault#readme
 */
/**
 * Loads the reVault engine and provides archive, key, Vault, agent, and
 * platform operations.
 *
 * Loading the engine does not open a Vault or Lockbox. Most applications
 * should call load() once, then use the Vault and Lockbox classes shown in the
 * package README.
 */
final class Revault
{
    private readonly BindingOperations $operations;
    private readonly Agent $agent;
    private readonly Platform $platform;

    /** Loads the bundled native library, or the explicit library path when supplied. */
    public function __construct(?string $nativeLibraryPath = null)
    {
        if ($nativeLibraryPath === '') {
            throw new \InvalidArgumentException('nativeLibraryPath must not be empty');
        }
        $inherited = getenv('REVAULT_LIBRARY');
        $selected = $nativeLibraryPath
            ?? (is_string($inherited) && $inherited !== '' ? $inherited : null)
            ?? self::nativeLibrary();
        $this->operations = BindingOperations::load($selected);
        $this->agent = new Agent($this->operations); $this->platform = new Platform($this->operations);
    }

    private static function nativeLibrary(): string
    {
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
        if (!is_file($bundled)) { throw new \RuntimeException("revault-api native library is missing for $target; install the matching platform package"); }
        return $bundled;
    }

    /** Returns the optional session agent controller without starting the agent. */
    public function agent(): Agent { return $this->agent; }
    /** Returns the operating system credential store facade. */
    public function platform(): Platform { return $this->platform; }
    public static function load(?string $nativeLibraryPath = null): self
    { return new self($nativeLibraryPath); }
    public static function runtime(): self { return new self(); }
    public static function agentSession(): AgentSession
    { $runtime = new self(); return new AgentSession($runtime->operations); }
    /** Returns the last error. */
    public function lastError(): string { return $this->operations->lastErrorMessage(); }
    /** Returns the last error details. */
    public function lastErrorDetails(): object { return $this->operations->bufferLastErrorDetails(); }

    /** Returns the newest Lockbox archive format version supported by this engine. */
    public function lockboxFormatVersion(): int
    {
        return $this->operations->lockboxFormatVersion();
    }

    /** Reads the format version from serialized Lockbox bytes without opening them. */
    public function lockboxProbeFormatVersion(string $bytes): int
    {
        return $this->operations->lockboxProbeFormatVersion($bytes);
    }

    /** Creates an in memory Lockbox protected by a 32 byte content key. */
    public function lockboxCreate(string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreate($key));
    }

    /** Creates a lockbox with explicit cache capacity, workload, worker policy, and job count. */
    public function lockboxCreateWithOptions(string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithOptions($key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    /** Creates an in memory Lockbox protected by the supplied password. */
    public function lockboxCreatePassword(string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreatePassword($password));
    }

    /** Creates an in memory Lockbox that the supplied contact can open. */
    public function lockboxCreateContact(OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateContact($contact->nativeHandle()));
    }

    /** Creates an in memory Lockbox and assigns its profile signing key. */
    public function lockboxCreateWithSigningKey(string $contentKey, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxCreateWithSigningKey($contentKey, $signingKey->nativeHandle()));
    }

    /** Opens serialized Lockbox bytes with a 32 byte content key. */
    public function lockboxOpen(string $archive, string $key): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpen($archive, $key));
    }

    /** Opens a lockbox with explicit cache capacity, workload, worker policy, and job count. */
    public function lockboxOpenWithOptions(string $archive, string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenWithOptions($archive, $key, $cacheMode, $cacheBytes, $workload, $worker, $jobs));
    }

    /** Opens serialized Lockbox bytes with the supplied password. */
    public function lockboxOpenPassword(string $archive, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenPassword($archive, $password));
    }

    /** Opens serialized Lockbox bytes with the supplied contact private key. */
    public function lockboxOpenContact(string $archive, OwnedHandle $contact): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxOpenContact($archive, $contact->nativeHandle()));
    }

    /** Reads public header, signature, and access slot metadata from a Lockbox file. */
    public function lockboxInspectFile(string $path): \Revault\FileInspection
    {
        return $this->operations->lockboxInspectFile($path);
    }

    /** Scans a damaged Lockbox file with its 32 byte content key. */
    public function lockboxRecoveryScanPath(string $path, string $key): \Revault\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScanPath($path, $key);
    }

    /** Scans damaged serialized Lockbox bytes with their 32 byte content key. */
    public function lockboxRecoveryScan(string $bytes, string $key): \Revault\RecoveryReport
    {
        return $this->operations->lockboxRecoveryScan($bytes, $key);
    }

    /** Builds a new Lockbox from recoverable records without changing the source. */
    public function lockboxRecoverySalvage(string $bytes, string $key, OwnedHandle $signingKey): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->lockboxRecoverySalvage($bytes, $key, $signingKey->nativeHandle()));
    }

    /** Generates a contact encryption key pair using secure random data. */
    public function keyContactGenerate(): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactGenerate());
    }

    /** Imports a contact key pair from its private binary record. */
    public function keyContactFromPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->keyContactFromPrivate($bytes));
    }

    /** Imports a contact public key from its binary representation. */
    public function keyContactPublicFromBytes(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->keyContactPublicFromBytes($bytes));
    }

    /** Generates a profile signing key pair using secure random data. */
    public function generateProfileSigningKeyPair(): ProfileSigningKeyPair
    {
        return new ProfileSigningKeyPair($this->operations, $this->operations->keySigningGenerate());
    }

    /** Imports a profile signing key pair from its private binary record. */
    public function profileSigningKeyPairFromPrivate(string $bytes): ProfileSigningKeyPair
    {
        return new ProfileSigningKeyPair($this->operations, $this->operations->keySigningFromPrivate($bytes));
    }

    /** Imports a profile signing public key from its binary representation. */
    public function profileSigningPublicKeyFromBytes(string $bytes): ProfileSigningPublicKey
    {
        return new ProfileSigningPublicKey($this->operations, $this->operations->keySigningPublicFromBytes($bytes));
    }

    /** Exports a private key in the requested KeyExportFormat. */
    public function vaultKeyExportPrivate(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPrivate($key->nativeHandle(), $format);
    }

    /** Exports a public key in the requested KeyExportFormat. */
    public function vaultKeyExportPublic(OwnedHandle $key, string $format): string
    {
        return $this->operations->vaultKeyExportPublic($key->nativeHandle(), $format);
    }

    /** Imports a private contact key from a detected supported encoding. */
    public function vaultKeyImportPrivate(string $bytes): ContactKeyPair
    {
        return new ContactKeyPair($this->operations, $this->operations->vaultKeyImportPrivate($bytes));
    }

    /** Imports a public contact key from a detected supported encoding. */
    public function vaultKeyImportPublic(string $bytes): ContactPublicKey
    {
        return new ContactPublicKey($this->operations, $this->operations->vaultKeyImportPublic($bytes));
    }

    /** Returns the stable fingerprint used to verify a public key. */
    public function vaultKeyFingerprint(OwnedHandle $key): string
    {
        return $this->operations->vaultKeyFingerprint($key->nativeHandle());
    }

    /** Encodes key bytes as hexadecimal text. */
    public function vaultKeyFormatHex(string $bytes): string
    {
        return $this->operations->vaultKeyFormatHex($bytes);
    }

    /** Decodes hexadecimal key text and rejects malformed input. */
    public function vaultKeyDecodeHex(string $text): string
    {
        return $this->operations->vaultKeyDecodeHex($text);
    }

    /** Encodes key bytes using Crockford Base32. */
    public function vaultKeyFormatCrockford(string $bytes): string
    {
        return $this->operations->vaultKeyFormatCrockford($bytes);
    }

    /** Groups a Crockford code for easier reading and transcription. */
    public function vaultKeyFormatCrockfordReading(string $code): string
    {
        return $this->operations->vaultKeyFormatCrockfordReading($code);
    }

    /** Decodes Crockford Base32 key text and rejects malformed input. */
    public function vaultKeyDecodeCrockford(string $code): string
    {
        return $this->operations->vaultKeyDecodeCrockford($code);
    }

    /** Encodes arbitrary bytes as hexadecimal text. */
    public function vaultKeyHexEncode(string $bytes): string
    {
        return $this->operations->vaultKeyHexEncode($bytes);
    }

    /** Decodes arbitrary hexadecimal text and rejects malformed input. */
    public function vaultKeyHexDecode(string $text): string
    {
        return $this->operations->vaultKeyHexDecode($text);
    }

    /** Opens an existing Vault directory with its passphrase. */
    public function vaultDirectoryOpen(string $root, string $password): Vault
    {
        return new Vault($this->operations, $this->operations->vaultDirectoryOpen($root, $password));
    }

    /** Returns the newest Vault structure version supported by this engine. */
    public function vaultStructureVersionCurrent(): int
    {
        return $this->operations->vaultStructureVersionCurrent();
    }

    /** Reads an existing Vault structure version without changing it. */
    public function vaultDirectoryProbeStructureVersion(string $root, string $password): int
    {
        return $this->operations->vaultDirectoryProbeStructureVersion($root, $password);
    }

    /** Opens or creates the default Vault without replacing existing state. */
    public function vaultDirectoryOpenOrCreateDefault(string $password): Vault
    {
        return new Vault($this->operations, $this->operations->vaultDirectoryOpenOrCreateDefault($password));
    }

    /** Replaces the default Vault and all persistent data it contains. */
    public function vaultDirectoryReplaceDefault(string $password): Vault
    {
        return new Vault($this->operations, $this->operations->vaultDirectoryReplaceDefault($password));
    }

    /** Changes the passphrase for an existing Vault at root. */
    public function vaultDirectoryChangePassword(string $root, string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangePassword($root, $oldPassword, $newPassword);
    }

    /** Changes the passphrase for the default Vault. */
    public function vaultDirectoryChangeDefaultPassword(string $oldPassword, string $newPassword): bool
    {
        return $this->operations->vaultDirectoryChangeDefaultPassword($oldPassword, $newPassword);
    }

    /** Replaces the Vault at root and all persistent data it contains. */
    public function vaultDirectoryReplace(string $root, string $password): Vault
    {
        return new Vault($this->operations, $this->operations->vaultDirectoryReplace($root, $password));
    }

    /** Opens the Vault at root, creating it only when absent. */
    public function vaultDirectoryOpenOrCreate(string $root, string $password): Vault
    {
        return new Vault($this->operations, $this->operations->vaultDirectoryOpenOrCreate($root, $password));
    }

    /** Writes a backup of the default Vault to path. */
    public function vaultBackupDefault(string $path, bool $overwrite): \Revault\VaultBackupManifest
    {
        return $this->operations->vaultBackupDefault($path, $overwrite);
    }

    /** Restores the default Vault from a backup at path. */
    public function vaultRestoreDefault(string $path, bool $overwrite): \Revault\VaultBackupManifest
    {
        return $this->operations->vaultRestoreDefault($path, $overwrite);
    }

    /** Opens an existing Vault metadata view that cannot load private keys. */
    public function vaultReadOnlyOpen(string $root, string $password): ReadOnlyVault
    {
        return new ReadOnlyVault($this->operations, $this->operations->vaultReadOnlyOpen($root, $password));
    }

    /** Opens the default Vault metadata view without loading private keys. */
    public function vaultReadOnlyOpenDefault(string $password): ReadOnlyVault
    {
        return new ReadOnlyVault($this->operations, $this->operations->vaultReadOnlyOpenDefault($password));
    }

    /** Returns the platform default Vault directory. */
    public function vaultDefaultDirectory(): string
    {
        return $this->operations->vaultDefaultDirectory();
    }

    /** Returns the path of the default Vault file. */
    public function vaultDefaultPath(): string
    {
        return $this->operations->vaultDefaultPath();
    }

    /** Returns the session agent log path. */
    public function vaultAgentLogPath(): string
    {
        return $this->operations->vaultAgentLogPath();
    }

    /** Returns the configured session agent log destination. */
    public function vaultAgentLogDestination(): string
    {
        return $this->operations->vaultAgentLogDestination();
    }

}

/** Base type for disposable API values; applications use its concrete subclasses. */
abstract class OwnedHandle
{
    /** Adopts an owned native handle; application code uses concrete subclasses. */
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}
    final public function nativeHandle(): CData { return $this->handle; }
    /** Release native memory; concrete handles implement free(). */
    public function close(): void { if (method_exists($this, 'free')) $this->free(); }
}

/**
 * An open Lockbox containing files, variables, and forms.
 *
 * Create or open a Lockbox with exactly one password, content key, or contact.
 * Mutations remain pending until commit(). Call close() in a finally block to
 * release the content key held by this process. See the package README and
 * bindings/e2e/php/conformance.php for complete examples.
 */
class Lockbox extends OwnedHandle
{
    /** Host path for handles returned by the path factory; null for bytes-only handles. */
    private ?string $backingPath = null;
    /** Create an in-memory archive protected by exactly one credential. */
    public static function createInMemory(?string $password = null, ?string $contentKey = null, ?OwnedHandle $contact = null, ?OwnedHandle $signingKey = null, ?array $options = null): self
    {
        if (count(array_filter([$password, $contentKey, $contact], static fn($value) => $value !== null)) !== 1) {
            throw new \InvalidArgumentException('Supply exactly one of password, contentKey, or contact.');
        }
        $runtime = Revault::runtime();
        $box = $password !== null ? $runtime->lockboxCreatePassword($password)
            : ($contact !== null ? $runtime->lockboxCreateContact($contact)
                : ($options !== null ? $runtime->lockboxCreateWithOptions($contentKey, $options['cacheMode'], $options['cacheBytes'] ?? 0, $options['workload'], $options['worker'], $options['jobs'] ?? 0)
                    : $runtime->lockboxCreate($contentKey)));
        if ($signingKey !== null) $box->setOwnerSigningKey($signingKey);
        return $box;
    }

    /** Open serialized archive bytes without consulting the Session Agent. */
    public static function openBytes(string $archive, ?string $password = null, ?string $contentKey = null, ?OwnedHandle $contact = null, ?array $options = null): self
    {
        if (count(array_filter([$password, $contentKey, $contact], static fn($value) => $value !== null)) !== 1) {
            throw new \InvalidArgumentException('Supply exactly one of password, contentKey, or contact.');
        }
        $runtime = Revault::runtime();
        if ($password !== null) return $runtime->lockboxOpenPassword($archive, $password);
        if ($contact !== null) return $runtime->lockboxOpenContact($archive, $contact);
        return $options === null ? $runtime->lockboxOpen($archive, $contentKey) : $runtime->lockboxOpenWithOptions($archive, $contentKey, $options['cacheMode'], $options['cacheBytes'] ?? 0, $options['workload'], $options['worker'], $options['jobs'] ?? 0);
    }

    /** Create a host archive file and return its process-local handle. */
    public static function create(string $path, ?string $password = null, ?string $contentKey = null, ?OwnedHandle $contact = null, ?OwnedHandle $signingKey = null, ?array $options = null, bool $overwrite = false): self
    {
        if (is_file($path) && !$overwrite) throw new \RuntimeException("Lockbox already exists: $path");
        $box = self::createInMemory($password, $contentKey, $contact, $signingKey, $options);
        file_put_contents($path, $box->toBytes());
        $box->backingPath = $path;
        return $box;
    }

    /** Open a host archive file without consulting the Session Agent. */
    public static function open(string $path, ?string $password = null, ?string $contentKey = null, ?OwnedHandle $contact = null, ?array $options = null): self
    {
        $box = self::openBytes(file_get_contents($path), $password, $contentKey, $contact, $options);
        $box->backingPath = $path;
        return $box;
    }

    /** Stages a file at the Lockbox path; replace controls an existing entry. */
    public function addFile(string $path, string $data, bool $replace): bool
    {
        return $this->operations->lockboxAddFile($this->handle, $path, $data, $replace);
    }

    /** Stages a file and its portable Unix permission bits. */
    public function addFileWithPermissions(string $path, string $data, int $permissions, bool $replace): bool
    {
        return $this->operations->lockboxAddFileWithPermissions($this->handle, $path, $data, $permissions, $replace);
    }

    /** Reads the complete file stored at the Lockbox path. */
    public function getFile(string $path): string
    {
        return $this->operations->lockboxGetFile($this->handle, $path);
    }

    /** Writes one Lockbox file to the host filesystem. */
    public function extractFile(string $source, string $destination, bool $replace): bool
    {
        return $this->operations->lockboxExtractFile($this->handle, $source, $destination, $replace);
    }

    /** Extracts the Lockbox with explicit size, count, link, and permission limits. */
    public function extractDirectory(string $destination, int $maxFileBytes, int $maxTotalBytes, int $maxFiles, bool $restoreSymlinks, bool $restorePermissions, bool $overwrite): bool
    {
        return $this->operations->lockboxExtractDirectory($this->handle, $destination, $maxFileBytes, $maxTotalBytes, $maxFiles, $restoreSymlinks, $restorePermissions, $overwrite);
    }

    /** Lists logical or physical content chunks for streaming diagnostics. */
    public function streamContent(bool $physical): \Revault\StreamChunkList
    {
        return $this->operations->lockboxStreamContent($this->handle, $physical);
    }

    /** Returns cache statistics for this lockbox. */
    public function cacheStats(): \Revault\CacheStats
    {
        return $this->operations->lockboxCacheStats($this->handle);
    }

    /** Returns import statistics for this lockbox. */
    public function importStats(): \Revault\ImportStats
    {
        return $this->operations->lockboxImportStats($this->handle);
    }

    /** Updates import stats. */
    public function resetImportStats(): bool
    {
        return $this->operations->lockboxResetImportStats($this->handle);
    }

    /** Returns page metadata for diagnostics without exposing plaintext secrets. */
    public function pageInspection(): \Revault\PageInspectionList
    {
        return $this->operations->lockboxPageInspection($this->handle);
    }

    /** Scans the open archive and returns its structured recovery report. */
    public function recoveryReport(): \Revault\RecoveryReport
    {
        return $this->operations->lockboxRecoveryReport($this->handle);
    }

    /** Renders the recovery report for a person, capped at maxEntries. */
    public function recoveryReportRender(bool $verbose, int $maxEntries): string
    {
        return $this->operations->lockboxRecoveryReportRender($this->handle, $verbose, $maxEntries);
    }

    /** Returns the current serialized archive size in bytes. */
    public function storageLen(): int
    {
        return $this->operations->lockboxStorageLen($this->handle);
    }

    /** Selects the predefined workload policy for later operations. */
    public function setWorkloadProfile(string $profile): bool
    {
        return $this->operations->lockboxSetWorkloadProfile($this->handle, $profile);
    }

    /** Selects worker scheduling and the maximum job count. */
    public function setWorkerPolicy(string $mode, int $jobs): bool
    {
        return $this->operations->lockboxSetWorkerPolicy($this->handle, $mode, $jobs);
    }

    /** Returns the cache, workload, and worker settings used by this Lockbox. */
    public function runtimeOptions(): \Revault\RuntimeOptions
    {
        return $this->operations->lockboxRuntimeOptions($this->handle);
    }

    /** Authenticates and publishes the staged changes. */
    public function commit(): bool
    {
        $committed = $this->operations->lockboxCommit($this->handle);
        if ($this->backingPath !== null) file_put_contents($this->backingPath, $this->toBytes());
        return $committed;
    }

    /** Stages a directory entry and optionally creates missing parents. */
    public function createDir(string $path, bool $createParents): bool
    {
        return $this->operations->lockboxCreateDir($this->handle, $path, $createParents);
    }

    /** Stages removal of a file, link, or empty directory at path. */
    public function delete(string $path): bool
    {
        return $this->operations->lockboxDelete($this->handle, $path);
    }

    /** Stages removal of a directory, optionally including its descendants. */
    public function removeDir(string $path, bool $recursive): bool
    {
        return $this->operations->lockboxRemoveDir($this->handle, $path, $recursive);
    }

    /** Stages every missing parent directory for path. */
    public function createParentDirs(string $path): bool
    {
        return $this->operations->lockboxCreateParentDirs($this->handle, $path);
    }

    /** Stages an atomic move from one Lockbox path to another. */
    public function rename(string $from, string $to): bool
    {
        return $this->operations->lockboxRename($this->handle, $from, $to);
    }

    /** Lists entries below path, optionally including descendants. */
    public function list(string $path, bool $recursive): \Revault\LockboxEntryList
    {
        return $this->operations->lockboxList($this->handle, $path, $recursive);
    }

    /** Lists entries using glob, type, recursion, and result limit filters. */
    public function listWithOptions(string $path, string $glob, bool $recursive, bool $includeFiles, bool $includeSymlinks, bool $includeDirectories, int $limit): \Revault\LockboxEntryList
    {
        return $this->operations->lockboxListWithOptions($this->handle, $path, $glob, $recursive, $includeFiles, $includeSymlinks, $includeDirectories, $limit);
    }

    /** Returns metadata for the selected lockbox entry. */
    public function stat(string $path): \Revault\OptionalLockboxEntry
    {
        return $this->operations->lockboxStat($this->handle, $path);
    }

    /** Stages a plain text variable; call commit() to publish the change. */
    public function setVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetVariable($this->handle, $name, $value);
    }

    /** Stores a secret variable from binary-safe PHP string bytes. */
    public function setSecretVariable(string $name, string $value): bool
    {
        return $this->operations->lockboxSetSecretVariable($this->handle, $name, $value);
    }

    /** Returns a plain variable, or null when it is absent. */
    public function getVariable(string $name): ?string
    {
        $value = $this->operations->lockboxGetVariable($this->handle, $name);
        return $value->getPresent() ? $value->getValue() : null;
    }

    /** Returns the encrypted Lockbox description, or null when unset. Example: set it, commit, then print `$box->description()`. */
    public function description(): ?string
    {
        return $this->getVariable('/.revault/description');
    }

    /** Stages encrypted description text; call commit() to publish it. Example: `$box->setDescription('Production credentials'); $box->commit();`. */
    public function setDescription(string $description): bool
    {
        return $this->setVariable('/.revault/description', $description);
    }

    /** Stages removal of the encrypted description; call commit(). Example: `$box->clearDescription(); $box->commit();`. */
    public function clearDescription(): bool
    {
        return $this->deleteVariable('/.revault/description');
    }

    /** Invokes the callback with temporary secret bytes, then wipes the native transfer. */
    public function withSecretVariable(string $name, callable $callback): mixed
    {
        return $this->operations->lockboxWithSecretVariable($this->handle, $name, $callback);
    }

    /** Stages removal of a variable. */
    public function deleteVariable(string $name): bool
    {
        return $this->operations->lockboxDeleteVariable($this->handle, $name);
    }

    /** Atomically renames variables using source and destination path pairs. */
    public function moveVariables(array $moves): bool
    {
        return $this->operations->lockboxMoveVariables($this->handle, DomainCodec::encodePathMoves($moves));
    }

    /** Lists variable names and metadata without exposing secret values. */
    public function listVariables(): \Revault\VariableList
    {
        return $this->operations->lockboxListVariables($this->handle);
    }

    /** Returns whether a variable is plain or secret. */
    public function variableSensitivity(string $name): \Revault\OptionalString
    {
        return $this->operations->lockboxVariableSensitivity($this->handle, $name);
    }

    /** Stages a symbolic link with its stored target text. */
    public function addSymlink(string $path, string $target, bool $replace): bool
    {
        return $this->operations->lockboxAddSymlink($this->handle, $path, $target, $replace);
    }

    /** Returns the target text stored for a symbolic link. */
    public function getSymlinkTarget(string $path): string
    {
        return $this->operations->lockboxGetSymlinkTarget($this->handle, $path);
    }

    /** Returns the stable public identifier stored in the Lockbox header. */
    public function id(): string
    {
        return $this->operations->lockboxId($this->handle);
    }

    /** Reports whether an entry exists at path. */
    public function exists(string $path): bool
    {
        return $this->operations->lockboxExists($this->handle, $path);
    }

    /** Reports whether path names a directory entry. */
    public function isDir(string $path): bool
    {
        return $this->operations->lockboxIsDir($this->handle, $path);
    }

    /** Returns the portable Unix permission bits stored for path. */
    public function permissions(string $path): int
    {
        return $this->operations->lockboxPermissions($this->handle, $path);
    }

    /** Stages portable Unix permission bits for path. */
    public function setPermissions(string $path, int $permissions): bool
    {
        return $this->operations->lockboxSetPermissions($this->handle, $path, $permissions);
    }

    /** Reads len bytes from a file starting at its logical offset. */
    public function readRange(string $path, int $offset, int $len): string
    {
        return $this->operations->lockboxReadRange($this->handle, $path, $offset, $len);
    }

    /** Adds a password access slot and returns its slot identifier. */
    public function addPassword(string $password): int
    {
        return $this->operations->lockboxAddPassword($this->handle, $password);
    }

    /** Grants a named contact access and returns the new slot identifier. */
    public function addContact(OwnedHandle $contact, string $name): int
    {
        return $this->operations->lockboxAddContact($this->handle, $contact->nativeHandle(), $name);
    }

    /** Removes an access slot; at least one usable slot must remain. */
    public function deleteKey(int $id): bool
    {
        return $this->operations->lockboxDeleteKey($this->handle, $id);
    }

    /** Lists public access slot metadata without returning credentials. */
    public function listKeySlots(): \Revault\KeySlotList
    {
        return $this->operations->lockboxListKeySlots($this->handle);
    }

    /** Assigns a profile signing key to the Lockbox owner role. */
    public function setOwnerSigningKey(OwnedHandle $key): bool
    {
        return $this->operations->lockboxSetOwnerSigningKey($this->handle, $key->nativeHandle());
    }

    /** Returns public signing and ownership metadata for the current revision. */
    public function ownerInspection(): \Revault\OwnerInspection
    {
        return $this->operations->lockboxOwnerInspection($this->handle);
    }

    /** Defines a reusable, versioned form from the supplied field definitions. */
    public function defineForm(string $alias, string $name, string $description, array $fields): \Revault\FormDefinition
    {
        return $this->operations->lockboxDefineForm($this->handle, $alias, $name, $description, DomainCodec::encodeFormFields($fields));
    }

    /** Lists the form definitions stored in this Lockbox. */
    public function listFormDefinitions(): \Revault\FormDefinitionList
    {
        return $this->operations->lockboxListFormDefinitions($this->handle);
    }

    /** Resolves a form alias, type identifier, or revision. */
    public function resolveForm(string $reference): \Revault\FormDefinition
    {
        return $this->operations->lockboxResolveForm($this->handle, $reference);
    }

    /** Lists every stored revision for a form type identifier. */
    public function listFormRevisions(string $typeId): \Revault\FormDefinitionList
    {
        return $this->operations->lockboxListFormRevisions($this->handle, $typeId);
    }

    /** Stages a form record at path using the referenced definition. */
    public function createFormRecord(string $path, string $typeReference, string $name): \Revault\FormRecord
    {
        return $this->operations->lockboxCreateFormRecord($this->handle, $path, $typeReference, $name);
    }

    /** Stages a plain field value in a form record. */
    public function setFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetFormField($this->handle, $path, $field, $value);
    }

    /** Stores a secret form field from binary-safe PHP string bytes. */
    public function setSecretFormField(string $path, string $field, string $value): bool
    {
        return $this->operations->lockboxSetSecretFormField($this->handle, $path, $field, $value);
    }

    /** Lists form records without exposing secret field values. */
    public function listFormRecords(): \Revault\FormRecordList
    {
        return $this->operations->lockboxListFormRecords($this->handle);
    }

    /** Returns a form record when path exists. */
    public function getFormRecord(string $path): \Revault\OptionalFormRecord
    {
        return $this->operations->lockboxGetFormRecord($this->handle, $path);
    }

    /** Stages removal of a form record. */
    public function deleteFormRecord(string $path): bool
    {
        return $this->operations->lockboxDeleteFormRecord($this->handle, $path);
    }

    /** Atomically renames form records using source and destination path pairs. */
    public function moveFormRecords(array $moves): bool
    {
        return $this->operations->lockboxMoveFormRecords($this->handle, DomainCodec::encodePathMoves($moves));
    }

    /** Returns a plain form field when it exists. */
    public function getFormField(string $path, string $field): \Revault\OptionalFormValue
    {
        return $this->operations->lockboxGetFormField($this->handle, $path, $field);
    }

    /** Invokes the callback with temporary field bytes, then wipes the native transfer. */
    public function withSecretFormField(string $path, string $field, callable $callback): mixed
    {
        return $this->operations->lockboxWithSecretFormField($this->handle, $path, $field, $callback);
    }

    /** Serializes the current Lockbox, including committed changes. */
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

/** A profile's contact-encryption identity used to decrypt content keys addressed to it. */
class ContactKeyPair extends OwnedHandle
{

    /** Returns the canonical public bytes paired with this identity. */
    public function publicBytes(): string
    {
        return $this->operations->keyContactPublic($this->handle);
    }

    /** Returns the private signing-key record for secure binary backup. */
    public function privateRecord(): string
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

/** A recipient's shareable encryption identity used when granting lockbox access. */
class ContactPublicKey extends OwnedHandle
{

    /** Releases this public contact key. */
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

/** A content key encrypted for one contact and recoverable only by its matching key pair. */
class WrappedContactKey extends OwnedHandle
{

    /** Returns the ephemeral public key stored in this wrapped key. */
    public function public(): string
    {
        return $this->operations->keyContactWrappedPublic($this->handle);
    }

    /** Returns the encrypted content key bytes. */
    public function ciphertext(): string
    {
        return $this->operations->keyContactWrappedCiphertext($this->handle);
    }

    /** Returns the complete wrapped key record for storage or transport. */
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

/** A Vault Profile signing identity used to authorize mutable Lockbox revisions. */
class ProfileSigningKeyPair extends OwnedHandle
{

    /** Returns the canonical public bytes for this signing identity. */
    public function public(): string
    {
        return $this->operations->keySigningPublic($this->handle);
    }

    /** Returns the private signing key record for secure backup. */
    public function private(): string
    {
        return $this->operations->keySigningPrivate($this->handle);
    }

    /** Creates an independently owned public verification-key handle. */
    public function publicKey(): ProfileSigningPublicKey
    {
        return new ProfileSigningPublicKey(
            $this->operations,
            $this->operations->keySigningPublicFromBytes($this->public()),
        );
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->keySigningFree($this->handle);
    }

}

/** The public half of a Vault Profile signing identity. */
class ProfileSigningPublicKey extends OwnedHandle
{

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->keySigningPublicFree($this->handle);
    }

}

/** Password-protected storage for Profile keys, contacts, forms, backups, and known lockbox paths. */
class VaultStore extends OwnedHandle
{

    /** Returns the canonical root directory of this Vault. */
    public function root(): string
    {
        return $this->operations->vaultDirectoryRoot($this->handle);
    }

    /** Returns the persistent structure version of this Vault. */
    public function structureVersion(): int
    {
        return $this->operations->vaultDirectoryStructureVersion($this->handle);
    }

    /** Lists private keys. */
    public function listPrivateKeys(): \Revault\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeys($this->handle);
    }

    /** Lists private key names. */
    public function listPrivateKeyNames(): \Revault\StringList
    {
        return $this->operations->vaultDirectoryListPrivateKeyNames($this->handle);
    }

    /** Lists contact names. */
    public function listContactNames(): \Revault\StringList
    {
        return $this->operations->vaultDirectoryListContactNames($this->handle);
    }

    /** Lists form aliases. */
    public function listFormAliases(): \Revault\StringList
    {
        return $this->operations->vaultDirectoryListFormAliases($this->handle);
    }

    /** Reports whether the named profile private key exists. */
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

    /** Reports whether the named contact exists. */
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
    public function listContacts(): \Revault\ContactList
    {
        return $this->operations->vaultDirectoryListContacts($this->handle);
    }

    /** Stores profile email. */
    public function storeProfileEmail(string $name, string $email): bool
    {
        return $this->operations->vaultDirectoryStoreProfileEmail($this->handle, $name, $email);
    }

    /** Returns the email recorded for a profile, when present. */
    public function profileEmail(string $name): \Revault\OptionalString
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

    /** Returns the number of stored key recovery backups. */
    public function backupCount(): int
    {
        return $this->operations->vaultDirectoryBackupCount($this->handle);
    }

    /** Restores a profile private key and signing key from recovery material. */
    public function restorePrivateKey(string $name, OwnedHandle $key, OwnedHandle $signingKey, bool $overwrite): bool
    {
        return $this->operations->vaultDirectoryRestorePrivateKey($this->handle, $name, $key->nativeHandle(), $signingKey->nativeHandle(), $overwrite);
    }

    /** Loads owner signing key. */
    public function loadProfileSigningKey(string $name): ProfileSigningKeyPair
    {
        return new ProfileSigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKey($this->handle, $name));
    }

    /** Loads owner signing key generation. */
    public function loadProfileSigningKeyGeneration(string $name, int $index): ProfileSigningKeyPair
    {
        return new ProfileSigningKeyPair($this->operations, $this->operations->vaultDirectoryLoadOwnerSigningKeyGeneration($this->handle, $name, $index));
    }

    /** Stores contact signing key. */
    public function storeContactSigningKey(string $name, OwnedHandle $key): bool
    {
        return $this->operations->vaultDirectoryStoreContactSigningKey($this->handle, $name, $key->nativeHandle());
    }

    /** Loads contact signing key. */
    public function loadContactSigningKey(string $name): ProfileSigningPublicKey
    {
        return new ProfileSigningPublicKey($this->operations, $this->operations->vaultDirectoryLoadContactSigningKey($this->handle, $name));
    }

    /** Lists profile generations. */
    public function listProfileGenerations(string $name): \Revault\ProfileHistory
    {
        return $this->operations->vaultDirectoryListProfileGenerations($this->handle, $name);
    }

    /** Updates private key. */
    public function rotatePrivateKey(string $name): \Revault\ProfileHistory
    {
        return $this->operations->vaultDirectoryRotatePrivateKey($this->handle, $name);
    }

    /** Stores lockbox. */
    public function rememberLockbox(string $id, string $path): bool
    {
        return $this->operations->vaultDirectoryRememberLockbox($this->handle, $id, $path);
    }

    /** Lists known lockboxes. */
    public function listKnownLockboxes(): \Revault\KnownLockboxList
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
    public function listAccessSlotLabels(string $id): \Revault\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryListAccessSlotLabels($this->handle, $id);
    }

    /** Finds access slot labels with the supplied name for one Lockbox. */
    public function findAccessSlotLabels(string $id, string $name): \Revault\AccessSlotLabelList
    {
        return $this->operations->vaultDirectoryFindAccessSlotLabels($this->handle, $id, $name);
    }

    /** Removes access slot label. */
    public function forgetAccessSlotLabel(string $id, int $slotId): bool
    {
        return $this->operations->vaultDirectoryForgetAccessSlotLabel($this->handle, $id, $slotId);
    }

    /** Defines a reusable, versioned form in the local vault. */
    public function defineForm(string $alias, string $name, string $description, array $fields): \Revault\FormDefinition
    {
        return $this->operations->vaultDirectoryDefineForm($this->handle, $alias, $name, $description, DomainCodec::encodeFormFields($fields));
    }

    /** Resolves a Vault form alias, type identifier, or revision. */
    public function resolveForm(string $reference): \Revault\FormDefinition
    {
        return $this->operations->vaultDirectoryResolveForm($this->handle, $reference);
    }

    /** Lists forms. */
    public function listForms(): \Revault\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListForms($this->handle);
    }

    /** Lists form revisions. */
    public function listFormRevisions(string $typeId): \Revault\FormDefinitionList
    {
        return $this->operations->vaultDirectoryListFormRevisions($this->handle, $typeId);
    }

    /** Adds any missing standard form definitions and returns the number added. */
    public function seedForms(): int
    {
        return $this->operations->vaultDirectorySeedForms($this->handle);
    }

    /** Stores password. */
    public function rememberPassword(string $id, string $password): bool
    {
        return $this->operations->vaultDirectoryRememberPassword($this->handle, $id, $password);
    }

    /** Returns the Lockbox password encrypted inside this Vault. */
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

/** A metadata view for discovery and diagnostics that never loads an owner signing key. */
class ReadOnlyVault
{
    /** Adopts a read only Vault handle returned by Revault. */
    public function __construct(protected readonly BindingOperations $operations, protected CData $handle) {}

    /** Lists profile names. */
    public function listProfileNames(): \Revault\StringList
    {
        return $this->operations->vaultReadOnlyListProfileNames($this->handle);
    }

    /** Lists contact names. */
    public function listContactNames(): \Revault\StringList
    {
        return $this->operations->vaultReadOnlyListContactNames($this->handle);
    }

    /** Lists form aliases. */
    public function listFormAliases(): \Revault\StringList
    {
        return $this->operations->vaultReadOnlyListFormAliases($this->handle);
    }

    /** Lists known lockboxes. */
    public function listKnownLockboxes(): \Revault\KnownLockboxList
    {
        return $this->operations->vaultReadOnlyListKnownLockboxes($this->handle);
    }

    /** Releases the native resources held by this object. */
    public function free(): void
    {
        $this->operations->vaultReadOnlyFree($this->handle);
    }

    /** Release the read-only Vault handle; repeated close is safe. */
    public function close(): void { $this->free(); }

}

/** Client for the session service that temporarily caches vault unlock and signing keys. */
class Agent
{
    /** Creates a controller for the optional session agent. */
    public function __construct(protected readonly BindingOperations $operations) {}

    /** Reports whether the session agent is running. */
    public function isRunning(): bool
    {
        return $this->operations->vaultIsRunning();
    }

    /** Removes all. */
    public function forgetAll(): bool
    {
        return $this->operations->vaultForgetAll();
    }

    /** Runs the session agent server until it is stopped. */
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

    /** Lists the content keys currently cached by the session agent. */
    public function list(): \Revault\AgentEntryList
    {
        return $this->operations->vaultAgentList();
    }

    /** Reports how the platform handles agent expiry during system sleep. */
    public function sleepSupport(): \Revault\SleepSupport
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

    /** Returns the cached signing identity for a Vault Profile. */
    public function profileSigningKey(string $vaultId, string $profile): ProfileSigningKeyPair
    {
        return new ProfileSigningKeyPair($this->operations, $this->operations->vaultAgentGetOwnerSigningKey($vaultId, $profile));
    }

    /** Caches a signing identity for a Vault Profile. */
    public function cacheProfileSigningKey(string $vaultId, string $profile, OwnedHandle $key, int $ttlSeconds): bool
    {
        return $this->operations->vaultAgentPutOwnerSigningKey($vaultId, $profile, $key->nativeHandle(), $ttlSeconds);
    }

    /** Removes a cached signing identity for a Vault Profile. */
    public function forgetProfileSigningKey(string $vaultId, string $profile): bool
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

/** A token kept alive while an operation needs secrets cached by the Session Agent. */
class AgentActivity extends OwnedHandle
{

}

/** Access to the platform credential store for a scoped Vault passphrase. */
class Platform
{
    /** Creates a facade for the operating system credential store. */
    public function __construct(protected readonly BindingOperations $operations) {}

    /** Returns availability and user presence guarantees for platform storage. */
    public function status(): \Revault\PlatformStatus
    {
        return $this->operations->vaultPlatformStatus();
    }

    /** Selects the application scope used for platform credentials. */
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

    /** Enables storage of the Vault passphrase in platform credentials. */
    public function enable(): bool
    {
        return $this->operations->vaultPlatformEnable();
    }

    /** Disables platform credential use without deleting the stored value. */
    public function disable(): bool
    {
        return $this->operations->vaultPlatformDisable();
    }

    /** Reports whether platform credential use is disabled. */
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

/** A session that opens lockboxes by host path, caches passwords, and closes locally used files. */
class LocalSession extends OwnedHandle
{

    /** Creates Lockbox password. */
    public function createLockboxPassword(string $path, string $password): Lockbox
    {
        return new Lockbox($this->operations, $this->operations->vaultCreateLockboxPassword($this->handle, $path, $password));
    }

    /** Opens Lockbox password. */
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

    /** Stores Lockbox password. */
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

/**
 * Persistent encrypted local store for profiles, keys, contacts, forms, and
 * remembered Lockbox access.
 *
 * open() requires an existing Vault. openOrCreate() preserves existing state
 * and creates only when absent. create() and replace() deliberately replace
 * persistent state.
 */
final class Vault extends VaultStore
{
    /** Opens an existing Vault with its passphrase. */
    public static function open(string $root, string $vaultPassphrase): self
    { return Revault::load()->vaultDirectoryOpen($root, $vaultPassphrase); }
    /** Opens the Vault at root, creating it only when it does not exist. */
    public static function openOrCreate(string $root, string $vaultPassphrase): self
    { return Revault::load()->vaultDirectoryOpenOrCreate($root, $vaultPassphrase); }
    /** Creates a fresh Vault and replaces any existing state at root. */
    public static function create(string $root, string $vaultPassphrase): self
    { return Revault::load()->vaultDirectoryReplace($root, $vaultPassphrase); }
    /** Explicit destructive alias for create(). */
    public static function replace(string $root, string $vaultPassphrase): self
    { return Revault::load()->vaultDirectoryReplace($root, $vaultPassphrase); }
}

/** Explicit Session Agent controller; cached content keys are temporary. */
final class AgentSession extends Agent
{
    private CData $local;
    /** Attach an explicit session controller to the packaged runtime. */
    public function __construct(BindingOperations $operations)
    { parent::__construct($operations); $this->local = $operations->vaultLocal(); }
    /** Return the process-wide explicit session controller. */
    public static function instance(): self { static $instance; return $instance ??= Revault::agentSession(); }
    /** Remove one cached lockbox key. */
    public function closeLockbox(string $lockboxPath): bool { return $this->operations->vaultCloseLockbox($this->local, $lockboxPath); }
    /** Remove all cached lockbox keys. */
    public function closeAll(): bool { return $this->operations->vaultCloseAll($this->local); }
    /** Create a password-protected lockbox file through this session. */
    public function createLockboxPassword(string $path, string $password): Lockbox { return new Lockbox($this->operations, $this->operations->vaultCreateLockboxPassword($this->local, $path, $password)); }
    /** Open a password-protected lockbox file through this session. */
    public function openLockboxPassword(string $path, string $password): Lockbox { return new Lockbox($this->operations, $this->operations->vaultOpenLockboxPassword($this->local, $path, $password)); }
    /** Create a content-key lockbox file through this session. */
    public function createLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox { return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContentKey($this->local, $path, $contentKey, $signingKey->nativeHandle())); }
    /** Create a contact-addressed lockbox file through this session. */
    public function createLockboxContact(string $path, OwnedHandle $contact, string $name, OwnedHandle $signingKey): Lockbox { return new Lockbox($this->operations, $this->operations->vaultCreateLockboxContact($this->local, $path, $contact->nativeHandle(), $name, $signingKey->nativeHandle())); }
    /** Open a content-key lockbox file through this session. */
    public function openLockboxContentKey(string $path, string $contentKey, OwnedHandle $signingKey): Lockbox { return new Lockbox($this->operations, $this->operations->vaultOpenLockboxContentKey($this->local, $path, $contentKey, $signingKey->nativeHandle())); }
    /** Cache a password-derived key for the requested TTL. */
    public function cacheLockboxPassword(string $path, string $password, int $ttlSeconds): bool { return $this->operations->vaultCacheLockboxPassword($this->local, $path, $password, $ttlSeconds); }
    /** Release the local session handle. */
    public function close(): void { $this->operations->vaultFree($this->local); }
    /** Release the local session handle (legacy alias). */
    public function free(): void { $this->close(); }
}

/** A caller owned secret buffer that close() overwrites. */
class SecretBytes
{
    private string $bytes;
    /** Copy a secret into mutable storage that can be wiped explicitly. */
    public function __construct(string $bytes) { $this->bytes = $bytes; }
    /** Return a copy for one native call. */
    public function bytes(): string { return $this->bytes; }
    /** Wipe the stored bytes in place. */
    public function close(): void { $this->bytes = str_repeat("\0", strlen($this->bytes)); }
}
/** A UTF-8 secret stored in the same wipeable buffer as SecretBytes. */
final class SecretString extends SecretBytes {}
final class LockboxCacheMode { public const BYTES = 'bytes', DISABLED = 'disabled', AUTOMATIC = 'automatic'; }
final class LockboxWorkload { public const INTERACTIVE = 'interactive', BULK_IMPORT = 'bulk-import', READ_MOSTLY = 'read-mostly'; }
final class LockboxWorker { public const AUTO = 'auto', SINGLE = 'single', THREADS = 'threads'; }
final class AgentActivityKind { public const OPEN = 'open', CLOSE = 'close', VARIABLES = 'variables', FORM = 'form', RECOVERY = 'recovery', VAULT = 'vault'; }
final class KeyExportFormat { public const LOCKBOX_PEM = 'lockbox-pem', JWK = 'jwk', JWKS = 'jwks', RAW_HEX = 'raw-hex'; }
