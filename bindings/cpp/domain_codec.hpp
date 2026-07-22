#pragma once

#include "domain_models.hpp"
#include "generated/flatbuffers/revault_bindings_generated.h"

namespace revault::detail {


inline bindings::LockboxEntry convert(const internal::LockboxEntryT& value);
inline bindings::PathMove convert(const internal::PathMoveT& value);
inline bindings::FormField convert(const internal::FormFieldT& value);
inline bindings::FormDefinition convert(const internal::FormDefinitionT& value);
inline bindings::FormValue convert(const internal::FormValueT& value);
inline bindings::FormRecord convert(const internal::FormRecordT& value);
inline bindings::RecoveryReport convert(const internal::RecoveryReportT& value);
inline bindings::KeySlot convert(const internal::KeySlotT& value);
inline bindings::CacheStats convert(const internal::CacheStatsT& value);
inline bindings::ImportStats convert(const internal::ImportStatsT& value);
inline bindings::PageObject convert(const internal::PageObjectT& value);
inline bindings::PageInspection convert(const internal::PageInspectionT& value);
inline bindings::FileInspection convert(const internal::FileInspectionT& value);
inline bindings::ProfileGeneration convert(const internal::ProfileGenerationT& value);
inline bindings::ProfileHistory convert(const internal::ProfileHistoryT& value);
inline bindings::KnownLockbox convert(const internal::KnownLockboxT& value);
inline bindings::AccessSlotLabel convert(const internal::AccessSlotLabelT& value);
inline bindings::StreamChunk convert(const internal::StreamChunkT& value);
inline bindings::RuntimeOptions convert(const internal::RuntimeOptionsT& value);
inline bindings::Variable convert(const internal::VariableT& value);
inline bindings::OwnerInspection convert(const internal::OwnerInspectionT& value);
inline bindings::Contact convert(const internal::ContactT& value);
inline bindings::AgentEntry convert(const internal::AgentEntryT& value);
inline bindings::SleepSupport convert(const internal::SleepSupportT& value);
inline bindings::PlatformStatus convert(const internal::PlatformStatusT& value);
inline bindings::VaultBackupManifest convert(const internal::VaultBackupManifestT& value);
inline bindings::ErrorDetails convert(const internal::ErrorDetailsT& value);

template <typename T, typename U>
std::vector<T> convert_owned_vector(const std::vector<std::unique_ptr<U>>& values) {
  std::vector<T> result; result.reserve(values.size());
  for (const auto& value : values) result.push_back(convert(*value));
  return result;
}

inline bindings::LockboxEntry convert(const internal::LockboxEntryT& value) {
  return {value.path, static_cast<bindings::LockboxEntryKind>(value.kind), value.length, value.permissions};
}
inline bindings::PathMove convert(const internal::PathMoveT& value) {
  return {value.source, value.destination};
}
inline bindings::FormField convert(const internal::FormFieldT& value) {
  return {value.id, value.label, value.kind, value.required};
}
inline bindings::FormDefinition convert(const internal::FormDefinitionT& value) {
  return {value.type_id, value.alias, value.revision, value.name, value.description, convert_owned_vector<bindings::FormField>(value.fields)};
}
inline bindings::FormValue convert(const internal::FormValueT& value) {
  return {value.field_id, value.label, value.kind, value.value, value.secret};
}
inline bindings::FormRecord convert(const internal::FormRecordT& value) {
  return {value.path, value.name, value.type_id, value.definition_alias, value.definition_revision, convert_owned_vector<bindings::FormValue>(value.values)};
}
inline bindings::RecoveryReport convert(const internal::RecoveryReportT& value) {
  return {convert_owned_vector<bindings::LockboxEntry>(value.intact_files), value.intact_file_count, value.partial_files, value.corrupt_records, value.toc_recovered, value.variables_recovered, value.variable_count, value.forms_recovered, value.form_definition_count, value.form_record_count};
}
inline bindings::KeySlot convert(const internal::KeySlotT& value) {
  return {value.id, value.protection, value.algorithm};
}
inline bindings::CacheStats convert(const internal::CacheStatsT& value) {
  return {value.limit_bytes, value.used_bytes, value.entries, value.hits, value.misses};
}
inline bindings::ImportStats convert(const internal::ImportStatsT& value) {
  return {value.host_stat_nanos, value.host_read_nanos, value.frame_prepare_nanos, value.page_write_nanos};
}
inline bindings::PageObject convert(const internal::PageObjectT& value) {
  return {value.id, value.kind, value.payload_len};
}
inline bindings::PageInspection convert(const internal::PageInspectionT& value) {
  return {value.offset, value.page_id, value.sequence, value.page_size, value.encrypted_body_len, value.unused_bytes, value.object_count, convert_owned_vector<bindings::PageObject>(value.objects)};
}
inline bindings::FileInspection convert(const internal::FileInspectionT& value) {
  return {value.lockbox_id, value.header_readable, value.key_directory_generation, value.key_directory_copy_count, value.owner_signed, convert_owned_vector<bindings::KeySlot>(value.key_slots)};
}
inline bindings::ProfileGeneration convert(const internal::ProfileGenerationT& value) {
  return {value.index, value.status, value.contact_fingerprint, value.created_at_unix_ms, value.retired_at_unix_ms, value.has_retired_at};
}
inline bindings::ProfileHistory convert(const internal::ProfileHistoryT& value) {
  return {value.name, value.active_generation, convert_owned_vector<bindings::ProfileGeneration>(value.generations)};
}
inline bindings::KnownLockbox convert(const internal::KnownLockboxT& value) {
  return {value.lockbox_id, value.path, value.last_seen_unix_ms};
}
inline bindings::AccessSlotLabel convert(const internal::AccessSlotLabelT& value) {
  return {value.lockbox_id, value.slot_id, value.name, value.updated_at_unix_ms};
}
inline bindings::StreamChunk convert(const internal::StreamChunkT& value) {
  return {value.path, value.file_offset, value.length, value.physical_offset, value.sparse, value.data};
}
inline bindings::RuntimeOptions convert(const internal::RuntimeOptionsT& value) {
  return {value.workload_profile, value.worker_policy};
}
inline bindings::Variable convert(const internal::VariableT& value) {
  return {value.name, value.sensitivity};
}
inline bindings::OwnerInspection convert(const internal::OwnerInspectionT& value) {
  return {value.signed_, value.fingerprint, value.has_fingerprint};
}
inline bindings::Contact convert(const internal::ContactT& value) {
  return {value.name, value.key};
}
inline bindings::AgentEntry convert(const internal::AgentEntryT& value) {
  return {value.id, value.path};
}
inline bindings::SleepSupport convert(const internal::SleepSupportT& value) {
  return {value.suspend_notifications, value.sleep_inhibition, value.supported};
}
inline bindings::PlatformStatus convert(const internal::PlatformStatusT& value) {
  return {value.supported, value.disabled, value.scope, value.backend, value.item};
}
inline bindings::VaultBackupManifest convert(const internal::VaultBackupManifestT& value) {
  return {value.format_version, value.created_at_unix_ms, value.vault_file_name, value.vault_size, value.vault_sha256};
}
inline bindings::ErrorDetails convert(const internal::ErrorDetailsT& value) {
  return {value.category, value.artifact_kind, value.found_version, value.supported_version, value.message, value.guidance};
}

template <typename T> T decode(std::span<const std::uint8_t> bytes);

template <> bindings::LockboxEntry decode<bindings::LockboxEntry>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::LockboxEntry>(bytes.data())->UnPack());
}
template <> bindings::PathMove decode<bindings::PathMove>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::PathMove>(bytes.data())->UnPack());
}
template <> bindings::FormField decode<bindings::FormField>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::FormField>(bytes.data())->UnPack());
}
template <> bindings::FormDefinition decode<bindings::FormDefinition>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::FormDefinition>(bytes.data())->UnPack());
}
template <> bindings::FormValue decode<bindings::FormValue>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::FormValue>(bytes.data())->UnPack());
}
template <> bindings::FormRecord decode<bindings::FormRecord>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::FormRecord>(bytes.data())->UnPack());
}
template <> bindings::RecoveryReport decode<bindings::RecoveryReport>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::RecoveryReport>(bytes.data())->UnPack());
}
template <> bindings::KeySlot decode<bindings::KeySlot>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::KeySlot>(bytes.data())->UnPack());
}
template <> bindings::CacheStats decode<bindings::CacheStats>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::CacheStats>(bytes.data())->UnPack());
}
template <> bindings::ImportStats decode<bindings::ImportStats>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::ImportStats>(bytes.data())->UnPack());
}
template <> bindings::PageObject decode<bindings::PageObject>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::PageObject>(bytes.data())->UnPack());
}
template <> bindings::PageInspection decode<bindings::PageInspection>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::PageInspection>(bytes.data())->UnPack());
}
template <> bindings::FileInspection decode<bindings::FileInspection>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::FileInspection>(bytes.data())->UnPack());
}
template <> bindings::ProfileGeneration decode<bindings::ProfileGeneration>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::ProfileGeneration>(bytes.data())->UnPack());
}
template <> bindings::ProfileHistory decode<bindings::ProfileHistory>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::ProfileHistory>(bytes.data())->UnPack());
}
template <> bindings::KnownLockbox decode<bindings::KnownLockbox>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::KnownLockbox>(bytes.data())->UnPack());
}
template <> bindings::AccessSlotLabel decode<bindings::AccessSlotLabel>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::AccessSlotLabel>(bytes.data())->UnPack());
}
template <> bindings::StreamChunk decode<bindings::StreamChunk>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::StreamChunk>(bytes.data())->UnPack());
}
template <> bindings::RuntimeOptions decode<bindings::RuntimeOptions>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::RuntimeOptions>(bytes.data())->UnPack());
}
template <> bindings::Variable decode<bindings::Variable>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::Variable>(bytes.data())->UnPack());
}
template <> bindings::OwnerInspection decode<bindings::OwnerInspection>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::OwnerInspection>(bytes.data())->UnPack());
}
template <> bindings::Contact decode<bindings::Contact>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::Contact>(bytes.data())->UnPack());
}
template <> bindings::AgentEntry decode<bindings::AgentEntry>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::AgentEntry>(bytes.data())->UnPack());
}
template <> bindings::SleepSupport decode<bindings::SleepSupport>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::SleepSupport>(bytes.data())->UnPack());
}
template <> bindings::PlatformStatus decode<bindings::PlatformStatus>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::PlatformStatus>(bytes.data())->UnPack());
}
template <> bindings::VaultBackupManifest decode<bindings::VaultBackupManifest>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::VaultBackupManifest>(bytes.data())->UnPack());
}
template <> bindings::ErrorDetails decode<bindings::ErrorDetails>(std::span<const std::uint8_t> bytes) {
  return convert(*flatbuffers::GetRoot<internal::ErrorDetails>(bytes.data())->UnPack());
}
template <> bindings::StreamChunkList decode<bindings::StreamChunkList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::StreamChunkList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::StreamChunk>(value->values);
}
template <> bindings::PageInspectionList decode<bindings::PageInspectionList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::PageInspectionList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::PageInspection>(value->values);
}
template <> bindings::LockboxEntryList decode<bindings::LockboxEntryList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::LockboxEntryList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::LockboxEntry>(value->entries);
}
template <> bindings::VariableList decode<bindings::VariableList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::VariableList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::Variable>(value->values);
}
template <> bindings::KeySlotList decode<bindings::KeySlotList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::KeySlotList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::KeySlot>(value->values);
}
template <> bindings::FormDefinitionList decode<bindings::FormDefinitionList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::FormDefinitionList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::FormDefinition>(value->values);
}
template <> bindings::FormRecordList decode<bindings::FormRecordList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::FormRecordList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::FormRecord>(value->values);
}
template <> bindings::ContactList decode<bindings::ContactList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::ContactList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::Contact>(value->values);
}
template <> bindings::KnownLockboxList decode<bindings::KnownLockboxList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::KnownLockboxList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::KnownLockbox>(value->values);
}
template <> bindings::AccessSlotLabelList decode<bindings::AccessSlotLabelList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::AccessSlotLabelList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::AccessSlotLabel>(value->values);
}
template <> bindings::AgentEntryList decode<bindings::AgentEntryList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::AgentEntryList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::AgentEntry>(value->values);
}
template <> bindings::ProfileHistoryList decode<bindings::ProfileHistoryList>(std::span<const std::uint8_t> bytes) {
  auto value = flatbuffers::GetRoot<internal::ProfileHistoryList>(bytes.data())->UnPack();
  return convert_owned_vector<bindings::ProfileHistory>(value->values);
}
template <> bindings::StringList decode<bindings::StringList>(std::span<const std::uint8_t> bytes) { return flatbuffers::GetRoot<internal::StringList>(bytes.data())->UnPack()->values; }
template <> bindings::OptionalLockboxEntry decode<bindings::OptionalLockboxEntry>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalLockboxEntry>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }
template <> bindings::OptionalFormRecord decode<bindings::OptionalFormRecord>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalFormRecord>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }
template <> bindings::OptionalFormValue decode<bindings::OptionalFormValue>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalFormValue>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }
template <> bindings::OptionalString decode<bindings::OptionalString>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalString>(bytes.data())->UnPack(); return value->present ? std::optional(value->value) : std::nullopt; }

std::string encode_moves(const bindings::PathMoveList& values) {
  internal::PathMoveListT transport;
  for (const auto& value : values) { auto item = std::make_unique<internal::PathMoveT>(); item->source = value.source; item->destination = value.destination; transport.values.push_back(std::move(item)); }
  flatbuffers::FlatBufferBuilder builder; builder.Finish(internal::PathMoveList::Pack(builder, &transport));
  return {reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize()};
}
std::string encode_fields(const bindings::FormFieldList& values) {
  internal::FormFieldListT transport;
  for (const auto& value : values) { auto item = std::make_unique<internal::FormFieldT>(); item->id = value.id; item->label = value.label; item->kind = value.kind; item->required = value.required; transport.values.push_back(std::move(item)); }
  flatbuffers::FlatBufferBuilder builder; builder.Finish(internal::FormFieldList::Pack(builder, &transport));
  return {reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize()};
}

}  // namespace revault::detail
