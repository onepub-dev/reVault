<?php
declare(strict_types=1);

namespace Revault;

use FFI;
use FFI\CData;
use RuntimeException;

/** Complete binary operation layer generated from revault_api.h. */
final class BindingOperations
{
    public function __construct(private readonly FFI $ffi) { if ($ffi->api_abi_version() !== 2) { throw new RuntimeException('revault-api native ABI mismatch; expected 2'); } }

    public static function load(string $library): self
    {
        return new self(FFI::cdef(file_get_contents(__DIR__ . '/../revault_ffi.h'), $library));
    }

    public function lastErrorMessage(): string
    {
        $value = $this->ffi->buffer_last_error(); return is_string($value) ? $value : FFI::string($value);
    }

    private function requireBool(bool $value): bool
    {
        if (!$value) { throw new RuntimeException($this->lastErrorMessage()); }
        return true;
    }

    private function requireHandle(CData $value): CData
    {
        if (FFI::isNull($value)) { throw new RuntimeException($this->lastErrorMessage()); }
        return $value;
    }

    private function take(CData $value): string
    {
        if (FFI::isNull($value->ptr)) { throw new RuntimeException($this->lastErrorMessage()); }
        try { return FFI::string($value->ptr, $value->len); }
        finally { $this->ffi->buffer_free($value); }
    }

    private function payload(CData $value): string
    {
        $frame = $this->take($value);
        if (strlen($frame) < 12 || substr($frame, 0, 4) !== 'LBWF') { throw new RuntimeException('invalid reVault binding frame'); }
        $length = unpack('Nlength', substr($frame, 8, 4))['length'];
        if ($length !== strlen($frame) - 12) { throw new RuntimeException('invalid reVault binding frame length'); }
        return substr($frame, 12);
    }

    private function decode(string $class, CData $value): object
    {
        $result = new $class(); $result->mergeFromString($this->payload($value)); return $result;
    }

    private function withBytes(string $value, callable $callback): mixed
    {
        $length = strlen($value); $bytes = $this->ffi->new('uint8_t[' . max(1, $length) . ']');
        if ($length > 0) { FFI::memcpy($bytes, $value, $length); }
        return $callback($bytes, $length);
    }

    private function withText(string $value, callable $callback): mixed
    {
        $length = strlen($value); $bytes = $this->ffi->new('char[' . max(1, $length) . ']');
        if ($length > 0) { FFI::memcpy($bytes, $value, $length); }
        return $callback($bytes, $length);
    }

    private function withSecretInput(string $value, callable $callback): mixed
    {
        return $this->withBytes($value, function (CData $bytes, int $length) use ($callback): mixed {
            try { return $callback($bytes, $length); }
            finally { FFI::memset($bytes, 0, max(1, $length)); }
        });
    }

    /** The callback receives mutable C bytes valid only for the duration of the call. */
    private function withSecret(callable $getter, callable $callback): mixed
    {
        $output = $this->ffi->new('void *');
        $this->requireBool((bool) $getter(FFI::addr($output)));
        if (FFI::isNull($output)) { return null; }
        try {
            $length = $this->ffi->new('size_t');
            $this->requireBool((bool) $this->ffi->secret_len($output, FFI::addr($length)));
            $bytes = $this->ffi->new('uint8_t[' . max(1, (int) $length->cdata) . ']');
            try {
                $this->requireBool((bool) $this->ffi->secret_copy($output, $bytes, $length->cdata));
                return $callback($bytes, (int) $length->cdata);
            } finally { FFI::memset($bytes, 0, max(1, (int) $length->cdata)); }
        } finally { $this->ffi->secret_free($output); }
    }

    public function freeBuffer(CData $value): void { $this->ffi->buffer_free($value); }

    public function bufferLastErrorDetails(): \Revault\Bindings\ErrorDetails
    {
        return $this->decode(\Revault\Bindings\ErrorDetails::class, $this->ffi->buffer_last_error_details());
    }

    public function lockboxFormatVersion(): int
    {
        return $this->ffi->lockbox_format_version();
    }

