package com.onepub.revault;

import com.google.flatbuffers.FlatBufferBuilder;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/** Internal conversion between native result bytes and public domain values. */
final class DomainCodec {
  private DomainCodec() {}
  static LockboxEntry lockboxEntry(byte[] bytes) {
    return new LockboxEntry(com.onepub.revault.internal.LockboxEntry.getRootAsLockboxEntry(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static PathMove pathMove(byte[] bytes) {
    return new PathMove(com.onepub.revault.internal.PathMove.getRootAsPathMove(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static FormField formField(byte[] bytes) {
    return new FormField(com.onepub.revault.internal.FormField.getRootAsFormField(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static FormDefinition formDefinition(byte[] bytes) {
    return new FormDefinition(com.onepub.revault.internal.FormDefinition.getRootAsFormDefinition(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static FormValue formValue(byte[] bytes) {
    return new FormValue(com.onepub.revault.internal.FormValue.getRootAsFormValue(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static FormRecord formRecord(byte[] bytes) {
    return new FormRecord(com.onepub.revault.internal.FormRecord.getRootAsFormRecord(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static RecoveryReport recoveryReport(byte[] bytes) {
    return new RecoveryReport(com.onepub.revault.internal.RecoveryReport.getRootAsRecoveryReport(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static KeySlot keySlot(byte[] bytes) {
    return new KeySlot(com.onepub.revault.internal.KeySlot.getRootAsKeySlot(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static CacheStats cacheStats(byte[] bytes) {
    return new CacheStats(com.onepub.revault.internal.CacheStats.getRootAsCacheStats(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static ImportStats importStats(byte[] bytes) {
    return new ImportStats(com.onepub.revault.internal.ImportStats.getRootAsImportStats(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static PageObject pageObject(byte[] bytes) {
    return new PageObject(com.onepub.revault.internal.PageObject.getRootAsPageObject(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static PageInspection pageInspection(byte[] bytes) {
    return new PageInspection(com.onepub.revault.internal.PageInspection.getRootAsPageInspection(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static FileInspection fileInspection(byte[] bytes) {
    return new FileInspection(com.onepub.revault.internal.FileInspection.getRootAsFileInspection(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static ProfileGeneration profileGeneration(byte[] bytes) {
    return new ProfileGeneration(com.onepub.revault.internal.ProfileGeneration.getRootAsProfileGeneration(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static ProfileHistory profileHistory(byte[] bytes) {
    return new ProfileHistory(com.onepub.revault.internal.ProfileHistory.getRootAsProfileHistory(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static KnownLockbox knownLockbox(byte[] bytes) {
    return new KnownLockbox(com.onepub.revault.internal.KnownLockbox.getRootAsKnownLockbox(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static AccessSlotLabel accessSlotLabel(byte[] bytes) {
    return new AccessSlotLabel(com.onepub.revault.internal.AccessSlotLabel.getRootAsAccessSlotLabel(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static StreamChunk streamChunk(byte[] bytes) {
    return new StreamChunk(com.onepub.revault.internal.StreamChunk.getRootAsStreamChunk(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static RuntimeOptions runtimeOptions(byte[] bytes) {
    return new RuntimeOptions(com.onepub.revault.internal.RuntimeOptions.getRootAsRuntimeOptions(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static Variable variable(byte[] bytes) {
    return new Variable(com.onepub.revault.internal.Variable.getRootAsVariable(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static OwnerInspection ownerInspection(byte[] bytes) {
    return new OwnerInspection(com.onepub.revault.internal.OwnerInspection.getRootAsOwnerInspection(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static Contact contact(byte[] bytes) {
    return new Contact(com.onepub.revault.internal.Contact.getRootAsContact(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static AgentEntry agentEntry(byte[] bytes) {
    return new AgentEntry(com.onepub.revault.internal.AgentEntry.getRootAsAgentEntry(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static SleepSupport sleepSupport(byte[] bytes) {
    return new SleepSupport(com.onepub.revault.internal.SleepSupport.getRootAsSleepSupport(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static PlatformStatus platformStatus(byte[] bytes) {
    return new PlatformStatus(com.onepub.revault.internal.PlatformStatus.getRootAsPlatformStatus(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static VaultBackupManifest vaultBackupManifest(byte[] bytes) {
    return new VaultBackupManifest(com.onepub.revault.internal.VaultBackupManifest.getRootAsVaultBackupManifest(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static ErrorDetails errorDetails(byte[] bytes) {
    return new ErrorDetails(com.onepub.revault.internal.ErrorDetails.getRootAsErrorDetails(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));
  }
  static java.util.List<StreamChunk> streamChunkList(byte[] bytes) {
    var view = com.onepub.revault.internal.StreamChunkList.getRootAsStreamChunkList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<StreamChunk>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new StreamChunk(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<PageInspection> pageInspectionList(byte[] bytes) {
    var view = com.onepub.revault.internal.PageInspectionList.getRootAsPageInspectionList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<PageInspection>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new PageInspection(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<LockboxEntry> lockboxEntryList(byte[] bytes) {
    var view = com.onepub.revault.internal.LockboxEntryList.getRootAsLockboxEntryList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<LockboxEntry>(view.entriesLength());
    for (int index = 0; index < view.entriesLength(); index++) result.add(new LockboxEntry(view.entries(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<Variable> variableList(byte[] bytes) {
    var view = com.onepub.revault.internal.VariableList.getRootAsVariableList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<Variable>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new Variable(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<KeySlot> keySlotList(byte[] bytes) {
    var view = com.onepub.revault.internal.KeySlotList.getRootAsKeySlotList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<KeySlot>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new KeySlot(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<FormDefinition> formDefinitionList(byte[] bytes) {
    var view = com.onepub.revault.internal.FormDefinitionList.getRootAsFormDefinitionList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<FormDefinition>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new FormDefinition(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<FormRecord> formRecordList(byte[] bytes) {
    var view = com.onepub.revault.internal.FormRecordList.getRootAsFormRecordList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<FormRecord>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new FormRecord(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<Contact> contactList(byte[] bytes) {
    var view = com.onepub.revault.internal.ContactList.getRootAsContactList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<Contact>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new Contact(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<KnownLockbox> knownLockboxList(byte[] bytes) {
    var view = com.onepub.revault.internal.KnownLockboxList.getRootAsKnownLockboxList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<KnownLockbox>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new KnownLockbox(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<AccessSlotLabel> accessSlotLabelList(byte[] bytes) {
    var view = com.onepub.revault.internal.AccessSlotLabelList.getRootAsAccessSlotLabelList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<AccessSlotLabel>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new AccessSlotLabel(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<AgentEntry> agentEntryList(byte[] bytes) {
    var view = com.onepub.revault.internal.AgentEntryList.getRootAsAgentEntryList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<AgentEntry>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new AgentEntry(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<ProfileHistory> profileHistoryList(byte[] bytes) {
    var view = com.onepub.revault.internal.ProfileHistoryList.getRootAsProfileHistoryList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));
    var result = new java.util.ArrayList<ProfileHistory>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new ProfileHistory(view.values(index)));
    return java.util.List.copyOf(result);
  }
  static java.util.List<String> stringList(byte[] bytes) { var view = com.onepub.revault.internal.StringList.getRootAsStringList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)); var result = new java.util.ArrayList<String>(view.valuesLength()); for (int i=0;i<view.valuesLength();i++) result.add(view.values(i)); return java.util.List.copyOf(result); }
  static LockboxEntry optionalLockboxEntry(byte[] bytes) { var value = com.onepub.revault.internal.OptionalLockboxEntry.getRootAsOptionalLockboxEntry(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new LockboxEntry(value); }
  static FormRecord optionalFormRecord(byte[] bytes) { var value = com.onepub.revault.internal.OptionalFormRecord.getRootAsOptionalFormRecord(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new FormRecord(value); }
  static FormValue optionalFormValue(byte[] bytes) { var value = com.onepub.revault.internal.OptionalFormValue.getRootAsOptionalFormValue(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new FormValue(value); }
  static String optionalString(byte[] bytes) { var value = com.onepub.revault.internal.OptionalString.getRootAsOptionalString(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)); return value.present() ? value.value() : null; }
  static byte[] encodePathMoves(java.util.List<PathMove> values) {
    var builder = new FlatBufferBuilder();
    var offsets = new int[values.size()];
    for (int index = 0; index < values.size(); index++) {
      var value = values.get(index);
      offsets[index] = com.onepub.revault.internal.PathMove.createPathMove(builder, builder.createString(value.source()), builder.createString(value.destination()));
    }
    var vector = com.onepub.revault.internal.PathMoveList.createValuesVector(builder, offsets);
    var root = com.onepub.revault.internal.PathMoveList.createPathMoveList(builder, vector);
    builder.finish(root);
    return builder.sizedByteArray();
  }
  static byte[] encodeFormFields(java.util.List<FormField> values) {
    var builder = new FlatBufferBuilder();
    var offsets = new int[values.size()];
    for (int index = 0; index < values.size(); index++) {
      var value = values.get(index);
      offsets[index] = com.onepub.revault.internal.FormField.createFormField(builder, builder.createString(value.id()), builder.createString(value.label()), builder.createString(value.kind()), value.required());
    }
    var vector = com.onepub.revault.internal.FormFieldList.createValuesVector(builder, offsets);
    var root = com.onepub.revault.internal.FormFieldList.createFormFieldList(builder, vector);
    builder.finish(root);
    return builder.sizedByteArray();
  }
}
