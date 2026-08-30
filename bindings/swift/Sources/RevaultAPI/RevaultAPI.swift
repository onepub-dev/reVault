// Swift API for encrypted reVault lockboxes and local vault metadata.
// See https://github.com/onepub-dev/reVault#readme for installation, security
// guidance, and complete examples.
import Foundation
import RevaultC
import FlatBuffers

/// Errors raised while validating or invoking the native reVault API.
public enum RevaultError: Error {
    /// The native operation failed with the associated diagnostic message.
    case native(String)
    /// A structured native response was not a valid reVault wire frame.
    case invalidFrame
}

/// Closed cache policy values accepted by lockbox creation/opening.
public enum CacheMode: String, Sendable { case bytes, pages }
/// Closed I/O workload policy values accepted by the native runtime.
public enum WorkloadProfile: String, Sendable { case interactive, bulkImport = "bulk-import" }
/// Closed worker scheduling policy values accepted by the native runtime.
public enum WorkerPolicy: String, Sendable { case auto, single }
/// Kinds of temporary activity retained by the optional agent.
public enum AgentActivityKind: String, Sendable { case lockbox, form, key }

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
    init() { precondition(api_abi_version() == 3, "revault-api native ABI mismatch; expected 3") }
    func lastErrorMessage() -> String { lastError() }

    func bufferLastErrorDetails() throws -> ErrorDetails {
        return DomainCodec.errorDetails(try take(buffer_last_error_details()))
    }

    func lockboxFormatVersion() throws -> UInt16 {
        return lockbox_format_version()
    }

    func lockboxProbeFormatVersion(_ bytes: Data) throws -> UInt16 {
        return bytes.withUnsafeBytes { bytesBytes in
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

    func lockboxCreatePasswordWithSigningKey(_ password: Data, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        return try password.withUnsafeBytes { passwordBytes in
            guard let value = lockbox_create_password_with_signing_key(passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count, signingKey) else { throw RevaultError.native(lastError()) }
            return value
        }
    }

    func lockboxCreateContact(_ contact: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        guard let value = lockbox_create_contact(contact) else { throw RevaultError.native(lastError()) }
        return value
    }

    func lockboxCreateContactWithSigningKey(_ contact: UnsafeMutableRawPointer, _ signingKey: UnsafeMutableRawPointer) throws -> UnsafeMutableRawPointer {
        guard let value = lockbox_create_contact_with_signing_key(contact, signingKey) else { throw RevaultError.native(lastError()) }
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

    func lockboxStreamContent(_ handle: UnsafeMutableRawPointer, _ physical: Bool) throws -> [StreamChunk] {
        return DomainCodec.streamChunkList(try take(lockbox_stream_content(handle, physical)))
    }

    func lockboxCacheStats(_ handle: UnsafeMutableRawPointer) throws -> CacheStats {
        return DomainCodec.cacheStats(try take(lockbox_cache_stats(handle)))
    }

    func lockboxImportStats(_ handle: UnsafeMutableRawPointer) throws -> ImportStats {
        return DomainCodec.importStats(try take(lockbox_import_stats(handle)))
    }

    func lockboxResetImportStats(_ handle: UnsafeMutableRawPointer) throws -> Bool {
        guard lockbox_reset_import_stats(handle) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxInspectFile(_ path: String) throws -> FileInspection {
        return try path.withCString { pathPointer in
            return DomainCodec.fileInspection(try take(lockbox_inspect_file(pathPointer, path.utf8.count)))
        }
    }

    func lockboxPageInspection(_ handle: UnsafeMutableRawPointer) throws -> [PageInspection] {
        return DomainCodec.pageInspectionList(try take(lockbox_page_inspection(handle)))
    }

    func lockboxRecoveryReport(_ handle: UnsafeMutableRawPointer) throws -> RecoveryReport {
        return DomainCodec.recoveryReport(try take(lockbox_recovery_report(handle)))
    }

    func lockboxRecoveryReportRender(_ handle: UnsafeMutableRawPointer, _ verbose: Bool, _ maxEntries: Int) throws -> String {
        return String(decoding: try take(lockbox_recovery_report_render(handle, verbose, maxEntries)), as: UTF8.self)
    }

    func lockboxRecoveryScanPath(_ path: String, _ key: Data) throws -> RecoveryReport {
        return try path.withCString { pathPointer in
            return try key.withUnsafeBytes { keyBytes in
                return DomainCodec.recoveryReport(try take(lockbox_recovery_scan_path(pathPointer, path.utf8.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count)))
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

    func lockboxRuntimeOptions(_ handle: UnsafeMutableRawPointer) throws -> RuntimeOptions {
        return DomainCodec.runtimeOptions(try take(lockbox_runtime_options(handle)))
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

    func lockboxList(_ handle: UnsafeMutableRawPointer, _ path: String, _ recursive: Bool) throws -> [LockboxEntry] {
        return try path.withCString { pathPointer in
            return DomainCodec.lockboxEntryList(try take(lockbox_list(handle, pathPointer, path.utf8.count, recursive)))
        }
    }

    func lockboxListWithOptions(_ handle: UnsafeMutableRawPointer, _ path: String, _ glob: String, _ recursive: Bool, _ includeFiles: Bool, _ includeSymlinks: Bool, _ includeDirectories: Bool, _ limit: Int) throws -> [LockboxEntry] {
        return try path.withCString { pathPointer in
            return try glob.withCString { globPointer in
                return DomainCodec.lockboxEntryList(try take(lockbox_list_with_options(handle, pathPointer, path.utf8.count, globPointer, glob.utf8.count, recursive, includeFiles, includeSymlinks, includeDirectories, limit)))
            }
        }
    }

    func lockboxStat(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> LockboxEntry? {
        return try path.withCString { pathPointer in
            return DomainCodec.optionalLockboxEntry(try take(lockbox_stat(handle, pathPointer, path.utf8.count)))
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
        defer { _ = secret.withUnsafeMutableBytes { $0.initializeMemory(as: UInt8.self, repeating: 0) } }
        return try name.withCString { namePointer in
            try secret.withUnsafeBytes { bytes in
                guard lockbox_set_secret_variable(handle, namePointer, name.utf8.count, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func lockboxGetVariable(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> String? {
        return try name.withCString { namePointer in
            return DomainCodec.optionalString(try take(lockbox_get_variable(handle, namePointer, name.utf8.count)))
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

    func lockboxMoveVariables(_ handle: UnsafeMutableRawPointer, _ movesFlatbuffer: Data) throws -> Bool {
        return try movesFlatbuffer.withUnsafeBytes { movesFlatbufferBytes in
            guard lockbox_move_variables(handle, movesFlatbufferBytes.bindMemory(to: UInt8.self).baseAddress, movesFlatbuffer.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxListVariables(_ handle: UnsafeMutableRawPointer) throws -> [Variable] {
        return DomainCodec.variableList(try take(lockbox_list_variables(handle)))
    }

    func lockboxVariableSensitivity(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> String? {
        return try name.withCString { namePointer in
            return DomainCodec.optionalString(try take(lockbox_variable_sensitivity(handle, namePointer, name.utf8.count)))
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
        return path.withCString { pathPointer in
            return lockbox_exists(handle, pathPointer, path.utf8.count)
        }
    }

    func lockboxIsDir(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return path.withCString { pathPointer in
            return lockbox_is_dir(handle, pathPointer, path.utf8.count)
        }
    }

    func lockboxPermissions(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> UInt32 {
        return path.withCString { pathPointer in
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

    func lockboxRecoveryScan(_ bytes: Data, _ key: Data) throws -> RecoveryReport {
        return try bytes.withUnsafeBytes { bytesBytes in
            return try key.withUnsafeBytes { keyBytes in
                return DomainCodec.recoveryReport(try take(lockbox_recovery_scan(bytesBytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, keyBytes.bindMemory(to: UInt8.self).baseAddress, key.count)))
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
        return password.withUnsafeBytes { passwordBytes in
            return lockbox_add_password(handle, passwordBytes.bindMemory(to: UInt8.self).baseAddress, password.count)
        }
    }

    func lockboxAddContact(_ handle: UnsafeMutableRawPointer, _ contact: UnsafeMutableRawPointer, _ name: String) throws -> UInt64 {
        return name.withCString { namePointer in
            return lockbox_add_contact(handle, contact, namePointer, name.utf8.count)
        }
    }

    func lockboxDeleteKey(_ handle: UnsafeMutableRawPointer, _ id: UInt64) throws -> Bool {
        guard lockbox_delete_key(handle, id) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxListKeySlots(_ handle: UnsafeMutableRawPointer) throws -> [KeySlot] {
        return DomainCodec.keySlotList(try take(lockbox_list_key_slots(handle)))
    }

    func lockboxSetOwnerSigningKey(_ handle: UnsafeMutableRawPointer, _ key: UnsafeMutableRawPointer) throws -> Bool {
        guard lockbox_set_owner_signing_key(handle, key) else { throw RevaultError.native(lastError()) }
        return true
    }

    func lockboxOwnerInspection(_ handle: UnsafeMutableRawPointer) throws -> OwnerInspection {
        return DomainCodec.ownerInspection(try take(lockbox_owner_inspection(handle)))
    }

    func lockboxDefineForm(_ handle: UnsafeMutableRawPointer, _ alias: String, _ name: String, _ description: String, _ fieldsFlatbuffer: Data) throws -> FormDefinition {
        return try alias.withCString { aliasPointer in
            return try name.withCString { namePointer in
                return try description.withCString { descriptionPointer in
                    return try fieldsFlatbuffer.withUnsafeBytes { fieldsFlatbufferBytes in
                        return DomainCodec.formDefinition(try take(lockbox_define_form(handle, aliasPointer, alias.utf8.count, namePointer, name.utf8.count, descriptionPointer, description.utf8.count, fieldsFlatbufferBytes.bindMemory(to: UInt8.self).baseAddress, fieldsFlatbuffer.count)))
                    }
                }
            }
        }
    }

    func lockboxListFormDefinitions(_ handle: UnsafeMutableRawPointer) throws -> [FormDefinition] {
        return DomainCodec.formDefinitionList(try take(lockbox_list_form_definitions(handle)))
    }

    func lockboxResolveForm(_ handle: UnsafeMutableRawPointer, _ reference: String) throws -> FormDefinition {
        return try reference.withCString { referencePointer in
            return DomainCodec.formDefinition(try take(lockbox_resolve_form(handle, referencePointer, reference.utf8.count)))
        }
    }

    func lockboxListFormRevisions(_ handle: UnsafeMutableRawPointer, _ typeId: String) throws -> [FormDefinition] {
        return try typeId.withCString { typeIdPointer in
            return DomainCodec.formDefinitionList(try take(lockbox_list_form_revisions(handle, typeIdPointer, typeId.utf8.count)))
        }
    }

    func lockboxCreateFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String, _ typeReference: String, _ name: String) throws -> FormRecord {
        return try path.withCString { pathPointer in
            return try typeReference.withCString { typeReferencePointer in
                return try name.withCString { namePointer in
                    return DomainCodec.formRecord(try take(lockbox_create_form_record(handle, pathPointer, path.utf8.count, typeReferencePointer, typeReference.utf8.count, namePointer, name.utf8.count)))
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
        defer { _ = secret.withUnsafeMutableBytes { $0.initializeMemory(as: UInt8.self, repeating: 0) } }
        return try path.withCString { pathPointer in
            try field.withCString { fieldPointer in
                try secret.withUnsafeBytes { bytes in
                    guard lockbox_set_secret_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count) else { throw RevaultError.native(lastError()) }
                    return true
                }
            }
        }
    }

    func lockboxListFormRecords(_ handle: UnsafeMutableRawPointer) throws -> [FormRecord] {
        return DomainCodec.formRecordList(try take(lockbox_list_form_records(handle)))
    }

    func lockboxGetFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> FormRecord? {
        return try path.withCString { pathPointer in
            return DomainCodec.optionalFormRecord(try take(lockbox_get_form_record(handle, pathPointer, path.utf8.count)))
        }
    }

    func lockboxDeleteFormRecord(_ handle: UnsafeMutableRawPointer, _ path: String) throws -> Bool {
        return try path.withCString { pathPointer in
            guard lockbox_delete_form_record(handle, pathPointer, path.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxMoveFormRecords(_ handle: UnsafeMutableRawPointer, _ movesFlatbuffer: Data) throws -> Bool {
        return try movesFlatbuffer.withUnsafeBytes { movesFlatbufferBytes in
            guard lockbox_move_form_records(handle, movesFlatbufferBytes.bindMemory(to: UInt8.self).baseAddress, movesFlatbuffer.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func lockboxGetFormField(_ handle: UnsafeMutableRawPointer, _ path: String, _ field: String) throws -> FormValue? {
        return try path.withCString { pathPointer in
            return try field.withCString { fieldPointer in
                return DomainCodec.optionalFormValue(try take(lockbox_get_form_field(handle, pathPointer, path.utf8.count, fieldPointer, field.utf8.count)))
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
        return root.withCString { rootPointer in
            return password.withUnsafeBytes { passwordBytes in
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

    func vaultDirectoryListPrivateKeys(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_directory_list_private_keys(handle)))
    }

    func vaultDirectoryListPrivateKeyNames(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_directory_list_private_key_names(handle)))
    }

    func vaultDirectoryListContactNames(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_directory_list_contact_names(handle)))
    }

    func vaultDirectoryListFormAliases(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_directory_list_form_aliases(handle)))
    }

    func vaultDirectoryPrivateKeyExists(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return name.withCString { namePointer in
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
        return name.withCString { namePointer in
            return vault_directory_contact_exists(handle, namePointer, name.utf8.count)
        }
    }

    func vaultDirectoryDeleteContact(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> Bool {
        return try name.withCString { namePointer in
            guard vault_directory_delete_contact(handle, namePointer, name.utf8.count) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryListContacts(_ handle: UnsafeMutableRawPointer) throws -> [Contact] {
        return DomainCodec.contactList(try take(vault_directory_list_contacts(handle)))
    }

    func vaultDirectoryStoreProfileEmail(_ handle: UnsafeMutableRawPointer, _ name: String, _ email: String) throws -> Bool {
        return try name.withCString { namePointer in
            return try email.withCString { emailPointer in
                guard vault_directory_store_profile_email(handle, namePointer, name.utf8.count, emailPointer, email.utf8.count) else { throw RevaultError.native(lastError()) }
                return true
            }
        }
    }

    func vaultDirectoryProfileEmail(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> String? {
        return try name.withCString { namePointer in
            return DomainCodec.optionalString(try take(vault_directory_profile_email(handle, namePointer, name.utf8.count)))
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

    func vaultDirectoryListProfileGenerations(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> ProfileHistory {
        return try name.withCString { namePointer in
            return DomainCodec.profileHistory(try take(vault_directory_list_profile_generations(handle, namePointer, name.utf8.count)))
        }
    }

    func vaultDirectoryRotatePrivateKey(_ handle: UnsafeMutableRawPointer, _ name: String) throws -> ProfileHistory {
        return try name.withCString { namePointer in
            return DomainCodec.profileHistory(try take(vault_directory_rotate_private_key(handle, namePointer, name.utf8.count)))
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

    func vaultDirectoryListKnownLockboxes(_ handle: UnsafeMutableRawPointer) throws -> [KnownLockbox] {
        return DomainCodec.knownLockboxList(try take(vault_directory_list_known_lockboxes(handle)))
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

    func vaultDirectoryListAccessSlotLabels(_ handle: UnsafeMutableRawPointer, _ id: Data) throws -> [AccessSlotLabel] {
        return try id.withUnsafeBytes { idBytes in
            return DomainCodec.accessSlotLabelList(try take(vault_directory_list_access_slot_labels(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count)))
        }
    }

    func vaultDirectoryFindAccessSlotLabels(_ handle: UnsafeMutableRawPointer, _ id: Data, _ name: String) throws -> [AccessSlotLabel] {
        return try id.withUnsafeBytes { idBytes in
            return try name.withCString { namePointer in
                return DomainCodec.accessSlotLabelList(try take(vault_directory_find_access_slot_labels(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, namePointer, name.utf8.count)))
            }
        }
    }

    func vaultDirectoryForgetAccessSlotLabel(_ handle: UnsafeMutableRawPointer, _ id: Data, _ slotId: UInt64) throws -> Bool {
        return try id.withUnsafeBytes { idBytes in
            guard vault_directory_forget_access_slot_label(handle, idBytes.bindMemory(to: UInt8.self).baseAddress, id.count, slotId) else { throw RevaultError.native(lastError()) }
            return true
        }
    }

    func vaultDirectoryDefineForm(_ handle: UnsafeMutableRawPointer, _ alias: String, _ name: String, _ description: String, _ fieldsFlatbuffer: Data) throws -> FormDefinition {
        return try alias.withCString { aliasPointer in
            return try name.withCString { namePointer in
                return try description.withCString { descriptionPointer in
                    return try fieldsFlatbuffer.withUnsafeBytes { fieldsFlatbufferBytes in
                        return DomainCodec.formDefinition(try take(vault_directory_define_form(handle, aliasPointer, alias.utf8.count, namePointer, name.utf8.count, descriptionPointer, description.utf8.count, fieldsFlatbufferBytes.bindMemory(to: UInt8.self).baseAddress, fieldsFlatbuffer.count)))
                    }
                }
            }
        }
    }

    func vaultDirectoryResolveForm(_ handle: UnsafeMutableRawPointer, _ reference: String) throws -> FormDefinition {
        return try reference.withCString { referencePointer in
            return DomainCodec.formDefinition(try take(vault_directory_resolve_form(handle, referencePointer, reference.utf8.count)))
        }
    }

    func vaultDirectoryListForms(_ handle: UnsafeMutableRawPointer) throws -> [FormDefinition] {
        return DomainCodec.formDefinitionList(try take(vault_directory_list_forms(handle)))
    }

    func vaultDirectoryListFormRevisions(_ handle: UnsafeMutableRawPointer, _ typeId: String) throws -> [FormDefinition] {
        return try typeId.withCString { typeIdPointer in
            return DomainCodec.formDefinitionList(try take(vault_directory_list_form_revisions(handle, typeIdPointer, typeId.utf8.count)))
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

    func vaultBackupDefault(_ path: String, _ overwrite: Bool) throws -> VaultBackupManifest {
        return try path.withCString { pathPointer in
            return DomainCodec.vaultBackupManifest(try take(vault_backup_default(pathPointer, path.utf8.count, overwrite)))
        }
    }

    func vaultRestoreDefault(_ path: String, _ overwrite: Bool) throws -> VaultBackupManifest {
        return try path.withCString { pathPointer in
            return DomainCodec.vaultBackupManifest(try take(vault_restore_default(pathPointer, path.utf8.count, overwrite)))
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

    func vaultReadOnlyListProfileNames(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_read_only_list_profile_names(handle)))
    }

    func vaultReadOnlyListContactNames(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_read_only_list_contact_names(handle)))
    }

    func vaultReadOnlyListFormAliases(_ handle: UnsafeMutableRawPointer) throws -> [String] {
        return DomainCodec.stringList(try take(vault_read_only_list_form_aliases(handle)))
    }

    func vaultReadOnlyListKnownLockboxes(_ handle: UnsafeMutableRawPointer) throws -> [KnownLockbox] {
        return DomainCodec.knownLockboxList(try take(vault_read_only_list_known_lockboxes(handle)))
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

    func vaultAgentList() throws -> [AgentEntry] {
        return DomainCodec.agentEntryList(try take(vault_agent_list()))
    }

    func vaultAgentSleepSupport() throws -> SleepSupport {
        return DomainCodec.sleepSupport(try take(vault_agent_sleep_support()))
    }

    func vaultPlatformStatus() throws -> PlatformStatus {
        return DomainCodec.platformStatus(try take(vault_platform_status()))
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

/// Base type for API values that retain sensitive state until released.
///
/// Applications receive concrete subclasses such as ``Lockbox`` and should
/// release them promptly after use. Do not share these values across concurrent
/// operations unless the specific API documents it.
public class OwnedHandle {
    fileprivate let operations: BindingOperations
    fileprivate var handle: UnsafeMutableRawPointer?
    fileprivate init(_ operations: BindingOperations, _ handle: UnsafeMutableRawPointer?) { self.operations = operations; self.handle = handle }
}

/// An open encrypted archive containing files, variables, secrets, and forms.
/// Commit pending mutations and release it when finished with decrypted data.
public final class Lockbox: OwnedHandle {}

/// A profile's contact-encryption identity used to decrypt keys addressed to it.
public final class ContactKeyPair: OwnedHandle {}

/// A recipient's shareable encryption identity used when granting lockbox access.
public final class ContactPublicKey: OwnedHandle {}

/// A content key encrypted for one contact and recoverable by its matching key pair.
public final class WrappedContactKey: OwnedHandle {}

/// A profile signing identity used to authorize mutable lockbox revisions.
public final class ProfileSigningKeyPair: OwnedHandle {}

/// The public profile identity readers use to verify authorized revisions.
public final class ProfileSigningPublicKey: OwnedHandle {}

/// Password-protected storage for Profile keys, contacts, forms, backups, and lockbox paths.
public final class Vault: OwnedHandle {}

/// A metadata view for discovery that never loads private profile signing material.
public final class ReadOnlyVault: OwnedHandle {}

/// Client for the session service that temporarily caches unlock and signing keys.
public final class AgentSession: OwnedHandle {}
/// Explicit controller for the single optional Session Agent process.


/// A token kept alive while an operation needs secrets cached by the agent.
public final class AgentActivity: OwnedHandle {}

/// Access to the platform credential store for a scoped Vault passphrase.
public final class Platform: OwnedHandle {}

/// Runtime API used to create/open lockboxes, reach Vault metadata, use the
/// explicit Session Agent, and access the platform credential store.
public final class Revault {
    fileprivate let operations = BindingOperations()
    /// Returns the agent.
    public lazy var agentSession = AgentSession(operations, try? operations.vaultLocal())
    /// Returns the platform.
    public lazy var platform = Platform(operations, nil)
    /// Returns the init.
    public init() {}
    /// Returns the last error.
    public func lastError() -> String { operations.lastErrorMessage() }
    /// Returns the last error details.
    public func lastErrorDetails() throws -> ErrorDetails { try operations.bufferLastErrorDetails() }

    /// Returns the newest Lockbox archive format version supported by this engine.
    public func lockboxFormatVersion() throws -> UInt16 {
        return try operations.lockboxFormatVersion()
    }

    /// Reads the format version from serialized Lockbox bytes without opening them.
    public func lockboxProbeFormatVersion(_ bytes: Data) throws -> UInt16 {
        return try operations.lockboxProbeFormatVersion(bytes)
    }

    /// Creates an in memory Lockbox protected by a 32 byte content key.
    public func lockboxCreate(_ key: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreate(key))
    }

    /// Creates a lockbox with explicit cache capacity, workload, worker policy, and job count.
    public func lockboxCreateWithOptions(_ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs))
    }

    /// Creates a lockbox using closed policy enums; the content key remains caller-owned.
    public func lockboxCreateWithOptions(_ key: Data, cacheMode: CacheMode, cacheBytes: UInt64,
        workload: WorkloadProfile, worker: WorkerPolicy, jobs: Int = 0) throws -> Lockbox {
        try lockboxCreateWithOptions(key, cacheMode.rawValue, cacheBytes, workload.rawValue, worker.rawValue, jobs)
    }

    /// Creates an in memory Lockbox protected by the supplied password.
    public func lockboxCreatePassword(_ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreatePassword(password))
    }

    /// Creates a password-protected Lockbox with its owner signing key.
    public func lockboxCreatePasswordWithProfileSigningKey(_ password: Data, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreatePasswordWithSigningKey(password, signingKey.handle!))
    }

    /// Creates an in memory Lockbox that the supplied contact can open.
    public func lockboxCreateContact(_ contact: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateContact(contact.handle!))
    }

    /// Creates a contact-protected Lockbox with its owner signing key.
    public func lockboxCreateContactWithProfileSigningKey(_ contact: OwnedHandle, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateContactWithSigningKey(contact.handle!, signingKey.handle!))
    }

    /// Creates an in memory Lockbox and assigns its profile signing key.
    public func lockboxCreateWithProfileSigningKey(_ contentKey: Data, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxCreateWithSigningKey(contentKey, signingKey.handle!))
    }

    /// Opens serialized Lockbox bytes with a 32 byte content key.
    public func lockboxOpen(_ archive: Data, _ key: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpen(archive, key))
    }

    /// Opens a lockbox with explicit cache capacity, workload, worker policy, and job count.
    public func lockboxOpenWithOptions(_ archive: Data, _ key: Data, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs))
    }

    /// Opens an existing archive using closed policy enums and a process-local content key.
    public func lockboxOpenWithOptions(_ archive: Data, _ key: Data, cacheMode: CacheMode, cacheBytes: UInt64,
        workload: WorkloadProfile, worker: WorkerPolicy, jobs: Int = 0) throws -> Lockbox {
        try lockboxOpenWithOptions(archive, key, cacheMode.rawValue, cacheBytes, workload.rawValue, worker.rawValue, jobs)
    }

    /// Opens serialized Lockbox bytes with the supplied password.
    public func lockboxOpenPassword(_ archive: Data, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenPassword(archive, password))
    }

    /// Opens serialized Lockbox bytes with the supplied contact private key.
    public func lockboxOpenContact(_ archive: Data, _ contact: OwnedHandle) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxOpenContact(archive, contact.handle!))
    }

    /// Reads public header, signature, and access slot metadata from a Lockbox file.
    public func lockboxInspectFile(_ path: String) throws -> FileInspection {
        return try operations.lockboxInspectFile(path)
    }

    /// Scans a damaged Lockbox file with its 32 byte content key.
    public func lockboxRecoveryScanPath(_ path: String, _ key: Data) throws -> RecoveryReport {
        return try operations.lockboxRecoveryScanPath(path, key)
    }

    /// Scans damaged serialized Lockbox bytes with their 32 byte content key.
    public func lockboxRecoveryScan(_ bytes: Data, _ key: Data) throws -> RecoveryReport {
        return try operations.lockboxRecoveryScan(bytes, key)
    }

    /// Builds a new Lockbox from recoverable records without changing the source.
    public func lockboxRecoverySalvage(_ bytes: Data, _ key: Data, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.lockboxRecoverySalvage(bytes, key, signingKey.handle!))
    }

    /// Generates a contact encryption key pair using secure random data.
    public func keyContactGenerate() throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.keyContactGenerate())
    }

    /// Imports a contact key pair from its private binary record.
    public func keyContactFromPrivate(_ bytes: Data) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.keyContactFromPrivate(bytes))
    }

    /// Imports a contact public key from its binary representation.
    public func keyContactPublicFromBytes(_ bytes: Data) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.keyContactPublicFromBytes(bytes))
    }

    /// Generates a profile signing identity.
    public func generateProfileSigningKeyPair() throws -> ProfileSigningKeyPair {
        return ProfileSigningKeyPair(operations, try operations.keySigningGenerate())
    }

    /// Imports a profile signing identity from its private record.
    public func profileSigningKeyPairFromPrivate(_ bytes: Data) throws -> ProfileSigningKeyPair {
        return ProfileSigningKeyPair(operations, try operations.keySigningFromPrivate(bytes))
    }

    /// Imports a profile signing public key from encoded public bytes.
    public func profileSigningPublicKeyFromBytes(_ bytes: Data) throws -> ProfileSigningPublicKey {
        return ProfileSigningPublicKey(operations, try operations.keySigningPublicFromBytes(bytes))
    }

    /// Exports a private key in the requested key format.
    public func vaultKeyExportPrivate(_ key: OwnedHandle, _ format: String) throws -> Data {
        return try operations.vaultKeyExportPrivate(key.handle!, format)
    }

    /// Exports a public key in the requested key format.
    public func vaultKeyExportPublic(_ key: OwnedHandle, _ format: String) throws -> Data {
        return try operations.vaultKeyExportPublic(key.handle!, format)
    }

    /// Imports a private contact key from a detected supported encoding.
    public func vaultKeyImportPrivate(_ bytes: Data) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultKeyImportPrivate(bytes))
    }

    /// Imports a public contact key from a detected supported encoding.
    public func vaultKeyImportPublic(_ bytes: Data) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.vaultKeyImportPublic(bytes))
    }

    /// Returns the stable fingerprint used to verify a public key.
    public func vaultKeyFingerprint(_ key: OwnedHandle) throws -> Data {
        return try operations.vaultKeyFingerprint(key.handle!)
    }

    /// Encodes key bytes as hexadecimal text.
    public func vaultKeyFormatHex(_ bytes: Data) throws -> String {
        return try operations.vaultKeyFormatHex(bytes)
    }

    /// Decodes hexadecimal key text and rejects malformed input.
    public func vaultKeyDecodeHex(_ text: String) throws -> Data {
        return try operations.vaultKeyDecodeHex(text)
    }

    /// Encodes key bytes using Crockford Base32.
    public func vaultKeyFormatCrockford(_ bytes: Data) throws -> String {
        return try operations.vaultKeyFormatCrockford(bytes)
    }

    /// Groups a Crockford code for easier reading and transcription.
    public func vaultKeyFormatCrockfordReading(_ code: String) throws -> String {
        return try operations.vaultKeyFormatCrockfordReading(code)
    }

    /// Decodes Crockford Base32 key text and rejects malformed input.
    public func vaultKeyDecodeCrockford(_ code: String) throws -> Data {
        return try operations.vaultKeyDecodeCrockford(code)
    }

    /// Encodes arbitrary bytes as hexadecimal text.
    public func vaultKeyHexEncode(_ bytes: Data) throws -> String {
        return try operations.vaultKeyHexEncode(bytes)
    }

    /// Decodes arbitrary hexadecimal text and rejects malformed input.
    public func vaultKeyHexDecode(_ text: String) throws -> Data {
        return try operations.vaultKeyHexDecode(text)
    }

    /// Opens an existing Vault directory with its passphrase.
    public func openVault(_ root: String, _ password: Data) throws -> Vault {
        return Vault(operations, try operations.vaultDirectoryOpen(root, password))
    }

    /// Returns the newest Vault structure version supported by this engine.
    public func vaultStructureVersionCurrent() throws -> UInt32 {
        return try operations.vaultStructureVersionCurrent()
    }

    /// Reads an existing Vault structure version without changing it.
    public func probeVaultStructureVersion(_ root: String, _ password: Data) throws -> UInt32 {
        return try operations.vaultDirectoryProbeStructureVersion(root, password)
    }

    /// Opens or creates the default Vault without replacing existing state.
    public func openOrCreateDefaultVault(_ password: Data) throws -> Vault {
        return Vault(operations, try operations.vaultDirectoryOpenOrCreateDefault(password))
    }

    /// Replaces the default Vault and all persistent data it contains.
    public func replaceDefaultVault(_ password: Data) throws -> Vault {
        return Vault(operations, try operations.vaultDirectoryReplaceDefault(password))
    }

    /// Changes the passphrase for an existing Vault.
    @discardableResult
    public func changeVaultPassword(_ root: String, _ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try operations.vaultDirectoryChangePassword(root, oldPassword, newPassword)
    }

    /// Changes the passphrase for the default Vault.
    @discardableResult
    public func changeDefaultVaultPassword(_ oldPassword: Data, _ newPassword: Data) throws -> Bool {
        return try operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword)
    }

    /// Replaces the selected Vault and all persistent data it contains.
    public func replaceVault(_ root: String, _ password: Data) throws -> Vault {
        return Vault(operations, try operations.vaultDirectoryReplace(root, password))
    }

    /// Opens the selected Vault, creating it only when absent.
    public func openOrCreateVault(_ root: String, _ password: Data) throws -> Vault {
        return Vault(operations, try operations.vaultDirectoryOpenOrCreate(root, password))
    }

    /// Writes a backup of the default Vault to the selected path.
    public func vaultBackupDefault(_ path: String, _ overwrite: Bool) throws -> VaultBackupManifest {
        return try operations.vaultBackupDefault(path, overwrite)
    }

    /// Restores the default Vault from the selected backup.
    public func vaultRestoreDefault(_ path: String, _ overwrite: Bool) throws -> VaultBackupManifest {
        return try operations.vaultRestoreDefault(path, overwrite)
    }

    /// Opens an existing Vault metadata view that cannot load private keys.
    public func openReadOnlyVault(_ root: String, _ password: Data) throws -> ReadOnlyVault {
        return ReadOnlyVault(operations, try operations.vaultReadOnlyOpen(root, password))
    }

    /// Opens the default Vault metadata view without loading private keys.
    public func openDefaultReadOnlyVault(_ password: Data) throws -> ReadOnlyVault {
        return ReadOnlyVault(operations, try operations.vaultReadOnlyOpenDefault(password))
    }

    /// Returns the platform default Vault directory.
    public func defaultVaultRoot() throws -> String {
        return try operations.vaultDefaultDirectory()
    }

    /// Returns the path of the default Vault file.
    public func vaultDefaultPath() throws -> String {
        return try operations.vaultDefaultPath()
    }

    /// Returns the session agent log path.
    public func vaultAgentLogPath() throws -> String {
        return try operations.vaultAgentLogPath()
    }

    /// Returns the configured session agent log destination.
    public func vaultAgentLogDestination() throws -> String {
        return try operations.vaultAgentLogDestination()
    }

    /// Opens an existing persistent Vault; it never creates or replaces it.
    public func openVault(at path: String, vaultPassphrase: Data) throws -> Vault {
        Vault(operations, try operations.vaultDirectoryOpen(path, vaultPassphrase))
    }

    /// Opens or creates a persistent Vault when no archive exists at path.
    public func openOrCreateVault(at path: String, vaultPassphrase: Data) throws -> Vault {
        Vault(operations, try operations.vaultDirectoryOpenOrCreate(path, vaultPassphrase))
    }

    /// Replaces persistent Vault data at path. This operation is destructive.
    public func replaceVault(at path: String, vaultPassphrase: Data) throws -> Vault {
        Vault(operations, try operations.vaultDirectoryReplace(path, vaultPassphrase))
    }

}

/// Persistent encrypted local metadata store. Its directory is a storage
/// detail; callers use the Vault facade rather than a directory domain object.


/// Returns the member.
extension Lockbox {
    /// Stages a file at the Lockbox path; replace controls an existing entry.
    @discardableResult
    public func addFile(_ path: String, _ data: Data, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddFile(handle!, path, data, replace)
    }

    /// Stages a file and its portable Unix permission bits.
    @discardableResult
    public func addFileWithPermissions(_ path: String, _ data: Data, _ permissions: UInt32, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddFileWithPermissions(handle!, path, data, permissions, replace)
    }

    /// Reads the complete file stored at the Lockbox path.
    public func getFile(_ path: String) throws -> Data {
        return try operations.lockboxGetFile(handle!, path)
    }

    /// Writes one Lockbox file to the host filesystem.
    @discardableResult
    public func extractFile(_ source: String, _ destination: String, _ replace: Bool) throws -> Bool {
        return try operations.lockboxExtractFile(handle!, source, destination, replace)
    }

    /// Extracts the Lockbox with explicit size, count, link, and permission limits.
    @discardableResult
    public func extractDirectory(_ destination: String, _ maxFileBytes: UInt64, _ maxTotalBytes: UInt64, _ maxFiles: Int, _ restoreSymlinks: Bool, _ restorePermissions: Bool, _ overwrite: Bool) throws -> Bool {
        return try operations.lockboxExtractDirectory(handle!, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite)
    }

    /// Lists logical or physical content chunks for streaming diagnostics.
    public func streamContent(_ physical: Bool) throws -> [StreamChunk] {
        return try operations.lockboxStreamContent(handle!, physical)
    }

    /// Returns cache statistics for this lockbox.
    public func cacheStats() throws -> CacheStats {
        return try operations.lockboxCacheStats(handle!)
    }

    /// Returns import statistics for this lockbox.
    public func importStats() throws -> ImportStats {
        return try operations.lockboxImportStats(handle!)
    }

    /// Updates import stats.
    @discardableResult
    public func resetImportStats() throws -> Bool {
        return try operations.lockboxResetImportStats(handle!)
    }

    /// Returns page metadata for diagnostics without exposing plaintext secrets.
    public func pageInspection() throws -> [PageInspection] {
        return try operations.lockboxPageInspection(handle!)
    }

    /// Scans the open archive and returns its structured recovery report.
    public func recoveryReport() throws -> RecoveryReport {
        return try operations.lockboxRecoveryReport(handle!)
    }

    /// Renders the recovery report for a person, capped at maxEntries.
    public func recoveryReportRender(_ verbose: Bool, _ maxEntries: Int) throws -> String {
        return try operations.lockboxRecoveryReportRender(handle!, verbose, maxEntries)
    }

    /// Returns the current serialized archive size in bytes.
    public func storageLen() throws -> UInt64 {
        return try operations.lockboxStorageLen(handle!)
    }

    /// Sets workload profile.
    @discardableResult
    public func setWorkloadProfile(_ profile: String) throws -> Bool {
        return try operations.lockboxSetWorkloadProfile(handle!, profile)
    }

    /// Selects a closed workload profile without passing undocumented strings.
    @discardableResult
    public func setWorkloadProfile(_ profile: WorkloadProfile) throws -> Bool {
        try setWorkloadProfile(profile.rawValue)
    }

    /// Sets worker policy.
    @discardableResult
    public func setWorkerPolicy(_ mode: String, _ jobs: Int) throws -> Bool {
        return try operations.lockboxSetWorkerPolicy(handle!, mode, jobs)
    }

    /// Selects a closed worker policy without passing undocumented strings.
    @discardableResult
    public func setWorkerPolicy(_ mode: WorkerPolicy, jobs: Int = 0) throws -> Bool {
        try setWorkerPolicy(mode.rawValue, jobs)
    }

    /// Returns the cache, workload, and worker settings used by this Lockbox.
    public func runtimeOptions() throws -> RuntimeOptions {
        return try operations.lockboxRuntimeOptions(handle!)
    }

    /// Authenticates and publishes the staged changes.
    @discardableResult
    public func commit() throws -> Bool {
        return try operations.lockboxCommit(handle!)
    }

    /// Stages a directory entry and optionally creates missing parents.
    @discardableResult
    public func createDir(_ path: String, _ createParents: Bool) throws -> Bool {
        return try operations.lockboxCreateDir(handle!, path, createParents)
    }

    /// Stages removal of a file, link, or empty directory at path.
    @discardableResult
    public func delete(_ path: String) throws -> Bool {
        return try operations.lockboxDelete(handle!, path)
    }

    /// Stages removal of a directory, optionally including its descendants.
    @discardableResult
    public func removeDir(_ path: String, _ recursive: Bool) throws -> Bool {
        return try operations.lockboxRemoveDir(handle!, path, recursive)
    }

    /// Stages every missing parent directory for path.
    @discardableResult
    public func createParentDirs(_ path: String) throws -> Bool {
        return try operations.lockboxCreateParentDirs(handle!, path)
    }

    /// Stages an atomic move from one Lockbox path to another.
    @discardableResult
    public func rename(_ from: String, _ to: String) throws -> Bool {
        return try operations.lockboxRename(handle!, from, to)
    }

    /// Lists entries below path, optionally including descendants.
    public func list(_ path: String, _ recursive: Bool) throws -> [LockboxEntry] {
        return try operations.lockboxList(handle!, path, recursive)
    }

    /// Lists entries using glob, type, recursion, and result limit filters.
    public func listWithOptions(_ path: String, _ glob: String, _ recursive: Bool, _ includeFiles: Bool, _ includeSymlinks: Bool, _ includeDirectories: Bool, _ limit: Int) throws -> [LockboxEntry] {
        return try operations.lockboxListWithOptions(handle!, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit)
    }

    /// Returns metadata for the selected lockbox entry.
    public func stat(_ path: String) throws -> LockboxEntry? {
        return try operations.lockboxStat(handle!, path)
    }

    /// Stages a plain text variable; commit to publish the change.
    @discardableResult
    public func setVariable(_ name: String, _ value: String) throws -> Bool {
        return try operations.lockboxSetVariable(handle!, name, value)
    }

    /// Sets secret variable.
    @discardableResult
    public func setSecretVariable(_ name: String, _ value: Data) throws -> Bool {
        return try operations.lockboxSetSecretVariable(handle!, name, value)
    }

    /// Returns a plain variable when it is present.
    public func getVariable(_ name: String) throws -> String? {
        return try operations.lockboxGetVariable(handle!, name)
    }

    /// Returns the encrypted Lockbox description, or `nil` when unset.
    /// Example: set it, commit, then `print(try box.description)`.
    public var description: String? {
        get throws { try getVariable("/.revault/description") }
    }

    /// Stages encrypted description text; call `commit()` to publish it.
    /// Example: `try box.setDescription("Production credentials"); try box.commit()`.
    @discardableResult
    public func setDescription(_ description: String) throws -> Bool {
        return try setVariable("/.revault/description", description)
    }

    /// Stages removal of the encrypted description; call `commit()`.
    /// Example: `try box.clearDescription(); try box.commit()`.
    @discardableResult
    public func clearDescription() throws -> Bool {
        return try deleteVariable("/.revault/description")
    }

    /// Invokes a callback with temporary secret variable bytes, then clears them.
    public func withSecretVariable<T>(_ name: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        return try operations.lockboxWithSecretVariable(handle!, name, callback)
    }

    /// Stages removal of a variable.
    @discardableResult
    public func deleteVariable(_ name: String) throws -> Bool {
        return try operations.lockboxDeleteVariable(handle!, name)
    }

    /// Updates variables.
    @discardableResult
    public func moveVariables(_ moves: [PathMove]) throws -> Bool {
        return try operations.lockboxMoveVariables(handle!, DomainCodec.encodePathMoves(moves))
    }

    /// Lists variable names and metadata without exposing secret values.
    public func listVariables() throws -> [Variable] {
        return try operations.lockboxListVariables(handle!)
    }

    /// Returns whether a variable is plain or secret.
    public func variableSensitivity(_ name: String) throws -> String? {
        return try operations.lockboxVariableSensitivity(handle!, name)
    }

    /// Stages a symbolic link with its stored target text.
    @discardableResult
    public func addSymlink(_ path: String, _ target: String, _ replace: Bool) throws -> Bool {
        return try operations.lockboxAddSymlink(handle!, path, target, replace)
    }

    /// Returns the target text stored for a symbolic link.
    public func getSymlinkTarget(_ path: String) throws -> String {
        return try operations.lockboxGetSymlinkTarget(handle!, path)
    }

    /// Returns the stable public identifier stored in the Lockbox header.
    public func id() throws -> Data {
        return try operations.lockboxId(handle!)
    }

    /// Reports whether an entry exists at path.
    @discardableResult
    public func exists(_ path: String) throws -> Bool {
        return try operations.lockboxExists(handle!, path)
    }

    /// Reports whether path names a directory entry.
    @discardableResult
    public func isDir(_ path: String) throws -> Bool {
        return try operations.lockboxIsDir(handle!, path)
    }

    /// Returns the portable Unix permission bits stored for path.
    public func permissions(_ path: String) throws -> UInt32 {
        return try operations.lockboxPermissions(handle!, path)
    }

    /// Stages portable Unix permission bits for path.
    @discardableResult
    public func setPermissions(_ path: String, _ permissions: UInt32) throws -> Bool {
        return try operations.lockboxSetPermissions(handle!, path, permissions)
    }

    /// Reads the requested byte range from a stored file.
    public func readRange(_ path: String, _ offset: UInt64, _ len: UInt64) throws -> Data {
        return try operations.lockboxReadRange(handle!, path, offset, len)
    }

    /// Adds a password access slot and returns its slot identifier.
    public func addPassword(_ password: Data) throws -> UInt64 {
        return try operations.lockboxAddPassword(handle!, password)
    }

    /// Grants a named contact access and returns the new slot identifier.
    public func addContact(_ contact: OwnedHandle, _ name: String) throws -> UInt64 {
        return try operations.lockboxAddContact(handle!, contact.handle!, name)
    }

    /// Removes an access slot; at least one usable slot must remain.
    @discardableResult
    public func deleteKey(_ id: UInt64) throws -> Bool {
        return try operations.lockboxDeleteKey(handle!, id)
    }

    /// Lists public access slot metadata without returning credentials.
    public func listKeySlots() throws -> [KeySlot] {
        return try operations.lockboxListKeySlots(handle!)
    }

    /// Assigns a profile signing key to the Lockbox owner role.
    @discardableResult
    public func setOwnerSigningKey(_ key: ProfileSigningKeyPair) throws -> Bool {
        return try operations.lockboxSetOwnerSigningKey(handle!, key.handle!)
    }

    /// Returns public signing and ownership metadata for the current revision.
    public func ownerInspection() throws -> OwnerInspection {
        return try operations.lockboxOwnerInspection(handle!)
    }

    /// Defines and stores a reusable versioned form.
    public func defineForm(_ alias: String, _ name: String, _ description: String, _ fields: [FormField]) throws -> FormDefinition {
        return try operations.lockboxDefineForm(handle!, alias, name, description, DomainCodec.encodeFormFields(fields))
    }

    /// Lists the form definitions stored in this Lockbox.
    public func listFormDefinitions() throws -> [FormDefinition] {
        return try operations.lockboxListFormDefinitions(handle!)
    }

    /// Resolves a form alias, type identifier, or revision.
    public func resolveForm(_ reference: String) throws -> FormDefinition {
        return try operations.lockboxResolveForm(handle!, reference)
    }

    /// Lists every stored revision for a form type identifier.
    public func listFormRevisions(_ typeId: String) throws -> [FormDefinition] {
        return try operations.lockboxListFormRevisions(handle!, typeId)
    }

    /// Stages a form record at path using the referenced definition.
    public func createFormRecord(_ path: String, _ typeReference: String, _ name: String) throws -> FormRecord {
        return try operations.lockboxCreateFormRecord(handle!, path, typeReference, name)
    }

    /// Stages a plain field value in a form record.
    @discardableResult
    public func setFormField(_ path: String, _ field: String, _ value: String) throws -> Bool {
        return try operations.lockboxSetFormField(handle!, path, field, value)
    }

    /// Sets secret form field.
    @discardableResult
    public func setSecretFormField(_ path: String, _ field: String, _ value: Data) throws -> Bool {
        return try operations.lockboxSetSecretFormField(handle!, path, field, value)
    }

    /// Lists form records without exposing secret field values.
    public func listFormRecords() throws -> [FormRecord] {
        return try operations.lockboxListFormRecords(handle!)
    }

    /// Returns the form record at path when present.
    public func getFormRecord(_ path: String) throws -> FormRecord? {
        return try operations.lockboxGetFormRecord(handle!, path)
    }

    /// Stages removal of a form record.
    @discardableResult
    public func deleteFormRecord(_ path: String) throws -> Bool {
        return try operations.lockboxDeleteFormRecord(handle!, path)
    }

    /// Updates form records.
    @discardableResult
    public func moveFormRecords(_ moves: [PathMove]) throws -> Bool {
        return try operations.lockboxMoveFormRecords(handle!, DomainCodec.encodePathMoves(moves))
    }

    /// Returns a plain form field when it exists.
    public func getFormField(_ path: String, _ field: String) throws -> FormValue? {
        return try operations.lockboxGetFormField(handle!, path, field)
    }

    /// Invokes a callback with temporary secret field bytes, then clears them.
    public func withSecretFormField<T>(_ path: String, _ field: String, _ callback: (UnsafeRawBufferPointer) throws -> T) throws -> T? {
        return try operations.lockboxWithSecretFormField(handle!, path, field, callback)
    }

    /// Serializes the current Lockbox, including committed changes.
    public func toBytes() throws -> Data {
        return try operations.lockboxToBytes(handle!)
    }

    /// Releases the native resources held by this object.
    public func free() throws -> Void {
        try operations.lockboxFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension ContactKeyPair {
    /// Returns the public bytes.
    public func publicBytes() throws -> Data {
        return try operations.keyContactPublic(handle!)
    }

    /// Returns the private bytes.
    public func privateBytes() throws -> Data {
        return try operations.keyContactPrivate(handle!)
    }

    /// Releases the native resources held by this object.
    public func free() throws -> Void {
        try operations.keyContactFree(handle!)
        handle = nil
    }

    /// Decrypts a wrapped content key for this contact.
    public func decrypt(_ wrapped: OwnedHandle) throws -> Data {
        return try operations.keyContactDecrypt(handle!, wrapped.handle!)
    }

}

/// Returns the member.
extension ContactPublicKey {
    /// Releases this public contact key.
    public func publicFree() throws -> Void {
        try operations.keyContactPublicFree(handle!)
        handle = nil
    }

    /// Encrypts a content key for the selected contact.
    public func encrypt(_ contentKey: Data) throws -> WrappedContactKey {
        return WrappedContactKey(operations, try operations.keyContactEncrypt(handle!, contentKey))
    }

}

/// Returns the member.
extension WrappedContactKey {
    /// Returns the public bytes.
    public func publicBytes() throws -> Data {
        return try operations.keyContactWrappedPublic(handle!)
    }

    /// Returns the encrypted content key bytes.
    public func ciphertext() throws -> Data {
        return try operations.keyContactWrappedCiphertext(handle!)
    }

    /// Returns the complete wrapped key record for storage or transport.
    public func encrypted() throws -> Data {
        return try operations.keyContactWrappedEncrypted(handle!)
    }

    /// Releases the native resources held by this object.
    public func free() throws -> Void {
        try operations.keyContactWrappedFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension ProfileSigningKeyPair {
    /// Returns the public bytes.
    public func publicBytes() throws -> Data {
        return try operations.keySigningPublic(handle!)
    }

    /// Returns the private signing-key record for secure binary backup.
    public func privateRecord() throws -> Data {
        return try operations.keySigningPrivate(handle!)
    }

    /// Creates an independently owned public verification-key handle.
    public func publicKey() throws -> ProfileSigningPublicKey {
        return ProfileSigningPublicKey(
            operations,
            try operations.keySigningPublicFromBytes(publicBytes())
        )
    }

    /// Wipes and releases the native signing-key handle.
    public func dispose() throws -> Void {
        try operations.keySigningFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension ProfileSigningPublicKey {
    /// Releases the native verification-key handle.
    public func dispose() throws -> Void {
        try operations.keySigningPublicFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension Vault {
    /// Returns the canonical root directory of this Vault.
    public func root() throws -> String {
        return try operations.vaultDirectoryRoot(handle!)
    }

    /// Returns the persistent structure version of this Vault.
    public func structureVersion() throws -> UInt32 {
        return try operations.vaultDirectoryStructureVersion(handle!)
    }

    /// Lists private keys.
    public func listPrivateKeys() throws -> [String] {
        return try operations.vaultDirectoryListPrivateKeys(handle!)
    }

    /// Lists private key names.
    public func listPrivateKeyNames() throws -> [String] {
        return try operations.vaultDirectoryListPrivateKeyNames(handle!)
    }

    /// Lists contact names.
    public func listContactNames() throws -> [String] {
        return try operations.vaultDirectoryListContactNames(handle!)
    }

    /// Lists form aliases.
    public func listFormAliases() throws -> [String] {
        return try operations.vaultDirectoryListFormAliases(handle!)
    }

    /// Reports whether the named profile private key exists.
    @discardableResult
    public func privateKeyExists(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryPrivateKeyExists(handle!, name)
    }

    /// Removes private key.
    @discardableResult
    public func deletePrivateKey(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryDeletePrivateKey(handle!, name)
    }

    /// Stores private key.
    @discardableResult
    public func storePrivateKey(_ name: String, _ key: OwnedHandle) throws -> Bool {
        return try operations.vaultDirectoryStorePrivateKey(handle!, name, key.handle!)
    }

    /// Loads private key.
    public func loadPrivateKey(_ name: String) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultDirectoryLoadPrivateKey(handle!, name))
    }

    /// Loads private key generation.
    public func loadPrivateKeyGeneration(_ name: String, _ index: UInt16) throws -> ContactKeyPair {
        return ContactKeyPair(operations, try operations.vaultDirectoryLoadPrivateKeyGeneration(handle!, name, index))
    }

    /// Stores contact.
    @discardableResult
    public func storeContact(_ name: String, _ key: OwnedHandle) throws -> Bool {
        return try operations.vaultDirectoryStoreContact(handle!, name, key.handle!)
    }

    /// Loads contact.
    public func loadContact(_ name: String) throws -> ContactPublicKey {
        return ContactPublicKey(operations, try operations.vaultDirectoryLoadContact(handle!, name))
    }

    /// Reports whether the named contact exists.
    @discardableResult
    public func contactExists(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryContactExists(handle!, name)
    }

    /// Removes contact.
    @discardableResult
    public func deleteContact(_ name: String) throws -> Bool {
        return try operations.vaultDirectoryDeleteContact(handle!, name)
    }

    /// Lists contacts.
    public func listContacts() throws -> [Contact] {
        return try operations.vaultDirectoryListContacts(handle!)
    }

    /// Stores profile email.
    @discardableResult
    public func storeProfileEmail(_ name: String, _ email: String) throws -> Bool {
        return try operations.vaultDirectoryStoreProfileEmail(handle!, name, email)
    }

    /// Returns the email recorded for a profile, when present.
    public func profileEmail(_ name: String) throws -> String? {
        return try operations.vaultDirectoryProfileEmail(handle!, name)
    }

    /// Stores backup.
    @discardableResult
    public func storeBackup(_ id: Data, _ bytes: Data) throws -> Bool {
        return try operations.vaultDirectoryStoreBackup(handle!, id, bytes)
    }

    /// Loads backup.
    public func loadBackup(_ id: Data) throws -> Data {
        return try operations.vaultDirectoryLoadBackup(handle!, id)
    }

    /// Returns the number of stored key recovery backups.
    public func backupCount() throws -> UInt64 {
        return try operations.vaultDirectoryBackupCount(handle!)
    }

    /// Restores a profile private key and signing key from recovery material.
    @discardableResult
    public func restorePrivateKey(_ name: String, _ key: OwnedHandle, _ signingKey: ProfileSigningKeyPair, _ overwrite: Bool) throws -> Bool {
        return try operations.vaultDirectoryRestorePrivateKey(handle!, name, key.handle!, signingKey.handle!, overwrite)
    }

    /// Loads the current profile signing identity.
    public func loadProfileSigningKey(_ name: String) throws -> ProfileSigningKeyPair {
        return ProfileSigningKeyPair(operations, try operations.vaultDirectoryLoadOwnerSigningKey(handle!, name))
    }

    /// Loads a historical profile signing identity generation.
    public func loadProfileSigningKeyGeneration(_ name: String, _ index: UInt16) throws -> ProfileSigningKeyPair {
        return ProfileSigningKeyPair(operations, try operations.vaultDirectoryLoadOwnerSigningKeyGeneration(handle!, name, index))
    }

    /// Stores contact signing key.
    @discardableResult
    public func storeContactSigningKey(_ name: String, _ key: ProfileSigningPublicKey) throws -> Bool {
        return try operations.vaultDirectoryStoreContactSigningKey(handle!, name, key.handle!)
    }

    /// Loads contact signing key.
    public func loadContactSigningKey(_ name: String) throws -> ProfileSigningPublicKey {
        return ProfileSigningPublicKey(operations, try operations.vaultDirectoryLoadContactSigningKey(handle!, name))
    }

    /// Lists profile generations.
    public func listProfileGenerations(_ name: String) throws -> ProfileHistory {
        return try operations.vaultDirectoryListProfileGenerations(handle!, name)
    }

    /// Updates private key.
    public func rotatePrivateKey(_ name: String) throws -> ProfileHistory {
        return try operations.vaultDirectoryRotatePrivateKey(handle!, name)
    }

    /// Stores lockbox.
    @discardableResult
    public func rememberLockbox(_ id: Data, _ path: String) throws -> Bool {
        return try operations.vaultDirectoryRememberLockbox(handle!, id, path)
    }

    /// Lists known lockboxes.
    public func listKnownLockboxes() throws -> [KnownLockbox] {
        return try operations.vaultDirectoryListKnownLockboxes(handle!)
    }

    /// Removes lockbox.
    @discardableResult
    public func forgetLockbox(_ path: String) throws -> Bool {
        return try operations.vaultDirectoryForgetLockbox(handle!, path)
    }

    /// Stores access slot label.
    @discardableResult
    public func rememberAccessSlotLabel(_ id: Data, _ slotId: UInt64, _ name: String) throws -> Bool {
        return try operations.vaultDirectoryRememberAccessSlotLabel(handle!, id, slotId, name)
    }

    /// Lists access slot labels.
    public func listAccessSlotLabels(_ id: Data) throws -> [AccessSlotLabel] {
        return try operations.vaultDirectoryListAccessSlotLabels(handle!, id)
    }

    /// Finds access slot labels with the supplied name for one Lockbox.
    public func findAccessSlotLabels(_ id: Data, _ name: String) throws -> [AccessSlotLabel] {
        return try operations.vaultDirectoryFindAccessSlotLabels(handle!, id, name)
    }

    /// Removes access slot label.
    @discardableResult
    public func forgetAccessSlotLabel(_ id: Data, _ slotId: UInt64) throws -> Bool {
        return try operations.vaultDirectoryForgetAccessSlotLabel(handle!, id, slotId)
    }

    /// Defines and stores a reusable versioned form.
    public func defineForm(_ alias: String, _ name: String, _ description: String, _ fields: [FormField]) throws -> FormDefinition {
        return try operations.vaultDirectoryDefineForm(handle!, alias, name, description, DomainCodec.encodeFormFields(fields))
    }

    /// Resolves a form alias, type identifier, or revision.
    public func resolveForm(_ reference: String) throws -> FormDefinition {
        return try operations.vaultDirectoryResolveForm(handle!, reference)
    }

    /// Lists forms.
    public func listForms() throws -> [FormDefinition] {
        return try operations.vaultDirectoryListForms(handle!)
    }

    /// Lists every stored revision for a form type identifier.
    public func listFormRevisions(_ typeId: String) throws -> [FormDefinition] {
        return try operations.vaultDirectoryListFormRevisions(handle!, typeId)
    }

    /// Adds missing standard form definitions and returns the number added.
    public func seedForms() throws -> Int {
        return try operations.vaultDirectorySeedForms(handle!)
    }

    /// Stores password.
    @discardableResult
    public func rememberPassword(_ id: Data, _ password: Data) throws -> Bool {
        return try operations.vaultDirectoryRememberPassword(handle!, id, password)
    }

    /// Returns the Lockbox password encrypted inside this Vault.
    public func rememberedPassword(_ id: Data) throws -> Data {
        return try operations.vaultDirectoryRememberedPassword(handle!, id)
    }

    /// Releases the native resources held by this object.
    public func free() throws -> Void {
        try operations.vaultDirectoryFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension ReadOnlyVault {
    /// Lists profile names.
    public func listProfileNames() throws -> [String] {
        return try operations.vaultReadOnlyListProfileNames(handle!)
    }

    /// Lists contact names.
    public func listContactNames() throws -> [String] {
        return try operations.vaultReadOnlyListContactNames(handle!)
    }

    /// Lists form aliases.
    public func listFormAliases() throws -> [String] {
        return try operations.vaultReadOnlyListFormAliases(handle!)
    }

    /// Lists known lockboxes.
    public func listKnownLockboxes() throws -> [KnownLockbox] {
        return try operations.vaultReadOnlyListKnownLockboxes(handle!)
    }

    /// Releases the native resources held by this object.
    public func free() throws -> Void {
        try operations.vaultReadOnlyFree(handle!)
        handle = nil
    }

}

/// Returns the member.
extension AgentSession {
    /// Reports whether running.
    @discardableResult
    public func isRunning() throws -> Bool {
        return try operations.vaultIsRunning()
    }

    /// Removes all.
    @discardableResult
    public func forgetAll() throws -> Bool {
        return try operations.vaultForgetAll()
    }

    /// Runs the session agent server until it is stopped.
    @discardableResult
    public func serve() throws -> Bool {
        return try operations.vaultAgentServe()
    }

    /// Verifies transport.
    @discardableResult
    public func verifyTransport() throws -> Bool {
        return try operations.vaultAgentVerifyTransport()
    }

    /// Returns get.
    public func get(_ id: Data) throws -> Data {
        return try operations.vaultAgentGet(id)
    }

    /// Stores put.
    @discardableResult
    public func put(_ id: Data, _ key: Data) throws -> Bool {
        return try operations.vaultAgentPut(id, key)
    }

    /// Removes forget.
    @discardableResult
    public func forget(_ id: Data) throws -> Bool {
        return try operations.vaultAgentForget(id)
    }

    /// Stops stop.
    @discardableResult
    public func stop() throws -> Bool {
        return try operations.vaultAgentStop()
    }

    /// Starts start.
    @discardableResult
    public func start() throws -> Bool {
        return try operations.vaultAgentStart()
    }

    /// Lists entries below path, optionally including descendants.
    public func list() throws -> [AgentEntry] {
        return try operations.vaultAgentList()
    }

    /// Reports how the platform handles agent expiry during system sleep.
    public func sleepSupport() throws -> SleepSupport {
        return try operations.vaultAgentSleepSupport()
    }

    /// Returns vault unlock key.
    public func getVaultUnlockKey(_ vaultId: String) throws -> Data {
        return try operations.vaultAgentGetVaultUnlockKey(vaultId)
    }

    /// Stores vault unlock key.
    @discardableResult
    public func putVaultUnlockKey(_ vaultId: String, _ key: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds)
    }

    /// Removes vault unlock key.
    @discardableResult
    public func forgetVaultUnlockKey(_ vaultId: String) throws -> Bool {
        return try operations.vaultAgentForgetVaultUnlockKey(vaultId)
    }

    /// Returns a profile signing identity cached by the Session Agent.
    public func profileSigningKey(_ vaultId: String, _ profile: String) throws -> ProfileSigningKeyPair {
        return ProfileSigningKeyPair(operations, try operations.vaultAgentGetOwnerSigningKey(vaultId, profile))
    }

    /// Caches a profile signing identity for the requested session TTL.
    @discardableResult
    public func cacheProfileSigningKey(_ vaultId: String, _ profile: String, _ key: ProfileSigningKeyPair, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key.handle!, ttlSeconds)
    }

    /// Removes one cached profile signing identity.
    @discardableResult
    public func forgetProfileSigningKey(_ vaultId: String, _ profile: String) throws -> Bool {
        return try operations.vaultAgentForgetOwnerSigningKey(vaultId, profile)
    }

    /// Starts activity.
    public func beginActivity(_ kind: String) throws -> AgentActivity {
        return AgentActivity(operations, try operations.vaultAgentBeginActivity(kind))
    }

    /// Stops activity.
    public func endActivity(_ handle: OwnedHandle) throws -> Void {
        try operations.vaultAgentEndActivity(handle.handle!)
    }

    private func localHandle() throws -> UnsafeMutableRawPointer {
        guard let handle else { throw RevaultError.native("Session Agent local handle is unavailable") }
        return handle
    }

    /// Creates a password-protected Lockbox at a host path.
    public func createLockboxPassword(_ path: String, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxPassword(localHandle(), path, password))
    }

    /// Opens a password-protected Lockbox at a host path.
    public func openLockboxPassword(_ path: String, _ password: Data) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultOpenLockboxPassword(localHandle(), path, password))
    }

    /// Creates a signed Lockbox from a content key at a host path.
    public func createLockboxContentKey(_ path: String, _ contentKey: Data, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxContentKey(localHandle(), path, contentKey, signingKey.handle!))
    }

    /// Creates a Lockbox for a contact at a host path.
    public func createLockboxContact(_ path: String, _ contact: OwnedHandle, _ name: String, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultCreateLockboxContact(localHandle(), path, contact.handle!, name, signingKey.handle!))
    }

    /// Opens a signed Lockbox from a content key at a host path.
    public func openLockboxContentKey(_ path: String, _ contentKey: Data, _ signingKey: ProfileSigningKeyPair) throws -> Lockbox {
        return Lockbox(operations, try operations.vaultOpenLockboxContentKey(localHandle(), path, contentKey, signingKey.handle!))
    }

    /// Caches a password for a host-path Lockbox until the supplied TTL.
    @discardableResult
    public func cacheLockboxPassword(_ path: String, _ password: Data, _ ttlSeconds: UInt64) throws -> Bool {
        return try operations.vaultCacheLockboxPassword(localHandle(), path, password, ttlSeconds)
    }

    /// Removes the agent's cached key for a host-path Lockbox.
    @discardableResult
    public func closeLockbox(_ path: String) throws -> Bool {
        return try operations.vaultCloseLockbox(localHandle(), path)
    }

    /// Removes all keys held by the local session handle.
    @discardableResult
    public func closeAll() throws -> Bool {
        return try operations.vaultCloseAll(localHandle())
    }

    /// Releases the local session handle in addition to the agent resources.
    public func free() throws -> Void {
        if let handle {
            try operations.vaultFree(handle)
            self.handle = nil
        }
    }

}

/// Returns the member.
extension AgentActivity {
}

/// Returns the member.
extension Platform {
    /// Returns availability and user presence guarantees for platform storage.
    public func status() throws -> PlatformStatus {
        return try operations.vaultPlatformStatus()
    }

    /// Sets scope.
    @discardableResult
    public func setScope(_ scope: String) throws -> Bool {
        return try operations.vaultPlatformSetScope(scope)
    }

    /// Removes password.
    @discardableResult
    public func forgetPassword() throws -> Bool {
        return try operations.vaultPlatformForgetPassword()
    }

    /// Stores password.
    @discardableResult
    public func putPassword(_ password: Data) throws -> Bool {
        return try operations.vaultPlatformPutPassword(password)
    }

    /// Enables storage of the Vault passphrase in platform credentials.
    @discardableResult
    public func enable() throws -> Bool {
        return try operations.vaultPlatformEnable()
    }

    /// Disables platform credential use without deleting the stored value.
    @discardableResult
    public func disable() throws -> Bool {
        return try operations.vaultPlatformDisable()
    }

    /// Reports whether platform credential use is disabled.
    @discardableResult
    public func disabled() throws -> Bool {
        return try operations.vaultPlatformDisabled()
    }

    /// Returns password.
    public func getPassword() throws -> Data {
        return try operations.vaultPlatformGetPassword()
    }

}

/// Returns the member.