    public function lockboxProbeFormatVersion(string $bytes): int
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->ffi->lockbox_probe_format_version($bytesPointer, $bytesLength));
    }

    public function lockboxCreate(string $key): CData
    {
        return $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireHandle($this->ffi->lockbox_create($keyPointer, $keyLength)));
    }

    public function lockboxCreateWithOptions(string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): CData
    {
        return $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireHandle($this->ffi->lockbox_create_with_options($keyPointer, $keyLength, $cacheMode, strlen($cacheMode), $cacheBytes, $workload, strlen($workload), $worker, strlen($worker), $jobs)));
    }

    public function lockboxCreatePassword(string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->lockbox_create_password($passwordPointer, $passwordLength)));
    }

    public function lockboxCreateContact(CData $contact): CData
    {
        return $this->requireHandle($this->ffi->lockbox_create_contact($contact));
    }

    public function lockboxCreateWithSigningKey(string $contentKey, CData $signingKey): CData
    {
        return $this->withBytes($contentKey, fn(CData $contentKeyPointer, int $contentKeyLength) => $this->requireHandle($this->ffi->lockbox_create_with_signing_key($contentKeyPointer, $contentKeyLength, $signingKey)));
    }

    public function lockboxOpen(string $archive, string $key): CData
    {
        return $this->withBytes($archive, fn(CData $archivePointer, int $archiveLength) => $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireHandle($this->ffi->lockbox_open($archivePointer, $archiveLength, $keyPointer, $keyLength))));
    }

    public function lockboxOpenWithOptions(string $archive, string $key, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): CData
    {
        return $this->withBytes($archive, fn(CData $archivePointer, int $archiveLength) => $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireHandle($this->ffi->lockbox_open_with_options($archivePointer, $archiveLength, $keyPointer, $keyLength, $cacheMode, strlen($cacheMode), $cacheBytes, $workload, strlen($workload), $worker, strlen($worker), $jobs))));
    }

    public function lockboxOpenPassword(string $archive, string $password): CData
    {
        return $this->withBytes($archive, fn(CData $archivePointer, int $archiveLength) => $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->lockbox_open_password($archivePointer, $archiveLength, $passwordPointer, $passwordLength))));
    }

    public function lockboxOpenContact(string $archive, CData $contact): CData
    {
        return $this->withBytes($archive, fn(CData $archivePointer, int $archiveLength) => $this->requireHandle($this->ffi->lockbox_open_contact($archivePointer, $archiveLength, $contact)));
    }

    public function lockboxAddFile(CData $handle, string $path, string $data, bool $replace): bool
    {
        return $this->withBytes($data, fn(CData $dataPointer, int $dataLength) => $this->requireBool((bool) ($this->ffi->lockbox_add_file($handle, $path, strlen($path), $dataPointer, $dataLength, $replace))));
    }

    public function lockboxAddFileWithPermissions(CData $handle, string $path, string $data, int $permissions, bool $replace): bool
    {
        return $this->withBytes($data, fn(CData $dataPointer, int $dataLength) => $this->requireBool((bool) ($this->ffi->lockbox_add_file_with_permissions($handle, $path, strlen($path), $dataPointer, $dataLength, $permissions, $replace))));
    }

    public function lockboxGetFile(CData $handle, string $path): string
    {
        return $this->take($this->ffi->lockbox_get_file($handle, $path, strlen($path)));
    }

    public function lockboxExtractFile(CData $handle, string $source, string $destination, bool $replace): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_extract_file($handle, $source, strlen($source), $destination, strlen($destination), $replace)));
    }

    public function lockboxExtractDirectory(CData $handle, string $destination, int $maxFileBytes, int $maxTotalBytes, int $maxFiles, bool $restoreSymlinks, bool $restorePermissions, bool $overwrite): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_extract_directory($handle, $destination, strlen($destination), $maxFileBytes, $maxTotalBytes, $maxFiles, $restoreSymlinks, $restorePermissions, $overwrite)));
    }

    public function lockboxStreamContent(CData $handle, bool $physical): \Revault\Bindings\StreamChunkList
    {
        return $this->decode(\Revault\Bindings\StreamChunkList::class, $this->ffi->lockbox_stream_content($handle, $physical));
    }

    public function lockboxCacheStats(CData $handle): \Revault\Bindings\CacheStats
    {
        return $this->decode(\Revault\Bindings\CacheStats::class, $this->ffi->lockbox_cache_stats($handle));
    }

    public function lockboxImportStats(CData $handle): \Revault\Bindings\ImportStats
    {
        return $this->decode(\Revault\Bindings\ImportStats::class, $this->ffi->lockbox_import_stats($handle));
    }

    public function lockboxResetImportStats(CData $handle): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_reset_import_stats($handle)));
    }

    public function lockboxInspectFile(string $path): \Revault\Bindings\FileInspection
    {
        return $this->decode(\Revault\Bindings\FileInspection::class, $this->ffi->lockbox_inspect_file($path, strlen($path)));
    }

    public function lockboxPageInspection(CData $handle): \Revault\Bindings\PageInspectionList
    {
        return $this->decode(\Revault\Bindings\PageInspectionList::class, $this->ffi->lockbox_page_inspection($handle));
    }

    public function lockboxRecoveryReport(CData $handle): \Revault\Bindings\RecoveryReport
    {
        return $this->decode(\Revault\Bindings\RecoveryReport::class, $this->ffi->lockbox_recovery_report($handle));
    }

    public function lockboxRecoveryReportRender(CData $handle, bool $verbose, int $maxEntries): string
    {
        return $this->take($this->ffi->lockbox_recovery_report_render($handle, $verbose, $maxEntries));
    }

    public function lockboxRecoveryScanPath(string $path, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->decode(\Revault\Bindings\RecoveryReport::class, $this->ffi->lockbox_recovery_scan_path($path, strlen($path), $keyPointer, $keyLength)));
    }

    public function lockboxStorageLen(CData $handle): int
    {
        return $this->ffi->lockbox_storage_len($handle);
    }

    public function lockboxSetWorkloadProfile(CData $handle, string $profile): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_workload_profile($handle, $profile, strlen($profile))));
    }

    public function lockboxSetWorkerPolicy(CData $handle, string $mode, int $jobs): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_worker_policy($handle, $mode, strlen($mode), $jobs)));
    }

    public function lockboxRuntimeOptions(CData $handle): \Revault\Bindings\RuntimeOptions
    {
        return $this->decode(\Revault\Bindings\RuntimeOptions::class, $this->ffi->lockbox_runtime_options($handle));
    }

    public function lockboxCommit(CData $handle): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_commit($handle)));
    }

    public function lockboxCreateDir(CData $handle, string $path, bool $createParents): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_create_dir($handle, $path, strlen($path), $createParents)));
    }

    public function lockboxDelete(CData $handle, string $path): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_delete($handle, $path, strlen($path))));
    }

    public function lockboxRemoveDir(CData $handle, string $path, bool $recursive): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_remove_dir($handle, $path, strlen($path), $recursive)));
    }

    public function lockboxCreateParentDirs(CData $handle, string $path): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_create_parent_dirs($handle, $path, strlen($path))));
    }

    public function lockboxRename(CData $handle, string $from, string $to): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_rename($handle, $from, strlen($from), $to, strlen($to))));
    }

    public function lockboxList(CData $handle, string $path, bool $recursive): \Revault\Bindings\LockboxEntryList
    {
        return $this->decode(\Revault\Bindings\LockboxEntryList::class, $this->ffi->lockbox_list($handle, $path, strlen($path), $recursive));
    }

    public function lockboxListWithOptions(CData $handle, string $path, string $glob, bool $recursive, bool $includeFiles, bool $includeSymlinks, bool $includeDirectories, int $limit): \Revault\Bindings\LockboxEntryList
    {
        return $this->decode(\Revault\Bindings\LockboxEntryList::class, $this->ffi->lockbox_list_with_options($handle, $path, strlen($path), $glob, strlen($glob), $recursive, $includeFiles, $includeSymlinks, $includeDirectories, $limit));
    }

    public function lockboxStat(CData $handle, string $path): \Revault\Bindings\OptionalLockboxEntry
    {
        return $this->decode(\Revault\Bindings\OptionalLockboxEntry::class, $this->ffi->lockbox_stat($handle, $path, strlen($path)));
    }

    public function lockboxSetVariable(CData $handle, string $name, string $value): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_variable($handle, $name, strlen($name), $value, strlen($value))));
    }

    public function lockboxSetSecretVariable(CData $handle, string $name, string $value): bool
    {
        return $this->withSecretInput($value, fn(CData $bytes, int $length): bool => $this->requireBool((bool) $this->ffi->lockbox_set_secret_variable($handle, $name, strlen($name), $bytes, $length)));
    }

    public function lockboxGetVariable(CData $handle, string $name): RevaultBindingsOptionalString
    {
        return $this->decode(RevaultBindingsOptionalString::class, $this->ffi->lockbox_get_variable($handle, $name, strlen($name)));
    }

    public function lockboxWithSecretVariable(CData $handle, string $name, callable $callback): mixed
    {
        return $this->withSecret(fn(CData $output): bool => (bool) $this->ffi->lockbox_get_secret_variable($handle, $name, strlen($name), $output), $callback);
    }

    public function lockboxDeleteVariable(CData $handle, string $name): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_delete_variable($handle, $name, strlen($name))));
    }

    public function lockboxMoveVariables(CData $handle, string $movesProto): bool
    {
        return $this->withBytes($movesProto, fn(CData $movesProtoPointer, int $movesProtoLength) => $this->requireBool((bool) ($this->ffi->lockbox_move_variables($handle, $movesProtoPointer, $movesProtoLength))));
    }

    public function lockboxListVariables(CData $handle): \Revault\Bindings\VariableList
    {
        return $this->decode(\Revault\Bindings\VariableList::class, $this->ffi->lockbox_list_variables($handle));
    }

    public function lockboxVariableSensitivity(CData $handle, string $name): \Revault\Bindings\OptionalString
    {
        return $this->decode(\Revault\Bindings\OptionalString::class, $this->ffi->lockbox_variable_sensitivity($handle, $name, strlen($name)));
    }

    public function lockboxAddSymlink(CData $handle, string $path, string $target, bool $replace): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_add_symlink($handle, $path, strlen($path), $target, strlen($target), $replace)));
    }

    public function lockboxGetSymlinkTarget(CData $handle, string $path): string
    {
        return $this->take($this->ffi->lockbox_get_symlink_target($handle, $path, strlen($path)));
    }

    public function lockboxId(CData $handle): string
    {
        return $this->take($this->ffi->lockbox_id($handle));
    }

    public function lockboxExists(CData $handle, string $path): bool
    {
        return (bool) ($this->ffi->lockbox_exists($handle, $path, strlen($path)));
    }

    public function lockboxIsDir(CData $handle, string $path): bool
    {
        return (bool) ($this->ffi->lockbox_is_dir($handle, $path, strlen($path)));
    }

    public function lockboxPermissions(CData $handle, string $path): int
    {
        return $this->ffi->lockbox_permissions($handle, $path, strlen($path));
    }

    public function lockboxSetPermissions(CData $handle, string $path, int $permissions): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_permissions($handle, $path, strlen($path), $permissions)));
    }

    public function lockboxReadRange(CData $handle, string $path, int $offset, int $len): string
    {
        return $this->take($this->ffi->lockbox_read_range($handle, $path, strlen($path), $offset, $len));
    }

    public function lockboxRecoveryScan(string $bytes, string $key): \Revault\Bindings\RecoveryReport
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->decode(\Revault\Bindings\RecoveryReport::class, $this->ffi->lockbox_recovery_scan($bytesPointer, $bytesLength, $keyPointer, $keyLength))));
    }

    public function lockboxRecoverySalvage(string $bytes, string $key, CData $signingKey): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireHandle($this->ffi->lockbox_recovery_salvage($bytesPointer, $bytesLength, $keyPointer, $keyLength, $signingKey))));
    }

    public function lockboxAddPassword(CData $handle, string $password): int
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->ffi->lockbox_add_password($handle, $passwordPointer, $passwordLength));
    }

    public function lockboxAddContact(CData $handle, CData $contact, string $name): int
    {
        return $this->ffi->lockbox_add_contact($handle, $contact, $name, strlen($name));
    }

    public function lockboxDeleteKey(CData $handle, int $id): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_delete_key($handle, $id)));
    }

    public function lockboxListKeySlots(CData $handle): \Revault\Bindings\KeySlotList
    {
        return $this->decode(\Revault\Bindings\KeySlotList::class, $this->ffi->lockbox_list_key_slots($handle));
    }

    public function lockboxSetOwnerSigningKey(CData $handle, CData $key): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_owner_signing_key($handle, $key)));
    }

    public function lockboxOwnerInspection(CData $handle): \Revault\Bindings\OwnerInspection
    {
        return $this->decode(\Revault\Bindings\OwnerInspection::class, $this->ffi->lockbox_owner_inspection($handle));
    }

    public function lockboxDefineForm(CData $handle, string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->withBytes($fieldsProto, fn(CData $fieldsProtoPointer, int $fieldsProtoLength) => $this->decode(\Revault\Bindings\FormDefinition::class, $this->ffi->lockbox_define_form($handle, $alias, strlen($alias), $name, strlen($name), $description, strlen($description), $fieldsProtoPointer, $fieldsProtoLength)));
    }

    public function lockboxListFormDefinitions(CData $handle): \Revault\Bindings\FormDefinitionList
    {
        return $this->decode(\Revault\Bindings\FormDefinitionList::class, $this->ffi->lockbox_list_form_definitions($handle));
    }

    public function lockboxResolveForm(CData $handle, string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->decode(\Revault\Bindings\FormDefinition::class, $this->ffi->lockbox_resolve_form($handle, $reference, strlen($reference)));
    }

    public function lockboxListFormRevisions(CData $handle, string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->decode(\Revault\Bindings\FormDefinitionList::class, $this->ffi->lockbox_list_form_revisions($handle, $typeId, strlen($typeId)));
    }

    public function lockboxCreateFormRecord(CData $handle, string $path, string $typeReference, string $name): \Revault\Bindings\FormRecord
    {
        return $this->decode(\Revault\Bindings\FormRecord::class, $this->ffi->lockbox_create_form_record($handle, $path, strlen($path), $typeReference, strlen($typeReference), $name, strlen($name)));
    }

    public function lockboxSetFormField(CData $handle, string $path, string $field, string $value): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_set_form_field($handle, $path, strlen($path), $field, strlen($field), $value, strlen($value))));
    }

    public function lockboxSetSecretFormField(CData $handle, string $path, string $field, string $value): bool
    {
        return $this->withSecretInput($value, fn(CData $bytes, int $length): bool => $this->requireBool((bool) $this->ffi->lockbox_set_secret_form_field($handle, $path, strlen($path), $field, strlen($field), $bytes, $length)));
    }

    public function lockboxListFormRecords(CData $handle): \Revault\Bindings\FormRecordList
    {
        return $this->decode(\Revault\Bindings\FormRecordList::class, $this->ffi->lockbox_list_form_records($handle));
    }

    public function lockboxGetFormRecord(CData $handle, string $path): \Revault\Bindings\OptionalFormRecord
    {
        return $this->decode(\Revault\Bindings\OptionalFormRecord::class, $this->ffi->lockbox_get_form_record($handle, $path, strlen($path)));
    }

    public function lockboxDeleteFormRecord(CData $handle, string $path): bool
    {
        return $this->requireBool((bool) ($this->ffi->lockbox_delete_form_record($handle, $path, strlen($path))));
    }

    public function lockboxMoveFormRecords(CData $handle, string $movesProto): bool
    {
        return $this->withBytes($movesProto, fn(CData $movesProtoPointer, int $movesProtoLength) => $this->requireBool((bool) ($this->ffi->lockbox_move_form_records($handle, $movesProtoPointer, $movesProtoLength))));
    }

    public function lockboxGetFormField(CData $handle, string $path, string $field): \Revault\Bindings\OptionalFormValue
    {
        return $this->decode(\Revault\Bindings\OptionalFormValue::class, $this->ffi->lockbox_get_form_field($handle, $path, strlen($path), $field, strlen($field)));
    }

    public function lockboxWithSecretFormField(CData $handle, string $path, string $field, callable $callback): mixed
    {
        return $this->withSecret(fn(CData $output): bool => (bool) $this->ffi->lockbox_get_secret_form_field($handle, $path, strlen($path), $field, strlen($field), $output), $callback);
    }

    public function lockboxToBytes(CData $handle): string
    {
        return $this->take($this->ffi->lockbox_to_bytes($handle));
    }

    public function lockboxFree(CData $handle): void
    {
        $this->ffi->lockbox_free($handle);
    }

    public function vaultIsRunning(): bool
    {
        return (bool) ($this->ffi->vault_is_running());
    }

    public function vaultForgetAll(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_forget_all()));
    }

    public function keyContactGenerate(): CData
    {
        return $this->requireHandle($this->ffi->key_contact_generate());
    }

    public function keyContactFromPrivate(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->key_contact_from_private($bytesPointer, $bytesLength)));
    }

    public function keyContactPublic(CData $handle): string
    {
        return $this->take($this->ffi->key_contact_public($handle));
    }

    public function keyContactPrivate(CData $handle): string
    {
        return $this->take($this->ffi->key_contact_private($handle));
    }

    public function keyContactPublicFromBytes(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->key_contact_public_from_bytes($bytesPointer, $bytesLength)));
    }

    public function keyContactPublicFree(CData $handle): void
    {
        $this->ffi->key_contact_public_free($handle);
    }

    public function keyContactFree(CData $handle): void
    {
        $this->ffi->key_contact_free($handle);
    }

    public function keyContactEncrypt(CData $contact, string $contentKey): CData
    {
        return $this->withBytes($contentKey, fn(CData $contentKeyPointer, int $contentKeyLength) => $this->requireHandle($this->ffi->key_contact_encrypt($contact, $contentKeyPointer, $contentKeyLength)));
    }

    public function keyContactDecrypt(CData $contact, CData $wrapped): string
    {
        return $this->take($this->ffi->key_contact_decrypt($contact, $wrapped));
    }

    public function keyContactWrappedPublic(CData $wrapped): string
    {
        return $this->take($this->ffi->key_contact_wrapped_public($wrapped));
    }

    public function keyContactWrappedCiphertext(CData $wrapped): string
    {
        return $this->take($this->ffi->key_contact_wrapped_ciphertext($wrapped));
    }

    public function keyContactWrappedEncrypted(CData $wrapped): string
    {
        return $this->take($this->ffi->key_contact_wrapped_encrypted($wrapped));
    }

    public function keyContactWrappedFree(CData $handle): void
    {
        $this->ffi->key_contact_wrapped_free($handle);
    }

    public function keySigningGenerate(): CData
    {
        return $this->requireHandle($this->ffi->key_signing_generate());
    }

    public function keySigningFromPrivate(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->key_signing_from_private($bytesPointer, $bytesLength)));
    }

    public function keySigningPublic(CData $handle): string
    {
        return $this->take($this->ffi->key_signing_public($handle));
    }

    public function keySigningPrivate(CData $handle): string
    {
        return $this->take($this->ffi->key_signing_private($handle));
    }

    public function keySigningPublicFromBytes(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->key_signing_public_from_bytes($bytesPointer, $bytesLength)));
    }

    public function keySigningPublicFree(CData $handle): void
    {
        $this->ffi->key_signing_public_free($handle);
    }

    public function keySigningFree(CData $handle): void
    {
        $this->ffi->key_signing_free($handle);
    }

    public function vaultKeyExportPrivate(CData $key, string $format): string
    {
        return $this->take($this->ffi->vault_key_export_private($key, $format, strlen($format)));
    }

    public function vaultKeyExportPublic(CData $key, string $format): string
    {
        return $this->take($this->ffi->vault_key_export_public($key, $format, strlen($format)));
    }

    public function vaultKeyImportPrivate(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->vault_key_import_private($bytesPointer, $bytesLength)));
    }

    public function vaultKeyImportPublic(string $bytes): CData
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireHandle($this->ffi->vault_key_import_public($bytesPointer, $bytesLength)));
    }

    public function vaultKeyFingerprint(CData $key): string
    {
        return $this->take($this->ffi->vault_key_fingerprint($key));
    }

    public function vaultKeyFormatHex(string $bytes): string
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->take($this->ffi->vault_key_format_hex($bytesPointer, $bytesLength)));
    }

    public function vaultKeyDecodeHex(string $text): string
    {
        return $this->take($this->ffi->vault_key_decode_hex($text, strlen($text)));
    }

    public function vaultKeyFormatCrockford(string $bytes): string
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->take($this->ffi->vault_key_format_crockford($bytesPointer, $bytesLength)));
    }

    public function vaultKeyFormatCrockfordReading(string $code): string
    {
        return $this->take($this->ffi->vault_key_format_crockford_reading($code, strlen($code)));
    }

    public function vaultKeyDecodeCrockford(string $code): string
    {
        return $this->take($this->ffi->vault_key_decode_crockford($code, strlen($code)));
    }

    public function vaultKeyHexEncode(string $bytes): string
    {
        return $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->take($this->ffi->vault_key_hex_encode($bytesPointer, $bytesLength)));
    }

    public function vaultKeyHexDecode(string $text): string
    {
        return $this->take($this->ffi->vault_key_hex_decode($text, strlen($text)));
    }

    public function vaultDirectoryOpen(string $root, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_directory_open($root, strlen($root), $passwordPointer, $passwordLength)));
    }

    public function vaultStructureVersionCurrent(): int
    {
        return $this->ffi->vault_structure_version_current();
    }

    public function vaultDirectoryProbeStructureVersion(string $root, string $password): int
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->ffi->vault_directory_probe_structure_version($root, strlen($root), $passwordPointer, $passwordLength));
    }

    public function vaultDirectoryOpenOrCreateDefault(string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_directory_open_or_create_default($passwordPointer, $passwordLength)));
    }

    public function vaultDirectoryReplaceDefault(string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_directory_replace_default($passwordPointer, $passwordLength)));
    }

    public function vaultDirectoryChangePassword(string $root, string $oldPassword, string $newPassword): bool
    {
        return $this->withBytes($oldPassword, fn(CData $oldPasswordPointer, int $oldPasswordLength) => $this->withBytes($newPassword, fn(CData $newPasswordPointer, int $newPasswordLength) => $this->requireBool((bool) ($this->ffi->vault_directory_change_password($root, strlen($root), $oldPasswordPointer, $oldPasswordLength, $newPasswordPointer, $newPasswordLength)))));
    }

    public function vaultDirectoryChangeDefaultPassword(string $oldPassword, string $newPassword): bool
    {
        return $this->withBytes($oldPassword, fn(CData $oldPasswordPointer, int $oldPasswordLength) => $this->withBytes($newPassword, fn(CData $newPasswordPointer, int $newPasswordLength) => $this->requireBool((bool) ($this->ffi->vault_directory_change_default_password($oldPasswordPointer, $oldPasswordLength, $newPasswordPointer, $newPasswordLength)))));
    }

    public function vaultDirectoryReplace(string $root, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_directory_replace($root, strlen($root), $passwordPointer, $passwordLength)));
    }

    public function vaultDirectoryOpenOrCreate(string $root, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_directory_open_or_create($root, strlen($root), $passwordPointer, $passwordLength)));
    }

    public function vaultDirectoryRoot(CData $handle): string
    {
        return $this->take($this->ffi->vault_directory_root($handle));
    }

    public function vaultDirectoryStructureVersion(CData $handle): int
    {
        return $this->ffi->vault_directory_structure_version($handle);
    }

    public function vaultDirectoryListPrivateKeys(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_directory_list_private_keys($handle));
    }

    public function vaultDirectoryListPrivateKeyNames(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_directory_list_private_key_names($handle));
    }

    public function vaultDirectoryListContactNames(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_directory_list_contact_names($handle));
    }

    public function vaultDirectoryListFormAliases(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_directory_list_form_aliases($handle));
    }

    public function vaultDirectoryPrivateKeyExists(CData $handle, string $name): bool
    {
        return (bool) ($this->ffi->vault_directory_private_key_exists($handle, $name, strlen($name)));
    }

    public function vaultDirectoryDeletePrivateKey(CData $handle, string $name): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_delete_private_key($handle, $name, strlen($name))));
    }

    public function vaultDirectoryStorePrivateKey(CData $handle, string $name, CData $key): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_store_private_key($handle, $name, strlen($name), $key)));
    }

    public function vaultDirectoryLoadPrivateKey(CData $handle, string $name): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_private_key($handle, $name, strlen($name)));
    }

    public function vaultDirectoryLoadPrivateKeyGeneration(CData $handle, string $name, int $index): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_private_key_generation($handle, $name, strlen($name), $index));
    }

    public function vaultDirectoryStoreContact(CData $handle, string $name, CData $key): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_store_contact($handle, $name, strlen($name), $key)));
    }

    public function vaultDirectoryLoadContact(CData $handle, string $name): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_contact($handle, $name, strlen($name)));
    }

    public function vaultDirectoryContactExists(CData $handle, string $name): bool
    {
        return (bool) ($this->ffi->vault_directory_contact_exists($handle, $name, strlen($name)));
    }

    public function vaultDirectoryDeleteContact(CData $handle, string $name): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_delete_contact($handle, $name, strlen($name))));
    }

    public function vaultDirectoryListContacts(CData $handle): \Revault\Bindings\ContactList
    {
        return $this->decode(\Revault\Bindings\ContactList::class, $this->ffi->vault_directory_list_contacts($handle));
    }

    public function vaultDirectoryStoreProfileEmail(CData $handle, string $name, string $email): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_store_profile_email($handle, $name, strlen($name), $email, strlen($email))));
    }

    public function vaultDirectoryProfileEmail(CData $handle, string $name): \Revault\Bindings\OptionalString
    {
        return $this->decode(\Revault\Bindings\OptionalString::class, $this->ffi->vault_directory_profile_email($handle, $name, strlen($name)));
    }

    public function vaultDirectoryStoreBackup(CData $handle, string $id, string $bytes): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->withBytes($bytes, fn(CData $bytesPointer, int $bytesLength) => $this->requireBool((bool) ($this->ffi->vault_directory_store_backup($handle, $idPointer, $idLength, $bytesPointer, $bytesLength)))));
    }

    public function vaultDirectoryLoadBackup(CData $handle, string $id): string
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->take($this->ffi->vault_directory_load_backup($handle, $idPointer, $idLength)));
    }

    public function vaultDirectoryBackupCount(CData $handle): int
    {
        return $this->ffi->vault_directory_backup_count($handle);
    }

    public function vaultDirectoryRestorePrivateKey(CData $handle, string $name, CData $key, CData $signingKey, bool $overwrite): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_restore_private_key($handle, $name, strlen($name), $key, $signingKey, $overwrite)));
    }

    public function vaultDirectoryLoadOwnerSigningKey(CData $handle, string $name): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_owner_signing_key($handle, $name, strlen($name)));
    }

    public function vaultDirectoryLoadOwnerSigningKeyGeneration(CData $handle, string $name, int $index): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_owner_signing_key_generation($handle, $name, strlen($name), $index));
    }

    public function vaultDirectoryStoreContactSigningKey(CData $handle, string $name, CData $key): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_store_contact_signing_key($handle, $name, strlen($name), $key)));
    }

    public function vaultDirectoryLoadContactSigningKey(CData $handle, string $name): CData
    {
        return $this->requireHandle($this->ffi->vault_directory_load_contact_signing_key($handle, $name, strlen($name)));
    }

    public function vaultDirectoryListProfileGenerations(CData $handle, string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->decode(\Revault\Bindings\ProfileHistory::class, $this->ffi->vault_directory_list_profile_generations($handle, $name, strlen($name)));
    }

    public function vaultDirectoryRotatePrivateKey(CData $handle, string $name): \Revault\Bindings\ProfileHistory
    {
        return $this->decode(\Revault\Bindings\ProfileHistory::class, $this->ffi->vault_directory_rotate_private_key($handle, $name, strlen($name)));
    }

    public function vaultDirectoryRememberLockbox(CData $handle, string $id, string $path): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->requireBool((bool) ($this->ffi->vault_directory_remember_lockbox($handle, $idPointer, $idLength, $path, strlen($path)))));
    }

    public function vaultDirectoryListKnownLockboxes(CData $handle): \Revault\Bindings\KnownLockboxList
    {
        return $this->decode(\Revault\Bindings\KnownLockboxList::class, $this->ffi->vault_directory_list_known_lockboxes($handle));
    }

    public function vaultDirectoryForgetLockbox(CData $handle, string $path): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_directory_forget_lockbox($handle, $path, strlen($path))));
    }

    public function vaultDirectoryRememberAccessSlotLabel(CData $handle, string $id, int $slotId, string $name): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->requireBool((bool) ($this->ffi->vault_directory_remember_access_slot_label($handle, $idPointer, $idLength, $slotId, $name, strlen($name)))));
    }

    public function vaultDirectoryListAccessSlotLabels(CData $handle, string $id): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->decode(\Revault\Bindings\AccessSlotLabelList::class, $this->ffi->vault_directory_list_access_slot_labels($handle, $idPointer, $idLength)));
    }

    public function vaultDirectoryFindAccessSlotLabels(CData $handle, string $id, string $name): \Revault\Bindings\AccessSlotLabelList
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->decode(\Revault\Bindings\AccessSlotLabelList::class, $this->ffi->vault_directory_find_access_slot_labels($handle, $idPointer, $idLength, $name, strlen($name))));
    }

    public function vaultDirectoryForgetAccessSlotLabel(CData $handle, string $id, int $slotId): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->requireBool((bool) ($this->ffi->vault_directory_forget_access_slot_label($handle, $idPointer, $idLength, $slotId))));
    }

    public function vaultDirectoryDefineForm(CData $handle, string $alias, string $name, string $description, string $fieldsProto): \Revault\Bindings\FormDefinition
    {
        return $this->withBytes($fieldsProto, fn(CData $fieldsProtoPointer, int $fieldsProtoLength) => $this->decode(\Revault\Bindings\FormDefinition::class, $this->ffi->vault_directory_define_form($handle, $alias, strlen($alias), $name, strlen($name), $description, strlen($description), $fieldsProtoPointer, $fieldsProtoLength)));
    }

    public function vaultDirectoryResolveForm(CData $handle, string $reference): \Revault\Bindings\FormDefinition
    {
        return $this->decode(\Revault\Bindings\FormDefinition::class, $this->ffi->vault_directory_resolve_form($handle, $reference, strlen($reference)));
    }

    public function vaultDirectoryListForms(CData $handle): \Revault\Bindings\FormDefinitionList
    {
        return $this->decode(\Revault\Bindings\FormDefinitionList::class, $this->ffi->vault_directory_list_forms($handle));
    }

    public function vaultDirectoryListFormRevisions(CData $handle, string $typeId): \Revault\Bindings\FormDefinitionList
    {
        return $this->decode(\Revault\Bindings\FormDefinitionList::class, $this->ffi->vault_directory_list_form_revisions($handle, $typeId, strlen($typeId)));
    }

    public function vaultDirectorySeedForms(CData $handle): int
    {
        return $this->ffi->vault_directory_seed_forms($handle);
    }

    public function vaultDirectoryRememberPassword(CData $handle, string $id, string $password): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireBool((bool) ($this->ffi->vault_directory_remember_password($handle, $idPointer, $idLength, $passwordPointer, $passwordLength)))));
    }

    public function vaultDirectoryRememberedPassword(CData $handle, string $id): string
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->take($this->ffi->vault_directory_remembered_password($handle, $idPointer, $idLength)));
    }

    public function vaultBackupDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->decode(\Revault\Bindings\VaultBackupManifest::class, $this->ffi->vault_backup_default($path, strlen($path), $overwrite));
    }

    public function vaultRestoreDefault(string $path, bool $overwrite): \Revault\Bindings\VaultBackupManifest
    {
        return $this->decode(\Revault\Bindings\VaultBackupManifest::class, $this->ffi->vault_restore_default($path, strlen($path), $overwrite));
    }

    public function vaultDirectoryFree(CData $handle): void
    {
        $this->ffi->vault_directory_free($handle);
    }

    public function vaultReadOnlyOpen(string $root, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_read_only_open($root, strlen($root), $passwordPointer, $passwordLength)));
    }

    public function vaultReadOnlyOpenDefault(string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_read_only_open_default($passwordPointer, $passwordLength)));
    }

    public function vaultReadOnlyListProfileNames(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_read_only_list_profile_names($handle));
    }

    public function vaultReadOnlyListContactNames(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_read_only_list_contact_names($handle));
    }

    public function vaultReadOnlyListFormAliases(CData $handle): \Revault\Bindings\StringList
    {
        return $this->decode(\Revault\Bindings\StringList::class, $this->ffi->vault_read_only_list_form_aliases($handle));
    }

    public function vaultReadOnlyListKnownLockboxes(CData $handle): \Revault\Bindings\KnownLockboxList
    {
        return $this->decode(\Revault\Bindings\KnownLockboxList::class, $this->ffi->vault_read_only_list_known_lockboxes($handle));
    }

    public function vaultReadOnlyFree(CData $handle): void
    {
        $this->ffi->vault_read_only_free($handle);
    }

    public function vaultAgentServe(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_serve()));
    }

    public function vaultAgentVerifyTransport(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_verify_transport()));
    }

    public function vaultAgentGet(string $id): string
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->take($this->ffi->vault_agent_get($idPointer, $idLength)));
    }

    public function vaultAgentPut(string $id, string $key): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireBool((bool) ($this->ffi->vault_agent_put($idPointer, $idLength, $keyPointer, $keyLength)))));
    }

    public function vaultAgentForget(string $id): bool
    {
        return $this->withBytes($id, fn(CData $idPointer, int $idLength) => $this->requireBool((bool) ($this->ffi->vault_agent_forget($idPointer, $idLength))));
    }

    public function vaultAgentStop(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_stop()));
    }

    public function vaultAgentStart(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_start()));
    }

    public function vaultAgentList(): \Revault\Bindings\AgentEntryList
    {
        return $this->decode(\Revault\Bindings\AgentEntryList::class, $this->ffi->vault_agent_list());
    }

    public function vaultAgentSleepSupport(): \Revault\Bindings\SleepSupport
    {
        return $this->decode(\Revault\Bindings\SleepSupport::class, $this->ffi->vault_agent_sleep_support());
    }

    public function vaultPlatformStatus(): \Revault\Bindings\PlatformStatus
    {
        return $this->decode(\Revault\Bindings\PlatformStatus::class, $this->ffi->vault_platform_status());
    }

    public function vaultPlatformSetScope(string $scope): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_platform_set_scope($scope, strlen($scope))));
    }

    public function vaultPlatformForgetPassword(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_platform_forget_password()));
    }

    public function vaultPlatformPutPassword(string $password): bool
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireBool((bool) ($this->ffi->vault_platform_put_password($passwordPointer, $passwordLength))));
    }

    public function vaultPlatformEnable(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_platform_enable()));
    }

    public function vaultPlatformDisable(): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_platform_disable()));
    }

    public function vaultPlatformDisabled(): bool
    {
        return (bool) ($this->ffi->vault_platform_disabled());
    }

    public function vaultPlatformGetPassword(): string
    {
        return $this->take($this->ffi->vault_platform_get_password());
    }

    public function vaultDefaultDirectory(): string
    {
        return $this->take($this->ffi->vault_default_directory());
    }

    public function vaultDefaultPath(): string
    {
        return $this->take($this->ffi->vault_default_path());
    }

    public function vaultAgentLogPath(): string
    {
        return $this->take($this->ffi->vault_agent_log_path());
    }

    public function vaultAgentLogDestination(): string
    {
        return $this->take($this->ffi->vault_agent_log_destination());
    }

    public function vaultAgentGetVaultUnlockKey(string $vaultId): string
    {
        return $this->take($this->ffi->vault_agent_get_vault_unlock_key($vaultId, strlen($vaultId)));
    }

    public function vaultAgentPutVaultUnlockKey(string $vaultId, string $key, int $ttlSeconds): bool
    {
        return $this->withBytes($key, fn(CData $keyPointer, int $keyLength) => $this->requireBool((bool) ($this->ffi->vault_agent_put_vault_unlock_key($vaultId, strlen($vaultId), $keyPointer, $keyLength, $ttlSeconds))));
    }

    public function vaultAgentForgetVaultUnlockKey(string $vaultId): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_forget_vault_unlock_key($vaultId, strlen($vaultId))));
    }

    public function vaultAgentGetOwnerSigningKey(string $vaultId, string $profile): CData
    {
        return $this->requireHandle($this->ffi->vault_agent_get_owner_signing_key($vaultId, strlen($vaultId), $profile, strlen($profile)));
    }

    public function vaultAgentPutOwnerSigningKey(string $vaultId, string $profile, CData $key, int $ttlSeconds): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_put_owner_signing_key($vaultId, strlen($vaultId), $profile, strlen($profile), $key, $ttlSeconds)));
    }

    public function vaultAgentForgetOwnerSigningKey(string $vaultId, string $profile): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_agent_forget_owner_signing_key($vaultId, strlen($vaultId), $profile, strlen($profile))));
    }

    public function vaultAgentBeginActivity(string $kind): CData
    {
        return $this->requireHandle($this->ffi->vault_agent_begin_activity($kind, strlen($kind)));
    }

    public function vaultAgentEndActivity(CData $handle): void
    {
        $this->ffi->vault_agent_end_activity($handle);
    }

    public function vaultLocal(): CData
    {
        return $this->requireHandle($this->ffi->vault_local());
    }

    public function vaultCreateLockboxPassword(CData $vault, string $path, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_create_lockbox_password($vault, $path, strlen($path), $passwordPointer, $passwordLength)));
    }

    public function vaultOpenLockboxPassword(CData $vault, string $path, string $password): CData
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireHandle($this->ffi->vault_open_lockbox_password($vault, $path, strlen($path), $passwordPointer, $passwordLength)));
    }

    public function vaultCreateLockboxContentKey(CData $vault, string $path, string $contentKey, CData $signingKey): CData
    {
        return $this->withBytes($contentKey, fn(CData $contentKeyPointer, int $contentKeyLength) => $this->requireHandle($this->ffi->vault_create_lockbox_content_key($vault, $path, strlen($path), $contentKeyPointer, $contentKeyLength, $signingKey)));
    }

    public function vaultCreateLockboxContact(CData $vault, string $path, CData $contact, string $name, CData $signingKey): CData
    {
        return $this->requireHandle($this->ffi->vault_create_lockbox_contact($vault, $path, strlen($path), $contact, $name, strlen($name), $signingKey));
    }

    public function vaultOpenLockboxContentKey(CData $vault, string $path, string $contentKey, CData $signingKey): CData
    {
        return $this->withBytes($contentKey, fn(CData $contentKeyPointer, int $contentKeyLength) => $this->requireHandle($this->ffi->vault_open_lockbox_content_key($vault, $path, strlen($path), $contentKeyPointer, $contentKeyLength, $signingKey)));
    }

    public function vaultCacheLockboxPassword(CData $vault, string $path, string $password, int $ttlSeconds): bool
    {
        return $this->withBytes($password, fn(CData $passwordPointer, int $passwordLength) => $this->requireBool((bool) ($this->ffi->vault_cache_lockbox_password($vault, $path, strlen($path), $passwordPointer, $passwordLength, $ttlSeconds))));
    }

    public function vaultCloseLockbox(CData $vault, string $path): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_close_lockbox($vault, $path, strlen($path))));
    }

    public function vaultCloseAll(CData $vault): bool
    {
        return $this->requireBool((bool) ($this->ffi->vault_close_all($vault)));
    }

    public function vaultFree(CData $vault): void
    {
        $this->ffi->vault_free($vault);
    }

}
