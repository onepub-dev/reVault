using Google.Protobuf;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using Revault.Bindings;

namespace Revault;

/// <summary>Generated complete, typed C# surface for every exported operation.</summary>
internal sealed class BindingOperations
{
    public BindingOperations() { if (RevaultNative.api_abi_version() != 2) throw new DllNotFoundException("revault-api native ABI mismatch; expected 2"); }
    private static string LastError() => Marshal.PtrToStringUTF8(RevaultNative.buffer_last_error()) ?? "native operation failed";
    private static bool Require(bool value) { if (!value) throw new InvalidOperationException(LastError()); return true; }
    private static IntPtr Require(IntPtr value) { if (value == IntPtr.Zero) throw new InvalidOperationException(LastError()); return value; }
    private static byte[] Take(RevaultBuffer value)
    {
        if (value.Ptr == IntPtr.Zero) throw new InvalidOperationException(LastError());
        var result = new byte[checked((int)value.Length)];
        Marshal.Copy(value.Ptr, result, 0, result.Length);
        RevaultNative.buffer_free(value);
        return result;
    }
    private static string TakeString(RevaultBuffer value) => Encoding.UTF8.GetString(Take(value));
    private delegate bool SecretGetter(out IntPtr handle);
    private static T? WithSecret<T>(SecretGetter getter, SecretCallback<T> callback)
    {
        Require(getter(out var handle));
        return WithSecret(handle, callback);
    }
    private static T? WithSecret<T>(IntPtr handle, SecretCallback<T> callback)
    {
        if (handle == IntPtr.Zero) return default;
        try
        {
            Require(RevaultNative.secret_len(handle, out var length));
            var bytes = new byte[checked((int)length)];
            try
            {
                unsafe
                {
                    fixed (byte* pointer = bytes)
                        Require(RevaultNative.secret_copy(handle, (IntPtr)pointer, length));
                }
                return callback(bytes);
            }
            finally { CryptographicOperations.ZeroMemory(bytes); }
        }
        finally { RevaultNative.secret_free(handle); }
    }
    private static T Frame<T>(RevaultBuffer value, MessageParser<T> parser) where T : IMessage<T>
    {
        var frame = Take(value);
        if (frame.Length < 12 || frame[0] != 'L' || frame[1] != 'B' || frame[2] != 'W' || frame[3] != 'F')
            throw new InvalidOperationException("invalid reVault binding frame");
        var length = System.Buffers.Binary.BinaryPrimitives.ReadUInt32BigEndian(frame.AsSpan(8, 4));
        if (length != frame.Length - 12) throw new InvalidOperationException("invalid reVault binding frame length");
        return parser.ParseFrom(frame, 12, checked((int)length));
    }

    public string LastErrorMessage() => LastError();

    public unsafe Revault.Bindings.ErrorDetails BufferLastErrorDetails()
    {
        return Frame(RevaultNative.buffer_last_error_details(), Revault.Bindings.ErrorDetails.Parser);
    }

    public unsafe ushort LockboxFormatVersion()
    {
        return RevaultNative.lockbox_format_version();
    }

