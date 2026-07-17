// Generated complete class-oriented Swift API. Do not edit.
import Foundation
import RevaultC
import SwiftProtobuf

public enum RevaultError: Error { case native(String); case invalidFrame }

private func lastError() -> String {
    guard let value = buffer_last_error() else { return "native reVault operation failed" }
    return String(cString: value)
}

private func take(_ buffer: RevaultBuffer) throws -> Data {
    guard let pointer = buffer.ptr else { throw RevaultError.native(lastError()) }
    let value = Data(bytes: pointer, count: buffer.len)
    buffer_free(buffer)
    return value
}

private func payload(_ buffer: RevaultBuffer) throws -> Data {
    let frame = try take(buffer)
    guard frame.count >= 12, Array(frame.prefix(4)) == [76, 66, 87, 70] else { throw RevaultError.invalidFrame }
    let length = frame[8..<12].reduce(0) { ($0 << 8) | Int($1) }
    guard length == frame.count - 12 else { throw RevaultError.invalidFrame }
    return Data(frame.dropFirst(12))
}

private func withSecret<T>(
    _ getter: (UnsafeMutablePointer<UnsafeMutableRawPointer?>) -> Bool,
    _ callback: (UnsafeRawBufferPointer) throws -> T
) throws -> T? {
    var handle: UnsafeMutableRawPointer?
    guard getter(&handle) else { throw RevaultError.native(lastError()) }
    guard let handle else { return nil }
    defer { secret_free(handle) }
    var length = 0
    guard secret_len(handle, &length) else { throw RevaultError.native(lastError()) }
    var bytes = [UInt8](repeating: 0, count: length)
    defer {
        bytes.withUnsafeMutableBytes { raw in
            _ = raw.initializeMemory(as: UInt8.self, repeating: 0)
        }
    }
    guard bytes.withUnsafeMutableBytes({ raw in
        secret_copy(handle, raw.bindMemory(to: UInt8.self).baseAddress, length)
    }) else {
        throw RevaultError.native(lastError())
    }
    return try bytes.withUnsafeBytes(callback)
}

final class BindingOperations {
    init() { precondition(api_abi_version() == 2, "revault-api native ABI mismatch; expected 2") }
    func lastErrorMessage() -> String { lastError() }

    func bufferLastErrorDetails() throws -> Revault_Bindings_ErrorDetails {
        return try Revault_Bindings_ErrorDetails(serializedBytes: payload(buffer_last_error_details()))
    }

    func lockboxFormatVersion() throws -> UInt16 {
        return lockbox_format_version()
    }

    func lockboxProbeFormatVersion(_ bytes: Data) throws -> UInt16 {
        return try bytes.withUnsafeBytes { bytesBytes in
            return lockbox_probe_format_version(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count)
        }
    }

