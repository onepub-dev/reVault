package revault

/*#include "../../rust/revault_bindings/revault_api.h"*/
import "C"
import "unsafe"

type Native struct{}
func (Native) ApiAbiVersion() C.uint32_t {
    return C.api_abi_version()
}
func (Native) BufferLastError() unsafe.Pointer {
    return unsafe.Pointer(C.buffer_last_error())
}
func (Native) BufferLastErrorDetails() C.RevaultBuffer {
    return C.buffer_last_error_details()
}
func (Native) BufferFree(value C.RevaultBuffer) {
    C.buffer_free(value)
}
func (Native) LockboxFormatVersion() C.uint16_t {
    return C.lockbox_format_version()
}
func (Native) LockboxProbeFormatVersion(bytes unsafe.Pointer, len C.size_t) C.uint16_t {
    return C.lockbox_probe_format_version((*C.uint8_t)(bytes), len)
}
func (Native) LockboxCreate(key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
    return C.lockbox_create((*C.uint8_t)(key), key_len)
}
func (Native) LockboxCreateWithOptions(key unsafe.Pointer, key_len C.size_t, cache_mode unsafe.Pointer, cache_len C.size_t, cache_bytes C.uint64_t, workload unsafe.Pointer, workload_len C.size_t, worker unsafe.Pointer, worker_len C.size_t, jobs C.size_t) unsafe.Pointer {
    return C.lockbox_create_with_options((*C.uint8_t)(key), key_len, (*C.char)(cache_mode), cache_len, cache_bytes, (*C.char)(workload), workload_len, (*C.char)(worker), worker_len, jobs)
}
func (Native) LockboxCreatePassword(password unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.lockbox_create_password((*C.uint8_t)(password), len)
}
func (Native) LockboxCreateContact(contact unsafe.Pointer) unsafe.Pointer {
    return C.lockbox_create_contact(contact)
}
func (Native) LockboxCreateWithSigningKey(content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
    return C.lockbox_create_with_signing_key((*C.uint8_t)(content_key), key_len, signing_key)
}
func (Native) LockboxOpen(archive unsafe.Pointer, archive_len C.size_t, key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
    return C.lockbox_open((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(key), key_len)
}
func (Native) LockboxOpenWithOptions(archive unsafe.Pointer, archive_len C.size_t, key unsafe.Pointer, key_len C.size_t, cache_mode unsafe.Pointer, cache_len C.size_t, cache_bytes C.uint64_t, workload unsafe.Pointer, workload_len C.size_t, worker unsafe.Pointer, worker_len C.size_t, jobs C.size_t) unsafe.Pointer {
    return C.lockbox_open_with_options((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(key), key_len, (*C.char)(cache_mode), cache_len, cache_bytes, (*C.char)(workload), workload_len, (*C.char)(worker), worker_len, jobs)
}
func (Native) LockboxOpenPassword(archive unsafe.Pointer, archive_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.lockbox_open_password((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(password), password_len)
}
func (Native) LockboxOpenContact(archive unsafe.Pointer, archive_len C.size_t, contact unsafe.Pointer) unsafe.Pointer {
    return C.lockbox_open_contact((*C.uint8_t)(archive), archive_len, contact)
}
func (Native) LockboxAddFile(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, data unsafe.Pointer, data_len C.size_t, replace C.bool) C.bool {
    return C.lockbox_add_file(handle, (*C.char)(path), path_len, (*C.uint8_t)(data), data_len, replace)
}
func (Native) LockboxAddFileWithPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, data unsafe.Pointer, data_len C.size_t, permissions C.uint32_t, replace C.bool) C.bool {
    return C.lockbox_add_file_with_permissions(handle, (*C.char)(path), path_len, (*C.uint8_t)(data), data_len, permissions, replace)
}
func (Native) LockboxGetFile(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
    return C.lockbox_get_file(handle, (*C.char)(path), path_len)
}
func (Native) LockboxExtractFile(handle unsafe.Pointer, source unsafe.Pointer, source_len C.size_t, destination unsafe.Pointer, destination_len C.size_t, replace C.bool) C.bool {
    return C.lockbox_extract_file(handle, (*C.char)(source), source_len, (*C.char)(destination), destination_len, replace)
}
func (Native) LockboxExtractDirectory(handle unsafe.Pointer, destination unsafe.Pointer, destination_len C.size_t, max_file_bytes C.uint64_t, max_total_bytes C.uint64_t, max_files C.size_t, restore_symlinks C.bool, restore_permissions C.bool, overwrite C.bool) C.bool {
    return C.lockbox_extract_directory(handle, (*C.char)(destination), destination_len, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
}
func (Native) LockboxStreamContent(handle unsafe.Pointer, physical C.bool) C.RevaultBuffer {
    return C.lockbox_stream_content(handle, physical)
}
func (Native) LockboxCacheStats(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_cache_stats(handle)
}
func (Native) LockboxImportStats(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_import_stats(handle)
}
func (Native) LockboxResetImportStats(handle unsafe.Pointer) C.bool {
    return C.lockbox_reset_import_stats(handle)
}
func (Native) LockboxInspectFile(path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
    return C.lockbox_inspect_file((*C.char)(path), path_len)
}
func (Native) LockboxPageInspection(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_page_inspection(handle)
}
func (Native) LockboxRecoveryReport(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_recovery_report(handle)
}
func (Native) LockboxRecoveryReportRender(handle unsafe.Pointer, verbose C.bool, max_entries C.size_t) C.RevaultBuffer {
    return C.lockbox_recovery_report_render(handle, verbose, max_entries)
}
func (Native) LockboxRecoveryScanPath(path unsafe.Pointer, path_len C.size_t, key unsafe.Pointer, key_len C.size_t) C.RevaultBuffer {
    return C.lockbox_recovery_scan_path((*C.char)(path), path_len, (*C.uint8_t)(key), key_len)
}
func (Native) LockboxStorageLen(handle unsafe.Pointer) C.uint64_t {
    return C.lockbox_storage_len(handle)
}
func (Native) LockboxSetWorkloadProfile(handle unsafe.Pointer, profile unsafe.Pointer, profile_len C.size_t) C.bool {
    return C.lockbox_set_workload_profile(handle, (*C.char)(profile), profile_len)
}
func (Native) LockboxSetWorkerPolicy(handle unsafe.Pointer, mode unsafe.Pointer, mode_len C.size_t, jobs C.size_t) C.bool {
    return C.lockbox_set_worker_policy(handle, (*C.char)(mode), mode_len, jobs)
}
func (Native) LockboxRuntimeOptions(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_runtime_options(handle)
}
func (Native) LockboxCommit(handle unsafe.Pointer) C.bool {
    return C.lockbox_commit(handle)
}
func (Native) LockboxCreateDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, create_parents C.bool) C.bool {
    return C.lockbox_create_dir(handle, (*C.char)(path), path_len, create_parents)
}
func (Native) LockboxDelete(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.lockbox_delete(handle, (*C.char)(path), path_len)
}
func (Native) LockboxRemoveDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, recursive C.bool) C.bool {
    return C.lockbox_remove_dir(handle, (*C.char)(path), path_len, recursive)
}
func (Native) LockboxCreateParentDirs(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.lockbox_create_parent_dirs(handle, (*C.char)(path), path_len)
}
func (Native) LockboxRename(handle unsafe.Pointer, from unsafe.Pointer, from_len C.size_t, to unsafe.Pointer, to_len C.size_t) C.bool {
    return C.lockbox_rename(handle, (*C.char)(from), from_len, (*C.char)(to), to_len)
}
func (Native) LockboxList(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, recursive C.bool) C.RevaultBuffer {
    return C.lockbox_list(handle, (*C.char)(path), path_len, recursive)
}
func (Native) LockboxListWithOptions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, glob unsafe.Pointer, glob_len C.size_t, recursive C.bool, include_files C.bool, include_symlinks C.bool, include_directories C.bool, limit C.size_t) C.RevaultBuffer {
    return C.lockbox_list_with_options(handle, (*C.char)(path), path_len, (*C.char)(glob), glob_len, recursive, include_files, include_symlinks, include_directories, limit)
}
func (Native) LockboxStat(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
    return C.lockbox_stat(handle, (*C.char)(path), path_len)
}
func (Native) LockboxSetVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, value unsafe.Pointer, value_len C.size_t, secret C.bool) C.bool {
    return C.lockbox_set_variable(handle, (*C.char)(name), name_len, (*C.char)(value), value_len, secret)
}
func (Native) LockboxGetVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.lockbox_get_variable(handle, (*C.char)(name), name_len)
}
func (Native) LockboxDeleteVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.lockbox_delete_variable(handle, (*C.char)(name), name_len)
}
func (Native) LockboxMoveVariables(handle unsafe.Pointer, moves_proto unsafe.Pointer, moves_len C.size_t) C.bool {
    return C.lockbox_move_variables(handle, (*C.uint8_t)(moves_proto), moves_len)
}
func (Native) LockboxListVariables(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_list_variables(handle)
}
func (Native) LockboxVariableSensitivity(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.lockbox_variable_sensitivity(handle, (*C.char)(name), name_len)
}
func (Native) LockboxAddSymlink(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, target unsafe.Pointer, target_len C.size_t, replace C.bool) C.bool {
    return C.lockbox_add_symlink(handle, (*C.char)(path), path_len, (*C.char)(target), target_len, replace)
}
func (Native) LockboxGetSymlinkTarget(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
    return C.lockbox_get_symlink_target(handle, (*C.char)(path), path_len)
}
func (Native) LockboxId(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_id(handle)
}
func (Native) LockboxExists(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.lockbox_exists(handle, (*C.char)(path), path_len)
}
func (Native) LockboxIsDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.lockbox_is_dir(handle, (*C.char)(path), path_len)
}
func (Native) LockboxPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.uint32_t {
    return C.lockbox_permissions(handle, (*C.char)(path), path_len)
}
func (Native) LockboxSetPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, permissions C.uint32_t) C.bool {
    return C.lockbox_set_permissions(handle, (*C.char)(path), path_len, permissions)
}
func (Native) LockboxReadRange(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, offset C.uint64_t, len C.uint64_t) C.RevaultBuffer {
    return C.lockbox_read_range(handle, (*C.char)(path), path_len, offset, len)
}
func (Native) LockboxRecoveryScan(bytes unsafe.Pointer, len C.size_t, key unsafe.Pointer, key_len C.size_t) C.RevaultBuffer {
    return C.lockbox_recovery_scan((*C.uint8_t)(bytes), len, (*C.uint8_t)(key), key_len)
}
func (Native) LockboxRecoverySalvage(bytes unsafe.Pointer, len C.size_t, key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
    return C.lockbox_recovery_salvage((*C.uint8_t)(bytes), len, (*C.uint8_t)(key), key_len, signing_key)
}
func (Native) LockboxAddPassword(handle unsafe.Pointer, password unsafe.Pointer, len C.size_t) C.uint64_t {
    return C.lockbox_add_password(handle, (*C.uint8_t)(password), len)
}
func (Native) LockboxAddContact(handle unsafe.Pointer, contact unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.uint64_t {
    return C.lockbox_add_contact(handle, contact, (*C.char)(name), name_len)
}
func (Native) LockboxDeleteKey(handle unsafe.Pointer, id C.uint64_t) C.bool {
    return C.lockbox_delete_key(handle, id)
}
func (Native) LockboxListKeySlots(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_list_key_slots(handle)
}
func (Native) LockboxSetOwnerSigningKey(handle unsafe.Pointer, key unsafe.Pointer) C.bool {
    return C.lockbox_set_owner_signing_key(handle, key)
}
func (Native) LockboxOwnerInspection(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_owner_inspection(handle)
}
func (Native) LockboxDefineForm(handle unsafe.Pointer, alias unsafe.Pointer, alias_len C.size_t, name unsafe.Pointer, name_len C.size_t, description unsafe.Pointer, description_len C.size_t, fields_proto unsafe.Pointer, fields_len C.size_t) C.RevaultBuffer {
    return C.lockbox_define_form(handle, (*C.char)(alias), alias_len, (*C.char)(name), name_len, (*C.char)(description), description_len, (*C.uint8_t)(fields_proto), fields_len)
}
func (Native) LockboxListFormDefinitions(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_list_form_definitions(handle)
}
func (Native) LockboxResolveForm(handle unsafe.Pointer, reference unsafe.Pointer, reference_len C.size_t) C.RevaultBuffer {
    return C.lockbox_resolve_form(handle, (*C.char)(reference), reference_len)
}
func (Native) LockboxListFormRevisions(handle unsafe.Pointer, type_id unsafe.Pointer, type_id_len C.size_t) C.RevaultBuffer {
    return C.lockbox_list_form_revisions(handle, (*C.char)(type_id), type_id_len)
}
func (Native) LockboxCreateFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, type_reference unsafe.Pointer, type_len C.size_t, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.lockbox_create_form_record(handle, (*C.char)(path), path_len, (*C.char)(type_reference), type_len, (*C.char)(name), name_len)
}
func (Native) LockboxSetFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t, value unsafe.Pointer, value_len C.size_t, secret C.bool) C.bool {
    return C.lockbox_set_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len, (*C.char)(value), value_len, secret)
}
func (Native) LockboxListFormRecords(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_list_form_records(handle)
}
func (Native) LockboxGetFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
    return C.lockbox_get_form_record(handle, (*C.char)(path), path_len)
}
func (Native) LockboxDeleteFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.lockbox_delete_form_record(handle, (*C.char)(path), path_len)
}
func (Native) LockboxMoveFormRecords(handle unsafe.Pointer, moves_proto unsafe.Pointer, moves_len C.size_t) C.bool {
    return C.lockbox_move_form_records(handle, (*C.uint8_t)(moves_proto), moves_len)
}
func (Native) LockboxGetFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t) C.RevaultBuffer {
    return C.lockbox_get_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len)
}
func (Native) LockboxToBytes(handle unsafe.Pointer) C.RevaultBuffer {
    return C.lockbox_to_bytes(handle)
}
func (Native) LockboxFree(handle unsafe.Pointer) {
    C.lockbox_free(handle)
}
func (Native) VaultIsRunning() C.bool {
    return C.vault_is_running()
}
func (Native) VaultForgetAll() C.bool {
    return C.vault_forget_all()
}
func (Native) KeyContactGenerate() unsafe.Pointer {
    return C.key_contact_generate()
}
func (Native) KeyContactFromPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.key_contact_from_private((*C.uint8_t)(bytes), len)
}
func (Native) KeyContactPublic(handle unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_public(handle)
}
func (Native) KeyContactPrivate(handle unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_private(handle)
}
func (Native) KeyContactPublicFromBytes(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.key_contact_public_from_bytes((*C.uint8_t)(bytes), len)
}
func (Native) KeyContactPublicFree(handle unsafe.Pointer) {
    C.key_contact_public_free(handle)
}
func (Native) KeyContactFree(handle unsafe.Pointer) {
    C.key_contact_free(handle)
}
func (Native) KeyContactEncrypt(contact unsafe.Pointer, content_key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
    return C.key_contact_encrypt(contact, (*C.uint8_t)(content_key), key_len)
}
func (Native) KeyContactDecrypt(contact unsafe.Pointer, wrapped unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_decrypt(contact, wrapped)
}
func (Native) KeyContactWrappedPublic(wrapped unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_wrapped_public(wrapped)
}
func (Native) KeyContactWrappedCiphertext(wrapped unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_wrapped_ciphertext(wrapped)
}
func (Native) KeyContactWrappedEncrypted(wrapped unsafe.Pointer) C.RevaultBuffer {
    return C.key_contact_wrapped_encrypted(wrapped)
}
func (Native) KeyContactWrappedFree(handle unsafe.Pointer) {
    C.key_contact_wrapped_free(handle)
}
func (Native) KeySigningGenerate() unsafe.Pointer {
    return C.key_signing_generate()
}
func (Native) KeySigningFromPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.key_signing_from_private((*C.uint8_t)(bytes), len)
}
func (Native) KeySigningPublic(handle unsafe.Pointer) C.RevaultBuffer {
    return C.key_signing_public(handle)
}
func (Native) KeySigningPrivate(handle unsafe.Pointer) C.RevaultBuffer {
    return C.key_signing_private(handle)
}
func (Native) KeySigningPublicFromBytes(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.key_signing_public_from_bytes((*C.uint8_t)(bytes), len)
}
func (Native) KeySigningPublicFree(handle unsafe.Pointer) {
    C.key_signing_public_free(handle)
}
func (Native) KeySigningFree(handle unsafe.Pointer) {
    C.key_signing_free(handle)
}
func (Native) VaultKeyExportPrivate(key unsafe.Pointer, format unsafe.Pointer, format_len C.size_t) C.RevaultBuffer {
    return C.vault_key_export_private(key, (*C.char)(format), format_len)
}
func (Native) VaultKeyExportPublic(key unsafe.Pointer, format unsafe.Pointer, format_len C.size_t) C.RevaultBuffer {
    return C.vault_key_export_public(key, (*C.char)(format), format_len)
}
func (Native) VaultKeyImportPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.vault_key_import_private((*C.uint8_t)(bytes), len)
}
func (Native) VaultKeyImportPublic(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.vault_key_import_public((*C.uint8_t)(bytes), len)
}
func (Native) VaultKeyFingerprint(key unsafe.Pointer) C.RevaultBuffer {
    return C.vault_key_fingerprint(key)
}
func (Native) VaultKeyFormatHex(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_format_hex((*C.uint8_t)(bytes), len)
}
func (Native) VaultKeyDecodeHex(text unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_decode_hex((*C.char)(text), len)
}
func (Native) VaultKeyFormatCrockford(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_format_crockford((*C.uint8_t)(bytes), len)
}
func (Native) VaultKeyFormatCrockfordReading(code unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_format_crockford_reading((*C.char)(code), len)
}
func (Native) VaultKeyDecodeCrockford(code unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_decode_crockford((*C.char)(code), len)
}
func (Native) VaultKeyHexEncode(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_hex_encode((*C.uint8_t)(bytes), len)
}
func (Native) VaultKeyHexDecode(text unsafe.Pointer, len C.size_t) C.RevaultBuffer {
    return C.vault_key_hex_decode((*C.char)(text), len)
}
func (Native) VaultDirectoryOpen(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_directory_open((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultStructureVersionCurrent() C.uint32_t {
    return C.vault_structure_version_current()
}
func (Native) VaultDirectoryProbeStructureVersion(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) C.uint32_t {
    return C.vault_directory_probe_structure_version((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryOpenOrCreateDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_directory_open_or_create_default((*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryReplaceDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_directory_replace_default((*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryChangePassword(root unsafe.Pointer, root_len C.size_t, old_password unsafe.Pointer, old_len C.size_t, new_password unsafe.Pointer, new_len C.size_t) C.bool {
    return C.vault_directory_change_password((*C.char)(root), root_len, (*C.uint8_t)(old_password), old_len, (*C.uint8_t)(new_password), new_len)
}
func (Native) VaultDirectoryChangeDefaultPassword(old_password unsafe.Pointer, old_len C.size_t, new_password unsafe.Pointer, new_len C.size_t) C.bool {
    return C.vault_directory_change_default_password((*C.uint8_t)(old_password), old_len, (*C.uint8_t)(new_password), new_len)
}
func (Native) VaultDirectoryReplace(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_directory_replace((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryOpenOrCreate(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_directory_open_or_create((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryRoot(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_root(handle)
}
func (Native) VaultDirectoryStructureVersion(handle unsafe.Pointer) C.uint32_t {
    return C.vault_directory_structure_version(handle)
}
func (Native) VaultDirectoryListPrivateKeys(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_private_keys(handle)
}
func (Native) VaultDirectoryListPrivateKeyNames(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_private_key_names(handle)
}
func (Native) VaultDirectoryListContactNames(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_contact_names(handle)
}
func (Native) VaultDirectoryListFormAliases(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_form_aliases(handle)
}
func (Native) VaultDirectoryPrivateKeyExists(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.vault_directory_private_key_exists(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryDeletePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.vault_directory_delete_private_key(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryStorePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
    return C.vault_directory_store_private_key(handle, (*C.char)(name), name_len, key)
}
func (Native) VaultDirectoryLoadPrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
    return C.vault_directory_load_private_key(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryLoadPrivateKeyGeneration(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, index C.uint16_t) unsafe.Pointer {
    return C.vault_directory_load_private_key_generation(handle, (*C.char)(name), name_len, index)
}
func (Native) VaultDirectoryStoreContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
    return C.vault_directory_store_contact(handle, (*C.char)(name), name_len, key)
}
func (Native) VaultDirectoryLoadContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
    return C.vault_directory_load_contact(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryContactExists(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.vault_directory_contact_exists(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryDeleteContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.vault_directory_delete_contact(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryListContacts(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_contacts(handle)
}
func (Native) VaultDirectoryStoreProfileEmail(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, email unsafe.Pointer, email_len C.size_t) C.bool {
    return C.vault_directory_store_profile_email(handle, (*C.char)(name), name_len, (*C.char)(email), email_len)
}
func (Native) VaultDirectoryProfileEmail(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_profile_email(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryStoreBackup(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, bytes unsafe.Pointer, len C.size_t) C.bool {
    return C.vault_directory_store_backup(handle, (*C.uint8_t)(id), id_len, (*C.uint8_t)(bytes), len)
}
func (Native) VaultDirectoryLoadBackup(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_load_backup(handle, (*C.uint8_t)(id), id_len)
}
func (Native) VaultDirectoryBackupCount(handle unsafe.Pointer) C.uint64_t {
    return C.vault_directory_backup_count(handle)
}
func (Native) VaultDirectoryRestorePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer, signing_key unsafe.Pointer, overwrite C.bool) C.bool {
    return C.vault_directory_restore_private_key(handle, (*C.char)(name), name_len, key, signing_key, overwrite)
}
func (Native) VaultDirectoryLoadOwnerSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
    return C.vault_directory_load_owner_signing_key(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryLoadOwnerSigningKeyGeneration(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, index C.uint16_t) unsafe.Pointer {
    return C.vault_directory_load_owner_signing_key_generation(handle, (*C.char)(name), name_len, index)
}
func (Native) VaultDirectoryStoreContactSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
    return C.vault_directory_store_contact_signing_key(handle, (*C.char)(name), name_len, key)
}
func (Native) VaultDirectoryLoadContactSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
    return C.vault_directory_load_contact_signing_key(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryListProfileGenerations(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_list_profile_generations(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryRotatePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_rotate_private_key(handle, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryRememberLockbox(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.vault_directory_remember_lockbox(handle, (*C.uint8_t)(id), id_len, (*C.char)(path), path_len)
}
func (Native) VaultDirectoryListKnownLockboxes(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_known_lockboxes(handle)
}
func (Native) VaultDirectoryForgetLockbox(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.vault_directory_forget_lockbox(handle, (*C.char)(path), path_len)
}
func (Native) VaultDirectoryRememberAccessSlotLabel(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, slot_id C.uint64_t, name unsafe.Pointer, name_len C.size_t) C.bool {
    return C.vault_directory_remember_access_slot_label(handle, (*C.uint8_t)(id), id_len, slot_id, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryListAccessSlotLabels(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_list_access_slot_labels(handle, (*C.uint8_t)(id), id_len)
}
func (Native) VaultDirectoryFindAccessSlotLabels(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_find_access_slot_labels(handle, (*C.uint8_t)(id), id_len, (*C.char)(name), name_len)
}
func (Native) VaultDirectoryForgetAccessSlotLabel(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, slot_id C.uint64_t) C.bool {
    return C.vault_directory_forget_access_slot_label(handle, (*C.uint8_t)(id), id_len, slot_id)
}
func (Native) VaultDirectoryDefineForm(handle unsafe.Pointer, alias unsafe.Pointer, alias_len C.size_t, name unsafe.Pointer, name_len C.size_t, description unsafe.Pointer, description_len C.size_t, fields_proto unsafe.Pointer, fields_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_define_form(handle, (*C.char)(alias), alias_len, (*C.char)(name), name_len, (*C.char)(description), description_len, (*C.uint8_t)(fields_proto), fields_len)
}
func (Native) VaultDirectoryResolveForm(handle unsafe.Pointer, reference unsafe.Pointer, reference_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_resolve_form(handle, (*C.char)(reference), reference_len)
}
func (Native) VaultDirectoryListForms(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_directory_list_forms(handle)
}
func (Native) VaultDirectoryListFormRevisions(handle unsafe.Pointer, type_id unsafe.Pointer, type_id_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_list_form_revisions(handle, (*C.char)(type_id), type_id_len)
}
func (Native) VaultDirectorySeedForms(handle unsafe.Pointer) C.size_t {
    return C.vault_directory_seed_forms(handle)
}
func (Native) VaultDirectoryRememberPassword(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, password unsafe.Pointer, password_len C.size_t) C.bool {
    return C.vault_directory_remember_password(handle, (*C.uint8_t)(id), id_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultDirectoryRememberedPassword(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
    return C.vault_directory_remembered_password(handle, (*C.uint8_t)(id), id_len)
}
func (Native) VaultBackupDefault(path unsafe.Pointer, path_len C.size_t, overwrite C.bool) C.RevaultBuffer {
    return C.vault_backup_default((*C.char)(path), path_len, overwrite)
}
func (Native) VaultRestoreDefault(path unsafe.Pointer, path_len C.size_t, overwrite C.bool) C.RevaultBuffer {
    return C.vault_restore_default((*C.char)(path), path_len, overwrite)
}
func (Native) VaultDirectoryFree(handle unsafe.Pointer) {
    C.vault_directory_free(handle)
}
func (Native) VaultReadOnlyOpen(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_read_only_open((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultReadOnlyOpenDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_read_only_open_default((*C.uint8_t)(password), password_len)
}
func (Native) VaultReadOnlyListProfileNames(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_read_only_list_profile_names(handle)
}
func (Native) VaultReadOnlyListContactNames(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_read_only_list_contact_names(handle)
}
func (Native) VaultReadOnlyListFormAliases(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_read_only_list_form_aliases(handle)
}
func (Native) VaultReadOnlyListKnownLockboxes(handle unsafe.Pointer) C.RevaultBuffer {
    return C.vault_read_only_list_known_lockboxes(handle)
}
func (Native) VaultReadOnlyFree(handle unsafe.Pointer) {
    C.vault_read_only_free(handle)
}
func (Native) VaultAgentServe() C.bool {
    return C.vault_agent_serve()
}
func (Native) VaultAgentVerifyTransport() C.bool {
    return C.vault_agent_verify_transport()
}
func (Native) VaultAgentGet(id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
    return C.vault_agent_get((*C.uint8_t)(id), id_len)
}
func (Native) VaultAgentPut(id unsafe.Pointer, id_len C.size_t, key unsafe.Pointer, key_len C.size_t) C.bool {
    return C.vault_agent_put((*C.uint8_t)(id), id_len, (*C.uint8_t)(key), key_len)
}
func (Native) VaultAgentForget(id unsafe.Pointer, id_len C.size_t) C.bool {
    return C.vault_agent_forget((*C.uint8_t)(id), id_len)
}
func (Native) VaultAgentStop() C.bool {
    return C.vault_agent_stop()
}
func (Native) VaultAgentStart() C.bool {
    return C.vault_agent_start()
}
func (Native) VaultAgentList() C.RevaultBuffer {
    return C.vault_agent_list()
}
func (Native) VaultAgentSleepSupport() C.RevaultBuffer {
    return C.vault_agent_sleep_support()
}
func (Native) VaultPlatformStatus() C.RevaultBuffer {
    return C.vault_platform_status()
}
func (Native) VaultPlatformSetScope(scope unsafe.Pointer, len C.size_t) C.bool {
    return C.vault_platform_set_scope((*C.char)(scope), len)
}
func (Native) VaultPlatformForgetPassword() C.bool {
    return C.vault_platform_forget_password()
}
func (Native) VaultPlatformPutPassword(password unsafe.Pointer, len C.size_t) C.bool {
    return C.vault_platform_put_password((*C.uint8_t)(password), len)
}
func (Native) VaultPlatformEnable() C.bool {
    return C.vault_platform_enable()
}
func (Native) VaultPlatformDisable() C.bool {
    return C.vault_platform_disable()
}
func (Native) VaultPlatformDisabled() C.bool {
    return C.vault_platform_disabled()
}
func (Native) VaultPlatformGetPassword() C.RevaultBuffer {
    return C.vault_platform_get_password()
}
func (Native) VaultDefaultDirectory() C.RevaultBuffer {
    return C.vault_default_directory()
}
func (Native) VaultDefaultPath() C.RevaultBuffer {
    return C.vault_default_path()
}
func (Native) VaultAgentLogPath() C.RevaultBuffer {
    return C.vault_agent_log_path()
}
func (Native) VaultAgentLogDestination() C.RevaultBuffer {
    return C.vault_agent_log_destination()
}
func (Native) VaultAgentGetVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t) C.RevaultBuffer {
    return C.vault_agent_get_vault_unlock_key((*C.char)(vault_id), vault_id_len)
}
func (Native) VaultAgentPutVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t, key unsafe.Pointer, key_len C.size_t, ttl_seconds C.uint64_t) C.bool {
    return C.vault_agent_put_vault_unlock_key((*C.char)(vault_id), vault_id_len, (*C.uint8_t)(key), key_len, ttl_seconds)
}
func (Native) VaultAgentForgetVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t) C.bool {
    return C.vault_agent_forget_vault_unlock_key((*C.char)(vault_id), vault_id_len)
}
func (Native) VaultAgentGetOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t) unsafe.Pointer {
    return C.vault_agent_get_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len)
}
func (Native) VaultAgentPutOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t, key unsafe.Pointer, ttl_seconds C.uint64_t) C.bool {
    return C.vault_agent_put_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len, key, ttl_seconds)
}
func (Native) VaultAgentForgetOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t) C.bool {
    return C.vault_agent_forget_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len)
}
func (Native) VaultAgentBeginActivity(kind unsafe.Pointer, len C.size_t) unsafe.Pointer {
    return C.vault_agent_begin_activity((*C.char)(kind), len)
}
func (Native) VaultAgentEndActivity(handle unsafe.Pointer) {
    C.vault_agent_end_activity(handle)
}
func (Native) VaultLocal() unsafe.Pointer {
    return C.vault_local()
}
func (Native) VaultCreateLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_create_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultOpenLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
    return C.vault_open_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len)
}
func (Native) VaultCreateLockboxContentKey(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
    return C.vault_create_lockbox_content_key(vault, (*C.char)(path), path_len, (*C.uint8_t)(content_key), key_len, signing_key)
}
func (Native) VaultCreateLockboxContact(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, contact unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
    return C.vault_create_lockbox_contact(vault, (*C.char)(path), path_len, contact, (*C.char)(name), name_len, signing_key)
}
func (Native) VaultOpenLockboxContentKey(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
    return C.vault_open_lockbox_content_key(vault, (*C.char)(path), path_len, (*C.uint8_t)(content_key), key_len, signing_key)
}
func (Native) VaultCacheLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t, ttl_seconds C.uint64_t) C.bool {
    return C.vault_cache_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len, ttl_seconds)
}
func (Native) VaultCloseLockbox(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
    return C.vault_close_lockbox(vault, (*C.char)(path), path_len)
}
func (Native) VaultCloseAll(vault unsafe.Pointer) C.bool {
    return C.vault_close_all(vault)
}
func (Native) VaultFree(vault unsafe.Pointer) {
    C.vault_free(vault)
}