    public unsafe ushort LockboxProbeFormatVersion(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return RevaultNative.lockbox_probe_format_version((IntPtr)bytesPointer, (nuint)bytes.Length); }
    }

    public unsafe IntPtr LockboxCreate(byte[] key)
    {
        fixed (byte* keyPointer = key)
        { return Require(RevaultNative.lockbox_create((IntPtr)keyPointer, (nuint)key.Length)); }
    }

    public unsafe IntPtr LockboxCreateWithOptions(byte[] key, string cacheMode, ulong cacheBytes, string workload, string worker, nuint jobs)
    {
        var cacheModeBytes = Encoding.UTF8.GetBytes(cacheMode);
        var workloadBytes = Encoding.UTF8.GetBytes(workload);
        var workerBytes = Encoding.UTF8.GetBytes(worker);
        fixed (byte* keyPointer = key)
        fixed (byte* cacheModeBytesPointer = cacheModeBytes)
        fixed (byte* workloadBytesPointer = workloadBytes)
        fixed (byte* workerBytesPointer = workerBytes)
        { return Require(RevaultNative.lockbox_create_with_options((IntPtr)keyPointer, (nuint)key.Length, (IntPtr)cacheModeBytesPointer, (nuint)cacheModeBytes.Length, cacheBytes, (IntPtr)workloadBytesPointer, (nuint)workloadBytes.Length, (IntPtr)workerBytesPointer, (nuint)workerBytes.Length, jobs)); }
    }

    public unsafe IntPtr LockboxCreatePassword(byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.lockbox_create_password((IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr LockboxCreateContact(IntPtr contact)
    {
        return Require(RevaultNative.lockbox_create_contact(contact));
    }

    public unsafe IntPtr LockboxCreateWithSigningKey(byte[] contentKey, IntPtr signingKey)
    {
        fixed (byte* contentKeyPointer = contentKey)
        { return Require(RevaultNative.lockbox_create_with_signing_key((IntPtr)contentKeyPointer, (nuint)contentKey.Length, signingKey)); }
    }

    public unsafe IntPtr LockboxOpen(byte[] archive, byte[] key)
    {
        fixed (byte* archivePointer = archive)
        fixed (byte* keyPointer = key)
        { return Require(RevaultNative.lockbox_open((IntPtr)archivePointer, (nuint)archive.Length, (IntPtr)keyPointer, (nuint)key.Length)); }
    }

    public unsafe IntPtr LockboxOpenWithOptions(byte[] archive, byte[] key, string cacheMode, ulong cacheBytes, string workload, string worker, nuint jobs)
    {
        var cacheModeBytes = Encoding.UTF8.GetBytes(cacheMode);
        var workloadBytes = Encoding.UTF8.GetBytes(workload);
        var workerBytes = Encoding.UTF8.GetBytes(worker);
        fixed (byte* archivePointer = archive)
        fixed (byte* keyPointer = key)
        fixed (byte* cacheModeBytesPointer = cacheModeBytes)
        fixed (byte* workloadBytesPointer = workloadBytes)
        fixed (byte* workerBytesPointer = workerBytes)
        { return Require(RevaultNative.lockbox_open_with_options((IntPtr)archivePointer, (nuint)archive.Length, (IntPtr)keyPointer, (nuint)key.Length, (IntPtr)cacheModeBytesPointer, (nuint)cacheModeBytes.Length, cacheBytes, (IntPtr)workloadBytesPointer, (nuint)workloadBytes.Length, (IntPtr)workerBytesPointer, (nuint)workerBytes.Length, jobs)); }
    }

    public unsafe IntPtr LockboxOpenPassword(byte[] archive, byte[] password)
    {
        fixed (byte* archivePointer = archive)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.lockbox_open_password((IntPtr)archivePointer, (nuint)archive.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr LockboxOpenContact(byte[] archive, IntPtr contact)
    {
        fixed (byte* archivePointer = archive)
        { return Require(RevaultNative.lockbox_open_contact((IntPtr)archivePointer, (nuint)archive.Length, contact)); }
    }

    public unsafe bool LockboxAddFile(IntPtr handle, string path, byte[] data, bool replace)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* dataPointer = data)
        { return Require(RevaultNative.lockbox_add_file(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)dataPointer, (nuint)data.Length, replace)); }
    }

    public unsafe bool LockboxAddFileWithPermissions(IntPtr handle, string path, byte[] data, uint permissions, bool replace)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* dataPointer = data)
        { return Require(RevaultNative.lockbox_add_file_with_permissions(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)dataPointer, (nuint)data.Length, permissions, replace)); }
    }

    public unsafe byte[] LockboxGetFile(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Take(RevaultNative.lockbox_get_file(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool LockboxExtractFile(IntPtr handle, string source, string destination, bool replace)
    {
        var sourceBytes = Encoding.UTF8.GetBytes(source);
        var destinationBytes = Encoding.UTF8.GetBytes(destination);
        fixed (byte* sourceBytesPointer = sourceBytes)
        fixed (byte* destinationBytesPointer = destinationBytes)
        { return Require(RevaultNative.lockbox_extract_file(handle, (IntPtr)sourceBytesPointer, (nuint)sourceBytes.Length, (IntPtr)destinationBytesPointer, (nuint)destinationBytes.Length, replace)); }
    }

    public unsafe bool LockboxExtractDirectory(IntPtr handle, string destination, ulong maxFileBytes, ulong maxTotalBytes, nuint maxFiles, bool restoreSymlinks, bool restorePermissions, bool overwrite)
    {
        var destinationBytes = Encoding.UTF8.GetBytes(destination);
        fixed (byte* destinationBytesPointer = destinationBytes)
        { return Require(RevaultNative.lockbox_extract_directory(handle, (IntPtr)destinationBytesPointer, (nuint)destinationBytes.Length, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite)); }
    }

    public unsafe Revault.Bindings.StreamChunkList LockboxStreamContent(IntPtr handle, bool physical)
    {
        return Frame(RevaultNative.lockbox_stream_content(handle, physical), Revault.Bindings.StreamChunkList.Parser);
    }

    public unsafe Revault.Bindings.CacheStats LockboxCacheStats(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_cache_stats(handle), Revault.Bindings.CacheStats.Parser);
    }

    public unsafe Revault.Bindings.ImportStats LockboxImportStats(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_import_stats(handle), Revault.Bindings.ImportStats.Parser);
    }

    public unsafe bool LockboxResetImportStats(IntPtr handle)
    {
        return Require(RevaultNative.lockbox_reset_import_stats(handle));
    }

    public unsafe Revault.Bindings.FileInspection LockboxInspectFile(string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.lockbox_inspect_file((IntPtr)pathBytesPointer, (nuint)pathBytes.Length), Revault.Bindings.FileInspection.Parser); }
    }

    public unsafe Revault.Bindings.PageInspectionList LockboxPageInspection(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_page_inspection(handle), Revault.Bindings.PageInspectionList.Parser);
    }

    public unsafe Revault.Bindings.RecoveryReport LockboxRecoveryReport(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_recovery_report(handle), Revault.Bindings.RecoveryReport.Parser);
    }

    public unsafe string LockboxRecoveryReportRender(IntPtr handle, bool verbose, nuint maxEntries)
    {
        return TakeString(RevaultNative.lockbox_recovery_report_render(handle, verbose, maxEntries));
    }

    public unsafe Revault.Bindings.RecoveryReport LockboxRecoveryScanPath(string path, byte[] key)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* keyPointer = key)
        { return Frame(RevaultNative.lockbox_recovery_scan_path((IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)keyPointer, (nuint)key.Length), Revault.Bindings.RecoveryReport.Parser); }
    }

    public unsafe ulong LockboxStorageLen(IntPtr handle)
    {
        return RevaultNative.lockbox_storage_len(handle);
    }

    public unsafe bool LockboxSetWorkloadProfile(IntPtr handle, string profile)
    {
        var profileBytes = Encoding.UTF8.GetBytes(profile);
        fixed (byte* profileBytesPointer = profileBytes)
        { return Require(RevaultNative.lockbox_set_workload_profile(handle, (IntPtr)profileBytesPointer, (nuint)profileBytes.Length)); }
    }

    public unsafe bool LockboxSetWorkerPolicy(IntPtr handle, string mode, nuint jobs)
    {
        var modeBytes = Encoding.UTF8.GetBytes(mode);
        fixed (byte* modeBytesPointer = modeBytes)
        { return Require(RevaultNative.lockbox_set_worker_policy(handle, (IntPtr)modeBytesPointer, (nuint)modeBytes.Length, jobs)); }
    }

    public unsafe Revault.Bindings.RuntimeOptions LockboxRuntimeOptions(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_runtime_options(handle), Revault.Bindings.RuntimeOptions.Parser);
    }

    public unsafe bool LockboxCommit(IntPtr handle)
    {
        return Require(RevaultNative.lockbox_commit(handle));
    }

    public unsafe bool LockboxCreateDir(IntPtr handle, string path, bool createParents)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_create_dir(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, createParents)); }
    }

    public unsafe bool LockboxDelete(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_delete(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool LockboxRemoveDir(IntPtr handle, string path, bool recursive)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_remove_dir(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, recursive)); }
    }

    public unsafe bool LockboxCreateParentDirs(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_create_parent_dirs(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool LockboxRename(IntPtr handle, string from, string to)
    {
        var fromBytes = Encoding.UTF8.GetBytes(from);
        var toBytes = Encoding.UTF8.GetBytes(to);
        fixed (byte* fromBytesPointer = fromBytes)
        fixed (byte* toBytesPointer = toBytes)
        { return Require(RevaultNative.lockbox_rename(handle, (IntPtr)fromBytesPointer, (nuint)fromBytes.Length, (IntPtr)toBytesPointer, (nuint)toBytes.Length)); }
    }

    public unsafe Revault.Bindings.LockboxEntryList LockboxList(IntPtr handle, string path, bool recursive)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.lockbox_list(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, recursive), Revault.Bindings.LockboxEntryList.Parser); }
    }

    public unsafe Revault.Bindings.LockboxEntryList LockboxListWithOptions(IntPtr handle, string path, string glob, bool recursive, bool includeFiles, bool includeSymlinks, bool includeDirectories, nuint limit)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var globBytes = Encoding.UTF8.GetBytes(glob);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* globBytesPointer = globBytes)
        { return Frame(RevaultNative.lockbox_list_with_options(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)globBytesPointer, (nuint)globBytes.Length, recursive, includeFiles, includeSymlinks, includeDirectories, limit), Revault.Bindings.LockboxEntryList.Parser); }
    }

    public unsafe Revault.Bindings.OptionalLockboxEntry LockboxStat(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.lockbox_stat(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length), Revault.Bindings.OptionalLockboxEntry.Parser); }
    }

    public unsafe bool LockboxSetVariable(IntPtr handle, string name, string value)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        var valueBytes = Encoding.UTF8.GetBytes(value);
        fixed (byte* nameBytesPointer = nameBytes)
        fixed (byte* valueBytesPointer = valueBytes)
        { return Require(RevaultNative.lockbox_set_variable(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, (IntPtr)valueBytesPointer, (nuint)valueBytes.Length)); }
    }

    public unsafe bool LockboxSetSecretVariable(IntPtr handle, string name, byte[] value)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        fixed (byte* valuePointer = value)
        { return Require(RevaultNative.lockbox_set_secret_variable(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, (IntPtr)valuePointer, (nuint)value.Length)); }
    }

    public unsafe Revault.Bindings.OptionalString LockboxGetVariable(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.lockbox_get_variable(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.OptionalString.Parser); }
    }

    public unsafe T? LockboxWithSecretVariable<T>(IntPtr handle, string name, SecretCallback<T> callback)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        IntPtr secret;
        fixed (byte* nameBytesPointer = nameBytes)
        { Require(RevaultNative.lockbox_get_secret_variable(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, out secret)); }
        return WithSecret(secret, callback);
    }

    public unsafe bool LockboxDeleteVariable(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.lockbox_delete_variable(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe bool LockboxMoveVariables(IntPtr handle, byte[] movesProto)
    {
        fixed (byte* movesProtoPointer = movesProto)
        { return Require(RevaultNative.lockbox_move_variables(handle, (IntPtr)movesProtoPointer, (nuint)movesProto.Length)); }
    }

    public unsafe Revault.Bindings.VariableList LockboxListVariables(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_list_variables(handle), Revault.Bindings.VariableList.Parser);
    }

    public unsafe Revault.Bindings.OptionalString LockboxVariableSensitivity(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.lockbox_variable_sensitivity(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.OptionalString.Parser); }
    }

    public unsafe bool LockboxAddSymlink(IntPtr handle, string path, string target, bool replace)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var targetBytes = Encoding.UTF8.GetBytes(target);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* targetBytesPointer = targetBytes)
        { return Require(RevaultNative.lockbox_add_symlink(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)targetBytesPointer, (nuint)targetBytes.Length, replace)); }
    }

    public unsafe string LockboxGetSymlinkTarget(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return TakeString(RevaultNative.lockbox_get_symlink_target(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe byte[] LockboxId(IntPtr handle)
    {
        return Take(RevaultNative.lockbox_id(handle));
    }

    public unsafe bool LockboxExists(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return RevaultNative.lockbox_exists(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length); }
    }

    public unsafe bool LockboxIsDir(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return RevaultNative.lockbox_is_dir(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length); }
    }

    public unsafe uint LockboxPermissions(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return RevaultNative.lockbox_permissions(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length); }
    }

    public unsafe bool LockboxSetPermissions(IntPtr handle, string path, uint permissions)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_set_permissions(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, permissions)); }
    }

    public unsafe byte[] LockboxReadRange(IntPtr handle, string path, ulong offset, ulong len)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Take(RevaultNative.lockbox_read_range(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, offset, len)); }
    }

    public unsafe Revault.Bindings.RecoveryReport LockboxRecoveryScan(byte[] bytes, byte[] key)
    {
        fixed (byte* bytesPointer = bytes)
        fixed (byte* keyPointer = key)
        { return Frame(RevaultNative.lockbox_recovery_scan((IntPtr)bytesPointer, (nuint)bytes.Length, (IntPtr)keyPointer, (nuint)key.Length), Revault.Bindings.RecoveryReport.Parser); }
    }

    public unsafe IntPtr LockboxRecoverySalvage(byte[] bytes, byte[] key, IntPtr signingKey)
    {
        fixed (byte* bytesPointer = bytes)
        fixed (byte* keyPointer = key)
        { return Require(RevaultNative.lockbox_recovery_salvage((IntPtr)bytesPointer, (nuint)bytes.Length, (IntPtr)keyPointer, (nuint)key.Length, signingKey)); }
    }

    public unsafe ulong LockboxAddPassword(IntPtr handle, byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return RevaultNative.lockbox_add_password(handle, (IntPtr)passwordPointer, (nuint)password.Length); }
    }

    public unsafe ulong LockboxAddContact(IntPtr handle, IntPtr contact, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return RevaultNative.lockbox_add_contact(handle, contact, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length); }
    }

    public unsafe bool LockboxDeleteKey(IntPtr handle, ulong id)
    {
        return Require(RevaultNative.lockbox_delete_key(handle, id));
    }

    public unsafe Revault.Bindings.KeySlotList LockboxListKeySlots(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_list_key_slots(handle), Revault.Bindings.KeySlotList.Parser);
    }

    public unsafe bool LockboxSetOwnerSigningKey(IntPtr handle, IntPtr key)
    {
        return Require(RevaultNative.lockbox_set_owner_signing_key(handle, key));
    }

    public unsafe Revault.Bindings.OwnerInspection LockboxOwnerInspection(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_owner_inspection(handle), Revault.Bindings.OwnerInspection.Parser);
    }

    public unsafe Revault.Bindings.FormDefinition LockboxDefineForm(IntPtr handle, string alias, string name, string description, byte[] fieldsProto)
    {
        var aliasBytes = Encoding.UTF8.GetBytes(alias);
        var nameBytes = Encoding.UTF8.GetBytes(name);
        var descriptionBytes = Encoding.UTF8.GetBytes(description);
        fixed (byte* aliasBytesPointer = aliasBytes)
        fixed (byte* nameBytesPointer = nameBytes)
        fixed (byte* descriptionBytesPointer = descriptionBytes)
        fixed (byte* fieldsProtoPointer = fieldsProto)
        { return Frame(RevaultNative.lockbox_define_form(handle, (IntPtr)aliasBytesPointer, (nuint)aliasBytes.Length, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, (IntPtr)descriptionBytesPointer, (nuint)descriptionBytes.Length, (IntPtr)fieldsProtoPointer, (nuint)fieldsProto.Length), Revault.Bindings.FormDefinition.Parser); }
    }

    public unsafe Revault.Bindings.FormDefinitionList LockboxListFormDefinitions(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_list_form_definitions(handle), Revault.Bindings.FormDefinitionList.Parser);
    }

    public unsafe Revault.Bindings.FormDefinition LockboxResolveForm(IntPtr handle, string reference)
    {
        var referenceBytes = Encoding.UTF8.GetBytes(reference);
        fixed (byte* referenceBytesPointer = referenceBytes)
        { return Frame(RevaultNative.lockbox_resolve_form(handle, (IntPtr)referenceBytesPointer, (nuint)referenceBytes.Length), Revault.Bindings.FormDefinition.Parser); }
    }

    public unsafe Revault.Bindings.FormDefinitionList LockboxListFormRevisions(IntPtr handle, string typeId)
    {
        var typeIdBytes = Encoding.UTF8.GetBytes(typeId);
        fixed (byte* typeIdBytesPointer = typeIdBytes)
        { return Frame(RevaultNative.lockbox_list_form_revisions(handle, (IntPtr)typeIdBytesPointer, (nuint)typeIdBytes.Length), Revault.Bindings.FormDefinitionList.Parser); }
    }

    public unsafe Revault.Bindings.FormRecord LockboxCreateFormRecord(IntPtr handle, string path, string typeReference, string name)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var typeReferenceBytes = Encoding.UTF8.GetBytes(typeReference);
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* typeReferenceBytesPointer = typeReferenceBytes)
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.lockbox_create_form_record(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)typeReferenceBytesPointer, (nuint)typeReferenceBytes.Length, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.FormRecord.Parser); }
    }

    public unsafe bool LockboxSetFormField(IntPtr handle, string path, string field, string value)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var fieldBytes = Encoding.UTF8.GetBytes(field);
        var valueBytes = Encoding.UTF8.GetBytes(value);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* fieldBytesPointer = fieldBytes)
        fixed (byte* valueBytesPointer = valueBytes)
        { return Require(RevaultNative.lockbox_set_form_field(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)fieldBytesPointer, (nuint)fieldBytes.Length, (IntPtr)valueBytesPointer, (nuint)valueBytes.Length)); }
    }

    public unsafe bool LockboxSetSecretFormField(IntPtr handle, string path, string field, byte[] value)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var fieldBytes = Encoding.UTF8.GetBytes(field);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* fieldBytesPointer = fieldBytes)
        fixed (byte* valuePointer = value)
        { return Require(RevaultNative.lockbox_set_secret_form_field(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)fieldBytesPointer, (nuint)fieldBytes.Length, (IntPtr)valuePointer, (nuint)value.Length)); }
    }

    public unsafe Revault.Bindings.FormRecordList LockboxListFormRecords(IntPtr handle)
    {
        return Frame(RevaultNative.lockbox_list_form_records(handle), Revault.Bindings.FormRecordList.Parser);
    }

    public unsafe Revault.Bindings.OptionalFormRecord LockboxGetFormRecord(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.lockbox_get_form_record(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length), Revault.Bindings.OptionalFormRecord.Parser); }
    }

    public unsafe bool LockboxDeleteFormRecord(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.lockbox_delete_form_record(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool LockboxMoveFormRecords(IntPtr handle, byte[] movesProto)
    {
        fixed (byte* movesProtoPointer = movesProto)
        { return Require(RevaultNative.lockbox_move_form_records(handle, (IntPtr)movesProtoPointer, (nuint)movesProto.Length)); }
    }

    public unsafe Revault.Bindings.OptionalFormValue LockboxGetFormField(IntPtr handle, string path, string field)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var fieldBytes = Encoding.UTF8.GetBytes(field);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* fieldBytesPointer = fieldBytes)
        { return Frame(RevaultNative.lockbox_get_form_field(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)fieldBytesPointer, (nuint)fieldBytes.Length), Revault.Bindings.OptionalFormValue.Parser); }
    }

    public unsafe T? LockboxWithSecretFormField<T>(IntPtr handle, string path, string field, SecretCallback<T> callback)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var fieldBytes = Encoding.UTF8.GetBytes(field);
        IntPtr secret;
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* fieldBytesPointer = fieldBytes)
        { Require(RevaultNative.lockbox_get_secret_form_field(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)fieldBytesPointer, (nuint)fieldBytes.Length, out secret)); }
        return WithSecret(secret, callback);
    }

    public unsafe byte[] LockboxToBytes(IntPtr handle)
    {
        return Take(RevaultNative.lockbox_to_bytes(handle));
    }

    public unsafe void LockboxFree(IntPtr handle)
    {
        RevaultNative.lockbox_free(handle);
    }

    public unsafe bool VaultIsRunning()
    {
        return RevaultNative.vault_is_running();
    }

    public unsafe bool VaultForgetAll()
    {
        return Require(RevaultNative.vault_forget_all());
    }

    public unsafe IntPtr KeyContactGenerate()
    {
        return Require(RevaultNative.key_contact_generate());
    }

    public unsafe IntPtr KeyContactFromPrivate(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.key_contact_from_private((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] KeyContactPublic(IntPtr handle)
    {
        return Take(RevaultNative.key_contact_public(handle));
    }

    public unsafe byte[] KeyContactPrivate(IntPtr handle)
    {
        return Take(RevaultNative.key_contact_private(handle));
    }

    public unsafe IntPtr KeyContactPublicFromBytes(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.key_contact_public_from_bytes((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe void KeyContactPublicFree(IntPtr handle)
    {
        RevaultNative.key_contact_public_free(handle);
    }

    public unsafe void KeyContactFree(IntPtr handle)
    {
        RevaultNative.key_contact_free(handle);
    }

    public unsafe IntPtr KeyContactEncrypt(IntPtr contact, byte[] contentKey)
    {
        fixed (byte* contentKeyPointer = contentKey)
        { return Require(RevaultNative.key_contact_encrypt(contact, (IntPtr)contentKeyPointer, (nuint)contentKey.Length)); }
    }

    public unsafe byte[] KeyContactDecrypt(IntPtr contact, IntPtr wrapped)
    {
        return Take(RevaultNative.key_contact_decrypt(contact, wrapped));
    }

    public unsafe byte[] KeyContactWrappedPublic(IntPtr wrapped)
    {
        return Take(RevaultNative.key_contact_wrapped_public(wrapped));
    }

    public unsafe byte[] KeyContactWrappedCiphertext(IntPtr wrapped)
    {
        return Take(RevaultNative.key_contact_wrapped_ciphertext(wrapped));
    }

    public unsafe byte[] KeyContactWrappedEncrypted(IntPtr wrapped)
    {
        return Take(RevaultNative.key_contact_wrapped_encrypted(wrapped));
    }

    public unsafe void KeyContactWrappedFree(IntPtr handle)
    {
        RevaultNative.key_contact_wrapped_free(handle);
    }

    public unsafe IntPtr KeySigningGenerate()
    {
        return Require(RevaultNative.key_signing_generate());
    }

    public unsafe IntPtr KeySigningFromPrivate(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.key_signing_from_private((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] KeySigningPublic(IntPtr handle)
    {
        return Take(RevaultNative.key_signing_public(handle));
    }

    public unsafe byte[] KeySigningPrivate(IntPtr handle)
    {
        return Take(RevaultNative.key_signing_private(handle));
    }

    public unsafe IntPtr KeySigningPublicFromBytes(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.key_signing_public_from_bytes((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe void KeySigningPublicFree(IntPtr handle)
    {
        RevaultNative.key_signing_public_free(handle);
    }

    public unsafe void KeySigningFree(IntPtr handle)
    {
        RevaultNative.key_signing_free(handle);
    }

    public unsafe byte[] VaultKeyExportPrivate(IntPtr key, string format)
    {
        var formatBytes = Encoding.UTF8.GetBytes(format);
        fixed (byte* formatBytesPointer = formatBytes)
        { return Take(RevaultNative.vault_key_export_private(key, (IntPtr)formatBytesPointer, (nuint)formatBytes.Length)); }
    }

    public unsafe byte[] VaultKeyExportPublic(IntPtr key, string format)
    {
        var formatBytes = Encoding.UTF8.GetBytes(format);
        fixed (byte* formatBytesPointer = formatBytes)
        { return Take(RevaultNative.vault_key_export_public(key, (IntPtr)formatBytesPointer, (nuint)formatBytes.Length)); }
    }

    public unsafe IntPtr VaultKeyImportPrivate(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.vault_key_import_private((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe IntPtr VaultKeyImportPublic(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.vault_key_import_public((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] VaultKeyFingerprint(IntPtr key)
    {
        return Take(RevaultNative.vault_key_fingerprint(key));
    }

    public unsafe string VaultKeyFormatHex(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return TakeString(RevaultNative.vault_key_format_hex((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] VaultKeyDecodeHex(string text)
    {
        var textBytes = Encoding.UTF8.GetBytes(text);
        fixed (byte* textBytesPointer = textBytes)
        { return Take(RevaultNative.vault_key_decode_hex((IntPtr)textBytesPointer, (nuint)textBytes.Length)); }
    }

    public unsafe string VaultKeyFormatCrockford(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return TakeString(RevaultNative.vault_key_format_crockford((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe string VaultKeyFormatCrockfordReading(string code)
    {
        var codeBytes = Encoding.UTF8.GetBytes(code);
        fixed (byte* codeBytesPointer = codeBytes)
        { return TakeString(RevaultNative.vault_key_format_crockford_reading((IntPtr)codeBytesPointer, (nuint)codeBytes.Length)); }
    }

    public unsafe byte[] VaultKeyDecodeCrockford(string code)
    {
        var codeBytes = Encoding.UTF8.GetBytes(code);
        fixed (byte* codeBytesPointer = codeBytes)
        { return Take(RevaultNative.vault_key_decode_crockford((IntPtr)codeBytesPointer, (nuint)codeBytes.Length)); }
    }

    public unsafe string VaultKeyHexEncode(byte[] bytes)
    {
        fixed (byte* bytesPointer = bytes)
        { return TakeString(RevaultNative.vault_key_hex_encode((IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] VaultKeyHexDecode(string text)
    {
        var textBytes = Encoding.UTF8.GetBytes(text);
        fixed (byte* textBytesPointer = textBytes)
        { return Take(RevaultNative.vault_key_hex_decode((IntPtr)textBytesPointer, (nuint)textBytes.Length)); }
    }

    public unsafe IntPtr VaultDirectoryOpen(string root, byte[] password)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_open((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe uint VaultStructureVersionCurrent()
    {
        return RevaultNative.vault_structure_version_current();
    }

    public unsafe uint VaultDirectoryProbeStructureVersion(string root, byte[] password)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* passwordPointer = password)
        { return RevaultNative.vault_directory_probe_structure_version((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length); }
    }

    public unsafe IntPtr VaultDirectoryOpenOrCreateDefault(byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_open_or_create_default((IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr VaultDirectoryReplaceDefault(byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_replace_default((IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe bool VaultDirectoryChangePassword(string root, byte[] oldPassword, byte[] newPassword)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* oldPasswordPointer = oldPassword)
        fixed (byte* newPasswordPointer = newPassword)
        { return Require(RevaultNative.vault_directory_change_password((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)oldPasswordPointer, (nuint)oldPassword.Length, (IntPtr)newPasswordPointer, (nuint)newPassword.Length)); }
    }

    public unsafe bool VaultDirectoryChangeDefaultPassword(byte[] oldPassword, byte[] newPassword)
    {
        fixed (byte* oldPasswordPointer = oldPassword)
        fixed (byte* newPasswordPointer = newPassword)
        { return Require(RevaultNative.vault_directory_change_default_password((IntPtr)oldPasswordPointer, (nuint)oldPassword.Length, (IntPtr)newPasswordPointer, (nuint)newPassword.Length)); }
    }

    public unsafe IntPtr VaultDirectoryReplace(string root, byte[] password)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_replace((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr VaultDirectoryOpenOrCreate(string root, byte[] password)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_open_or_create((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe string VaultDirectoryRoot(IntPtr handle)
    {
        return TakeString(RevaultNative.vault_directory_root(handle));
    }

    public unsafe uint VaultDirectoryStructureVersion(IntPtr handle)
    {
        return RevaultNative.vault_directory_structure_version(handle);
    }

    public unsafe Revault.Bindings.StringList VaultDirectoryListPrivateKeys(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_private_keys(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.StringList VaultDirectoryListPrivateKeyNames(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_private_key_names(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.StringList VaultDirectoryListContactNames(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_contact_names(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.StringList VaultDirectoryListFormAliases(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_form_aliases(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe bool VaultDirectoryPrivateKeyExists(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return RevaultNative.vault_directory_private_key_exists(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length); }
    }

    public unsafe bool VaultDirectoryDeletePrivateKey(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_delete_private_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe bool VaultDirectoryStorePrivateKey(IntPtr handle, string name, IntPtr key)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_store_private_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, key)); }
    }

    public unsafe IntPtr VaultDirectoryLoadPrivateKey(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_private_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe IntPtr VaultDirectoryLoadPrivateKeyGeneration(IntPtr handle, string name, ushort index)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_private_key_generation(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, index)); }
    }

    public unsafe bool VaultDirectoryStoreContact(IntPtr handle, string name, IntPtr key)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_store_contact(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, key)); }
    }

    public unsafe IntPtr VaultDirectoryLoadContact(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_contact(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe bool VaultDirectoryContactExists(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return RevaultNative.vault_directory_contact_exists(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length); }
    }

    public unsafe bool VaultDirectoryDeleteContact(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_delete_contact(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe Revault.Bindings.ContactList VaultDirectoryListContacts(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_contacts(handle), Revault.Bindings.ContactList.Parser);
    }

    public unsafe bool VaultDirectoryStoreProfileEmail(IntPtr handle, string name, string email)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        var emailBytes = Encoding.UTF8.GetBytes(email);
        fixed (byte* nameBytesPointer = nameBytes)
        fixed (byte* emailBytesPointer = emailBytes)
        { return Require(RevaultNative.vault_directory_store_profile_email(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, (IntPtr)emailBytesPointer, (nuint)emailBytes.Length)); }
    }

    public unsafe Revault.Bindings.OptionalString VaultDirectoryProfileEmail(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.vault_directory_profile_email(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.OptionalString.Parser); }
    }

    public unsafe bool VaultDirectoryStoreBackup(IntPtr handle, byte[] id, byte[] bytes)
    {
        fixed (byte* idPointer = id)
        fixed (byte* bytesPointer = bytes)
        { return Require(RevaultNative.vault_directory_store_backup(handle, (IntPtr)idPointer, (nuint)id.Length, (IntPtr)bytesPointer, (nuint)bytes.Length)); }
    }

    public unsafe byte[] VaultDirectoryLoadBackup(IntPtr handle, byte[] id)
    {
        fixed (byte* idPointer = id)
        { return Take(RevaultNative.vault_directory_load_backup(handle, (IntPtr)idPointer, (nuint)id.Length)); }
    }

    public unsafe ulong VaultDirectoryBackupCount(IntPtr handle)
    {
        return RevaultNative.vault_directory_backup_count(handle);
    }

    public unsafe bool VaultDirectoryRestorePrivateKey(IntPtr handle, string name, IntPtr key, IntPtr signingKey, bool overwrite)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_restore_private_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, key, signingKey, overwrite)); }
    }

    public unsafe IntPtr VaultDirectoryLoadOwnerSigningKey(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_owner_signing_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe IntPtr VaultDirectoryLoadOwnerSigningKeyGeneration(IntPtr handle, string name, ushort index)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_owner_signing_key_generation(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, index)); }
    }

    public unsafe bool VaultDirectoryStoreContactSigningKey(IntPtr handle, string name, IntPtr key)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_store_contact_signing_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, key)); }
    }

    public unsafe IntPtr VaultDirectoryLoadContactSigningKey(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_load_contact_signing_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe Revault.Bindings.ProfileHistory VaultDirectoryListProfileGenerations(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.vault_directory_list_profile_generations(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.ProfileHistory.Parser); }
    }

    public unsafe Revault.Bindings.ProfileHistory VaultDirectoryRotatePrivateKey(IntPtr handle, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.vault_directory_rotate_private_key(handle, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.ProfileHistory.Parser); }
    }

    public unsafe bool VaultDirectoryRememberLockbox(IntPtr handle, byte[] id, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* idPointer = id)
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.vault_directory_remember_lockbox(handle, (IntPtr)idPointer, (nuint)id.Length, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe Revault.Bindings.KnownLockboxList VaultDirectoryListKnownLockboxes(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_known_lockboxes(handle), Revault.Bindings.KnownLockboxList.Parser);
    }

    public unsafe bool VaultDirectoryForgetLockbox(IntPtr handle, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.vault_directory_forget_lockbox(handle, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool VaultDirectoryRememberAccessSlotLabel(IntPtr handle, byte[] id, ulong slotId, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* idPointer = id)
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_directory_remember_access_slot_label(handle, (IntPtr)idPointer, (nuint)id.Length, slotId, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length)); }
    }

    public unsafe Revault.Bindings.AccessSlotLabelList VaultDirectoryListAccessSlotLabels(IntPtr handle, byte[] id)
    {
        fixed (byte* idPointer = id)
        { return Frame(RevaultNative.vault_directory_list_access_slot_labels(handle, (IntPtr)idPointer, (nuint)id.Length), Revault.Bindings.AccessSlotLabelList.Parser); }
    }

    public unsafe Revault.Bindings.AccessSlotLabelList VaultDirectoryFindAccessSlotLabels(IntPtr handle, byte[] id, string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* idPointer = id)
        fixed (byte* nameBytesPointer = nameBytes)
        { return Frame(RevaultNative.vault_directory_find_access_slot_labels(handle, (IntPtr)idPointer, (nuint)id.Length, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length), Revault.Bindings.AccessSlotLabelList.Parser); }
    }

    public unsafe bool VaultDirectoryForgetAccessSlotLabel(IntPtr handle, byte[] id, ulong slotId)
    {
        fixed (byte* idPointer = id)
        { return Require(RevaultNative.vault_directory_forget_access_slot_label(handle, (IntPtr)idPointer, (nuint)id.Length, slotId)); }
    }

    public unsafe Revault.Bindings.FormDefinition VaultDirectoryDefineForm(IntPtr handle, string alias, string name, string description, byte[] fieldsProto)
    {
        var aliasBytes = Encoding.UTF8.GetBytes(alias);
        var nameBytes = Encoding.UTF8.GetBytes(name);
        var descriptionBytes = Encoding.UTF8.GetBytes(description);
        fixed (byte* aliasBytesPointer = aliasBytes)
        fixed (byte* nameBytesPointer = nameBytes)
        fixed (byte* descriptionBytesPointer = descriptionBytes)
        fixed (byte* fieldsProtoPointer = fieldsProto)
        { return Frame(RevaultNative.vault_directory_define_form(handle, (IntPtr)aliasBytesPointer, (nuint)aliasBytes.Length, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, (IntPtr)descriptionBytesPointer, (nuint)descriptionBytes.Length, (IntPtr)fieldsProtoPointer, (nuint)fieldsProto.Length), Revault.Bindings.FormDefinition.Parser); }
    }

    public unsafe Revault.Bindings.FormDefinition VaultDirectoryResolveForm(IntPtr handle, string reference)
    {
        var referenceBytes = Encoding.UTF8.GetBytes(reference);
        fixed (byte* referenceBytesPointer = referenceBytes)
        { return Frame(RevaultNative.vault_directory_resolve_form(handle, (IntPtr)referenceBytesPointer, (nuint)referenceBytes.Length), Revault.Bindings.FormDefinition.Parser); }
    }

    public unsafe Revault.Bindings.FormDefinitionList VaultDirectoryListForms(IntPtr handle)
    {
        return Frame(RevaultNative.vault_directory_list_forms(handle), Revault.Bindings.FormDefinitionList.Parser);
    }

    public unsafe Revault.Bindings.FormDefinitionList VaultDirectoryListFormRevisions(IntPtr handle, string typeId)
    {
        var typeIdBytes = Encoding.UTF8.GetBytes(typeId);
        fixed (byte* typeIdBytesPointer = typeIdBytes)
        { return Frame(RevaultNative.vault_directory_list_form_revisions(handle, (IntPtr)typeIdBytesPointer, (nuint)typeIdBytes.Length), Revault.Bindings.FormDefinitionList.Parser); }
    }

    public unsafe nuint VaultDirectorySeedForms(IntPtr handle)
    {
        return RevaultNative.vault_directory_seed_forms(handle);
    }

    public unsafe bool VaultDirectoryRememberPassword(IntPtr handle, byte[] id, byte[] password)
    {
        fixed (byte* idPointer = id)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_directory_remember_password(handle, (IntPtr)idPointer, (nuint)id.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe byte[] VaultDirectoryRememberedPassword(IntPtr handle, byte[] id)
    {
        fixed (byte* idPointer = id)
        { return Take(RevaultNative.vault_directory_remembered_password(handle, (IntPtr)idPointer, (nuint)id.Length)); }
    }

    public unsafe Revault.Bindings.VaultBackupManifest VaultBackupDefault(string path, bool overwrite)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.vault_backup_default((IntPtr)pathBytesPointer, (nuint)pathBytes.Length, overwrite), Revault.Bindings.VaultBackupManifest.Parser); }
    }

    public unsafe Revault.Bindings.VaultBackupManifest VaultRestoreDefault(string path, bool overwrite)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Frame(RevaultNative.vault_restore_default((IntPtr)pathBytesPointer, (nuint)pathBytes.Length, overwrite), Revault.Bindings.VaultBackupManifest.Parser); }
    }

    public unsafe void VaultDirectoryFree(IntPtr handle)
    {
        RevaultNative.vault_directory_free(handle);
    }

    public unsafe IntPtr VaultReadOnlyOpen(string root, byte[] password)
    {
        var rootBytes = Encoding.UTF8.GetBytes(root);
        fixed (byte* rootBytesPointer = rootBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_read_only_open((IntPtr)rootBytesPointer, (nuint)rootBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr VaultReadOnlyOpenDefault(byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_read_only_open_default((IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe Revault.Bindings.StringList VaultReadOnlyListProfileNames(IntPtr handle)
    {
        return Frame(RevaultNative.vault_read_only_list_profile_names(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.StringList VaultReadOnlyListContactNames(IntPtr handle)
    {
        return Frame(RevaultNative.vault_read_only_list_contact_names(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.StringList VaultReadOnlyListFormAliases(IntPtr handle)
    {
        return Frame(RevaultNative.vault_read_only_list_form_aliases(handle), Revault.Bindings.StringList.Parser);
    }

    public unsafe Revault.Bindings.KnownLockboxList VaultReadOnlyListKnownLockboxes(IntPtr handle)
    {
        return Frame(RevaultNative.vault_read_only_list_known_lockboxes(handle), Revault.Bindings.KnownLockboxList.Parser);
    }

    public unsafe void VaultReadOnlyFree(IntPtr handle)
    {
        RevaultNative.vault_read_only_free(handle);
    }

    public unsafe bool VaultAgentServe()
    {
        return Require(RevaultNative.vault_agent_serve());
    }

    public unsafe bool VaultAgentVerifyTransport()
    {
        return Require(RevaultNative.vault_agent_verify_transport());
    }

    public unsafe byte[] VaultAgentGet(byte[] id)
    {
        fixed (byte* idPointer = id)
        { return Take(RevaultNative.vault_agent_get((IntPtr)idPointer, (nuint)id.Length)); }
    }

    public unsafe bool VaultAgentPut(byte[] id, byte[] key)
    {
        fixed (byte* idPointer = id)
        fixed (byte* keyPointer = key)
        { return Require(RevaultNative.vault_agent_put((IntPtr)idPointer, (nuint)id.Length, (IntPtr)keyPointer, (nuint)key.Length)); }
    }

    public unsafe bool VaultAgentForget(byte[] id)
    {
        fixed (byte* idPointer = id)
        { return Require(RevaultNative.vault_agent_forget((IntPtr)idPointer, (nuint)id.Length)); }
    }

    public unsafe bool VaultAgentStop()
    {
        return Require(RevaultNative.vault_agent_stop());
    }

    public unsafe bool VaultAgentStart()
    {
        return Require(RevaultNative.vault_agent_start());
    }

    public unsafe Revault.Bindings.AgentEntryList VaultAgentList()
    {
        return Frame(RevaultNative.vault_agent_list(), Revault.Bindings.AgentEntryList.Parser);
    }

    public unsafe Revault.Bindings.SleepSupport VaultAgentSleepSupport()
    {
        return Frame(RevaultNative.vault_agent_sleep_support(), Revault.Bindings.SleepSupport.Parser);
    }

    public unsafe Revault.Bindings.PlatformStatus VaultPlatformStatus()
    {
        return Frame(RevaultNative.vault_platform_status(), Revault.Bindings.PlatformStatus.Parser);
    }

    public unsafe bool VaultPlatformSetScope(string scope)
    {
        var scopeBytes = Encoding.UTF8.GetBytes(scope);
        fixed (byte* scopeBytesPointer = scopeBytes)
        { return Require(RevaultNative.vault_platform_set_scope((IntPtr)scopeBytesPointer, (nuint)scopeBytes.Length)); }
    }

    public unsafe bool VaultPlatformForgetPassword()
    {
        return Require(RevaultNative.vault_platform_forget_password());
    }

    public unsafe bool VaultPlatformPutPassword(byte[] password)
    {
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_platform_put_password((IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe bool VaultPlatformEnable()
    {
        return Require(RevaultNative.vault_platform_enable());
    }

    public unsafe bool VaultPlatformDisable()
    {
        return Require(RevaultNative.vault_platform_disable());
    }

    public unsafe bool VaultPlatformDisabled()
    {
        return RevaultNative.vault_platform_disabled();
    }

    public unsafe byte[] VaultPlatformGetPassword()
    {
        return Take(RevaultNative.vault_platform_get_password());
    }

    public unsafe string VaultDefaultDirectory()
    {
        return TakeString(RevaultNative.vault_default_directory());
    }

    public unsafe string VaultDefaultPath()
    {
        return TakeString(RevaultNative.vault_default_path());
    }

    public unsafe string VaultAgentLogPath()
    {
        return TakeString(RevaultNative.vault_agent_log_path());
    }

    public unsafe string VaultAgentLogDestination()
    {
        return TakeString(RevaultNative.vault_agent_log_destination());
    }

    public unsafe byte[] VaultAgentGetVaultUnlockKey(string vaultId)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        { return Take(RevaultNative.vault_agent_get_vault_unlock_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length)); }
    }

    public unsafe bool VaultAgentPutVaultUnlockKey(string vaultId, byte[] key, ulong ttlSeconds)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        fixed (byte* keyPointer = key)
        { return Require(RevaultNative.vault_agent_put_vault_unlock_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length, (IntPtr)keyPointer, (nuint)key.Length, ttlSeconds)); }
    }

    public unsafe bool VaultAgentForgetVaultUnlockKey(string vaultId)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        { return Require(RevaultNative.vault_agent_forget_vault_unlock_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length)); }
    }

    public unsafe IntPtr VaultAgentGetOwnerSigningKey(string vaultId, string profile)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        var profileBytes = Encoding.UTF8.GetBytes(profile);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        fixed (byte* profileBytesPointer = profileBytes)
        { return Require(RevaultNative.vault_agent_get_owner_signing_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length, (IntPtr)profileBytesPointer, (nuint)profileBytes.Length)); }
    }

    public unsafe bool VaultAgentPutOwnerSigningKey(string vaultId, string profile, IntPtr key, ulong ttlSeconds)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        var profileBytes = Encoding.UTF8.GetBytes(profile);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        fixed (byte* profileBytesPointer = profileBytes)
        { return Require(RevaultNative.vault_agent_put_owner_signing_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length, (IntPtr)profileBytesPointer, (nuint)profileBytes.Length, key, ttlSeconds)); }
    }

    public unsafe bool VaultAgentForgetOwnerSigningKey(string vaultId, string profile)
    {
        var vaultIdBytes = Encoding.UTF8.GetBytes(vaultId);
        var profileBytes = Encoding.UTF8.GetBytes(profile);
        fixed (byte* vaultIdBytesPointer = vaultIdBytes)
        fixed (byte* profileBytesPointer = profileBytes)
        { return Require(RevaultNative.vault_agent_forget_owner_signing_key((IntPtr)vaultIdBytesPointer, (nuint)vaultIdBytes.Length, (IntPtr)profileBytesPointer, (nuint)profileBytes.Length)); }
    }

    public unsafe IntPtr VaultAgentBeginActivity(string kind)
    {
        var kindBytes = Encoding.UTF8.GetBytes(kind);
        fixed (byte* kindBytesPointer = kindBytes)
        { return Require(RevaultNative.vault_agent_begin_activity((IntPtr)kindBytesPointer, (nuint)kindBytes.Length)); }
    }

    public unsafe void VaultAgentEndActivity(IntPtr handle)
    {
        RevaultNative.vault_agent_end_activity(handle);
    }

    public unsafe IntPtr VaultLocal()
    {
        return Require(RevaultNative.vault_local());
    }

    public unsafe IntPtr VaultCreateLockboxPassword(IntPtr vault, string path, byte[] password)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_create_lockbox_password(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr VaultOpenLockboxPassword(IntPtr vault, string path, byte[] password)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_open_lockbox_password(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length)); }
    }

    public unsafe IntPtr VaultCreateLockboxContentKey(IntPtr vault, string path, byte[] contentKey, IntPtr signingKey)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* contentKeyPointer = contentKey)
        { return Require(RevaultNative.vault_create_lockbox_content_key(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)contentKeyPointer, (nuint)contentKey.Length, signingKey)); }
    }

    public unsafe IntPtr VaultCreateLockboxContact(IntPtr vault, string path, IntPtr contact, string name, IntPtr signingKey)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        var nameBytes = Encoding.UTF8.GetBytes(name);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* nameBytesPointer = nameBytes)
        { return Require(RevaultNative.vault_create_lockbox_contact(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, contact, (IntPtr)nameBytesPointer, (nuint)nameBytes.Length, signingKey)); }
    }

    public unsafe IntPtr VaultOpenLockboxContentKey(IntPtr vault, string path, byte[] contentKey, IntPtr signingKey)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* contentKeyPointer = contentKey)
        { return Require(RevaultNative.vault_open_lockbox_content_key(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)contentKeyPointer, (nuint)contentKey.Length, signingKey)); }
    }

    public unsafe bool VaultCacheLockboxPassword(IntPtr vault, string path, byte[] password, ulong ttlSeconds)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        fixed (byte* passwordPointer = password)
        { return Require(RevaultNative.vault_cache_lockbox_password(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length, (IntPtr)passwordPointer, (nuint)password.Length, ttlSeconds)); }
    }

    public unsafe bool VaultCloseLockbox(IntPtr vault, string path)
    {
        var pathBytes = Encoding.UTF8.GetBytes(path);
        fixed (byte* pathBytesPointer = pathBytes)
        { return Require(RevaultNative.vault_close_lockbox(vault, (IntPtr)pathBytesPointer, (nuint)pathBytes.Length)); }
    }

    public unsafe bool VaultCloseAll(IntPtr vault)
    {
        return Require(RevaultNative.vault_close_all(vault));
    }

    public unsafe void VaultFree(IntPtr vault)
    {
        RevaultNative.vault_free(vault);
    }

}