    func lockboxCreate(_ key: Data) throws -> UnsafeMutableRawPointer {
        return try key.withUnsafeBytes { keyBytes in
            guard let value = lockbox_create(keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func lockboxCreateWithOptions(_ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> UnsafeMutableRawPointer {
        return try key.withUnsafeBytes { keyBytes in
            return try cacheMode.withCString { cacheModePointer in
                return try workload.withCString { workloadPointer in
                    return try worker.withCString { workerPointer in
                        guard let value = lockbox_create_with_options(keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count, cacheModePointer, cacheMode.utf8.count, cacheBytes, workloadPointer, workload.utf8.count, workerPointer, worker.utf8.count, jobs) else { throw RevaultError.native(lastError()) }
                        return value
                    }
                }
            }
        }
    }

    func lockboxCreatePassword(_ password: Data) throws -> UnsafeMutableRawPointer {
        return try password.withUnsafeBytes { passwordBytes in
            guard let value = lockbox_create_password(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func lockboxCreateContact(_ contact: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        guard let value = lockbox_create_contact(contact) else { throw RevaultError.native(lastError()) }
        return value
    }

    func lockboxCreateWithSigningKey(_ contentKey: Data, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try contentKey.withUnsafeBytes { contentKeyBytes in
            guard let value = lockbox_create_with_signing_key(contentKeyBytes.bindMemory(to: UInt8.self).baseAddress, contentKey.count, signingKey) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func lockboxOpen(_ archive: Data, _ key: Data) throws -> UnsafeMutableRawPointer {
        return try archive.withUnsafeBytes { archiveBytes in
            return try key.withUnsafeBytes { keyBytes in
                guard let value = lockbox_open(archiveBytes.bindMemory(to: UInt8.self).baseAddress, archive.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func lockboxOpenWithOptions(_ archive: Data, _ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> UnsafeMutableRawPointer {
        return try archive.withUnsafeBytes { archiveBytes in
            return try key.withUnsafeBytes { keyBytes in
                return try cacheMode.withCString { cacheModePointer in
                    return try workload.withCString { workloadPointer in
                        return try worker.withCString { workerPointer in
                            guard let value = lockbox_open_with_options(archiveBytes.bindMemory(to: UInt8.self).baseAddress, archive.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count, cacheModePointer, cacheMode.utf8.count, cacheBytes, workloadPointer, workload.utf8.count, workerPointer, worker.utf8.count, jobs) else { throw RevaultError.native(lastError()) }
                            return value
                        }
                    }
                }
            }
        }
    }

    func lockboxOpenPassword(_ archive: Data, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try archive.withUnsafeBytes { archiveBytes in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = lockbox_open_password(archiveBytes.bindMemory(to: UInt8.self).baseAddress, archive.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func lockboxOpenContact(_ archive: Data, _ contact: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try archive.withUnsafeBytes { archiveBytes in
            guard let value = lockbox_open_contact(archiveBytes.bindMemory(to: UInt8.self).baseAddress, archive.count, contact) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func lockboxAddFile(_ handle: UnsafeMutableRawPointer, _ path: String, _ data: Data, _ replace: Bool) throws -> Bool {
        return try path.withCString { pathPointer in
            return try data.withUnsafeBytes { dataBytes in
                guard lockbox_add_file(handle, pathPointer, path.utf8.count, dataBytes.bindMemory(to: UInt8.self).baseAddress, data.count, replace) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxAddFileWithPermissions(_ handle: UnsafeMutableRawPointer, _ path: String, _ data: Data, _ permissions: UInt32, _ replace: Bool) throws -> Bool {
        return try path.withCString { pathPointer in
            return try data.withUnsafeBytes { dataBytes in
                guard lockbox_add_file_with_permissions(handle, pathPointer, path.utf8.count, dataBytes.bindMemory(to: UInt8.self).baseAddress, data.count, permissions, replace) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxGetFile(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Data {
        return try path.withCString { pathPointer in
            return try take(lockbox_get_file(handle, pathPointer, path.utf8.count))
        }
    }

    func lockboxExtractFile(_ handle: UnsafeMutableRawPointer, _ source: String, _ destination: String, _ replace: Bool) throws -> Bool {
        return try source.withCString { sourcePointer in
            return try destination.withCString { destinationPointer in
                guard lockbox_extract_file(handle, sourcePointer, source.utf8.count, destinationPointer, destination.utf8.count, replace) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxExtractDirectory(_ handle: UnsafeMutableRawPointer, _ destination: String, _ maxFileBytes: UInt64, _ maxTotalBytes: UInt64, _ maxFiles: Int, _ restoreSymlinks: Bool, _ restorePermissions: Bool, _ overwrite: Bool) throws -> Bool {
        return try destination.withCString { destinationPointer in
            guard lockbox_extract_directory(handle, destinationPointer, destination.utf8.count, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxStreamContent(_ handle: UnsafeMutableRawPointer, _ physical: Bool) throws -> Revault_Bindings_StreamChunkList {
        return try Revault_Bindings_StreamChunkList(serializedBytes: payload(lockbox_stream_content(handle, physical)))
    }

    func lockboxCacheStats(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_CacheStats {
        return try Revault_Bindings_CacheStats(serializedBytes: payload(lockbox_cache_stats(handle)))
    }

    func lockboxImportStats(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_ImportStats {
        return try Revault_Bindings_ImportStats(serializedBytes: payload(lockbox_import_stats(handle)))
    }

    func lockboxResetImportStats(_ handle: UnsafeMutableRawPointer) throws -> Bool {
        guard lockbox_reset_import_stats(handle) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxInspectFile(_ path: String) throws -> Revault_Bindings_FileInspection {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_FileInspection(serializedBytes: payload(lockbox_inspect_file(pathPointer, path.utf8.count)))
        }
    }

    func lockboxPageInspection(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_PageInspectionList {
        return try Revault_Bindings_PageInspectionList(serializedBytes: payload(lockbox_page_inspection(handle)))
    }

    func lockboxRecoveryReport(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_RecoveryReport {
        return try Revault_Bindings_RecoveryReport(serializedBytes: payload(lockbox_recovery_report(handle)))
    }

    func lockboxRecoveryReportRender(_ handle: UnsafeMutableRawPointer, _ verbose: Bool, _ maxEntries: Int) throws -> String {
        return String(decoding: try take(lockbox_recovery_report_render(handle, verbose, maxEntries)), as: UTF8.self)
    }

    func lockboxRecoveryScanPath(_ path: String, _ key: Data) throws -> Revault_Bindings_RecoveryReport {
        return try path.withCString { pathPointer in
            return try key.withUnsafeBytes { keyBytes in
                return try Revault_Bindings_RecoveryReport(serializedBytes: payload(lockbox_recovery_scan_path(pathPointer, path.utf8.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count)))
            }
        }
    }

    func lockboxStorageLen(_ handle: UnsafeMutableRawPointer) throws -> UInt64 {
        return lockbox_storage_len(handle)
    }

    func lockboxSetWorkloadProfile(_ handle: UnsafeMutableRawPointer, _ profile: String) throws -> Bool {
        return try profile.withCString { profilePointer in
            guard lockbox_set_workload_profile(handle, profilePointer, profile.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxSetWorkerPolicy(_ handle: UnsafeMutableRawPointer, _ mode: String, _ jobs: Int) throws -> Bool {
        return try mode.withCString { modePointer in
            guard lockbox_set_worker_policy(handle, modePointer, mode.utf8.count, jobs) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxRuntimeOptions(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_RuntimeOptions {
        return try Revault_Bindings_RuntimeOptions(serializedBytes: payload(lockbox_runtime_options(handle)))
    }

    func lockboxCommit(_ handle: UnsafeMutableRawPointer) throws -> Bool {
        guard lockbox_commit(handle) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxCreateDir(_ handle: UnsafeMutableRawPointer, _ path: String, _ createParents: Bool) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_create_dir(handle, pathPointer, path.utf8.count, createParents) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxDelete(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_delete(handle, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxRemoveDir(_ handle: UnsafeMutableRawPointer, _ path: String, _ recursive: Bool) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_remove_dir(handle, pathPointer, path.utf8.count, recursive) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxCreateParentDirs(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_create_parent_dirs(handle, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxRename(_ handle: UnsafeMutableRawPointer, _ from: String, _ to: String) throws -> Bool {
        return try from.withCString { fromPointer in
            return try to.withCString { toPointer in
                guard lockbox_rename(handle, fromPointer, from.utf8.count, toPointer, to.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxList(_ handle: UnsafeMutableRawPointer, _ path: String, _ recursive: Bool) throws -> Revault_Bindings_LockboxEntryList {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_LockboxEntryList(serializedBytes: payload(lockbox_list(handle, pathPointer, path.utf8.count, recursive)))
        }
    }

    func lockboxListWithOptions(_ handle: UnsafeMutableRawPointer, _ path: String, _ glob: String, _ recursive: Bool, _ includeFiles: Bool, _ includeSymlinks: Bool, _ includeDirectories: Bool, _ limit: Int) throws -> Revault_Bindings_LockboxEntryList {
        return try path.withCString { pathPointer in
            return try glob.withCString { globPointer in
                return try Revault_Bindings_LockboxEntryList(serializedBytes: payload(lockbox_list_with_options(handle, pathPointer, path.utf8.count, globPointer, glob.utf8.count, recursive, includeFiles, includeSymlinks, includeDirectories, limit)))
            }
        }
    }

    func lockboxStat(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Revault_Bindings_OptionalLockboxEntry {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_OptionalLockboxEntry(serializedBytes: payload(lockbox_stat(handle, pathPointer, path.utf8.count)))
        }
    }

    func lockboxSetVariable(_ handle: UnsafeMutableRawPointer, _ name: String, _ value: String) throws -> Bool {
        return try name.withCString { namePointer in
            return try value.withCString { valuePointer in
                guard lockbox_set_variable(handle, namePointer, name.utf8.count, valuePointer, value.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxSetSecretVariable(_ handle: UnsafeMutableRawPointer, _ name: String, _ value: Data) throws -> Bool {
        var secret = [UInt8](value)
        defer { secret.withUnsafeMutableBytes { $0.initializeMemory(as: UInt8.self, repeating: 0) } }
        return try name.withCString { namePointer in
            try secret.withUnsafeBytes { bytes in
                guard lockbox_set_secret_variable(handle, namePointer, name.utf8.count, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxGetVariable(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> String? {
        return try name.withCString { namePointer in
            let value = try Revault_Bindings_OptionalString(serializedBytes: payload(lockbox_get_variable(handle, namePointer, name.utf8.count)))
            return value.present ? value.value : nil
        }
    }

    func lockboxWithSecretVariable<T>(_ handle: UnsafeMutableRawPointer, _ name: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        try name.withCString { namePointer in
            try withSecret({ lockbox_get_secret_variable(handle, namePointer, name.utf8.count, $0) }, callback)
        }
    }

    func lockboxDeleteVariable(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            guard lockbox_delete_variable(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxMoveVariables(_ handle: UnsafeMutableRawPointer, _ movesProto: Data) throws -> Bool {
        return try movesProto.withUnsafeBytes { movesProtoBytes in
            guard lockbox_move_variables(handle, movesProtoBytes.bindMemory(to: UInt8.self).baseAddress, movesProto.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxListVariables(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_VariableList {
        return try Revault_Bindings_VariableList(serializedBytes: payload(lockbox_list_variables(handle)))
    }

    func lockboxVariableSensitivity(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Revault_Bindings_OptionalString {
        return try name.withCString { namePointer in
            return try Revault_Bindings_OptionalString(serializedBytes: payload(lockbox_variable_sensitivity(handle, namePointer, name.utf8.count)))
        }
    }

    func lockboxAddSymlink(_ handle: UnsafeMutableRawPointer, _ path: String, _ target: String, _ replace: Bool) throws -> Bool {
        return try path.withCString { pathPointer in
            return try target.withCString { targetPointer in
                guard lockbox_add_symlink(handle, pathPointer, path.utf8.count, targetPointer, target.utf8.count, replace) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxGetSymlinkTarget(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> String {
        return try path.withCString { pathPointer in
            return String(decoding: try take(lockbox_get_symlink_target(handle, pathPointer, path.utf8.count)), as: UTF8.self)
        }
    }

    func lockboxId(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(lockbox_id(handle))
    }

    func lockboxExists(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            return lockbox_exists(handle, pathPointer, path.utf8.count)
        }
    }

    func lockboxIsDir(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            return lockbox_is_dir(handle, pathPointer, path.utf8.count)
        }
    }

    func lockboxPermissions(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> UInt32 {
        return try path.withCString { pathPointer in
            return lockbox_permissions(handle, pathPointer, path.utf8.count)
        }
    }

    func lockboxSetPermissions(_ handle: UnsafeMutableRawPointer, _ path: String, _ permissions: UInt32) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_set_permissions(handle, pathPointer, path.utf8.count, permissions) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxReadRange(_ handle: UnsafeMutableRawPointer, _ path: String, _ offset: UInt64, _ len: UInt64) throws -> Data {
        return try path.withCString { pathPointer in
            return try take(lockbox_read_range(handle, pathPointer, path.utf8.count, offset, len))
        }
    }

    func lockboxRecoveryScan(_ bytes: Data, _ key: Data) throws -> Revault_Bindings_RecoveryReport {
        return try bytes.withUnsafeBytes { bytesBytes in
            return try key.withUnsafeBytes { keyBytes in
                return try Revault_Bindings_RecoveryReport(serializedBytes: payload(lockbox_recovery_scan(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count)))
            }
        }
    }

    func lockboxRecoverySalvage(_ bytes: Data, _ key: Data, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            return try key.withUnsafeBytes { keyBytes in
                guard let value = lockbox_recovery_salvage(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count, signingKey) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func lockboxAddPassword(_ handle: UnsafeMutableRawPointer, _ password: Data) throws -> UInt64 {
        return try password.withUnsafeBytes { passwordBytes in
            return lockbox_add_password(handle, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count)
        }
    }

    func lockboxAddContact(_ handle: UnsafeMutableRawPointer, _ contact: UnsafeMutableRawPointer, _ name: String) throws -> UInt64 {
        return try name.withCString { namePointer in
            return lockbox_add_contact(handle, contact, namePointer, name.utf8.count)
        }
    }

    func lockboxDeleteKey(_ handle: UnsafeMutableRawPointer, _ id: UInt64) throws -> Bool {
        guard lockbox_delete_key(handle, id) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxListKeySlots(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_KeySlotList {
        return try Revault_Bindings_KeySlotList(serializedBytes: payload(lockbox_list_key_slots(handle)))
    }

    func lockboxSetOwnerSigningKey(_ handle: UnsafeMutableRawPointer, _ key: UnsafeMutableRawPointer) throws -> Bool {
        guard lockbox_set_owner_signing_key(handle, key) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxOwnerInspection(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_OwnerInspection {
        return try Revault_Bindings_OwnerInspection(serializedBytes: payload(lockbox_owner_inspection(handle)))
    }

    func lockboxDefineForm(_ handle: UnsafeMutableRawPointer, _ alias: String, _ name: String, _ description: String, _ fieldsProto: Data) throws -> Revault_Bindings_FormDefinition {
        return try alias.withCString { aliasPointer in
            return try name.withCString { namePointer in
                return try description.withCString { descriptionPointer in
                    return try fieldsProto.withUnsafeBytes { fieldsProtoBytes in
                        return try Revault_Bindings_FormDefinition(serializedBytes: payload(lockbox_define_form(handle, aliasPointer, alias.utf8.count, namePointer, name.utf8.count, descriptionPointer, description.utf8.count, fieldsProtoBytes.bindMemory(to: UInt8.self).baseAddress, fieldsProto.count)))
                    }
                }
            }
        }
    }

    func lockboxListFormDefinitions(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_FormDefinitionList {
        return try Revault_Bindings_FormDefinitionList(serializedBytes: payload(lockbox_list_form_definitions(handle)))
    }

    func lockboxResolveForm(_ handle: UnsafeMutableRawPointer, _ reference: String) throws -> Revault_Bindings_FormDefinition {
        return try reference.withCString { referencePointer in
            return try Revault_Bindings_FormDefinition(serializedBytes: payload(lockbox_resolve_form(handle, referencePointer, reference.utf8.count)))
        }
    }

    func lockboxListFormRevisions(_ handle: UnsafeMutableRawPointer, _ typeId: String) throws -> Revault_Bindings_FormDefinitionList {
        return try typeId.withCString { typeIdPointer in
            return try Revault_Bindings_FormDefinitionList(serializedBytes: payload(lockbox_list_form_revisions(handle, typeIdPointer, typeId.utf8.count)))
        }
    }

    func lockboxCreateFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String, _ typeReference: String, _ name: String) throws -> Revault_Bindings_FormRecord {
        return try path.withCString { pathPointer in
            return try typeReference.withCString { typeReferencePointer in
                return try name.withCString { namePointer in
                    return try Revault_Bindings_FormRecord(serializedBytes: payload(lockbox_create_form_record(handle, pathPointer, path.utf8.count, typeReferencePointer, typeReference.utf8.count, namePointer, name.utf8.count)))
                }
            }
        }
    }

    func lockboxSetFormField(_ handle: UnsafeMutableRawPointer, _ path: String, _ field: String, _ value: String) throws -> Bool {
        return try path.withCString { pathPointer in
            return try field.withCString { fieldPointer in
                return try value.withCString { valuePointer in
                    guard lockbox_set_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count, valuePointer, value.utf8.count) else { throw RevaultError.native(lastError()) }
                    return true
                }
            }
        }
    }

    func lockboxSetSecretFormField(_ handle: UnsafeMutableRawPointer, _ path: String, _ field: String, _ value: Data) throws -> Bool {
        var secret = [UInt8](value)
        defer { secret.withUnsafeMutableBytes { $0.initializeMemory(as: UInt8.self, repeating: 0) } }
        return try path.withCString { pathPointer in
            try field.withCString { fieldPointer in
                try secret.withUnsafeBytes { bytes in
                    guard lockbox_set_secret_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
                    return true
                }
            }
        }
    }

    func lockboxListFormRecords(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_FormRecordList {
        return try Revault_Bindings_FormRecordList(serializedBytes: payload(lockbox_list_form_records(handle)))
    }

    func lockboxGetFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Revault_Bindings_OptionalFormRecord {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_OptionalFormRecord(serializedBytes: payload(lockbox_get_form_record(handle, pathPointer, path.utf8.count)))
        }
    }

    func lockboxDeleteFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_delete_form_record(handle, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxMoveFormRecords(_ handle: UnsafeMutableRawPointer, _ movesProto: Data) throws -> Bool {
        return try movesProto.withUnsafeBytes { movesProtoBytes in
            guard lockbox_move_form_records(handle, movesProtoBytes.bindMemory(to: UInt8.self).baseAddress, movesProto.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxGetFormField(_ handle: UnsafeMutableRawPointer, _ path: String, _ field: String) throws -> Revault_Bindings_OptionalFormValue {
        return try path.withCString { pathPointer in
            return try field.withCString { fieldPointer in
                return try Revault_Bindings_OptionalFormValue(serializedBytes: payload(lockbox_get_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count)))
            }
        }
    }

    func lockboxWithSecretFormField<T>(_ handle: UnsafeMutableRawPointer, _ path: String, _ field: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        try path.withCString { pathPointer in
            try field.withCString { fieldPointer in
                try withSecret({ lockbox_get_secret_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count, $0) }, callback)
            }
        }
    }

    func lockboxToBytes(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(lockbox_to_bytes(handle))
    }

    func lockboxFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        lockbox_free(handle)
    }

    func vaultIsRunning() throws -> Bool {
        return vault_is_running()
    }

    func vaultForgetAll() throws -> Bool {
        guard vault_forget_all() else { throw RevaultError.native(lastError()) }
        return true
    }

    func keyContactGenerate() throws -> UnsafeMutableRawPointer {
        guard let value = key_contact_generate() else { throw RevaultError.native(lastError()) }
        return value
    }

    func keyContactFromPrivate(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = key_contact_from_private(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func keyContactPublic(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_public(handle))
    }

    func keyContactPrivate(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_private(handle))
    }

    func keyContactPublicFromBytes(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = key_contact_public_from_bytes(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func keyContactPublicFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        key_contact_public_free(handle)
    }

    func keyContactFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        key_contact_free(handle)
    }

    func keyContactEncrypt(_ contact: UnsafeMutableRawPointer, _ contentKey: Data) throws -> UnsafeMutableRawPointer {
        return try contentKey.withUnsafeBytes { contentKeyBytes in
            guard let value = key_contact_encrypt(contact, contentKeyBytes.bindMemory(to: UInt8.self).baseAddress, contentKey.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func keyContactDecrypt(_ contact: UnsafeMutableRawPointer, _ wrapped: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_decrypt(contact, wrapped))
    }

    func keyContactWrappedPublic(_ wrapped: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_wrapped_public(wrapped))
    }

    func keyContactWrappedCiphertext(_ wrapped: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_wrapped_ciphertext(wrapped))
    }

    func keyContactWrappedEncrypted(_ wrapped: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_contact_wrapped_encrypted(wrapped))
    }

    func keyContactWrappedFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        key_contact_wrapped_free(handle)
    }

    func keySigningGenerate() throws -> UnsafeMutableRawPointer {
        guard let value = key_signing_generate() else { throw RevaultError.native(lastError()) }
        return value
    }

    func keySigningFromPrivate(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = key_signing_from_private(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func keySigningPublic(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_signing_public(handle))
    }

    func keySigningPrivate(_ handle: UnsafeMutableRawPointer) throws -> Data {
        return try take(key_signing_private(handle))
    }

    func keySigningPublicFromBytes(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = key_signing_public_from_bytes(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func keySigningPublicFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        key_signing_public_free(handle)
    }

    func keySigningFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        key_signing_free(handle)
    }

    func vaultKeyExportPrivate(_ key: UnsafeMutableRawPointer, _ format: String) throws -> Data {
        return try format.withCString { formatPointer in
            return try take(vault_key_export_private(key, formatPointer, format.utf8.count))
        }
    }

    func vaultKeyExportPublic(_ key: UnsafeMutableRawPointer, _ format: String) throws -> Data {
        return try format.withCString { formatPointer in
            return try take(vault_key_export_public(key, formatPointer, format.utf8.count))
        }
    }

    func vaultKeyImportPrivate(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = vault_key_import_private(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultKeyImportPublic(_ bytes: Data) throws -> UnsafeMutableRawPointer {
        return try bytes.withUnsafeBytes { bytesBytes in
            guard let value = vault_key_import_public(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultKeyFingerprint(_ key: UnsafeMutableRawPointer) throws -> Data {
        return try take(vault_key_fingerprint(key))
    }

    func vaultKeyFormatHex(_ bytes: Data) throws -> String {
        return try bytes.withUnsafeBytes { bytesBytes in
            return String(decoding: try take(vault_key_format_hex(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count)), as: UTF8.self)
        }
    }

    func vaultKeyDecodeHex(_ text: String) throws -> Data {
        return try text.withCString { textPointer in
            return try take(vault_key_decode_hex(textPointer, text.utf8.count))
        }
    }

    func vaultKeyFormatCrockford(_ bytes: Data) throws -> String {
        return try bytes.withUnsafeBytes { bytesBytes in
            return String(decoding: try take(vault_key_format_crockford(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count)), as: UTF8.self)
        }
    }

    func vaultKeyFormatCrockfordReading(_ code: String) throws -> String {
        return try code.withCString { codePointer in
            return String(decoding: try take(vault_key_format_crockford_reading(codePointer, code.utf8.count)), as: UTF8.self)
        }
    }

    func vaultKeyDecodeCrockford(_ code: String) throws -> Data {
        return try code.withCString { codePointer in
            return try take(vault_key_decode_crockford(codePointer, code.utf8.count))
        }
    }

    func vaultKeyHexEncode(_ bytes: Data) throws -> String {
        return try bytes.withUnsafeBytes { bytesBytes in
            return String(decoding: try take(vault_key_hex_encode(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count)), as: UTF8.self)
        }
    }

    func vaultKeyHexDecode(_ text: String) throws -> Data {
        return try text.withCString { textPointer in
            return try take(vault_key_hex_decode(textPointer, text.utf8.count))
        }
    }

    func vaultDirectoryOpen(_ root: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try root.withCString { rootPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_directory_open(rootPointer, root.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultStructureVersionCurrent() throws -> UInt32 {
        return vault_structure_version_current()
    }

    func vaultDirectoryProbeStructureVersion(_ root: String, _ password: Data) throws -> UInt32 {
        return try root.withCString { rootPointer in
            return try password.withUnsafeBytes { passwordBytes in
                return vault_directory_probe_structure_version(rootPointer, root.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count)
            }
        }
    }

    func vaultDirectoryOpenOrCreateDefault(_ password: Data) throws -> UnsafeMutableRawPointer {
        return try password.withUnsafeBytes { passwordBytes in
            guard let value = vault_directory_open_or_create_default(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryReplaceDefault(_ password: Data) throws -> UnsafeMutableRawPointer {
        return try password.withUnsafeBytes { passwordBytes in
            guard let value = vault_directory_replace_default(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryChangePassword(_ root: String, _ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try root.withCString { rootPointer in
            return try oldPassword.withUnsafeBytes { oldPasswordBytes in
                return try newPassword.withUnsafeBytes { newPasswordBytes in
                    guard vault_directory_change_password(rootPointer, root.utf8.count, oldPasswordBytes.bindMemory(to: UInt8.self).baseAddress, oldPassword.count, newPasswordBytes.bindMemory(to: UInt8.self).baseAddress, newPassword.count) else { throw RevaultError.native(lastError()) }
                    return true
                }
            }
        }
    }

    func vaultDirectoryChangeDefaultPassword(_ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try oldPassword.withUnsafeBytes { oldPasswordBytes in
            return try newPassword.withUnsafeBytes { newPasswordBytes in
                guard vault_directory_change_default_password(oldPasswordBytes.bindMemory(to: UInt8.self).baseAddress, oldPassword.count, newPasswordBytes.bindMemory(to: UInt8.self).baseAddress, newPassword.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryReplace(_ root: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try root.withCString { rootPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_directory_replace(rootPointer, root.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultDirectoryOpenOrCreate(_ root: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try root.withCString { rootPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_directory_open_or_create(rootPointer, root.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultDirectoryRoot(_ handle: UnsafeMutableRawPointer) throws -> String {
        return String(decoding: try take(vault_directory_root(handle)), as: UTF8.self)
    }

    func vaultDirectoryStructureVersion(_ handle: UnsafeMutableRawPointer) throws -> UInt32 {
        return vault_directory_structure_version(handle)
    }

    func vaultDirectoryListPrivateKeys(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_directory_list_private_keys(handle)))
    }

    func vaultDirectoryListPrivateKeyNames(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_directory_list_private_key_names(handle)))
    }

    func vaultDirectoryListContactNames(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_directory_list_contact_names(handle)))
    }

    func vaultDirectoryListFormAliases(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_directory_list_form_aliases(handle)))
    }

    func vaultDirectoryPrivateKeyExists(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            return vault_directory_private_key_exists(handle, namePointer, name.utf8.count)
        }
    }

    func vaultDirectoryDeletePrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_delete_private_key(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryStorePrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String, _ key: UnsafeMutableRawPointer) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_store_private_key(handle, namePointer, name.utf8.count, key) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryLoadPrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_private_key(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryLoadPrivateKeyGeneration(_ handle: UnsafeMutableRawPointer, _ name: String, _ index: UInt16) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_private_key_generation(handle, namePointer, name.utf8.count, index) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryStoreContact(_ handle: UnsafeMutableRawPointer, _ name: String, _ key: UnsafeMutableRawPointer) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_store_contact(handle, namePointer, name.utf8.count, key) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryLoadContact(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_contact(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryContactExists(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            return vault_directory_contact_exists(handle, namePointer, name.utf8.count)
        }
    }

    func vaultDirectoryDeleteContact(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_delete_contact(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryListContacts(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_ContactList {
        return try Revault_Bindings_ContactList(serializedBytes: payload(vault_directory_list_contacts(handle)))
    }

    func vaultDirectoryStoreProfileEmail(_ handle: UnsafeMutableRawPointer, _ name: String, _ email: String) throws -> Bool {
        return try name.withCString { namePointer in
            return try email.withCString { emailPointer in
                guard vault_directory_store_profile_email(handle, namePointer, name.utf8.count, emailPointer, email.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryProfileEmail(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Revault_Bindings_OptionalString {
        return try name.withCString { namePointer in
            return try Revault_Bindings_OptionalString(serializedBytes: payload(vault_directory_profile_email(handle, namePointer, name.utf8.count)))
        }
    }

    func vaultDirectoryStoreBackup(_ handle: UnsafeMutableRawPointer, _ id: Data, _ bytes: Data) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            return try bytes.withUnsafeBytes { bytesBytes in
                guard vault_directory_store_backup(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryLoadBackup(_ handle: UnsafeMutableRawPointer, _ id: Data) throws -> Data {
        return try id.withUnsafeBytes { idBytes in
            return try take(vault_directory_load_backup(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count))
        }
    }

    func vaultDirectoryBackupCount(_ handle: UnsafeMutableRawPointer) throws -> UInt64 {
        return vault_directory_backup_count(handle)
    }

    func vaultDirectoryRestorePrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String, _ key: UnsafeMutableRawPointer, _ signingKey: UnsafeMutableRawPointer, _ overwrite: Bool) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_restore_private_key(handle, namePointer, name.utf8.count, key, signingKey, overwrite) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryLoadOwnerSigningKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_owner_signing_key(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryLoadOwnerSigningKeyGeneration(_ handle: UnsafeMutableRawPointer, _ name: String, _ index: UInt16) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_owner_signing_key_generation(handle, namePointer, name.utf8.count, index) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryStoreContactSigningKey(_ handle: UnsafeMutableRawPointer, _ name: String, _ key: UnsafeMutableRawPointer) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_store_contact_signing_key(handle, namePointer, name.utf8.count, key) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryLoadContactSigningKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> UnsafeMutableRawPointer {
        return try name.withCString { namePointer in
            guard let value = vault_directory_load_contact_signing_key(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultDirectoryListProfileGenerations(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Revault_Bindings_ProfileHistory {
        return try name.withCString { namePointer in
            return try Revault_Bindings_ProfileHistory(serializedBytes: payload(vault_directory_list_profile_generations(handle, namePointer, name.utf8.count)))
        }
    }

    func vaultDirectoryRotatePrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Revault_Bindings_ProfileHistory {
        return try name.withCString { namePointer in
            return try Revault_Bindings_ProfileHistory(serializedBytes: payload(vault_directory_rotate_private_key(handle, namePointer, name.utf8.count)))
        }
    }

    func vaultDirectoryRememberLockbox(_ handle: UnsafeMutableRawPointer, _ id: Data, _ path: String) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            return try path.withCString { pathPointer in
                guard vault_directory_remember_lockbox(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryListKnownLockboxes(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_KnownLockboxList {
        return try Revault_Bindings_KnownLockboxList(serializedBytes: payload(vault_directory_list_known_lockboxes(handle)))
    }

    func vaultDirectoryForgetLockbox(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard vault_directory_forget_lockbox(handle, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryRememberAccessSlotLabel(_ handle: UnsafeMutableRawPointer, _ id: Data, _ slotId: UInt64, _ name: String) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            return try name.withCString { namePointer in
                guard vault_directory_remember_access_slot_label(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, slotId, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryListAccessSlotLabels(_ handle: UnsafeMutableRawPointer, _ id: Data) throws -> Revault_Bindings_AccessSlotLabelList {
        return try id.withUnsafeBytes { idBytes in
            return try Revault_Bindings_AccessSlotLabelList(serializedBytes: payload(vault_directory_list_access_slot_labels(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count)))
        }
    }

    func vaultDirectoryFindAccessSlotLabels(_ handle: UnsafeMutableRawPointer, _ id: Data, _ name: String) throws -> Revault_Bindings_AccessSlotLabelList {
        return try id.withUnsafeBytes { idBytes in
            return try name.withCString { namePointer in
                return try Revault_Bindings_AccessSlotLabelList(serializedBytes: payload(vault_directory_find_access_slot_labels(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, namePointer, name.utf8.count)))
            }
        }
    }

    func vaultDirectoryForgetAccessSlotLabel(_ handle: UnsafeMutableRawPointer, _ id: Data, _ slotId: UInt64) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            guard vault_directory_forget_access_slot_label(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, slotId) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryDefineForm(_ handle: UnsafeMutableRawPointer, _ alias: String, _ name: String, _ description: String, _ fieldsProto: Data) throws -> Revault_Bindings_FormDefinition {
        return try alias.withCString { aliasPointer in
            return try name.withCString { namePointer in
                return try description.withCString { descriptionPointer in
                    return try fieldsProto.withUnsafeBytes { fieldsProtoBytes in
                        return try Revault_Bindings_FormDefinition(serializedBytes: payload(vault_directory_define_form(handle, aliasPointer, alias.utf8.count, namePointer, name.utf8.count, descriptionPointer, description.utf8.count, fieldsProtoBytes.bindMemory(to: UInt8.self).baseAddress, fieldsProto.count)))
                    }
                }
            }
        }
    }

    func vaultDirectoryResolveForm(_ handle: UnsafeMutableRawPointer, _ reference: String) throws -> Revault_Bindings_FormDefinition {
        return try reference.withCString { referencePointer in
            return try Revault_Bindings_FormDefinition(serializedBytes: payload(vault_directory_resolve_form(handle, referencePointer, reference.utf8.count)))
        }
    }

    func vaultDirectoryListForms(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_FormDefinitionList {
        return try Revault_Bindings_FormDefinitionList(serializedBytes: payload(vault_directory_list_forms(handle)))
    }

    func vaultDirectoryListFormRevisions(_ handle: UnsafeMutableRawPointer, _ typeId: String) throws -> Revault_Bindings_FormDefinitionList {
        return try typeId.withCString { typeIdPointer in
            return try Revault_Bindings_FormDefinitionList(serializedBytes: payload(vault_directory_list_form_revisions(handle, typeIdPointer, typeId.utf8.count)))
        }
    }

    func vaultDirectorySeedForms(_ handle: UnsafeMutableRawPointer) throws -> Int {
        return vault_directory_seed_forms(handle)
    }

    func vaultDirectoryRememberPassword(_ handle: UnsafeMutableRawPointer, _ id: Data, _ password: Data) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            return try password.withUnsafeBytes { passwordBytes in
                guard vault_directory_remember_password(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryRememberedPassword(_ handle: UnsafeMutableRawPointer, _ id: Data) throws -> Data {
        return try id.withUnsafeBytes { idBytes in
            return try take(vault_directory_remembered_password(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count))
        }
    }

    func vaultBackupDefault(_ path: String, _ overwrite: Bool) throws -> Revault_Bindings_VaultBackupManifest {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_VaultBackupManifest(serializedBytes: payload(vault_backup_default(pathPointer, path.utf8.count, overwrite)))
        }
    }

    func vaultRestoreDefault(_ path: String, _ overwrite: Bool) throws -> Revault_Bindings_VaultBackupManifest {
        return try path.withCString { pathPointer in
            return try Revault_Bindings_VaultBackupManifest(serializedBytes: payload(vault_restore_default(pathPointer, path.utf8.count, overwrite)))
        }
    }

    func vaultDirectoryFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        vault_directory_free(handle)
    }

    func vaultReadOnlyOpen(_ root: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try root.withCString { rootPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_read_only_open(rootPointer, root.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultReadOnlyOpenDefault(_ password: Data) throws -> UnsafeMutableRawPointer {
        return try password.withUnsafeBytes { passwordBytes in
            guard let value = vault_read_only_open_default(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultReadOnlyListProfileNames(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_read_only_list_profile_names(handle)))
    }

    func vaultReadOnlyListContactNames(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_read_only_list_contact_names(handle)))
    }

    func vaultReadOnlyListFormAliases(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_StringList {
        return try Revault_Bindings_StringList(serializedBytes: payload(vault_read_only_list_form_aliases(handle)))
    }

    func vaultReadOnlyListKnownLockboxes(_ handle: UnsafeMutableRawPointer) throws -> Revault_Bindings_KnownLockboxList {
        return try Revault_Bindings_KnownLockboxList(serializedBytes: payload(vault_read_only_list_known_lockboxes(handle)))
    }

    func vaultReadOnlyFree(_ handle: UnsafeMutableRawPointer) throws -> Void {
        vault_read_only_free(handle)
    }

    func vaultAgentServe() throws -> Bool {
        guard vault_agent_serve() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultAgentVerifyTransport() throws -> Bool {
        guard vault_agent_verify_transport() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultAgentGet(_ id: Data) throws -> Data {
        return try id.withUnsafeBytes { idBytes in
            return try take(vault_agent_get(idBytes.bindMemory(to: UInt8.self).baseAddress, id.count))
        }
    }

    func vaultAgentPut(_ id: Data, _ key: Data) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            return try key.withUnsafeBytes { keyBytes in
                guard vault_agent_put(idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultAgentForget(_ id: Data) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            guard vault_agent_forget(idBytes.bindMemory(to: UInt8.self).baseAddress, id.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultAgentStop() throws -> Bool {
        guard vault_agent_stop() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultAgentStart() throws -> Bool {
        guard vault_agent_start() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultAgentList() throws -> Revault_Bindings_AgentEntryList {
        return try Revault_Bindings_AgentEntryList(serializedBytes: payload(vault_agent_list()))
    }

    func vaultAgentSleepSupport() throws -> Revault_Bindings_SleepSupport {
        return try Revault_Bindings_SleepSupport(serializedBytes: payload(vault_agent_sleep_support()))
    }

    func vaultPlatformStatus() throws -> Revault_Bindings_PlatformStatus {
        return try Revault_Bindings_PlatformStatus(serializedBytes: payload(vault_platform_status()))
    }

    func vaultPlatformSetScope(_ scope: String) throws -> Bool {
        return try scope.withCString { scopePointer in
            guard vault_platform_set_scope(scopePointer, scope.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultPlatformForgetPassword() throws -> Bool {
        guard vault_platform_forget_password() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultPlatformPutPassword(_ password: Data) throws -> Bool {
        return try password.withUnsafeBytes { passwordBytes in
            guard vault_platform_put_password(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultPlatformEnable() throws -> Bool {
        guard vault_platform_enable() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultPlatformDisable() throws -> Bool {
        guard vault_platform_disable() else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultPlatformDisabled() throws -> Bool {
        return vault_platform_disabled()
    }

    func vaultPlatformGetPassword() throws -> Data {
        return try take(vault_platform_get_password())
    }

    func vaultDefaultDirectory() throws -> String {
        return String(decoding: try take(vault_default_directory()), as: UTF8.self)
    }

    func vaultDefaultPath() throws -> String {
        return String(decoding: try take(vault_default_path()), as: UTF8.self)
    }

    func vaultAgentLogPath() throws -> String {
        return String(decoding: try take(vault_agent_log_path()), as: UTF8.self)
    }

    func vaultAgentLogDestination() throws -> String {
        return String(decoding: try take(vault_agent_log_destination()), as: UTF8.self)
    }

    func vaultAgentGetVaultUnlockKey(_ vaultId: String) throws -> Data {
        return try vaultId.withCString { vaultIdPointer in
            return try take(vault_agent_get_vault_unlock_key(vaultIdPointer, vaultId.utf8.count))
        }
    }

    func vaultAgentPutVaultUnlockKey(_ vaultId: String, _ key: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try vaultId.withCString { vaultIdPointer in
            return try key.withUnsafeBytes { keyBytes in
                guard vault_agent_put_vault_unlock_key(vaultIdPointer, vaultId.utf8.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count, ttlSeconds) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultAgentForgetVaultUnlockKey(_ vaultId: String) throws -> Bool {
        return try vaultId.withCString { vaultIdPointer in
            guard vault_agent_forget_vault_unlock_key(vaultIdPointer, vaultId.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultAgentGetOwnerSigningKey(_ vaultId: String, _ profile: String) throws -> UnsafeMutableRawPointer {
        return try vaultId.withCString { vaultIdPointer in
            return try profile.withCString { profilePointer in
                guard let value = vault_agent_get_owner_signing_key(vaultIdPointer, vaultId.utf8.count, profilePointer, profile.utf8.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultAgentPutOwnerSigningKey(_ vaultId: String, _ profile: String, _ key: UnsafeMutableRawPointer, _ ttlSeconds: UInt64) throws -> Bool {
        return try vaultId.withCString { vaultIdPointer in
            return try profile.withCString { profilePointer in
                guard vault_agent_put_owner_signing_key(vaultIdPointer, vaultId.utf8.count, profilePointer, profile.utf8.count, key, ttlSeconds) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultAgentForgetOwnerSigningKey(_ vaultId: String, _ profile: String) throws -> Bool {
        return try vaultId.withCString { vaultIdPointer in
            return try profile.withCString { profilePointer in
                guard vault_agent_forget_owner_signing_key(vaultIdPointer, vaultId.utf8.count, profilePointer, profile.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultAgentBeginActivity(_ kind: String) throws -> UnsafeMutableRawPointer {
        return try kind.withCString { kindPointer in
            guard let value = vault_agent_begin_activity(kindPointer, kind.utf8.count) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func vaultAgentEndActivity(_ handle: UnsafeMutableRawPointer) throws -> Void {
        vault_agent_end_activity(handle)
    }

    func vaultLocal() throws -> UnsafeMutableRawPointer {
        guard let value = vault_local() else { throw RevaultError.native(lastError()) }
        return value
    }

    func vaultCreateLockboxPassword(_ vault: UnsafeMutableRawPointer, _ path: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try path.withCString { pathPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_create_lockbox_password(vault, pathPointer, path.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultOpenLockboxPassword(_ vault: UnsafeMutableRawPointer, _ path: String, _ password: Data) throws -> UnsafeMutableRawPointer {
        return try path.withCString { pathPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard let value = vault_open_lockbox_password(vault, pathPointer, path.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultCreateLockboxContentKey(_ vault: UnsafeMutableRawPointer, _ path: String, _ contentKey: Data, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try path.withCString { pathPointer in
            return try contentKey.withUnsafeBytes { contentKeyBytes in
                guard let value = vault_create_lockbox_content_key(vault, pathPointer, path.utf8.count, contentKeyBytes.bindMemory(to: UInt8.self).baseAddress, contentKey.count, signingKey) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultCreateLockboxContact(_ vault: UnsafeMutableRawPointer, _ path: String, _ contact: UnsafeMutableRawPointer, _ name: String, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try path.withCString { pathPointer in
            return try name.withCString { namePointer in
                guard let value = vault_create_lockbox_contact(vault, pathPointer, path.utf8.count, contact, namePointer, name.utf8.count, signingKey) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultOpenLockboxContentKey(_ vault: UnsafeMutableRawPointer, _ path: String, _ contentKey: Data, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try path.withCString { pathPointer in
            return try contentKey.withUnsafeBytes { contentKeyBytes in
                guard let value = vault_open_lockbox_content_key(vault, pathPointer, path.utf8.count, contentKeyBytes.bindMemory(to: UInt8.self).baseAddress, contentKey.count, signingKey) else { throw RevaultError.native(lastError()) }
                return value
            }
        }
    }

    func vaultCacheLockboxPassword(_ vault: UnsafeMutableRawPointer, _ path: String, _ password: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try path.withCString { pathPointer in
            return try password.withUnsafeBytes { passwordBytes in
                guard vault_cache_lockbox_password(vault, pathPointer, path.utf8.count, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count, ttlSeconds) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultCloseLockbox(_ vault: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard vault_close_lockbox(vault, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultCloseAll(_ vault: UnsafeMutableRawPointer) throws -> Bool {
        guard vault_close_all(vault) else { throw RevaultError.native(lastError()) }
        return true
    }

    func vaultFree(_ vault: UnsafeMutableRawPointer) throws -> Void {
        vault_free(vault)
    }

}

public class OwnedHandle {
    fileprivate let operations: BindingOperations
    fileprivate var handle: UnsafeMutableRawPointer?
    fileprivate init(_ operations: BindingOperations, _ handle: UnsafeMutableRawPointer?) { self.operations = operations; self.handle = handle }
}

public final class Lockbox: OwnedHandle {}

public final class ContactKeyPair: OwnedHandle {}

public final class ContactPublicKey: OwnedHandle {}

public final class WrappedContactKey: OwnedHandle {}

public final class SigningKeyPair: OwnedHandle {}

public final class SigningPublicKey: OwnedHandle {}

public final class VaultDirectory: OwnedHandle {}

public final class ReadOnlyVaultDirectory: OwnedHandle {}

public final class Agent: OwnedHandle {}

public final class AgentActivity: OwnedHandle {}

public final class Platform: OwnedHandle {}

public final class LocalVault: OwnedHandle {}

public final class Vault {
    fileprivate let operations = BindingOperations()
    public lazy var agent = Agent(operations, nil)
    public lazy var platform = Platform(operations, nil)
    public init() {}
    public func lastError() -> String { operations.lastErrorMessage() }
    public func lastErrorDetails() throws -> Revault_Bindings_ErrorDetails { try operations.bufferLastErrorDetails() }

    public func lockboxFormatVersion() throws -> UInt16 {
        return try operations.lockboxFormatVersion()
    }

    public func lockboxProbeFormatVersion(_ bytes: Data) throws -> UInt16 {
        return try operations.lockboxProbeFormatVersion(bytes)
    }

    public func lockboxCreate(_ key: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreate(key))
    }

    public func lockboxCreateWithOptions(_ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs))
    }

    public func lockboxCreatePassword(_ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreatePassword(password))
    }

    public func lockboxCreateContact(_ contact: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateContact(contact.handle!))
    }

    public func lockboxCreateWithSigningKey(_ contentKey: Data, _ signingKey: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateWithSigningKey(contentKey, signingKey.handle!))
    }

    public func lockboxOpen(_ archive: Data, _ key: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpen(archive, key))
    }

    public func lockboxOpenWithOptions(_ archive: Data, _ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs))
    }

    public func lockboxOpenPassword(_ archive: Data, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenPassword(archive, password))
    }

    public func lockboxOpenContact(_ archive: Data, _ contact: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenContact(archive, contact.handle!))
    }

    public func lockboxInspectFile(_ path: String) throws -> Revault_Bindings_FileInspection {
        return try operations.lockboxInspectFile(path)
    }

    public func lockboxRecoveryScanPath(_ path: String, _ key: Data) throws -> Revault_Bindings_RecoveryReport {
        return try operations.lockboxRecoveryScanPath(path, key)
    }

    public func lockboxRecoveryScan(_ bytes: Data, _ key: Data) throws -> Revault_Bindings_RecoveryReport {
        return try operations.lockboxRecoveryScan(bytes, key)
    }

    public func lockboxRecoverySalvage(_ bytes: Data, _ key: Data, _ signingKey: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxRecoverySalvage(bytes, key, signingKey.handle!))
    }

    public func keyContactGenerate() throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.keyContactGenerate())
    }

    public func keyContactFromPrivate(_ bytes: Data) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.keyContactFromPrivate(bytes))
    }

    public func keyContactPublicFromBytes(_ bytes: Data) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.keyContactPublicFromBytes(bytes))
    }

    public func keySigningGenerate() throws -> SigningKeyPair {
        return SigningKeyPair(operations, try operations.keySigningGenerate())
    }

    public func keySigningFromPrivate(_ bytes: Data) throws -> SigningKeyPair {
        return SigningKeyPair(operations, try operations.keySigningFromPrivate(bytes))
    }

    public func keySigningPublicFromBytes(_ bytes: Data) throws -> SigningPublicKey {
        return SigningPublicKey(operations, try operations.keySigningPublicFromBytes(bytes))
    }

    public func vaultKeyExportPrivate(_ key: OwnedHandle, _ format: String) throws -> Data {
        return try operations.vaultKeyExportPrivate(key.handle!, format)
    }

    public func vaultKeyExportPublic(_ key: OwnedHandle, _ format: String) throws -> Data {
        return try operations.vaultKeyExportPublic(key.handle!, format)
    }

    public func vaultKeyImportPrivate(_ bytes: Data) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultKeyImportPrivate(bytes))
    }

    public func vaultKeyImportPublic(_ bytes: Data) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.vaultKeyImportPublic(bytes))
    }

    public func vaultKeyFingerprint(_ key: OwnedHandle) throws -> Data {
        return try operations.vaultKeyFingerprint(key.handle!)
    }

    public func vaultKeyFormatHex(_ bytes: Data) throws -> String {
        return try operations.vaultKeyFormatHex(bytes)
    }

    public func vaultKeyDecodeHex(_ text: String) throws -> Data {
        return try operations.vaultKeyDecodeHex(text)
    }

    public func vaultKeyFormatCrockford(_ bytes: Data) throws -> String {
        return try operations.vaultKeyFormatCrockford(bytes)
    }

    public func vaultKeyFormatCrockfordReading(_ code: String) throws -> String {
        return try operations.vaultKeyFormatCrockfordReading(code)
    }

    public func vaultKeyDecodeCrockford(_ code: String) throws -> Data {
        return try operations.vaultKeyDecodeCrockford(code)
    }

    public func vaultKeyHexEncode(_ bytes: Data) throws -> String {
        return try operations.vaultKeyHexEncode(bytes)
    }

    public func vaultKeyHexDecode(_ text: String) throws -> Data {
        return try operations.vaultKeyHexDecode(text)
    }

    public func vaultDirectoryOpen(_ root: String, _ password: Data) throws -> VaultDirectory {
        return VaultDirectory(operations, try operations.vaultDirectoryOpen(root, password))
    }

    public func vaultStructureVersionCurrent() throws -> UInt32 {
        return try operations.vaultStructureVersionCurrent()
    }

    public func vaultDirectoryProbeStructureVersion(_ root: String, _ password: Data) throws -> UInt32 {
        return try operations.vaultDirectoryProbeStructureVersion(root, password)
    }

    public func vaultDirectoryOpenOrCreateDefault(_ password: Data) throws -> VaultDirectory {
        return VaultDirectory(operations, try operations.vaultDirectoryOpenOrCreateDefault(password))
    }

    public func vaultDirectoryReplaceDefault(_ password: Data) throws -> VaultDirectory {
        return VaultDirectory(operations, try operations.vaultDirectoryReplaceDefault(password))
    }

    public func vaultDirectoryChangePassword(_ root: String, _ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try operations.vaultDirectoryChangePassword(root, oldPassword, newPassword)
    }

    public func vaultDirectoryChangeDefaultPassword(_ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword)
    }

    public func vaultDirectoryReplace(_ root: String, _ password: Data) throws -> VaultDirectory {
        return VaultDirectory(operations, try operations.vaultDirectoryReplace(root, password))
    }

    public func vaultDirectoryOpenOrCreate(_ root: String, _ password: Data) throws -> VaultDirectory {
        return VaultDirectory(operations, try operations.vaultDirectoryOpenOrCreate(root, password))
    }

    public func vaultBackupDefault(_ path: String, _ overwrite: Bool) throws -> Revault_Bindings_VaultBackupManifest {
        return try operations.vaultBackupDefault(path, overwrite)
    }

    public func vaultRestoreDefault(_ path: String, _ overwrite: Bool) throws -> Revault_Bindings_VaultBackupManifest {
        return try operations.vaultRestoreDefault(path, overwrite)
    }

    public func vaultReadOnlyOpen(_ root: String, _ password: Data) throws -> ReadOnlyVaultDirectory {
        return ReadOnlyVaultDirectory(operations, try operations.vaultReadOnlyOpen(root, password))
    }

    public func vaultReadOnlyOpenDefault(_ password: Data) throws -> ReadOnlyVaultDirectory {
        return ReadOnlyVaultDirectory(operations, try operations.vaultReadOnlyOpenDefault(password))
    }

    public func vaultDefaultDirectory() throws -> String {
        return try operations.vaultDefaultDirectory()
    }

    public func vaultDefaultPath() throws -> String {
        return try operations.vaultDefaultPath()
    }

    public func vaultAgentLogPath() throws -> String {
        return try operations.vaultAgentLogPath()
    }

    public func vaultAgentLogDestination() throws -> String {
        return try operations.vaultAgentLogDestination()
    }

    public func vaultLocal() throws -> LocalVault {
        return LocalVault(operations, try operations.vaultLocal())
    }

}

public extension Lockbox {
    public func addFile(_ path: String, _ data: Data, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddFile(handle!, path, data, replace)
    }

    public func addFileWithPermissions(_ path: String, _ data: Data, _ permissions: UInt32, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddFileWithPermissions(handle!, path, data, permissions, replace)
    }

    public func getFile(_ path: String) throws -> Data {
        return try operations.lockboxGetFile(handle!, path)
    }

    public func extractFile(_ source: String, _ destination: String, _ replace: Bool) throws -> Bool {
        return try operations.lockboxExtractFile(handle!, source, destination, replace)
    }

    public func extractDirectory(_ destination: String, _ maxFileBytes: UInt64, _ maxTotalBytes: UInt64, _ maxFiles: Int, _ restoreSymlinks: Bool, _ restorePermissions: Bool, _ overwrite: Bool) throws -> Bool {
        return try operations.lockboxExtractDirectory(handle!, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite)
    }

    public func streamContent(_ physical: Bool) throws -> Revault_Bindings_StreamChunkList {
        return try operations.lockboxStreamContent(handle!, physical)
    }

    public func cacheStats() throws -> Revault_Bindings_CacheStats {
        return try operations.lockboxCacheStats(handle!)
    }

    public func importStats() throws -> Revault_Bindings_ImportStats {
        return try operations.lockboxImportStats(handle!)
    }

    public func resetImportStats() throws -> Bool {
        return try operations.lockboxResetImportStats(handle!)
    }

    public func pageInspection() throws -> Revault_Bindings_PageInspectionList {
        return try operations.lockboxPageInspection(handle!)
    }

    public func recoveryReport() throws -> Revault_Bindings_RecoveryReport {
        return try operations.lockboxRecoveryReport(handle!)
    }

    public func recoveryReportRender(_ verbose: Bool, _ maxEntries: Int) throws -> String {
        return try operations.lockboxRecoveryReportRender(handle!, verbose, maxEntries)
    }

    public func storageLen() throws -> UInt64 {
        return try operations.lockboxStorageLen(handle!)
    }

    public func setWorkloadProfile(_ profile: String) throws -> Bool {
        return try operations.lockboxSetWorkloadProfile(handle!, profile)
    }

    public func setWorkerPolicy(_ mode: String, _ jobs: Int) throws -> Bool {
        return try operations.lockboxSetWorkerPolicy(handle!, mode, jobs)
    }

    public func runtimeOptions() throws -> Revault_Bindings_RuntimeOptions {
        return try operations.lockboxRuntimeOptions(handle!)
    }

    public func commit() throws -> Bool {
        return try operations.lockboxCommit(handle!)
    }

    public func createDir(_ path: String, _ createParents: Bool) throws -> Bool {
        return try operations.lockboxCreateDir(handle!, path, createParents)
    }

    public func delete(_ path: String) throws -> Bool {
        return try operations.lockboxDelete(handle!, path)
    }

    public func removeDir(_ path: String, _ recursive: Bool) throws -> Bool {
        return try operations.lockboxRemoveDir(handle!, path, recursive)
    }

    public func createParentDirs(_ path: String) throws -> Bool {
        return try operations.lockboxCreateParentDirs(handle!, path)
    }

    public func rename(_ from: String, _ to: String) throws -> Bool {
        return try operations.lockboxRename(handle!, from, to)
    }

    public func list(_ path: String, _ recursive: Bool) throws -> Revault_Bindings_LockboxEntryList {
        return try operations.lockboxList(handle!, path, recursive)
    }

    public func listWithOptions(_ path: String, _ glob: String, _ recursive: Bool, _ includeFiles: Bool, _ includeSymlinks: Bool, _ includeDirectories: Bool, _ limit: Int) throws -> Revault_Bindings_LockboxEntryList {
        return try operations.lockboxListWithOptions(handle!, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit)
    }

    public func stat(_ path: String) throws -> Revault_Bindings_OptionalLockboxEntry {
        return try operations.lockboxStat(handle!, path)
    }

    public func setVariable(_ name: String, _ value: String) throws -> Bool {
        return try operations.lockboxSetVariable(handle!, name, value)
    }

    public func setSecretVariable(_ name: String, _ value: Data) throws -> Bool {
        return try operations.lockboxSetSecretVariable(handle!, name, value)
    }

    public func getVariable(_ name: String) throws -> String? {
        return try operations.lockboxGetVariable(handle!, name)
    }

    public func withSecretVariable<T>(_ name: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        return try operations.lockboxWithSecretVariable(handle!, name, callback)
    }

    public func deleteVariable(_ name: String) throws -> Bool {
        return try operations.lockboxDeleteVariable(handle!, name)
    }

    public func moveVariables(_ movesProto: Data) throws -> Bool {
        return try operations.lockboxMoveVariables(handle!, movesProto)
    }

    public func listVariables() throws -> Revault_Bindings_VariableList {
        return try operations.lockboxListVariables(handle!)
    }

    public func variableSensitivity(_ name: String) throws -> Revault_Bindings_OptionalString {
        return try operations.lockboxVariableSensitivity(handle!, name)
    }

    public func addSymlink(_ path: String, _ target: String, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddSymlink(handle!, path, target, replace)
    }

    public func getSymlinkTarget(_ path: String) throws -> String {
        return try operations.lockboxGetSymlinkTarget(handle!, path)
    }

    public func id() throws -> Data {
        return try operations.lockboxId(handle!)
    }

    public func exists(_ path: String) throws -> Bool {
        return try operations.lockboxExists(handle!, path)
    }

    public func isDir(_ path: String) throws -> Bool {
        return try operations.lockboxIsDir(handle!, path)
    }

    public func permissions(_ path: String) throws -> UInt32 {
        return try operations.lockboxPermissions(handle!, path)
    }

    public func setPermissions(_ path: String, _ permissions: UInt32) throws -> Bool {
        return try operations.lockboxSetPermissions(handle!, path, permissions)
    }

    public func readRange(_ path: String, _ offset: UInt64, _ len: UInt64) throws -> Data {
        return try operations.lockboxReadRange(handle!, path, offset, len)
    }

    public func addPassword(_ password: Data) throws -> UInt64 {
        return try operations.lockboxAddPassword(handle!, password)
    }

    public func addContact(_ contact: OwnedHandle, _ name: String) throws -> UInt64 {
        return try operations.lockboxAddContact(handle!, contact.handle!, name)
    }

    public func deleteKey(_ id: UInt64) throws -> Bool {
        return try operations.lockboxDeleteKey(handle!, id)
    }

    public func listKeySlots() throws -> Revault_Bindings_KeySlotList {
        return try operations.lockboxListKeySlots(handle!)
    }

    public func setOwnerSigningKey(_ key: OwnedHandle) throws -> Bool {
        return try operations.lockboxSetOwnerSigningKey(handle!, key.handle!)
    }

    public func ownerInspection() throws -> Revault_Bindings_OwnerInspection {
        return try operations.lockboxOwnerInspection(handle!)
    }

    public func defineForm(_ alias: String, _ name: String, _ description: String, _ fieldsProto: Data) throws -> Revault_Bindings_FormDefinition {
        return try operations.lockboxDefineForm(handle!, alias, name, description, fieldsProto)
    }

    public func listFormDefinitions() throws -> Revault_Bindings_FormDefinitionList {
        return try operations.lockboxListFormDefinitions(handle!)
    }

    public func resolveForm(_ reference: String) throws -> Revault_Bindings_FormDefinition {
        return try operations.lockboxResolveForm(handle!, reference)
    }

    public func listFormRevisions(_ typeId: String) throws -> Revault_Bindings_FormDefinitionList {
        return try operations.lockboxListFormRevisions(handle!, typeId)
    }

    public func createFormRecord(_ path: String, _ typeReference: String, _ name: String) throws -> Revault_Bindings_FormRecord {
        return try operations.lockboxCreateFormRecord(handle!, path, typeReference, name)
    }

    public func setFormField(_ path: String, _ field: String, _ value: String) throws -> Bool {
        return try operations.lockboxSetFormField(handle!, path, field, value)
    }

    public func setSecretFormField(_ path: String, _ field: String, _ value: Data) throws -> Bool {
        return try operations.lockboxSetSecretFormField(handle!, path, field, value)
    }

    public func listFormRecords() throws -> Revault_Bindings_FormRecordList {
        return try operations.lockboxListFormRecords(handle!)
    }

    public func getFormRecord(_ path: String) throws -> Revault_Bindings_OptionalFormRecord {
        return try operations.lockboxGetFormRecord(handle!, path)
    }

    public func deleteFormRecord(_ path: String) throws -> Bool {
        return try operations.lockboxDeleteFormRecord(handle!, path)
    }

    public func moveFormRecords(_ movesProto: Data) throws -> Bool {
        return try operations.lockboxMoveFormRecords(handle!, movesProto)
    }

    public func getFormField(_ path: String, _ field: String) throws -> Revault_Bindings_OptionalFormValue {
        return try operations.lockboxGetFormField(handle!, path, field)
    }

    public func withSecretFormField<T>(_ path: String, _ field: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        return try operations.lockboxWithSecretFormField(handle!, path, field, callback)
    }

    public func toBytes() throws -> Data {
        return try operations.lockboxToBytes(handle!)
    }

    public func free() throws -> Void {
        try operations.lockboxFree(handle!)
        handle = nil
    }

}

public extension ContactKeyPair {
    public func publicBytes() throws -> Data {
        return try operations.keyContactPublic(handle!)
    }

    public func privateBytes() throws -> Data {
        return try operations.keyContactPrivate(handle!)
    }

    public func free() throws -> Void {
        try operations.keyContactFree(handle!)
        handle = nil
    }

    public func decrypt(_ wrapped: OwnedHandle) throws -> Data {
        return try operations.keyContactDecrypt(handle!, wrapped.handle!)
    }

}

public extension ContactPublicKey {
    public func publicFree() throws -> Void {
        try operations.keyContactPublicFree(handle!)
        handle = nil
    }

    public func encrypt(_ contentKey: Data) throws -> WrappedContactKey {
        return WrappedContactKey(operations, try operations.keyContactEncrypt(handle!, contentKey))
    }

}

public extension WrappedContactKey {
    public func publicBytes() throws -> Data {
        return try operations.keyContactWrappedPublic(handle!)
    }

    public func ciphertext() throws -> Data {
        return try operations.keyContactWrappedCiphertext(handle!)
    }

    public func encrypted() throws -> Data {
        return try operations.keyContactWrappedEncrypted(handle!)
    }

    public func free() throws -> Void {
        try operations.keyContactWrappedFree(handle!)
        handle = nil
    }

}

public extension SigningKeyPair {
    public func publicBytes() throws -> Data {
        return try operations.keySigningPublic(handle!)
    }

    public func privateBytes() throws -> Data {
        return try operations.keySigningPrivate(handle!)
    }

    public func free() throws -> Void {
        try operations.keySigningFree(handle!)
        handle = nil
    }

}

public extension SigningPublicKey {
    public func publicFree() throws -> Void {
        try operations.keySigningPublicFree(handle!)
        handle = nil
    }

}

public extension VaultDirectory {
    public func root() throws -> String {
        return try operations.vaultDirectoryRoot(handle!)
    }

    public func structureVersion() throws -> UInt32 {
        return try operations.vaultDirectoryStructureVersion(handle!)
    }

    public func listPrivateKeys() throws -> Revault_Bindings_StringList {
        return try operations.vaultDirectoryListPrivateKeys(handle!)
    }

    public func listPrivateKeyNames() throws -> Revault_Bindings_StringList {
        return try operations.vaultDirectoryListPrivateKeyNames(handle!)
    }

    public func listContactNames() throws -> Revault_Bindings_StringList {
        return try operations.vaultDirectoryListContactNames(handle!)
    }

    public func listFormAliases() throws -> Revault_Bindings_StringList {
        return try operations.vaultDirectoryListFormAliases(handle!)
    }

    public func privateKeyExists(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryPrivateKeyExists(handle!, name)
    }

    public func deletePrivateKey(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryDeletePrivateKey(handle!, name)
    }

    public func storePrivateKey(_ name: String, _ key: OwnedHandle) throws -> Bool {
        return try operations.vaultDirectoryStorePrivateKey(handle!, name, key.handle!)
    }

    public func loadPrivateKey(_ name: String) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultDirectoryLoadPrivateKey(handle!, name))
    }

    public func loadPrivateKeyGeneration(_ name: String, _ index: UInt16) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultDirectoryLoadPrivateKeyGeneration(handle!, name, index))
    }

    public func storeContact(_ name: String, _ key: OwnedHandle) throws -> Bool {
        return try operations.vaultDirectoryStoreContact(handle!, name, key.handle!)
    }

    public func loadContact(_ name: String) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.vaultDirectoryLoadContact(handle!, name))
    }

    public func contactExists(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryContactExists(handle!, name)
    }

    public func deleteContact(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryDeleteContact(handle!, name)
    }

    public func listContacts() throws -> Revault_Bindings_ContactList {
        return try operations.vaultDirectoryListContacts(handle!)
    }

    public func storeProfileEmail(_ name: String, _ email: String) throws -> Bool {
        return try operations.vaultDirectoryStoreProfileEmail(handle!, name, email)
    }

    public func profileEmail(_ name: String) throws -> Revault_Bindings_OptionalString {
        return try operations.vaultDirectoryProfileEmail(handle!, name)
    }

    public func storeBackup(_ id: Data, _ bytes: Data) throws -> Bool {
        return try operations.vaultDirectoryStoreBackup(handle!, id, bytes)
    }

    public func loadBackup(_ id: Data) throws -> Data {
        return try operations.vaultDirectoryLoadBackup(handle!, id)
    }

    public func backupCount() throws -> UInt64 {
        return try operations.vaultDirectoryBackupCount(handle!)
    }

    public func restorePrivateKey(_ name: String, _ key: OwnedHandle, _ signingKey: OwnedHandle, _ overwrite: Bool) throws -> Bool {
        return try operations.vaultDirectoryRestorePrivateKey(handle!, name, key.handle!, signingKey.handle!, overwrite)
    }

    public func loadOwnerSigningKey(_ name: String) throws -> SigningKeyPair {
        return SigningKeyPair(operations, try operations.vaultDirectoryLoadOwnerSigningKey(handle!, name))
    }

    public func loadOwnerSigningKeyGeneration(_ name: String, _ index: UInt16) throws -> SigningKeyPair {
        return SigningKeyPair(operations, try operations.vaultDirectoryLoadOwnerSigningKeyGeneration(handle!, name, index))
    }

    public func storeContactSigningKey(_ name: String, _ key: OwnedHandle) throws -> Bool {
        return try operations.vaultDirectoryStoreContactSigningKey(handle!, name, key.handle!)
    }

    public func loadContactSigningKey(_ name: String) throws -> SigningPublicKey {
        return SigningPublicKey(operations, try operations.vaultDirectoryLoadContactSigningKey(handle!, name))
    }

    public func listProfileGenerations(_ name: String) throws -> Revault_Bindings_ProfileHistory {
        return try operations.vaultDirectoryListProfileGenerations(handle!, name)
    }

    public func rotatePrivateKey(_ name: String) throws -> Revault_Bindings_ProfileHistory {
        return try operations.vaultDirectoryRotatePrivateKey(handle!, name)
    }

    public func rememberLockbox(_ id: Data, _ path: String) throws -> Bool {
        return try operations.vaultDirectoryRememberLockbox(handle!, id, path)
    }

    public func listKnownLockboxes() throws -> Revault_Bindings_KnownLockboxList {
        return try operations.vaultDirectoryListKnownLockboxes(handle!)
    }

    public func forgetLockbox(_ path: String) throws -> Bool {
        return try operations.vaultDirectoryForgetLockbox(handle!, path)
    }

    public func rememberAccessSlotLabel(_ id: Data, _ slotId: UInt64, _ name: String) throws -> Bool {
        return try operations.vaultDirectoryRememberAccessSlotLabel(handle!, id, slotId, name)
    }

    public func listAccessSlotLabels(_ id: Data) throws -> Revault_Bindings_AccessSlotLabelList {
        return try operations.vaultDirectoryListAccessSlotLabels(handle!, id)
    }

    public func findAccessSlotLabels(_ id: Data, _ name: String) throws -> Revault_Bindings_AccessSlotLabelList {
        return try operations.vaultDirectoryFindAccessSlotLabels(handle!, id, name)
    }

    public func forgetAccessSlotLabel(_ id: Data, _ slotId: UInt64) throws -> Bool {
        return try operations.vaultDirectoryForgetAccessSlotLabel(handle!, id, slotId)
    }

    public func defineForm(_ alias: String, _ name: String, _ description: String, _ fieldsProto: Data) throws -> Revault_Bindings_FormDefinition {
        return try operations.vaultDirectoryDefineForm(handle!, alias, name, description, fieldsProto)
    }

    public func resolveForm(_ reference: String) throws -> Revault_Bindings_FormDefinition {
        return try operations.vaultDirectoryResolveForm(handle!, reference)
    }

    public func listForms() throws -> Revault_Bindings_FormDefinitionList {
        return try operations.vaultDirectoryListForms(handle!)
    }

    public func listFormRevisions(_ typeId: String) throws -> Revault_Bindings_FormDefinitionList {
        return try operations.vaultDirectoryListFormRevisions(handle!, typeId)
    }

    public func seedForms() throws -> Int {
        return try operations.vaultDirectorySeedForms(handle!)
    }

    public func rememberPassword(_ id: Data, _ password: Data) throws -> Bool {
        return try operations.vaultDirectoryRememberPassword(handle!, id, password)
    }

    public func rememberedPassword(_ id: Data) throws -> Data {
        return try operations.vaultDirectoryRememberedPassword(handle!, id)
    }

    public func free() throws -> Void {
        try operations.vaultDirectoryFree(handle!)
        handle = nil
    }

}

public extension ReadOnlyVaultDirectory {
    public func listProfileNames() throws -> Revault_Bindings_StringList {
        return try operations.vaultReadOnlyListProfileNames(handle!)
    }

    public func listContactNames() throws -> Revault_Bindings_StringList {
        return try operations.vaultReadOnlyListContactNames(handle!)
    }

    public func listFormAliases() throws -> Revault_Bindings_StringList {
        return try operations.vaultReadOnlyListFormAliases(handle!)
    }

    public func listKnownLockboxes() throws -> Revault_Bindings_KnownLockboxList {
        return try operations.vaultReadOnlyListKnownLockboxes(handle!)
    }

    public func free() throws -> Void {
        try operations.vaultReadOnlyFree(handle!)
        handle = nil
    }

}

public extension Agent {
    public func isRunning() throws -> Bool {
        return try operations.vaultIsRunning()
    }

    public func forgetAll() throws -> Bool {
        return try operations.vaultForgetAll()
    }

    public func serve() throws -> Bool {
        return try operations.vaultAgentServe()
    }

    public func verifyTransport() throws -> Bool {
        return try operations.vaultAgentVerifyTransport()
    }

    public func get(_ id: Data) throws -> Data {
        return try operations.vaultAgentGet(id)
    }

    public func put(_ id: Data, _ key: Data) throws -> Bool {
        return try operations.vaultAgentPut(id, key)
    }

    public func forget(_ id: Data) throws -> Bool {
        return try operations.vaultAgentForget(id)
    }

    public func stop() throws -> Bool {
        return try operations.vaultAgentStop()
    }

    public func start() throws -> Bool {
        return try operations.vaultAgentStart()
    }

    public func list() throws -> Revault_Bindings_AgentEntryList {
        return try operations.vaultAgentList()
    }

    public func sleepSupport() throws -> Revault_Bindings_SleepSupport {
        return try operations.vaultAgentSleepSupport()
    }

    public func getVaultUnlockKey(_ vaultId: String) throws -> Data {
        return try operations.vaultAgentGetVaultUnlockKey(vaultId)
    }

    public func putVaultUnlockKey(_ vaultId: String, _ key: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds)
    }

    public func forgetVaultUnlockKey(_ vaultId: String) throws -> Bool {
        return try operations.vaultAgentForgetVaultUnlockKey(vaultId)
    }

    public func getOwnerSigningKey(_ vaultId: String, _ profile: String) throws -> SigningKeyPair {
        return SigningKeyPair(operations, try operations.vaultAgentGetOwnerSigningKey(vaultId, profile))
    }

    public func putOwnerSigningKey(_ vaultId: String, _ profile: String, _ key: OwnedHandle, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key.handle!, ttlSeconds)
    }

    public func forgetOwnerSigningKey(_ vaultId: String, _ profile: String) throws -> Bool {
        return try operations.vaultAgentForgetOwnerSigningKey(vaultId, profile)
    }

    public func beginActivity(_ kind: String) throws -> AgentActivity {
        return AgentActivity(operations, try operations.vaultAgentBeginActivity(kind))
    }

    public func endActivity(_ handle: OwnedHandle) throws -> Void {
        try operations.vaultAgentEndActivity(handle.handle!)
    }

}

public extension AgentActivity {
}

public extension Platform {
    public func status() throws -> Revault_Bindings_PlatformStatus {
        return try operations.vaultPlatformStatus()
    }

    public func setScope(_ scope: String) throws -> Bool {
        return try operations.vaultPlatformSetScope(scope)
    }

    public func forgetPassword() throws -> Bool {
        return try operations.vaultPlatformForgetPassword()
    }

    public func putPassword(_ password: Data) throws -> Bool {
        return try operations.vaultPlatformPutPassword(password)
    }

    public func enable() throws -> Bool {
        return try operations.vaultPlatformEnable()
    }

    public func disable() throws -> Bool {
        return try operations.vaultPlatformDisable()
    }

    public func disabled() throws -> Bool {
        return try operations.vaultPlatformDisabled()
    }

    public func getPassword() throws -> Data {
        return try operations.vaultPlatformGetPassword()
    }

}

public extension LocalVault {
    public func createLockboxPassword(_ path: String, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxPassword(handle!, path, password))
    }

    public func openLockboxPassword(_ path: String, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultOpenLockboxPassword(handle!, path, password))
    }

    public func createLockboxContentKey(_ path: String, _ contentKey: Data, _ signingKey: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxContentKey(handle!, path, contentKey, signingKey.handle!))
    }

    public func createLockboxContact(_ path: String, _ contact: OwnedHandle, _ name: String, _ signingKey: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxContact(handle!, path, contact.handle!, name, signingKey.handle!))
    }

    public func openLockboxContentKey(_ path: String, _ contentKey: Data, _ signingKey: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultOpenLockboxContentKey(handle!, path, contentKey, signingKey.handle!))
    }

    public func cacheLockboxPassword(_ path: String, _ password: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultCacheLockboxPassword(handle!, path, password, ttlSeconds)
    }

    public func closeLockbox(_ path: String) throws -> Bool {
        return try operations.vaultCloseLockbox(handle!, path)
    }

    public func closeAll() throws -> Bool {
        return try operations.vaultCloseAll(handle!)
    }

    public func free() throws -> Void {
        try operations.vaultFree(handle!)
        handle = nil
    }

}
