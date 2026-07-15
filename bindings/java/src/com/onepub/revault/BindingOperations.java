package com.onepub.revault;

import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.MessageLite;
import java.lang.foreign.Arena;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;

/** Generated complete, typed Java surface for every exported binding operation. */
public final class BindingOperations {
  private static final MemoryLayout BUFFER = MemoryLayout.structLayout(
      ValueLayout.ADDRESS.withName("ptr"), ValueLayout.JAVA_LONG.withName("len"));
  private final RevaultNativeApi api;

  public BindingOperations(RevaultNativeApi api) {
    this.api = api;
    try {
      if ((int) api.api_abi_version.invokeExact() != 1) throw new IllegalStateException("revault-api native ABI mismatch; expected 1");
    } catch (RuntimeException error) { throw error; }
      catch (Throwable error) { throw new IllegalStateException(error); }
  }

  @FunctionalInterface private interface Parser<T extends MessageLite> { T parse(byte[] value) throws InvalidProtocolBufferException; }

  private static Object call(java.lang.invoke.MethodHandle method, Object... args) {
    try { return method.invokeWithArguments(args); }
    catch (Throwable error) { throw error instanceof RuntimeException runtime ? runtime : new IllegalStateException(error); }
  }
  private static MemorySegment bytes(Arena arena, byte[] value) {
    if (value.length == 0) return MemorySegment.NULL;
    var result = arena.allocate(value.length); result.copyFrom(MemorySegment.ofArray(value)); return result;
  }
  private static MemorySegment text(Arena arena, String value) {
    if (value.isEmpty()) return MemorySegment.NULL;
    return bytes(arena, value.getBytes(StandardCharsets.UTF_8));
  }
  private String lastError() {
    var pointer = (MemorySegment) call(api.buffer_last_error);
    return pointer.address() == 0 ? "native operation failed" : pointer.reinterpret(Long.MAX_VALUE).getString(0);
  }
  private boolean require(boolean value) { if (!value) throw new IllegalStateException(lastError()); return true; }
  private MemorySegment require(MemorySegment value) { if (value.address() == 0) throw new IllegalStateException(lastError()); return value; }
  private byte[] take(MemorySegment value) {
    var pointer = value.get(ValueLayout.ADDRESS, 0);
    var length = value.get(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS.byteSize());
    if (pointer.address() == 0) throw new IllegalStateException(lastError());
    var result = pointer.reinterpret(length).toArray(ValueLayout.JAVA_BYTE);
    call(api.buffer_free, value); return result;
  }
  private String takeString(MemorySegment value) { return new String(take(value), StandardCharsets.UTF_8); }
  private <T extends MessageLite> T frame(MemorySegment value, Parser<T> parser) {
    var bytes = take(value);
    if (bytes.length < 12 || bytes[0] != 'L' || bytes[1] != 'B' || bytes[2] != 'W' || bytes[3] != 'F')
      throw new IllegalArgumentException("invalid reVault binding frame");
    int length = java.nio.ByteBuffer.wrap(bytes, 8, 4).order(ByteOrder.BIG_ENDIAN).getInt();
    if (length != bytes.length - 12) throw new IllegalArgumentException("invalid reVault binding frame length");
    try { return parser.parse(java.util.Arrays.copyOfRange(bytes, 12, bytes.length)); }
    catch (InvalidProtocolBufferException error) { throw new IllegalArgumentException("invalid reVault protobuf payload", error); }
  }

  public String lastErrorMessage() { return lastError(); }

