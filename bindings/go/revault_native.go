package revault

/*#include "../../rust/revault_bindings/revault_api.h"*/
import "C"
import "unsafe"

// native exposes the stable ABI directly for binding authors.
//
// Application code should use Vault and Lockbox, which manage ownership,
// errors, structured results, and callback-scoped secrets. native methods use
// C pointers and lengths and follow the ownership rules in revault_api.h.
// See https://github.com/onepub-dev/reVault#readme.
type native struct{}

func (native) ApiAbiVersion() C.uint32_t {
	return C.api_abi_version()
}
func (native) BufferLastError() unsafe.Pointer {
	return unsafe.Pointer(C.buffer_last_error())
}
func (native) BufferLastErrorDetails() C.RevaultBuffer {
	return C.buffer_last_error_details()
}
func (native) BufferFree(value C.RevaultBuffer) {
	C.buffer_free(value)
}
func (native) SecretLen(handle unsafe.Pointer, length *C.size_t) C.bool {
	return C.secret_len(handle, length)
}
func (native) SecretCopy(handle unsafe.Pointer, destination unsafe.Pointer, length C.size_t) C.bool {
	return C.secret_copy(handle, (*C.uint8_t)(destination), length)
}
func (native) SecretFree(handle unsafe.Pointer) { C.secret_free(handle) }
func (native) LockboxFormatVersion() C.uint16_t {
	return C.lockbox_format_version()
}
func (native) LockboxProbeFormatVersion(bytes unsafe.Pointer, len C.size_t) C.uint16_t {
	return C.lockbox_probe_format_version((*C.uint8_t)(bytes), len)
}
func (native) LockboxCreate(key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
	return C.lockbox_create((*C.uint8_t)(key), key_len)
}
func (native) LockboxCreateWithOptions(key unsafe.Pointer, key_len C.size_t, cache_mode unsafe.Pointer, cache_len C.size_t, cache_bytes C.uint64_t, workload unsafe.Pointer, workload_len C.size_t, worker unsafe.Pointer, worker_len C.size_t, jobs C.size_t) unsafe.Pointer {
	return C.lockbox_create_with_options((*C.uint8_t)(key), key_len, (*C.char)(cache_mode), cache_len, cache_bytes, (*C.char)(workload), workload_len, (*C.char)(worker), worker_len, jobs)
}
func (native) LockboxCreatePassword(password unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.lockbox_create_password((*C.uint8_t)(password), len)
}
func (native) LockboxCreateContact(contact unsafe.Pointer) unsafe.Pointer {
	return C.lockbox_create_contact(contact)
}
func (native) LockboxCreateWithSigningKey(content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
	return C.lockbox_create_with_signing_key((*C.uint8_t)(content_key), key_len, signing_key)
}
func (native) LockboxOpen(archive unsafe.Pointer, archive_len C.size_t, key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
	return C.lockbox_open((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(key), key_len)
}
func (native) LockboxOpenWithOptions(archive unsafe.Pointer, archive_len C.size_t, key unsafe.Pointer, key_len C.size_t, cache_mode unsafe.Pointer, cache_len C.size_t, cache_bytes C.uint64_t, workload unsafe.Pointer, workload_len C.size_t, worker unsafe.Pointer, worker_len C.size_t, jobs C.size_t) unsafe.Pointer {
	return C.lockbox_open_with_options((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(key), key_len, (*C.char)(cache_mode), cache_len, cache_bytes, (*C.char)(workload), workload_len, (*C.char)(worker), worker_len, jobs)
}
func (native) LockboxOpenPassword(archive unsafe.Pointer, archive_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.lockbox_open_password((*C.uint8_t)(archive), archive_len, (*C.uint8_t)(password), password_len)
}
func (native) LockboxOpenContact(archive unsafe.Pointer, archive_len C.size_t, contact unsafe.Pointer) unsafe.Pointer {
	return C.lockbox_open_contact((*C.uint8_t)(archive), archive_len, contact)
}
func (native) LockboxAddFile(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, data unsafe.Pointer, data_len C.size_t, replace C.bool) C.bool {
	return C.lockbox_add_file(handle, (*C.char)(path), path_len, (*C.uint8_t)(data), data_len, replace)
}
func (native) LockboxAddFileWithPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, data unsafe.Pointer, data_len C.size_t, permissions C.uint32_t, replace C.bool) C.bool {
	return C.lockbox_add_file_with_permissions(handle, (*C.char)(path), path_len, (*C.uint8_t)(data), data_len, permissions, replace)
}
func (native) LockboxGetFile(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
	return C.lockbox_get_file(handle, (*C.char)(path), path_len)
}
func (native) LockboxExtractFile(handle unsafe.Pointer, source unsafe.Pointer, source_len C.size_t, destination unsafe.Pointer, destination_len C.size_t, replace C.bool) C.bool {
	return C.lockbox_extract_file(handle, (*C.char)(source), source_len, (*C.char)(destination), destination_len, replace)
}
func (native) LockboxExtractDirectory(handle unsafe.Pointer, destination unsafe.Pointer, destination_len C.size_t, max_file_bytes C.uint64_t, max_total_bytes C.uint64_t, max_files C.size_t, restore_symlinks C.bool, restore_permissions C.bool, overwrite C.bool) C.bool {
	return C.lockbox_extract_directory(handle, (*C.char)(destination), destination_len, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
}
func (native) LockboxStreamContent(handle unsafe.Pointer, physical C.bool) C.RevaultBuffer {
	return C.lockbox_stream_content(handle, physical)
}
func (native) LockboxCacheStats(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_cache_stats(handle)
}
func (native) LockboxImportStats(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_import_stats(handle)
}
func (native) LockboxResetImportStats(handle unsafe.Pointer) C.bool {
	return C.lockbox_reset_import_stats(handle)
}
func (native) LockboxInspectFile(path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
	return C.lockbox_inspect_file((*C.char)(path), path_len)
}
func (native) LockboxPageInspection(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_page_inspection(handle)
}
func (native) LockboxRecoveryReport(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_recovery_report(handle)
}
func (native) LockboxRecoveryReportRender(handle unsafe.Pointer, verbose C.bool, max_entries C.size_t) C.RevaultBuffer {
	return C.lockbox_recovery_report_render(handle, verbose, max_entries)
}
func (native) LockboxRecoveryScanPath(path unsafe.Pointer, path_len C.size_t, key unsafe.Pointer, key_len C.size_t) C.RevaultBuffer {
	return C.lockbox_recovery_scan_path((*C.char)(path), path_len, (*C.uint8_t)(key), key_len)
}
func (native) LockboxStorageLen(handle unsafe.Pointer) C.uint64_t {
	return C.lockbox_storage_len(handle)
}
func (native) LockboxSetWorkloadProfile(handle unsafe.Pointer, profile unsafe.Pointer, profile_len C.size_t) C.bool {
	return C.lockbox_set_workload_profile(handle, (*C.char)(profile), profile_len)
}
func (native) LockboxSetWorkerPolicy(handle unsafe.Pointer, mode unsafe.Pointer, mode_len C.size_t, jobs C.size_t) C.bool {
	return C.lockbox_set_worker_policy(handle, (*C.char)(mode), mode_len, jobs)
}
func (native) LockboxRuntimeOptions(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_runtime_options(handle)
}
func (native) LockboxCommit(handle unsafe.Pointer) C.bool {
	return C.lockbox_commit(handle)
}
func (native) LockboxCreateDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, create_parents C.bool) C.bool {
	return C.lockbox_create_dir(handle, (*C.char)(path), path_len, create_parents)
}
func (native) LockboxDelete(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.lockbox_delete(handle, (*C.char)(path), path_len)
}
func (native) LockboxRemoveDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, recursive C.bool) C.bool {
	return C.lockbox_remove_dir(handle, (*C.char)(path), path_len, recursive)
}
func (native) LockboxCreateParentDirs(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.lockbox_create_parent_dirs(handle, (*C.char)(path), path_len)
}
func (native) LockboxRename(handle unsafe.Pointer, from unsafe.Pointer, from_len C.size_t, to unsafe.Pointer, to_len C.size_t) C.bool {
	return C.lockbox_rename(handle, (*C.char)(from), from_len, (*C.char)(to), to_len)
}
func (native) LockboxList(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, recursive C.bool) C.RevaultBuffer {
	return C.lockbox_list(handle, (*C.char)(path), path_len, recursive)
}
func (native) LockboxListWithOptions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, glob unsafe.Pointer, glob_len C.size_t, recursive C.bool, include_files C.bool, include_symlinks C.bool, include_directories C.bool, limit C.size_t) C.RevaultBuffer {
	return C.lockbox_list_with_options(handle, (*C.char)(path), path_len, (*C.char)(glob), glob_len, recursive, include_files, include_symlinks, include_directories, limit)
}
func (native) LockboxStat(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
	return C.lockbox_stat(handle, (*C.char)(path), path_len)
}
func (native) LockboxSetVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, value unsafe.Pointer, value_len C.size_t) C.bool {
	return C.lockbox_set_variable(handle, (*C.char)(name), name_len, (*C.char)(value), value_len)
}
func (native) LockboxSetSecretVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, value unsafe.Pointer, value_len C.size_t) C.bool {
	return C.lockbox_set_secret_variable(handle, (*C.char)(name), name_len, (*C.uint8_t)(value), value_len)
}
func (native) LockboxGetVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.lockbox_get_variable(handle, (*C.char)(name), name_len)
}
func (native) LockboxGetSecretVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, output *unsafe.Pointer) C.bool {
	return C.lockbox_get_secret_variable(handle, (*C.char)(name), name_len, output)
}
func (native) LockboxDeleteVariable(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.lockbox_delete_variable(handle, (*C.char)(name), name_len)
}
func (native) LockboxMoveVariables(handle unsafe.Pointer, moves_proto unsafe.Pointer, moves_len C.size_t) C.bool {
	return C.lockbox_move_variables(handle, (*C.uint8_t)(moves_proto), moves_len)
}
func (native) LockboxListVariables(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_list_variables(handle)
}
func (native) LockboxVariableSensitivity(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.lockbox_variable_sensitivity(handle, (*C.char)(name), name_len)
}
func (native) LockboxAddSymlink(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, target unsafe.Pointer, target_len C.size_t, replace C.bool) C.bool {
	return C.lockbox_add_symlink(handle, (*C.char)(path), path_len, (*C.char)(target), target_len, replace)
}
func (native) LockboxGetSymlinkTarget(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
	return C.lockbox_get_symlink_target(handle, (*C.char)(path), path_len)
}
func (native) LockboxId(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_id(handle)
}
func (native) LockboxExists(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.lockbox_exists(handle, (*C.char)(path), path_len)
}
func (native) LockboxIsDir(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.lockbox_is_dir(handle, (*C.char)(path), path_len)
}
func (native) LockboxPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.uint32_t {
	return C.lockbox_permissions(handle, (*C.char)(path), path_len)
}
func (native) LockboxSetPermissions(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, permissions C.uint32_t) C.bool {
	return C.lockbox_set_permissions(handle, (*C.char)(path), path_len, permissions)
}
func (native) LockboxReadRange(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, offset C.uint64_t, len C.uint64_t) C.RevaultBuffer {
	return C.lockbox_read_range(handle, (*C.char)(path), path_len, offset, len)
}
func (native) LockboxRecoveryScan(bytes unsafe.Pointer, len C.size_t, key unsafe.Pointer, key_len C.size_t) C.RevaultBuffer {
	return C.lockbox_recovery_scan((*C.uint8_t)(bytes), len, (*C.uint8_t)(key), key_len)
}
func (native) LockboxRecoverySalvage(bytes unsafe.Pointer, len C.size_t, key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
	return C.lockbox_recovery_salvage((*C.uint8_t)(bytes), len, (*C.uint8_t)(key), key_len, signing_key)
}
func (native) LockboxAddPassword(handle unsafe.Pointer, password unsafe.Pointer, len C.size_t) C.uint64_t {
	return C.lockbox_add_password(handle, (*C.uint8_t)(password), len)
}
func (native) LockboxAddContact(handle unsafe.Pointer, contact unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.uint64_t {
	return C.lockbox_add_contact(handle, contact, (*C.char)(name), name_len)
}
func (native) LockboxDeleteKey(handle unsafe.Pointer, id C.uint64_t) C.bool {
	return C.lockbox_delete_key(handle, id)
}
func (native) LockboxListKeySlots(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_list_key_slots(handle)
}
func (native) LockboxSetOwnerSigningKey(handle unsafe.Pointer, key unsafe.Pointer) C.bool {
	return C.lockbox_set_owner_signing_key(handle, key)
}
func (native) LockboxOwnerInspection(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_owner_inspection(handle)
}
func (native) LockboxDefineForm(handle unsafe.Pointer, alias unsafe.Pointer, alias_len C.size_t, name unsafe.Pointer, name_len C.size_t, description unsafe.Pointer, description_len C.size_t, fields_proto unsafe.Pointer, fields_len C.size_t) C.RevaultBuffer {
	return C.lockbox_define_form(handle, (*C.char)(alias), alias_len, (*C.char)(name), name_len, (*C.char)(description), description_len, (*C.uint8_t)(fields_proto), fields_len)
}
func (native) LockboxListFormDefinitions(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_list_form_definitions(handle)
}
func (native) LockboxResolveForm(handle unsafe.Pointer, reference unsafe.Pointer, reference_len C.size_t) C.RevaultBuffer {
	return C.lockbox_resolve_form(handle, (*C.char)(reference), reference_len)
}
func (native) LockboxListFormRevisions(handle unsafe.Pointer, type_id unsafe.Pointer, type_id_len C.size_t) C.RevaultBuffer {
	return C.lockbox_list_form_revisions(handle, (*C.char)(type_id), type_id_len)
}
func (native) LockboxCreateFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, type_reference unsafe.Pointer, type_len C.size_t, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.lockbox_create_form_record(handle, (*C.char)(path), path_len, (*C.char)(type_reference), type_len, (*C.char)(name), name_len)
}
func (native) LockboxSetFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t, value unsafe.Pointer, value_len C.size_t) C.bool {
	return C.lockbox_set_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len, (*C.char)(value), value_len)
}
func (native) LockboxSetSecretFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t, value unsafe.Pointer, value_len C.size_t) C.bool {
	return C.lockbox_set_secret_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len, (*C.uint8_t)(value), value_len)
}
func (native) LockboxListFormRecords(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_list_form_records(handle)
}
func (native) LockboxGetFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.RevaultBuffer {
	return C.lockbox_get_form_record(handle, (*C.char)(path), path_len)
}
func (native) LockboxDeleteFormRecord(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.lockbox_delete_form_record(handle, (*C.char)(path), path_len)
}
func (native) LockboxMoveFormRecords(handle unsafe.Pointer, moves_proto unsafe.Pointer, moves_len C.size_t) C.bool {
	return C.lockbox_move_form_records(handle, (*C.uint8_t)(moves_proto), moves_len)
}
func (native) LockboxGetFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t) C.RevaultBuffer {
	return C.lockbox_get_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len)
}
func (native) LockboxGetSecretFormField(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, field unsafe.Pointer, field_len C.size_t, output *unsafe.Pointer) C.bool {
	return C.lockbox_get_secret_form_field(handle, (*C.char)(path), path_len, (*C.char)(field), field_len, output)
}
func (native) LockboxToBytes(handle unsafe.Pointer) C.RevaultBuffer {
	return C.lockbox_to_bytes(handle)
}
func (native) LockboxFree(handle unsafe.Pointer) {
	C.lockbox_free(handle)
}
func (native) VaultIsRunning() C.bool {
	return C.vault_is_running()
}
func (native) VaultForgetAll() C.bool {
	return C.vault_forget_all()
}
func (native) KeyContactGenerate() unsafe.Pointer {
	return C.key_contact_generate()
}
func (native) KeyContactFromPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.key_contact_from_private((*C.uint8_t)(bytes), len)
}
func (native) KeyContactPublic(handle unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_public(handle)
}
func (native) KeyContactPrivate(handle unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_private(handle)
}
func (native) KeyContactPublicFromBytes(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.key_contact_public_from_bytes((*C.uint8_t)(bytes), len)
}
func (native) KeyContactPublicFree(handle unsafe.Pointer) {
	C.key_contact_public_free(handle)
}
func (native) KeyContactFree(handle unsafe.Pointer) {
	C.key_contact_free(handle)
}
func (native) KeyContactEncrypt(contact unsafe.Pointer, content_key unsafe.Pointer, key_len C.size_t) unsafe.Pointer {
	return C.key_contact_encrypt(contact, (*C.uint8_t)(content_key), key_len)
}
func (native) KeyContactDecrypt(contact unsafe.Pointer, wrapped unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_decrypt(contact, wrapped)
}
func (native) KeyContactWrappedPublic(wrapped unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_wrapped_public(wrapped)
}
func (native) KeyContactWrappedCiphertext(wrapped unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_wrapped_ciphertext(wrapped)
}
func (native) KeyContactWrappedEncrypted(wrapped unsafe.Pointer) C.RevaultBuffer {
	return C.key_contact_wrapped_encrypted(wrapped)
}
func (native) KeyContactWrappedFree(handle unsafe.Pointer) {
	C.key_contact_wrapped_free(handle)
}
func (native) KeySigningGenerate() unsafe.Pointer {
	return C.key_signing_generate()
}
func (native) KeySigningFromPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.key_signing_from_private((*C.uint8_t)(bytes), len)
}
func (native) KeySigningPublic(handle unsafe.Pointer) C.RevaultBuffer {
	return C.key_signing_public(handle)
}
func (native) KeySigningPrivate(handle unsafe.Pointer) C.RevaultBuffer {
	return C.key_signing_private(handle)
}
func (native) KeySigningPublicFromBytes(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.key_signing_public_from_bytes((*C.uint8_t)(bytes), len)
}
func (native) KeySigningPublicFree(handle unsafe.Pointer) {
	C.key_signing_public_free(handle)
}
func (native) KeySigningFree(handle unsafe.Pointer) {
	C.key_signing_free(handle)
}
func (native) VaultKeyExportPrivate(key unsafe.Pointer, format unsafe.Pointer, format_len C.size_t) C.RevaultBuffer {
	return C.vault_key_export_private(key, (*C.char)(format), format_len)
}
func (native) VaultKeyExportPublic(key unsafe.Pointer, format unsafe.Pointer, format_len C.size_t) C.RevaultBuffer {
	return C.vault_key_export_public(key, (*C.char)(format), format_len)
}
func (native) VaultKeyImportPrivate(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.vault_key_import_private((*C.uint8_t)(bytes), len)
}
func (native) VaultKeyImportPublic(bytes unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.vault_key_import_public((*C.uint8_t)(bytes), len)
}
func (native) VaultKeyFingerprint(key unsafe.Pointer) C.RevaultBuffer {
	return C.vault_key_fingerprint(key)
}
func (native) VaultKeyFormatHex(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_format_hex((*C.uint8_t)(bytes), len)
}
func (native) VaultKeyDecodeHex(text unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_decode_hex((*C.char)(text), len)
}
func (native) VaultKeyFormatCrockford(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_format_crockford((*C.uint8_t)(bytes), len)
}
func (native) VaultKeyFormatCrockfordReading(code unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_format_crockford_reading((*C.char)(code), len)
}
func (native) VaultKeyDecodeCrockford(code unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_decode_crockford((*C.char)(code), len)
}
func (native) VaultKeyHexEncode(bytes unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_hex_encode((*C.uint8_t)(bytes), len)
}
func (native) VaultKeyHexDecode(text unsafe.Pointer, len C.size_t) C.RevaultBuffer {
	return C.vault_key_hex_decode((*C.char)(text), len)
}
func (native) VaultDirectoryOpen(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_directory_open((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultStructureVersionCurrent() C.uint32_t {
	return C.vault_structure_version_current()
}
func (native) VaultDirectoryProbeStructureVersion(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) C.uint32_t {
	return C.vault_directory_probe_structure_version((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryOpenOrCreateDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_directory_open_or_create_default((*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryReplaceDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_directory_replace_default((*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryChangePassword(root unsafe.Pointer, root_len C.size_t, old_password unsafe.Pointer, old_len C.size_t, new_password unsafe.Pointer, new_len C.size_t) C.bool {
	return C.vault_directory_change_password((*C.char)(root), root_len, (*C.uint8_t)(old_password), old_len, (*C.uint8_t)(new_password), new_len)
}
func (native) VaultDirectoryChangeDefaultPassword(old_password unsafe.Pointer, old_len C.size_t, new_password unsafe.Pointer, new_len C.size_t) C.bool {
	return C.vault_directory_change_default_password((*C.uint8_t)(old_password), old_len, (*C.uint8_t)(new_password), new_len)
}
func (native) VaultDirectoryReplace(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_directory_replace((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryOpenOrCreate(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_directory_open_or_create((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryRoot(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_root(handle)
}
func (native) VaultDirectoryStructureVersion(handle unsafe.Pointer) C.uint32_t {
	return C.vault_directory_structure_version(handle)
}
func (native) VaultDirectoryListPrivateKeys(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_private_keys(handle)
}
func (native) VaultDirectoryListPrivateKeyNames(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_private_key_names(handle)
}
func (native) VaultDirectoryListContactNames(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_contact_names(handle)
}
func (native) VaultDirectoryListFormAliases(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_form_aliases(handle)
}
func (native) VaultDirectoryPrivateKeyExists(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.vault_directory_private_key_exists(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryDeletePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.vault_directory_delete_private_key(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryStorePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
	return C.vault_directory_store_private_key(handle, (*C.char)(name), name_len, key)
}
func (native) VaultDirectoryLoadPrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
	return C.vault_directory_load_private_key(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryLoadPrivateKeyGeneration(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, index C.uint16_t) unsafe.Pointer {
	return C.vault_directory_load_private_key_generation(handle, (*C.char)(name), name_len, index)
}
func (native) VaultDirectoryStoreContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
	return C.vault_directory_store_contact(handle, (*C.char)(name), name_len, key)
}
func (native) VaultDirectoryLoadContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
	return C.vault_directory_load_contact(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryContactExists(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.vault_directory_contact_exists(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryDeleteContact(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.vault_directory_delete_contact(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryListContacts(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_contacts(handle)
}
func (native) VaultDirectoryStoreProfileEmail(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, email unsafe.Pointer, email_len C.size_t) C.bool {
	return C.vault_directory_store_profile_email(handle, (*C.char)(name), name_len, (*C.char)(email), email_len)
}
func (native) VaultDirectoryProfileEmail(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_profile_email(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryStoreBackup(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, bytes unsafe.Pointer, len C.size_t) C.bool {
	return C.vault_directory_store_backup(handle, (*C.uint8_t)(id), id_len, (*C.uint8_t)(bytes), len)
}
func (native) VaultDirectoryLoadBackup(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_load_backup(handle, (*C.uint8_t)(id), id_len)
}
func (native) VaultDirectoryBackupCount(handle unsafe.Pointer) C.uint64_t {
	return C.vault_directory_backup_count(handle)
}
func (native) VaultDirectoryRestorePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer, signing_key unsafe.Pointer, overwrite C.bool) C.bool {
	return C.vault_directory_restore_private_key(handle, (*C.char)(name), name_len, key, signing_key, overwrite)
}
func (native) VaultDirectoryLoadOwnerSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
	return C.vault_directory_load_owner_signing_key(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryLoadOwnerSigningKeyGeneration(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, index C.uint16_t) unsafe.Pointer {
	return C.vault_directory_load_owner_signing_key_generation(handle, (*C.char)(name), name_len, index)
}
func (native) VaultDirectoryStoreContactSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, key unsafe.Pointer) C.bool {
	return C.vault_directory_store_contact_signing_key(handle, (*C.char)(name), name_len, key)
}
func (native) VaultDirectoryLoadContactSigningKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) unsafe.Pointer {
	return C.vault_directory_load_contact_signing_key(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryListProfileGenerations(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_list_profile_generations(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryRotatePrivateKey(handle unsafe.Pointer, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_rotate_private_key(handle, (*C.char)(name), name_len)
}
func (native) VaultDirectoryRememberLockbox(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.vault_directory_remember_lockbox(handle, (*C.uint8_t)(id), id_len, (*C.char)(path), path_len)
}
func (native) VaultDirectoryListKnownLockboxes(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_known_lockboxes(handle)
}
func (native) VaultDirectoryForgetLockbox(handle unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.vault_directory_forget_lockbox(handle, (*C.char)(path), path_len)
}
func (native) VaultDirectoryRememberAccessSlotLabel(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, slot_id C.uint64_t, name unsafe.Pointer, name_len C.size_t) C.bool {
	return C.vault_directory_remember_access_slot_label(handle, (*C.uint8_t)(id), id_len, slot_id, (*C.char)(name), name_len)
}
func (native) VaultDirectoryListAccessSlotLabels(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_list_access_slot_labels(handle, (*C.uint8_t)(id), id_len)
}
func (native) VaultDirectoryFindAccessSlotLabels(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, name unsafe.Pointer, name_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_find_access_slot_labels(handle, (*C.uint8_t)(id), id_len, (*C.char)(name), name_len)
}
func (native) VaultDirectoryForgetAccessSlotLabel(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, slot_id C.uint64_t) C.bool {
	return C.vault_directory_forget_access_slot_label(handle, (*C.uint8_t)(id), id_len, slot_id)
}
func (native) VaultDirectoryDefineForm(handle unsafe.Pointer, alias unsafe.Pointer, alias_len C.size_t, name unsafe.Pointer, name_len C.size_t, description unsafe.Pointer, description_len C.size_t, fields_proto unsafe.Pointer, fields_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_define_form(handle, (*C.char)(alias), alias_len, (*C.char)(name), name_len, (*C.char)(description), description_len, (*C.uint8_t)(fields_proto), fields_len)
}
func (native) VaultDirectoryResolveForm(handle unsafe.Pointer, reference unsafe.Pointer, reference_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_resolve_form(handle, (*C.char)(reference), reference_len)
}
func (native) VaultDirectoryListForms(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_directory_list_forms(handle)
}
func (native) VaultDirectoryListFormRevisions(handle unsafe.Pointer, type_id unsafe.Pointer, type_id_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_list_form_revisions(handle, (*C.char)(type_id), type_id_len)
}
func (native) VaultDirectorySeedForms(handle unsafe.Pointer) C.size_t {
	return C.vault_directory_seed_forms(handle)
}
func (native) VaultDirectoryRememberPassword(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t, password unsafe.Pointer, password_len C.size_t) C.bool {
	return C.vault_directory_remember_password(handle, (*C.uint8_t)(id), id_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultDirectoryRememberedPassword(handle unsafe.Pointer, id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
	return C.vault_directory_remembered_password(handle, (*C.uint8_t)(id), id_len)
}
func (native) VaultBackupDefault(path unsafe.Pointer, path_len C.size_t, overwrite C.bool) C.RevaultBuffer {
	return C.vault_backup_default((*C.char)(path), path_len, overwrite)
}
func (native) VaultRestoreDefault(path unsafe.Pointer, path_len C.size_t, overwrite C.bool) C.RevaultBuffer {
	return C.vault_restore_default((*C.char)(path), path_len, overwrite)
}
func (native) VaultDirectoryFree(handle unsafe.Pointer) {
	C.vault_directory_free(handle)
}
func (native) VaultReadOnlyOpen(root unsafe.Pointer, root_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_read_only_open((*C.char)(root), root_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultReadOnlyOpenDefault(password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_read_only_open_default((*C.uint8_t)(password), password_len)
}
func (native) VaultReadOnlyListProfileNames(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_read_only_list_profile_names(handle)
}
func (native) VaultReadOnlyListContactNames(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_read_only_list_contact_names(handle)
}
func (native) VaultReadOnlyListFormAliases(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_read_only_list_form_aliases(handle)
}
func (native) VaultReadOnlyListKnownLockboxes(handle unsafe.Pointer) C.RevaultBuffer {
	return C.vault_read_only_list_known_lockboxes(handle)
}
func (native) VaultReadOnlyFree(handle unsafe.Pointer) {
	C.vault_read_only_free(handle)
}
func (native) VaultAgentServe() C.bool {
	return C.vault_agent_serve()
}
func (native) VaultAgentVerifyTransport() C.bool {
	return C.vault_agent_verify_transport()
}
func (native) VaultAgentGet(id unsafe.Pointer, id_len C.size_t) C.RevaultBuffer {
	return C.vault_agent_get((*C.uint8_t)(id), id_len)
}
func (native) VaultAgentPut(id unsafe.Pointer, id_len C.size_t, key unsafe.Pointer, key_len C.size_t) C.bool {
	return C.vault_agent_put((*C.uint8_t)(id), id_len, (*C.uint8_t)(key), key_len)
}
func (native) VaultAgentForget(id unsafe.Pointer, id_len C.size_t) C.bool {
	return C.vault_agent_forget((*C.uint8_t)(id), id_len)
}
func (native) VaultAgentStop() C.bool {
	return C.vault_agent_stop()
}
func (native) VaultAgentStart() C.bool {
	return C.vault_agent_start()
}
func (native) VaultAgentList() C.RevaultBuffer {
	return C.vault_agent_list()
}
func (native) VaultAgentSleepSupport() C.RevaultBuffer {
	return C.vault_agent_sleep_support()
}
func (native) VaultPlatformStatus() C.RevaultBuffer {
	return C.vault_platform_status()
}
func (native) VaultPlatformSetScope(scope unsafe.Pointer, len C.size_t) C.bool {
	return C.vault_platform_set_scope((*C.char)(scope), len)
}
func (native) VaultPlatformForgetPassword() C.bool {
	return C.vault_platform_forget_password()
}
func (native) VaultPlatformPutPassword(password unsafe.Pointer, len C.size_t) C.bool {
	return C.vault_platform_put_password((*C.uint8_t)(password), len)
}
func (native) VaultPlatformEnable() C.bool {
	return C.vault_platform_enable()
}
func (native) VaultPlatformDisable() C.bool {
	return C.vault_platform_disable()
}
func (native) VaultPlatformDisabled() C.bool {
	return C.vault_platform_disabled()
}
func (native) VaultPlatformGetPassword() C.RevaultBuffer {
	return C.vault_platform_get_password()
}
func (native) VaultDefaultDirectory() C.RevaultBuffer {
	return C.vault_default_directory()
}
func (native) VaultDefaultPath() C.RevaultBuffer {
	return C.vault_default_path()
}
func (native) VaultAgentLogPath() C.RevaultBuffer {
	return C.vault_agent_log_path()
}
func (native) VaultAgentLogDestination() C.RevaultBuffer {
	return C.vault_agent_log_destination()
}
func (native) VaultAgentGetVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t) C.RevaultBuffer {
	return C.vault_agent_get_vault_unlock_key((*C.char)(vault_id), vault_id_len)
}
func (native) VaultAgentPutVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t, key unsafe.Pointer, key_len C.size_t, ttl_seconds C.uint64_t) C.bool {
	return C.vault_agent_put_vault_unlock_key((*C.char)(vault_id), vault_id_len, (*C.uint8_t)(key), key_len, ttl_seconds)
}
func (native) VaultAgentForgetVaultUnlockKey(vault_id unsafe.Pointer, vault_id_len C.size_t) C.bool {
	return C.vault_agent_forget_vault_unlock_key((*C.char)(vault_id), vault_id_len)
}
func (native) VaultAgentGetOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t) unsafe.Pointer {
	return C.vault_agent_get_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len)
}
func (native) VaultAgentPutOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t, key unsafe.Pointer, ttl_seconds C.uint64_t) C.bool {
	return C.vault_agent_put_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len, key, ttl_seconds)
}
func (native) VaultAgentForgetOwnerSigningKey(vault_id unsafe.Pointer, vault_len C.size_t, profile unsafe.Pointer, profile_len C.size_t) C.bool {
	return C.vault_agent_forget_owner_signing_key((*C.char)(vault_id), vault_len, (*C.char)(profile), profile_len)
}
func (native) VaultAgentBeginActivity(kind unsafe.Pointer, len C.size_t) unsafe.Pointer {
	return C.vault_agent_begin_activity((*C.char)(kind), len)
}
func (native) VaultAgentEndActivity(handle unsafe.Pointer) {
	C.vault_agent_end_activity(handle)
}
func (native) VaultLocal() unsafe.Pointer {
	return C.vault_local()
}
func (native) VaultCreateLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_create_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultOpenLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t) unsafe.Pointer {
	return C.vault_open_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len)
}
func (native) VaultCreateLockboxContentKey(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
	return C.vault_create_lockbox_content_key(vault, (*C.char)(path), path_len, (*C.uint8_t)(content_key), key_len, signing_key)
}
func (native) VaultCreateLockboxContact(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, contact unsafe.Pointer, name unsafe.Pointer, name_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
	return C.vault_create_lockbox_contact(vault, (*C.char)(path), path_len, contact, (*C.char)(name), name_len, signing_key)
}
func (native) VaultOpenLockboxContentKey(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, content_key unsafe.Pointer, key_len C.size_t, signing_key unsafe.Pointer) unsafe.Pointer {
	return C.vault_open_lockbox_content_key(vault, (*C.char)(path), path_len, (*C.uint8_t)(content_key), key_len, signing_key)
}
func (native) VaultCacheLockboxPassword(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t, password unsafe.Pointer, password_len C.size_t, ttl_seconds C.uint64_t) C.bool {
	return C.vault_cache_lockbox_password(vault, (*C.char)(path), path_len, (*C.uint8_t)(password), password_len, ttl_seconds)
}
func (native) VaultCloseLockbox(vault unsafe.Pointer, path unsafe.Pointer, path_len C.size_t) C.bool {
	return C.vault_close_lockbox(vault, (*C.char)(path), path_len)
}
func (native) VaultCloseAll(vault unsafe.Pointer) C.bool {
	return C.vault_close_all(vault)
}
func (native) VaultFree(vault unsafe.Pointer) {
	C.vault_free(vault)
}