  public revault.bindings.RevaultBindings.ErrorDetails bufferLastErrorDetails() {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.buffer_last_error_details, arena), revault.bindings.RevaultBindings.ErrorDetails::parseFrom);
    }
  }

  public short lockboxFormatVersion() {
    return (short) call(api.lockbox_format_version);
  }

  public short lockboxProbeFormatVersion(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return (short) call(api.lockbox_probe_format_version, bytes(arena, bytes), (long) bytes.length);
    }
  }

  public MemorySegment lockboxCreate(byte[] key) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_create, bytes(arena, key), (long) key.length));
    }
  }

  public MemorySegment lockboxCreateWithOptions(byte[] key, String cacheMode, long cacheBytes, String workload, String worker, long jobs) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_create_with_options, bytes(arena, key), (long) key.length, text(arena, cacheMode), (long) cacheMode.getBytes(StandardCharsets.UTF_8).length, cacheBytes, text(arena, workload), (long) workload.getBytes(StandardCharsets.UTF_8).length, text(arena, worker), (long) worker.getBytes(StandardCharsets.UTF_8).length, jobs));
    }
  }

  public MemorySegment lockboxCreatePassword(byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_create_password, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment lockboxCreateContact(MemorySegment contact) {
    return require((MemorySegment) call(api.lockbox_create_contact, contact));
  }

  public MemorySegment lockboxCreateWithSigningKey(byte[] contentKey, MemorySegment signingKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_create_with_signing_key, bytes(arena, contentKey), (long) contentKey.length, signingKey));
    }
  }

  public MemorySegment lockboxOpen(byte[] archive, byte[] key) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_open, bytes(arena, archive), (long) archive.length, bytes(arena, key), (long) key.length));
    }
  }

  public MemorySegment lockboxOpenWithOptions(byte[] archive, byte[] key, String cacheMode, long cacheBytes, String workload, String worker, long jobs) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_open_with_options, bytes(arena, archive), (long) archive.length, bytes(arena, key), (long) key.length, text(arena, cacheMode), (long) cacheMode.getBytes(StandardCharsets.UTF_8).length, cacheBytes, text(arena, workload), (long) workload.getBytes(StandardCharsets.UTF_8).length, text(arena, worker), (long) worker.getBytes(StandardCharsets.UTF_8).length, jobs));
    }
  }

  public MemorySegment lockboxOpenPassword(byte[] archive, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_open_password, bytes(arena, archive), (long) archive.length, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment lockboxOpenContact(byte[] archive, MemorySegment contact) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_open_contact, bytes(arena, archive), (long) archive.length, contact));
    }
  }

  public boolean lockboxAddFile(MemorySegment handle, String path, byte[] data, boolean replace) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_add_file, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, data), (long) data.length, replace));
    }
  }

  public boolean lockboxAddFileWithPermissions(MemorySegment handle, String path, byte[] data, int permissions, boolean replace) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_add_file_with_permissions, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, data), (long) data.length, permissions, replace));
    }
  }

  public byte[] lockboxGetFile(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.lockbox_get_file, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxExtractFile(MemorySegment handle, String source, String destination, boolean replace) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_extract_file, handle, text(arena, source), (long) source.getBytes(StandardCharsets.UTF_8).length, text(arena, destination), (long) destination.getBytes(StandardCharsets.UTF_8).length, replace));
    }
  }

  public boolean lockboxExtractDirectory(MemorySegment handle, String destination, long maxFileBytes, long maxTotalBytes, long maxFiles, boolean restoreSymlinks, boolean restorePermissions, boolean overwrite) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_extract_directory, handle, text(arena, destination), (long) destination.getBytes(StandardCharsets.UTF_8).length, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite));
    }
  }

  public revault.bindings.RevaultBindings.StreamChunkList lockboxStreamContent(MemorySegment handle, boolean physical) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_stream_content, arena, handle, physical), revault.bindings.RevaultBindings.StreamChunkList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.CacheStats lockboxCacheStats(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_cache_stats, arena, handle), revault.bindings.RevaultBindings.CacheStats::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.ImportStats lockboxImportStats(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_import_stats, arena, handle), revault.bindings.RevaultBindings.ImportStats::parseFrom);
    }
  }

  public boolean lockboxResetImportStats(MemorySegment handle) {
    return require((boolean) call(api.lockbox_reset_import_stats, handle));
  }

  public revault.bindings.RevaultBindings.FileInspection lockboxInspectFile(String path) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_inspect_file, arena, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FileInspection::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.PageInspectionList lockboxPageInspection(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_page_inspection, arena, handle), revault.bindings.RevaultBindings.PageInspectionList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.RecoveryReport lockboxRecoveryReport(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_recovery_report, arena, handle), revault.bindings.RevaultBindings.RecoveryReport::parseFrom);
    }
  }

  public String lockboxRecoveryReportRender(MemorySegment handle, boolean verbose, long maxEntries) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.lockbox_recovery_report_render, arena, handle, verbose, maxEntries));
    }
  }

  public revault.bindings.RevaultBindings.RecoveryReport lockboxRecoveryScanPath(String path, byte[] key) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_recovery_scan_path, arena, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, key), (long) key.length), revault.bindings.RevaultBindings.RecoveryReport::parseFrom);
    }
  }

  public long lockboxStorageLen(MemorySegment handle) {
    return (long) call(api.lockbox_storage_len, handle);
  }

  public boolean lockboxSetWorkloadProfile(MemorySegment handle, String profile) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_set_workload_profile, handle, text(arena, profile), (long) profile.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxSetWorkerPolicy(MemorySegment handle, String mode, long jobs) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_set_worker_policy, handle, text(arena, mode), (long) mode.getBytes(StandardCharsets.UTF_8).length, jobs));
    }
  }

  public revault.bindings.RevaultBindings.RuntimeOptions lockboxRuntimeOptions(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_runtime_options, arena, handle), revault.bindings.RevaultBindings.RuntimeOptions::parseFrom);
    }
  }

  public boolean lockboxCommit(MemorySegment handle) {
    return require((boolean) call(api.lockbox_commit, handle));
  }

  public boolean lockboxCreateDir(MemorySegment handle, String path, boolean createParents) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_create_dir, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, createParents));
    }
  }

  public boolean lockboxDelete(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_delete, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxRemoveDir(MemorySegment handle, String path, boolean recursive) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_remove_dir, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, recursive));
    }
  }

  public boolean lockboxCreateParentDirs(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_create_parent_dirs, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxRename(MemorySegment handle, String from, String to) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_rename, handle, text(arena, from), (long) from.getBytes(StandardCharsets.UTF_8).length, text(arena, to), (long) to.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.LockboxEntryList lockboxList(MemorySegment handle, String path, boolean recursive) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, recursive), revault.bindings.RevaultBindings.LockboxEntryList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.LockboxEntryList lockboxListWithOptions(MemorySegment handle, String path, String glob, boolean recursive, boolean includeFiles, boolean includeSymlinks, boolean includeDirectories, long limit) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_with_options, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, glob), (long) glob.getBytes(StandardCharsets.UTF_8).length, recursive, includeFiles, includeSymlinks, includeDirectories, limit), revault.bindings.RevaultBindings.LockboxEntryList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.OptionalLockboxEntry lockboxStat(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_stat, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.OptionalLockboxEntry::parseFrom);
    }
  }

  public boolean lockboxSetVariable(MemorySegment handle, String name, String value, boolean secret) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_set_variable, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, text(arena, value), (long) value.getBytes(StandardCharsets.UTF_8).length, secret));
    }
  }

  public String lockboxGetVariable(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.lockbox_get_variable, arena, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxDeleteVariable(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_delete_variable, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxMoveVariables(MemorySegment handle, byte[] movesProto) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_move_variables, handle, bytes(arena, movesProto), (long) movesProto.length));
    }
  }

  public revault.bindings.RevaultBindings.VariableList lockboxListVariables(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_variables, arena, handle), revault.bindings.RevaultBindings.VariableList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.OptionalString lockboxVariableSensitivity(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_variable_sensitivity, arena, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.OptionalString::parseFrom);
    }
  }

  public boolean lockboxAddSymlink(MemorySegment handle, String path, String target, boolean replace) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_add_symlink, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, target), (long) target.getBytes(StandardCharsets.UTF_8).length, replace));
    }
  }

  public String lockboxGetSymlinkTarget(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.lockbox_get_symlink_target, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public byte[] lockboxId(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.lockbox_id, arena, handle));
    }
  }

  public boolean lockboxExists(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return (boolean) call(api.lockbox_exists, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public boolean lockboxIsDir(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return (boolean) call(api.lockbox_is_dir, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public int lockboxPermissions(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return (int) call(api.lockbox_permissions, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public boolean lockboxSetPermissions(MemorySegment handle, String path, int permissions) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_set_permissions, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, permissions));
    }
  }

  public byte[] lockboxReadRange(MemorySegment handle, String path, long offset, long len) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.lockbox_read_range, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, offset, len));
    }
  }

  public revault.bindings.RevaultBindings.RecoveryReport lockboxRecoveryScan(byte[] bytes, byte[] key) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_recovery_scan, arena, bytes(arena, bytes), (long) bytes.length, bytes(arena, key), (long) key.length), revault.bindings.RevaultBindings.RecoveryReport::parseFrom);
    }
  }

  public MemorySegment lockboxRecoverySalvage(byte[] bytes, byte[] key, MemorySegment signingKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_recovery_salvage, bytes(arena, bytes), (long) bytes.length, bytes(arena, key), (long) key.length, signingKey));
    }
  }

  public long lockboxAddPassword(MemorySegment handle, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return (long) call(api.lockbox_add_password, handle, bytes(arena, password), (long) password.length);
    }
  }

  public long lockboxAddContact(MemorySegment handle, MemorySegment contact, String name) {
    try (var arena = Arena.ofConfined()) {
      return (long) call(api.lockbox_add_contact, handle, contact, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public boolean lockboxDeleteKey(MemorySegment handle, long id) {
    return require((boolean) call(api.lockbox_delete_key, handle, id));
  }

  public revault.bindings.RevaultBindings.KeySlotList lockboxListKeySlots(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_key_slots, arena, handle), revault.bindings.RevaultBindings.KeySlotList::parseFrom);
    }
  }

  public boolean lockboxSetOwnerSigningKey(MemorySegment handle, MemorySegment key) {
    return require((boolean) call(api.lockbox_set_owner_signing_key, handle, key));
  }

  public revault.bindings.RevaultBindings.OwnerInspection lockboxOwnerInspection(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_owner_inspection, arena, handle), revault.bindings.RevaultBindings.OwnerInspection::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinition lockboxDefineForm(MemorySegment handle, String alias, String name, String description, byte[] fieldsProto) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_define_form, arena, handle, text(arena, alias), (long) alias.getBytes(StandardCharsets.UTF_8).length, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, text(arena, description), (long) description.getBytes(StandardCharsets.UTF_8).length, bytes(arena, fieldsProto), (long) fieldsProto.length), revault.bindings.RevaultBindings.FormDefinition::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinitionList lockboxListFormDefinitions(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_form_definitions, arena, handle), revault.bindings.RevaultBindings.FormDefinitionList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinition lockboxResolveForm(MemorySegment handle, String reference) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_resolve_form, arena, handle, text(arena, reference), (long) reference.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormDefinition::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinitionList lockboxListFormRevisions(MemorySegment handle, String typeId) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_form_revisions, arena, handle, text(arena, typeId), (long) typeId.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormDefinitionList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormRecord lockboxCreateFormRecord(MemorySegment handle, String path, String typeReference, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_create_form_record, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, typeReference), (long) typeReference.getBytes(StandardCharsets.UTF_8).length, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormRecord::parseFrom);
    }
  }

  public boolean lockboxSetFormField(MemorySegment handle, String path, String field, String value, boolean secret) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_set_form_field, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, field), (long) field.getBytes(StandardCharsets.UTF_8).length, text(arena, value), (long) value.getBytes(StandardCharsets.UTF_8).length, secret));
    }
  }

  public revault.bindings.RevaultBindings.FormRecordList lockboxListFormRecords(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_list_form_records, arena, handle), revault.bindings.RevaultBindings.FormRecordList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormRecord lockboxGetFormRecord(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_get_form_record, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormRecord::parseFrom);
    }
  }

  public boolean lockboxDeleteFormRecord(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_delete_form_record, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean lockboxMoveFormRecords(MemorySegment handle, byte[] movesProto) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.lockbox_move_form_records, handle, bytes(arena, movesProto), (long) movesProto.length));
    }
  }

  public revault.bindings.RevaultBindings.FormValue lockboxGetFormField(MemorySegment handle, String path, String field) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.lockbox_get_form_field, arena, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, field), (long) field.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormValue::parseFrom);
    }
  }

  public byte[] lockboxToBytes(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.lockbox_to_bytes, arena, handle));
    }
  }

  public void lockboxFree(MemorySegment handle) {
    call(api.lockbox_free, handle);
  }

  public boolean vaultIsRunning() {
    return (boolean) call(api.vault_is_running);
  }

  public boolean vaultForgetAll() {
    return require((boolean) call(api.vault_forget_all));
  }

  public MemorySegment keyContactGenerate() {
    return require((MemorySegment) call(api.key_contact_generate));
  }

  public MemorySegment keyContactFromPrivate(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.key_contact_from_private, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] keyContactPublic(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_public, arena, handle));
    }
  }

  public byte[] keyContactPrivate(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_private, arena, handle));
    }
  }

  public MemorySegment keyContactPublicFromBytes(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.key_contact_public_from_bytes, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public void keyContactPublicFree(MemorySegment handle) {
    call(api.key_contact_public_free, handle);
  }

  public void keyContactFree(MemorySegment handle) {
    call(api.key_contact_free, handle);
  }

  public MemorySegment keyContactEncrypt(MemorySegment contact, byte[] contentKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.key_contact_encrypt, contact, bytes(arena, contentKey), (long) contentKey.length));
    }
  }

  public byte[] keyContactDecrypt(MemorySegment contact, MemorySegment wrapped) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_decrypt, arena, contact, wrapped));
    }
  }

  public byte[] keyContactWrappedPublic(MemorySegment wrapped) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_wrapped_public, arena, wrapped));
    }
  }

  public byte[] keyContactWrappedCiphertext(MemorySegment wrapped) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_wrapped_ciphertext, arena, wrapped));
    }
  }

  public byte[] keyContactWrappedEncrypted(MemorySegment wrapped) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_contact_wrapped_encrypted, arena, wrapped));
    }
  }

  public void keyContactWrappedFree(MemorySegment handle) {
    call(api.key_contact_wrapped_free, handle);
  }

  public MemorySegment keySigningGenerate() {
    return require((MemorySegment) call(api.key_signing_generate));
  }

  public MemorySegment keySigningFromPrivate(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.key_signing_from_private, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] keySigningPublic(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_signing_public, arena, handle));
    }
  }

  public byte[] keySigningPrivate(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.key_signing_private, arena, handle));
    }
  }

  public MemorySegment keySigningPublicFromBytes(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.key_signing_public_from_bytes, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public void keySigningPublicFree(MemorySegment handle) {
    call(api.key_signing_public_free, handle);
  }

  public void keySigningFree(MemorySegment handle) {
    call(api.key_signing_free, handle);
  }

  public byte[] vaultKeyExportPrivate(MemorySegment key, String format) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_export_private, arena, key, text(arena, format), (long) format.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public byte[] vaultKeyExportPublic(MemorySegment key, String format) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_export_public, arena, key, text(arena, format), (long) format.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultKeyImportPrivate(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_key_import_private, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public MemorySegment vaultKeyImportPublic(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_key_import_public, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] vaultKeyFingerprint(MemorySegment key) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_fingerprint, arena, key));
    }
  }

  public String vaultKeyFormatHex(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_key_format_hex, arena, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] vaultKeyDecodeHex(String text) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_decode_hex, arena, text(arena, text), (long) text.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public String vaultKeyFormatCrockford(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_key_format_crockford, arena, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public String vaultKeyFormatCrockfordReading(String code) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_key_format_crockford_reading, arena, text(arena, code), (long) code.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public byte[] vaultKeyDecodeCrockford(String code) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_decode_crockford, arena, text(arena, code), (long) code.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public String vaultKeyHexEncode(byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_key_hex_encode, arena, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] vaultKeyHexDecode(String text) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_key_hex_decode, arena, text(arena, text), (long) text.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultDirectoryOpen(String root, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_open, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public int vaultStructureVersionCurrent() {
    return (int) call(api.vault_structure_version_current);
  }

  public int vaultDirectoryProbeStructureVersion(String root, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return (int) call(api.vault_directory_probe_structure_version, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length);
    }
  }

  public MemorySegment vaultDirectoryOpenOrCreateDefault(byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_open_or_create_default, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment vaultDirectoryReplaceDefault(byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_replace_default, bytes(arena, password), (long) password.length));
    }
  }

  public boolean vaultDirectoryChangePassword(String root, byte[] oldPassword, byte[] newPassword) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_change_password, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, oldPassword), (long) oldPassword.length, bytes(arena, newPassword), (long) newPassword.length));
    }
  }

  public boolean vaultDirectoryChangeDefaultPassword(byte[] oldPassword, byte[] newPassword) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_change_default_password, bytes(arena, oldPassword), (long) oldPassword.length, bytes(arena, newPassword), (long) newPassword.length));
    }
  }

  public MemorySegment vaultDirectoryReplace(String root, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_replace, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment vaultDirectoryOpenOrCreate(String root, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_open_or_create, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public String vaultDirectoryRoot(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_directory_root, arena, handle));
    }
  }

  public int vaultDirectoryStructureVersion(MemorySegment handle) {
    return (int) call(api.vault_directory_structure_version, handle);
  }

  public revault.bindings.RevaultBindings.StringList vaultDirectoryListPrivateKeys(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_private_keys, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultDirectoryListPrivateKeyNames(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_private_key_names, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultDirectoryListContactNames(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_contact_names, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultDirectoryListFormAliases(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_form_aliases, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public boolean vaultDirectoryPrivateKeyExists(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return (boolean) call(api.vault_directory_private_key_exists, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public boolean vaultDirectoryDeletePrivateKey(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_delete_private_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultDirectoryStorePrivateKey(MemorySegment handle, String name, MemorySegment key) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_store_private_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, key));
    }
  }

  public MemorySegment vaultDirectoryLoadPrivateKey(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_private_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultDirectoryLoadPrivateKeyGeneration(MemorySegment handle, String name, short index) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_private_key_generation, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, index));
    }
  }

  public boolean vaultDirectoryStoreContact(MemorySegment handle, String name, MemorySegment key) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_store_contact, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, key));
    }
  }

  public MemorySegment vaultDirectoryLoadContact(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_contact, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultDirectoryContactExists(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return (boolean) call(api.vault_directory_contact_exists, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length);
    }
  }

  public boolean vaultDirectoryDeleteContact(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_delete_contact, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.ContactList vaultDirectoryListContacts(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_contacts, arena, handle), revault.bindings.RevaultBindings.ContactList::parseFrom);
    }
  }

  public boolean vaultDirectoryStoreProfileEmail(MemorySegment handle, String name, String email) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_store_profile_email, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, text(arena, email), (long) email.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.OptionalString vaultDirectoryProfileEmail(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_profile_email, arena, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.OptionalString::parseFrom);
    }
  }

  public boolean vaultDirectoryStoreBackup(MemorySegment handle, byte[] id, byte[] bytes) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_store_backup, handle, bytes(arena, id), (long) id.length, bytes(arena, bytes), (long) bytes.length));
    }
  }

  public byte[] vaultDirectoryLoadBackup(MemorySegment handle, byte[] id) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_directory_load_backup, arena, handle, bytes(arena, id), (long) id.length));
    }
  }

  public long vaultDirectoryBackupCount(MemorySegment handle) {
    return (long) call(api.vault_directory_backup_count, handle);
  }

  public boolean vaultDirectoryRestorePrivateKey(MemorySegment handle, String name, MemorySegment key, MemorySegment signingKey, boolean overwrite) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_restore_private_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, key, signingKey, overwrite));
    }
  }

  public MemorySegment vaultDirectoryLoadOwnerSigningKey(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_owner_signing_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultDirectoryLoadOwnerSigningKeyGeneration(MemorySegment handle, String name, short index) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_owner_signing_key_generation, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, index));
    }
  }

  public boolean vaultDirectoryStoreContactSigningKey(MemorySegment handle, String name, MemorySegment key) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_store_contact_signing_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, key));
    }
  }

  public MemorySegment vaultDirectoryLoadContactSigningKey(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_directory_load_contact_signing_key, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.ProfileHistory vaultDirectoryListProfileGenerations(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_profile_generations, arena, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.ProfileHistory::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.ProfileHistory vaultDirectoryRotatePrivateKey(MemorySegment handle, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_rotate_private_key, arena, handle, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.ProfileHistory::parseFrom);
    }
  }

  public boolean vaultDirectoryRememberLockbox(MemorySegment handle, byte[] id, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_remember_lockbox, handle, bytes(arena, id), (long) id.length, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.KnownLockboxList vaultDirectoryListKnownLockboxes(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_known_lockboxes, arena, handle), revault.bindings.RevaultBindings.KnownLockboxList::parseFrom);
    }
  }

  public boolean vaultDirectoryForgetLockbox(MemorySegment handle, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_forget_lockbox, handle, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultDirectoryRememberAccessSlotLabel(MemorySegment handle, byte[] id, long slotId, String name) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_remember_access_slot_label, handle, bytes(arena, id), (long) id.length, slotId, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public revault.bindings.RevaultBindings.AccessSlotLabelList vaultDirectoryListAccessSlotLabels(MemorySegment handle, byte[] id) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_access_slot_labels, arena, handle, bytes(arena, id), (long) id.length), revault.bindings.RevaultBindings.AccessSlotLabelList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.AccessSlotLabelList vaultDirectoryFindAccessSlotLabels(MemorySegment handle, byte[] id, String name) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_find_access_slot_labels, arena, handle, bytes(arena, id), (long) id.length, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.AccessSlotLabelList::parseFrom);
    }
  }

  public boolean vaultDirectoryForgetAccessSlotLabel(MemorySegment handle, byte[] id, long slotId) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_forget_access_slot_label, handle, bytes(arena, id), (long) id.length, slotId));
    }
  }

  public revault.bindings.RevaultBindings.FormDefinition vaultDirectoryDefineForm(MemorySegment handle, String alias, String name, String description, byte[] fieldsProto) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_define_form, arena, handle, text(arena, alias), (long) alias.getBytes(StandardCharsets.UTF_8).length, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, text(arena, description), (long) description.getBytes(StandardCharsets.UTF_8).length, bytes(arena, fieldsProto), (long) fieldsProto.length), revault.bindings.RevaultBindings.FormDefinition::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinition vaultDirectoryResolveForm(MemorySegment handle, String reference) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_resolve_form, arena, handle, text(arena, reference), (long) reference.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormDefinition::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinitionList vaultDirectoryListForms(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_forms, arena, handle), revault.bindings.RevaultBindings.FormDefinitionList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.FormDefinitionList vaultDirectoryListFormRevisions(MemorySegment handle, String typeId) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_directory_list_form_revisions, arena, handle, text(arena, typeId), (long) typeId.getBytes(StandardCharsets.UTF_8).length), revault.bindings.RevaultBindings.FormDefinitionList::parseFrom);
    }
  }

  public long vaultDirectorySeedForms(MemorySegment handle) {
    return (long) call(api.vault_directory_seed_forms, handle);
  }

  public boolean vaultDirectoryRememberPassword(MemorySegment handle, byte[] id, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_directory_remember_password, handle, bytes(arena, id), (long) id.length, bytes(arena, password), (long) password.length));
    }
  }

  public byte[] vaultDirectoryRememberedPassword(MemorySegment handle, byte[] id) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_directory_remembered_password, arena, handle, bytes(arena, id), (long) id.length));
    }
  }

  public revault.bindings.RevaultBindings.VaultBackupManifest vaultBackupDefault(String path, boolean overwrite) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_backup_default, arena, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, overwrite), revault.bindings.RevaultBindings.VaultBackupManifest::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.VaultBackupManifest vaultRestoreDefault(String path, boolean overwrite) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_restore_default, arena, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, overwrite), revault.bindings.RevaultBindings.VaultBackupManifest::parseFrom);
    }
  }

  public void vaultDirectoryFree(MemorySegment handle) {
    call(api.vault_directory_free, handle);
  }

  public MemorySegment vaultReadOnlyOpen(String root, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_read_only_open, text(arena, root), (long) root.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment vaultReadOnlyOpenDefault(byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_read_only_open_default, bytes(arena, password), (long) password.length));
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultReadOnlyListProfileNames(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_read_only_list_profile_names, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultReadOnlyListContactNames(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_read_only_list_contact_names, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.StringList vaultReadOnlyListFormAliases(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_read_only_list_form_aliases, arena, handle), revault.bindings.RevaultBindings.StringList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.KnownLockboxList vaultReadOnlyListKnownLockboxes(MemorySegment handle) {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_read_only_list_known_lockboxes, arena, handle), revault.bindings.RevaultBindings.KnownLockboxList::parseFrom);
    }
  }

  public void vaultReadOnlyFree(MemorySegment handle) {
    call(api.vault_read_only_free, handle);
  }

  public boolean vaultAgentServe() {
    return require((boolean) call(api.vault_agent_serve));
  }

  public boolean vaultAgentVerifyTransport() {
    return require((boolean) call(api.vault_agent_verify_transport));
  }

  public byte[] vaultAgentGet(byte[] id) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_agent_get, arena, bytes(arena, id), (long) id.length));
    }
  }

  public boolean vaultAgentPut(byte[] id, byte[] key) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_put, bytes(arena, id), (long) id.length, bytes(arena, key), (long) key.length));
    }
  }

  public boolean vaultAgentForget(byte[] id) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_forget, bytes(arena, id), (long) id.length));
    }
  }

  public boolean vaultAgentStop() {
    return require((boolean) call(api.vault_agent_stop));
  }

  public boolean vaultAgentStart() {
    return require((boolean) call(api.vault_agent_start));
  }

  public revault.bindings.RevaultBindings.AgentEntryList vaultAgentList() {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_agent_list, arena), revault.bindings.RevaultBindings.AgentEntryList::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.SleepSupport vaultAgentSleepSupport() {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_agent_sleep_support, arena), revault.bindings.RevaultBindings.SleepSupport::parseFrom);
    }
  }

  public revault.bindings.RevaultBindings.PlatformStatus vaultPlatformStatus() {
    try (var arena = Arena.ofConfined()) {
      return frame((MemorySegment) call(api.vault_platform_status, arena), revault.bindings.RevaultBindings.PlatformStatus::parseFrom);
    }
  }

  public boolean vaultPlatformSetScope(String scope) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_platform_set_scope, text(arena, scope), (long) scope.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultPlatformForgetPassword() {
    return require((boolean) call(api.vault_platform_forget_password));
  }

  public boolean vaultPlatformPutPassword(byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_platform_put_password, bytes(arena, password), (long) password.length));
    }
  }

  public boolean vaultPlatformEnable() {
    return require((boolean) call(api.vault_platform_enable));
  }

  public boolean vaultPlatformDisable() {
    return require((boolean) call(api.vault_platform_disable));
  }

  public boolean vaultPlatformDisabled() {
    return (boolean) call(api.vault_platform_disabled);
  }

  public byte[] vaultPlatformGetPassword() {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_platform_get_password, arena));
    }
  }

  public String vaultDefaultDirectory() {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_default_directory, arena));
    }
  }

  public String vaultDefaultPath() {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_default_path, arena));
    }
  }

  public String vaultAgentLogPath() {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_agent_log_path, arena));
    }
  }

  public String vaultAgentLogDestination() {
    try (var arena = Arena.ofConfined()) {
      return takeString((MemorySegment) call(api.vault_agent_log_destination, arena));
    }
  }

  public byte[] vaultAgentGetVaultUnlockKey(String vaultId) {
    try (var arena = Arena.ofConfined()) {
      return take((MemorySegment) call(api.vault_agent_get_vault_unlock_key, arena, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultAgentPutVaultUnlockKey(String vaultId, byte[] key, long ttlSeconds) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_put_vault_unlock_key, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length, bytes(arena, key), (long) key.length, ttlSeconds));
    }
  }

  public boolean vaultAgentForgetVaultUnlockKey(String vaultId) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_forget_vault_unlock_key, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultAgentGetOwnerSigningKey(String vaultId, String profile) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_agent_get_owner_signing_key, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length, text(arena, profile), (long) profile.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultAgentPutOwnerSigningKey(String vaultId, String profile, MemorySegment key, long ttlSeconds) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_put_owner_signing_key, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length, text(arena, profile), (long) profile.getBytes(StandardCharsets.UTF_8).length, key, ttlSeconds));
    }
  }

  public boolean vaultAgentForgetOwnerSigningKey(String vaultId, String profile) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_agent_forget_owner_signing_key, text(arena, vaultId), (long) vaultId.getBytes(StandardCharsets.UTF_8).length, text(arena, profile), (long) profile.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public MemorySegment vaultAgentBeginActivity(String kind) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_agent_begin_activity, text(arena, kind), (long) kind.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public void vaultAgentEndActivity(MemorySegment handle) {
    call(api.vault_agent_end_activity, handle);
  }

  public MemorySegment vaultLocal() {
    return require((MemorySegment) call(api.vault_local));
  }

  public MemorySegment vaultCreateLockboxPassword(MemorySegment vault, String path, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_create_lockbox_password, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment vaultOpenLockboxPassword(MemorySegment vault, String path, byte[] password) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_open_lockbox_password, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length));
    }
  }

  public MemorySegment vaultCreateLockboxContentKey(MemorySegment vault, String path, byte[] contentKey, MemorySegment signingKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_create_lockbox_content_key, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, contentKey), (long) contentKey.length, signingKey));
    }
  }

  public MemorySegment vaultCreateLockboxContact(MemorySegment vault, String path, MemorySegment contact, String name, MemorySegment signingKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_create_lockbox_contact, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, contact, text(arena, name), (long) name.getBytes(StandardCharsets.UTF_8).length, signingKey));
    }
  }

  public MemorySegment vaultOpenLockboxContentKey(MemorySegment vault, String path, byte[] contentKey, MemorySegment signingKey) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.vault_open_lockbox_content_key, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, contentKey), (long) contentKey.length, signingKey));
    }
  }

  public boolean vaultCacheLockboxPassword(MemorySegment vault, String path, byte[] password, long ttlSeconds) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_cache_lockbox_password, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, bytes(arena, password), (long) password.length, ttlSeconds));
    }
  }

  public boolean vaultCloseLockbox(MemorySegment vault, String path) {
    try (var arena = Arena.ofConfined()) {
      return require((boolean) call(api.vault_close_lockbox, vault, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length));
    }
  }

  public boolean vaultCloseAll(MemorySegment vault) {
    return require((boolean) call(api.vault_close_all, vault));
  }

  public void vaultFree(MemorySegment vault) {
    call(api.vault_free, vault);
  }

}
