#pragma once

#include <algorithm>
#include <cstdint>
#include <string>
#include <string_view>
#include <stdexcept>
#include <optional>
#include <functional>
#include <span>
#include <vector>

#include <revault_api.h>
#include <revault_bindings.pb.h>

namespace revault {

inline void require_compatible_abi() {
  if (api_abi_version() != 2)
    throw std::runtime_error("revault-api native ABI mismatch; expected 2");
}

class VaultDirectory;
class ReadOnlyVaultDirectory;
class Agent;
class LocalVault;
class WrappedContactKey;

class Buffer {
 public:
  Buffer() = default;
  explicit Buffer(RevaultBuffer value) : value_(value) {}
  Buffer(const Buffer&) = delete;
  Buffer& operator=(const Buffer&) = delete;
  Buffer(Buffer&& other) noexcept : value_(other.value_) { other.value_ = {}; }
  Buffer& operator=(Buffer&& other) noexcept {
    if (this != &other) { reset(); value_ = other.value_; other.value_ = {}; }
    return *this;
  }
  ~Buffer() { reset(); }
  const std::uint8_t* data() const { return value_.ptr; }
  std::size_t size() const { return value_.len; }
  std::vector<std::uint8_t> bytes() const { return {data(), data() + size()}; }
  explicit operator bool() const { return value_.ptr != nullptr; }
 private:
  void reset() { if (value_.ptr) buffer_free(value_); value_ = {}; }
  RevaultBuffer value_{};
};

inline std::vector<std::uint8_t> take_bytes(RevaultBuffer result) {
  if (!result.ptr) throw std::runtime_error(buffer_last_error());
  Buffer owned(result);
  return owned.bytes();
}

inline std::string take_string(RevaultBuffer result) {
  const auto bytes = take_bytes(result);
  return {reinterpret_cast<const char*>(bytes.data()), bytes.size()};
}

inline bool with_secret_handle(
    void* handle,
    const std::function<void(std::span<const std::uint8_t>)>& callback) {
  if (!handle) return false;
  try {
    std::size_t length{};
    if (!secret_len(handle, &length)) throw std::runtime_error(buffer_last_error());
    std::vector<std::uint8_t> bytes(length);
    if (!secret_copy(handle, bytes.data(), bytes.size()))
      throw std::runtime_error(buffer_last_error());
    try { callback(bytes); }
    catch (...) {
      std::fill(bytes.begin(), bytes.end(), 0);
      throw;
    }
    std::fill(bytes.begin(), bytes.end(), 0);
    secret_free(handle);
    return true;
  } catch (...) {
    secret_free(handle);
    throw;
  }
}

template <typename Message>
Message take_message(RevaultBuffer result) {
  if (!result.ptr) throw std::runtime_error(buffer_last_error());
  Buffer owned(result);
  if (owned.size() < 12 || std::string_view(
          reinterpret_cast<const char*>(owned.data()), 4) != "LBWF")
    throw std::runtime_error("invalid reVault binary frame");
  const auto* bytes = owned.data();
  const std::size_t length = (static_cast<std::size_t>(bytes[8]) << 24) |
                             (static_cast<std::size_t>(bytes[9]) << 16) |
                             (static_cast<std::size_t>(bytes[10]) << 8) |
                             static_cast<std::size_t>(bytes[11]);
  if (length + 12 != owned.size())
    throw std::runtime_error("invalid reVault binary frame length");
  Message message;
  if (!message.ParseFromArray(bytes + 12, static_cast<int>(length)))
    throw std::runtime_error("invalid reVault protobuf payload");
  return message;
}

inline bindings::ErrorDetails last_error_details() {
  return take_message<bindings::ErrorDetails>(buffer_last_error_details());
}

inline std::string encode_moves(
    const std::vector<std::pair<std::string, std::string>>& moves) {
  bindings::PathMoveList message;
  for (const auto& [source, destination] : moves) {
    auto* item = message.add_values();
    item->set_source(source); item->set_destination(destination);
  }
  return message.SerializeAsString();
}

class ContactPublicKey {
 public:
  explicit ContactPublicKey(const std::vector<std::uint8_t>& bytes)
      : handle_(key_contact_public_from_bytes(bytes.data(), bytes.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  ContactPublicKey(const ContactPublicKey&) = delete;
  ContactPublicKey& operator=(const ContactPublicKey&) = delete;
  ContactPublicKey(ContactPublicKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~ContactPublicKey() { if (handle_) key_contact_public_free(handle_); }
  static ContactPublicKey import(const std::vector<std::uint8_t>& bytes) {
    return ContactPublicKey(vault_key_import_public(bytes.data(), bytes.size()));
  }
  std::vector<std::uint8_t> export_as(const std::string& format) const {
    return take_bytes(vault_key_export_public(handle_, format.data(), format.size()));
  }
  std::vector<std::uint8_t> fingerprint() const {
    return take_bytes(vault_key_fingerprint(handle_));
  }
  WrappedContactKey encrypt(const std::vector<std::uint8_t>& content_key) const;
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit ContactPublicKey(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

class WrappedContactKey {
 public:
  WrappedContactKey(const WrappedContactKey&) = delete;
  WrappedContactKey& operator=(const WrappedContactKey&) = delete;
  WrappedContactKey(WrappedContactKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~WrappedContactKey() { if (handle_) key_contact_wrapped_free(handle_); }
  std::vector<std::uint8_t> public_bytes() const { return take_bytes(key_contact_wrapped_public(handle_)); }
  std::vector<std::uint8_t> ciphertext() const { return take_bytes(key_contact_wrapped_ciphertext(handle_)); }
  std::vector<std::uint8_t> encrypted_bytes() const { return take_bytes(key_contact_wrapped_encrypted(handle_)); }
  void* native_handle() const { return handle_; }
 private:
  friend class ContactKeyPair;
  friend class ContactPublicKey;
  explicit WrappedContactKey(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

inline WrappedContactKey ContactPublicKey::encrypt(
    const std::vector<std::uint8_t>& content_key) const {
  return WrappedContactKey(key_contact_encrypt(
      handle_, content_key.data(), content_key.size()));
}

class ContactKeyPair {
 public:
  ContactKeyPair() : handle_(key_contact_generate()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static ContactKeyPair from_private_record(const std::vector<std::uint8_t>& bytes) {
    return ContactKeyPair(key_contact_from_private(bytes.data(), bytes.size()));
  }
  static ContactKeyPair import(const std::vector<std::uint8_t>& bytes) {
    return ContactKeyPair(vault_key_import_private(bytes.data(), bytes.size()));
  }
  ContactKeyPair(const ContactKeyPair&) = delete;
  ContactKeyPair& operator=(const ContactKeyPair&) = delete;
  ContactKeyPair(ContactKeyPair&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~ContactKeyPair() { if (handle_) key_contact_free(handle_); }
  std::vector<std::uint8_t> public_bytes() const { return take_bytes(key_contact_public(handle_)); }
  std::vector<std::uint8_t> private_record() const { return take_bytes(key_contact_private(handle_)); }
  ContactPublicKey public_key() const { return ContactPublicKey(public_bytes()); }
  std::vector<std::uint8_t> export_as(const std::string& format) const {
    return take_bytes(vault_key_export_private(handle_, format.data(), format.size()));
  }
  std::vector<std::uint8_t> decrypt(const WrappedContactKey& wrapped) const {
    return take_bytes(key_contact_decrypt(handle_, wrapped.native_handle()));
  }
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit ContactKeyPair(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

class SigningPublicKey {
 public:
  explicit SigningPublicKey(const std::vector<std::uint8_t>& bytes)
      : handle_(key_signing_public_from_bytes(bytes.data(), bytes.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  SigningPublicKey(const SigningPublicKey&) = delete;
  SigningPublicKey& operator=(const SigningPublicKey&) = delete;
  SigningPublicKey(SigningPublicKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~SigningPublicKey() { if (handle_) key_signing_public_free(handle_); }
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit SigningPublicKey(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

class SigningKeyPair {
 public:
  SigningKeyPair() : handle_(key_signing_generate()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static SigningKeyPair from_private_record(const std::vector<std::uint8_t>& bytes) {
    return SigningKeyPair(key_signing_from_private(bytes.data(), bytes.size()));
  }
  SigningKeyPair(const SigningKeyPair&) = delete;
  SigningKeyPair& operator=(const SigningKeyPair&) = delete;
  SigningKeyPair(SigningKeyPair&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~SigningKeyPair() { if (handle_) key_signing_free(handle_); }
  std::vector<std::uint8_t> public_bytes() const { return take_bytes(key_signing_public(handle_)); }
  std::vector<std::uint8_t> private_record() const { return take_bytes(key_signing_private(handle_)); }
  SigningPublicKey public_key() const { return SigningPublicKey(public_bytes()); }
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  friend class Agent;
  explicit SigningKeyPair(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

struct LockboxOptions {
  std::string cache_mode{"bytes"};
  std::uint64_t cache_bytes{64 * 1024 * 1024};
  std::string workload{"interactive"};
  std::string worker{"auto"};
  std::size_t jobs{0};
};

class Lockbox {
 public:
  static std::uint16_t format_version() { return lockbox_format_version(); }
  static std::uint16_t probe_format_version(const std::vector<std::uint8_t>& bytes) {
    const auto value = lockbox_probe_format_version(bytes.data(), bytes.size());
    if (!value) throw std::runtime_error(buffer_last_error());
    return value;
  }
  explicit Lockbox(const std::vector<std::uint8_t>& key) : handle_(lockbox_create(key.data(), key.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  Lockbox(const Lockbox&) = delete;
  Lockbox& operator=(const Lockbox&) = delete;
  Lockbox(Lockbox&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  Lockbox& operator=(Lockbox&& other) noexcept {
    if (this != &other) {
      if (handle_) lockbox_free(handle_);
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }
  ~Lockbox() { if (handle_) lockbox_free(handle_); }

  static Lockbox create_with_password(const std::string& password) {
    return adopt(lockbox_create_password(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  static Lockbox create_with_options(const std::vector<std::uint8_t>& key,
                                     const LockboxOptions& options) {
    return adopt(lockbox_create_with_options(
        key.data(), key.size(), options.cache_mode.data(), options.cache_mode.size(),
        options.cache_bytes, options.workload.data(), options.workload.size(),
        options.worker.data(), options.worker.size(), options.jobs));
  }
  static Lockbox create_for_contact(const ContactPublicKey& contact) {
    return adopt(lockbox_create_contact(contact.native_handle()));
  }
  static Lockbox create_signed(const std::vector<std::uint8_t>& key,
                               const SigningKeyPair& signing_key) {
    return adopt(lockbox_create_with_signing_key(
        key.data(), key.size(), signing_key.native_handle()));
  }
  static Lockbox open(const std::vector<std::uint8_t>& archive,
                      const std::vector<std::uint8_t>& key) {
    return adopt(lockbox_open(archive.data(), archive.size(), key.data(), key.size()));
  }
  static Lockbox open_with_password(const std::vector<std::uint8_t>& archive,
                                    const std::string& password) {
    return adopt(lockbox_open_password(
        archive.data(), archive.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  static Lockbox open_with_options(const std::vector<std::uint8_t>& archive,
                                   const std::vector<std::uint8_t>& key,
                                   const LockboxOptions& options) {
    return adopt(lockbox_open_with_options(
        archive.data(), archive.size(), key.data(), key.size(),
        options.cache_mode.data(), options.cache_mode.size(), options.cache_bytes,
        options.workload.data(), options.workload.size(), options.worker.data(),
        options.worker.size(), options.jobs));
  }
  static Lockbox open_with_contact(const std::vector<std::uint8_t>& archive,
                                   const ContactKeyPair& contact) {
    return adopt(lockbox_open_contact(archive.data(), archive.size(), contact.native_handle()));
  }

  void add_file(const std::string& path, const std::vector<std::uint8_t>& data, bool replace = false) {
    if (!lockbox_add_file(handle_, path.data(), path.size(), data.data(), data.size(), replace))
      throw std::runtime_error(buffer_last_error());
  }
  void commit() {
    if (!lockbox_commit(handle_)) throw std::runtime_error(buffer_last_error());
  }
  std::uint64_t storage_length() const { return lockbox_storage_len(handle_); }
  std::vector<std::uint8_t> id() const { return take_bytes(lockbox_id(handle_)); }
  static bindings::FileInspection inspect_file(const std::string& path) {
    return take_message<bindings::FileInspection>(
        lockbox_inspect_file(path.data(), path.size()));
  }
  std::string render_recovery_report(bool verbose = false,
                                     std::size_t max_entries = 100) const {
    return take_string(lockbox_recovery_report_render(handle_, verbose, max_entries));
  }
  static bindings::RecoveryReport scan_path(const std::string& path,
                                            const std::vector<std::uint8_t>& key) {
    return take_message<bindings::RecoveryReport>(lockbox_recovery_scan_path(
        path.data(), path.size(), key.data(), key.size()));
  }
  static bindings::RecoveryReport scan(const std::vector<std::uint8_t>& archive,
                                       const std::vector<std::uint8_t>& key) {
    return take_message<bindings::RecoveryReport>(lockbox_recovery_scan(
        archive.data(), archive.size(), key.data(), key.size()));
  }
  static Lockbox salvage(const std::vector<std::uint8_t>& archive,
                         const std::vector<std::uint8_t>& key,
                         const SigningKeyPair* signing_key = nullptr) {
    return adopt(lockbox_recovery_salvage(
        archive.data(), archive.size(), key.data(), key.size(),
        signing_key ? signing_key->native_handle() : nullptr));
  }
  std::vector<std::uint8_t> get_file(const std::string& path) const {
    auto result = lockbox_get_file(handle_, path.data(), path.size());
    if (!result.ptr) throw std::runtime_error(buffer_last_error());
    std::vector<std::uint8_t> value(result.ptr, result.ptr + result.len);
    buffer_free(result);
    return value;
  }
  void add_file_with_permissions(const std::string& path, const std::vector<std::uint8_t>& data,
                                 std::uint32_t permissions, bool replace = false) {
    if (!lockbox_add_file_with_permissions(handle_, path.data(), path.size(), data.data(), data.size(), permissions, replace))
      throw std::runtime_error(buffer_last_error());
  }
  void create_dir(const std::string& path, bool create_parents = true) {
    if (!lockbox_create_dir(handle_, path.data(), path.size(), create_parents)) throw std::runtime_error(buffer_last_error());
  }
  void create_parent_dirs(const std::string& path) {
    if (!lockbox_create_parent_dirs(handle_, path.data(), path.size())) throw std::runtime_error(buffer_last_error());
  }
  void extract_file(const std::string& source, const std::string& destination,
                    bool replace = false) const {
    if (!lockbox_extract_file(handle_, source.data(), source.size(), destination.data(),
                              destination.size(), replace))
      throw std::runtime_error(buffer_last_error());
  }
  void extract_directory(const std::string& destination, std::uint64_t max_file_bytes,
                         std::uint64_t max_total_bytes, std::size_t max_files,
                         bool restore_symlinks, bool restore_permissions,
                         bool overwrite) const {
    if (!lockbox_extract_directory(handle_, destination.data(), destination.size(),
                                   max_file_bytes, max_total_bytes, max_files,
                                   restore_symlinks, restore_permissions, overwrite))
      throw std::runtime_error(buffer_last_error());
  }
  void remove_dir(const std::string& path, bool recursive = false) {
    if (!lockbox_remove_dir(handle_, path.data(), path.size(), recursive)) throw std::runtime_error(buffer_last_error());
  }
  void remove(const std::string& path) {
    if (!lockbox_delete(handle_, path.data(), path.size())) throw std::runtime_error(buffer_last_error());
  }
  void rename(const std::string& from, const std::string& to) {
    if (!lockbox_rename(handle_, from.data(), from.size(), to.data(), to.size())) throw std::runtime_error(buffer_last_error());
  }
  void add_symlink(const std::string& path, const std::string& target, bool replace = false) {
    if (!lockbox_add_symlink(handle_, path.data(), path.size(), target.data(), target.size(), replace)) throw std::runtime_error(buffer_last_error());
  }
  Buffer symlink_target(const std::string& path) const {
    return checked(lockbox_get_symlink_target(handle_, path.data(), path.size()));
  }
  Buffer read_range(const std::string& path, std::uint64_t offset, std::uint64_t length) const {
    return checked(lockbox_read_range(handle_, path.data(), path.size(), offset, length));
  }
  bindings::LockboxEntryList list(const std::string& path = "/", bool recursive = false) const {
    return decoded<bindings::LockboxEntryList>(lockbox_list(handle_, path.data(), path.size(), recursive));
  }
  bindings::OptionalLockboxEntry stat(const std::string& path) const {
    return decoded<bindings::OptionalLockboxEntry>(lockbox_stat(handle_, path.data(), path.size()));
  }
  bindings::LockboxEntryList list_with_options(
      const std::string& path, const std::string& glob, bool recursive,
      bool include_files, bool include_symlinks, bool include_directories,
      std::size_t limit) const {
    return decoded<bindings::LockboxEntryList>(lockbox_list_with_options(
        handle_, path.data(), path.size(), glob.data(), glob.size(), recursive,
        include_files, include_symlinks, include_directories, limit));
  }
  bool exists(const std::string& path) const { return lockbox_exists(handle_, path.data(), path.size()); }
  bool is_dir(const std::string& path) const { return lockbox_is_dir(handle_, path.data(), path.size()); }
  std::uint32_t permissions(const std::string& path) const { return lockbox_permissions(handle_, path.data(), path.size()); }
  void set_permissions(const std::string& path, std::uint32_t value) {
    if (!lockbox_set_permissions(handle_, path.data(), path.size(), value)) throw std::runtime_error(buffer_last_error());
  }
  void set_variable(const std::string& name, const std::string& value) {
    if (!lockbox_set_variable(handle_, name.data(), name.size(), value.data(), value.size())) throw std::runtime_error(buffer_last_error());
  }
  void set_secret_variable(const std::string& name, std::span<const std::uint8_t> value) {
    if (!lockbox_set_secret_variable(handle_, name.data(), name.size(), value.data(), value.size())) throw std::runtime_error(buffer_last_error());
  }
  std::optional<std::string> get_variable(const std::string& name) const {
    auto value = decoded<bindings::OptionalString>(lockbox_get_variable(handle_, name.data(), name.size()));
    return value.present() ? std::optional<std::string>(value.value()) : std::nullopt;
  }
  bool with_secret_variable(const std::string& name,
      const std::function<void(std::span<const std::uint8_t>)>& callback) const {
    void* secret{};
    if (!lockbox_get_secret_variable(handle_, name.data(), name.size(), &secret))
      throw std::runtime_error(buffer_last_error());
    return with_secret_handle(secret, callback);
  }
  void delete_variable(const std::string& name) {
    if (!lockbox_delete_variable(handle_, name.data(), name.size())) throw std::runtime_error(buffer_last_error());
  }
  void move_variables(const std::vector<std::pair<std::string, std::string>>& moves) {
    const auto encoded = encode_moves(moves);
    if (!lockbox_move_variables(handle_, reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()))
      throw std::runtime_error(buffer_last_error());
  }
  bindings::VariableList list_variables() const { return decoded<bindings::VariableList>(lockbox_list_variables(handle_)); }
  bindings::OptionalString variable_sensitivity(const std::string& name) const {
    return decoded<bindings::OptionalString>(
        lockbox_variable_sensitivity(handle_, name.data(), name.size()));
  }
  void reset_import_stats() const {
    if (!lockbox_reset_import_stats(handle_)) throw std::runtime_error(buffer_last_error());
  }
  bindings::CacheStats cache_stats() const { return decoded<bindings::CacheStats>(lockbox_cache_stats(handle_)); }
  bindings::ImportStats import_stats() const { return decoded<bindings::ImportStats>(lockbox_import_stats(handle_)); }
  bindings::KeySlotList list_key_slots() const { return decoded<bindings::KeySlotList>(lockbox_list_key_slots(handle_)); }
  std::uint64_t add_password(const std::string& password) {
    auto id = lockbox_add_password(handle_, reinterpret_cast<const std::uint8_t*>(password.data()), password.size());
    if (id == UINT64_MAX) throw std::runtime_error(buffer_last_error());
    return id;
  }
  std::uint64_t add_contact(const ContactPublicKey& contact, const std::string& name) {
    auto id = lockbox_add_contact(handle_, contact.native_handle(), name.data(), name.size());
    if (id == UINT64_MAX) throw std::runtime_error(buffer_last_error());
    return id;
  }
  void delete_key(std::uint64_t id) {
    if (!lockbox_delete_key(handle_, id)) throw std::runtime_error(buffer_last_error());
  }
  bindings::OwnerInspection owner_inspection() const { return decoded<bindings::OwnerInspection>(lockbox_owner_inspection(handle_)); }
  void set_owner_signing_key(const SigningKeyPair& key) {
    if (!lockbox_set_owner_signing_key(handle_, key.native_handle()))
      throw std::runtime_error(buffer_last_error());
  }
  bindings::StreamChunkList stream_content(bool physical = false) const { return decoded<bindings::StreamChunkList>(lockbox_stream_content(handle_, physical)); }
  bindings::PageInspectionList page_inspection() const { return decoded<bindings::PageInspectionList>(lockbox_page_inspection(handle_)); }
  bindings::RecoveryReport recovery_report() const { return decoded<bindings::RecoveryReport>(lockbox_recovery_report(handle_)); }
  bindings::RuntimeOptions runtime_options() const { return decoded<bindings::RuntimeOptions>(lockbox_runtime_options(handle_)); }
  Buffer to_bytes() const { return checked(lockbox_to_bytes(handle_)); }

  bindings::FormDefinition define_form(const std::string& alias,
                                       const std::string& name,
                                       const std::string& description,
                                       const bindings::FormFieldList& fields) {
    const std::string encoded = fields.SerializeAsString();
    return decoded<bindings::FormDefinition>(lockbox_define_form(
        handle_, alias.data(), alias.size(), name.data(), name.size(),
        description.data(), description.size(),
        reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()));
  }
  bindings::FormDefinitionList list_form_definitions() const {
    return decoded<bindings::FormDefinitionList>(lockbox_list_form_definitions(handle_));
  }
  bindings::FormDefinition resolve_form(const std::string& reference) const {
    return decoded<bindings::FormDefinition>(
        lockbox_resolve_form(handle_, reference.data(), reference.size()));
  }
  bindings::FormDefinitionList list_form_revisions(const std::string& type_id) const {
    return decoded<bindings::FormDefinitionList>(
        lockbox_list_form_revisions(handle_, type_id.data(), type_id.size()));
  }
  bindings::FormRecord create_form_record(const std::string& path,
                                          const std::string& type_reference,
                                          const std::string& name) {
    return decoded<bindings::FormRecord>(lockbox_create_form_record(
        handle_, path.data(), path.size(), type_reference.data(), type_reference.size(),
        name.data(), name.size()));
  }
  void set_form_field(const std::string& path, const std::string& field,
                      const std::string& value) {
    if (!lockbox_set_form_field(handle_, path.data(), path.size(), field.data(), field.size(),
                                value.data(), value.size()))
      throw std::runtime_error(buffer_last_error());
  }
  void set_secret_form_field(const std::string& path, const std::string& field,
                             std::span<const std::uint8_t> value) {
    if (!lockbox_set_secret_form_field(handle_, path.data(), path.size(), field.data(), field.size(),
                                       value.data(), value.size()))
      throw std::runtime_error(buffer_last_error());
  }
  bindings::FormRecordList list_form_records() const {
    return decoded<bindings::FormRecordList>(lockbox_list_form_records(handle_));
  }
  bindings::OptionalFormRecord get_form_record(const std::string& path) const {
    return decoded<bindings::OptionalFormRecord>(
        lockbox_get_form_record(handle_, path.data(), path.size()));
  }
  void delete_form_record(const std::string& path) {
    if (!lockbox_delete_form_record(handle_, path.data(), path.size()))
      throw std::runtime_error(buffer_last_error());
  }
  void move_form_records(const std::vector<std::pair<std::string, std::string>>& moves) {
    const auto encoded = encode_moves(moves);
    if (!lockbox_move_form_records(handle_, reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()))
      throw std::runtime_error(buffer_last_error());
  }
  bindings::OptionalFormValue get_form_field(const std::string& path,
                                             const std::string& field) const {
    return decoded<bindings::OptionalFormValue>(lockbox_get_form_field(
        handle_, path.data(), path.size(), field.data(), field.size()));
  }
  bool with_secret_form_field(const std::string& path, const std::string& field,
      const std::function<void(std::span<const std::uint8_t>)>& callback) const {
    void* secret{};
    if (!lockbox_get_secret_form_field(handle_, path.data(), path.size(), field.data(), field.size(), &secret))
      throw std::runtime_error(buffer_last_error());
    return with_secret_handle(secret, callback);
  }
  void set_workload_profile(const std::string& profile) {
    if (!lockbox_set_workload_profile(handle_, profile.data(), profile.size()))
      throw std::runtime_error(buffer_last_error());
  }
  void set_worker_policy(const std::string& mode, std::size_t jobs = 0) {
    if (!lockbox_set_worker_policy(handle_, mode.data(), mode.size(), jobs))
      throw std::runtime_error(buffer_last_error());
  }
  void* native_handle() const { return handle_; }

 private:
  friend class LocalVault;
  explicit Lockbox(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static Lockbox adopt(void* handle) { return Lockbox(handle); }
  static Buffer checked(RevaultBuffer result) {
    if (!result.ptr) throw std::runtime_error(buffer_last_error());
    return Buffer(result);
  }
  template <typename Message>
  static Message decoded(RevaultBuffer result) {
    Buffer owned = checked(result);
    if (owned.size() < 12 || std::string_view(
            reinterpret_cast<const char*>(owned.data()), 4) != "LBWF")
      throw std::runtime_error("invalid reVault binary frame");
    const auto* bytes = owned.data();
    const std::size_t length = (static_cast<std::size_t>(bytes[8]) << 24) |
                               (static_cast<std::size_t>(bytes[9]) << 16) |
                               (static_cast<std::size_t>(bytes[10]) << 8) |
                               static_cast<std::size_t>(bytes[11]);
    if (length + 12 != owned.size())
      throw std::runtime_error("invalid reVault binary frame length");
    Message message;
    if (!message.ParseFromArray(bytes + 12, static_cast<int>(length)))
      throw std::runtime_error("invalid reVault protobuf payload");
    return message;
  }
  void* handle_;
};

class VaultDirectory {
 public:
  static std::uint32_t current_structure_version() { return vault_structure_version_current(); }
  static std::uint32_t probe_structure_version(const std::string& root, const std::string& password) {
    const auto value = vault_directory_probe_structure_version(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()), password.size());
    if (!value) throw std::runtime_error(buffer_last_error());
    return value;
  }
  static VaultDirectory open(const std::string& root, const std::string& password) {
    return adopt(vault_directory_open(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  static VaultDirectory open_or_create(const std::string& root,
                                       const std::string& password) {
    return adopt(vault_directory_open_or_create(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  static VaultDirectory replace(const std::string& root, const std::string& password) {
    return adopt(vault_directory_replace(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  static VaultDirectory open_or_create_default(const std::string& password) {
    return adopt(vault_directory_open_or_create_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  static VaultDirectory replace_default(const std::string& password) {
    return adopt(vault_directory_replace_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  static void change_password(const std::string& root, const std::string& old_password,
                              const std::string& new_password) {
    if (!vault_directory_change_password(
            root.data(), root.size(),
            reinterpret_cast<const std::uint8_t*>(old_password.data()), old_password.size(),
            reinterpret_cast<const std::uint8_t*>(new_password.data()), new_password.size()))
      throw std::runtime_error(buffer_last_error());
  }
  static void change_default_password(const std::string& old_password,
                                      const std::string& new_password) {
    if (!vault_directory_change_default_password(
            reinterpret_cast<const std::uint8_t*>(old_password.data()), old_password.size(),
            reinterpret_cast<const std::uint8_t*>(new_password.data()), new_password.size()))
      throw std::runtime_error(buffer_last_error());
  }
  static std::string default_directory() { return take_string(vault_default_directory()); }
  static std::string default_path() { return take_string(vault_default_path()); }
  static bindings::VaultBackupManifest backup_default(const std::string& path,
                                                       bool overwrite = false) {
    return take_message<bindings::VaultBackupManifest>(
        vault_backup_default(path.data(), path.size(), overwrite));
  }
  static bindings::VaultBackupManifest restore_default(const std::string& path,
                                                        bool overwrite = false) {
    return take_message<bindings::VaultBackupManifest>(
        vault_restore_default(path.data(), path.size(), overwrite));
  }

  VaultDirectory(const VaultDirectory&) = delete;
  VaultDirectory& operator=(const VaultDirectory&) = delete;
  VaultDirectory(VaultDirectory&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  VaultDirectory& operator=(VaultDirectory&& other) noexcept {
    if (this != &other) {
      if (handle_) vault_directory_free(handle_);
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }
  ~VaultDirectory() { if (handle_) vault_directory_free(handle_); }

  std::string root() const { return take_string(vault_directory_root(handle_)); }
  std::uint32_t structure_version() const { return vault_directory_structure_version(handle_); }
  bindings::StringList list_private_keys() const {
    return take_message<bindings::StringList>(vault_directory_list_private_keys(handle_));
  }
  bindings::StringList list_private_key_names() const {
    return take_message<bindings::StringList>(vault_directory_list_private_key_names(handle_));
  }
  bindings::StringList list_contact_names() const {
    return take_message<bindings::StringList>(vault_directory_list_contact_names(handle_));
  }
  bindings::StringList list_form_aliases() const {
    return take_message<bindings::StringList>(vault_directory_list_form_aliases(handle_));
  }
  bool private_key_exists(const std::string& name) const {
    return vault_directory_private_key_exists(handle_, name.data(), name.size());
  }
  void delete_private_key(const std::string& name) const {
    checked(vault_directory_delete_private_key(handle_, name.data(), name.size()));
  }
  void store_private_key(const std::string& name, const ContactKeyPair& key) const {
    checked(vault_directory_store_private_key(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  ContactKeyPair load_private_key(const std::string& name) const {
    return ContactKeyPair(vault_directory_load_private_key(handle_, name.data(), name.size()));
  }
  ContactKeyPair load_private_key_generation(const std::string& name,
                                             std::uint16_t index) const {
    return ContactKeyPair(vault_directory_load_private_key_generation(
        handle_, name.data(), name.size(), index));
  }
  void store_contact(const std::string& name, const ContactPublicKey& key) const {
    checked(vault_directory_store_contact(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  ContactPublicKey load_contact(const std::string& name) const {
    return ContactPublicKey(vault_directory_load_contact(handle_, name.data(), name.size()));
  }
  bool contact_exists(const std::string& name) const {
    return vault_directory_contact_exists(handle_, name.data(), name.size());
  }
  void delete_contact(const std::string& name) const {
    checked(vault_directory_delete_contact(handle_, name.data(), name.size()));
  }
  bindings::ContactList list_contacts() const {
    return take_message<bindings::ContactList>(vault_directory_list_contacts(handle_));
  }
  void store_profile_email(const std::string& name, const std::string& email) const {
    checked(vault_directory_store_profile_email(
        handle_, name.data(), name.size(), email.data(), email.size()));
  }
  bindings::OptionalString profile_email(const std::string& name) const {
    return take_message<bindings::OptionalString>(
        vault_directory_profile_email(handle_, name.data(), name.size()));
  }
  void store_backup(const std::vector<std::uint8_t>& id,
                    const std::vector<std::uint8_t>& bytes) const {
    checked(vault_directory_store_backup(
        handle_, id.data(), id.size(), bytes.data(), bytes.size()));
  }
  std::vector<std::uint8_t> load_backup(const std::vector<std::uint8_t>& id) const {
    return take_bytes(vault_directory_load_backup(handle_, id.data(), id.size()));
  }
  std::uint64_t backup_count() const { return vault_directory_backup_count(handle_); }
  void restore_private_key(const std::string& name, const ContactKeyPair& key,
                           const SigningKeyPair& signing_key, bool overwrite = false) const {
    checked(vault_directory_restore_private_key(
        handle_, name.data(), name.size(), key.native_handle(),
        signing_key.native_handle(), overwrite));
  }
  SigningKeyPair load_owner_signing_key(const std::string& name) const {
    return SigningKeyPair(vault_directory_load_owner_signing_key(
        handle_, name.data(), name.size()));
  }
  SigningKeyPair load_owner_signing_key_generation(const std::string& name,
                                                   std::uint16_t index) const {
    return SigningKeyPair(vault_directory_load_owner_signing_key_generation(
        handle_, name.data(), name.size(), index));
  }
  void store_contact_signing_key(const std::string& name,
                                 const SigningPublicKey& key) const {
    checked(vault_directory_store_contact_signing_key(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  SigningPublicKey load_contact_signing_key(const std::string& name) const {
    return SigningPublicKey(vault_directory_load_contact_signing_key(
        handle_, name.data(), name.size()));
  }
  bindings::ProfileHistory list_profile_generations(const std::string& name) const {
    return take_message<bindings::ProfileHistory>(
        vault_directory_list_profile_generations(handle_, name.data(), name.size()));
  }
  bindings::ProfileHistory rotate_private_key(const std::string& name) const {
    return take_message<bindings::ProfileHistory>(
        vault_directory_rotate_private_key(handle_, name.data(), name.size()));
  }
  void remember_lockbox(const std::vector<std::uint8_t>& id,
                        const std::string& path) const {
    checked(vault_directory_remember_lockbox(
        handle_, id.data(), id.size(), path.data(), path.size()));
  }
  bindings::KnownLockboxList list_known_lockboxes() const {
    return take_message<bindings::KnownLockboxList>(
        vault_directory_list_known_lockboxes(handle_));
  }
  void forget_lockbox(const std::string& path) const {
    checked(vault_directory_forget_lockbox(handle_, path.data(), path.size()));
  }
  void remember_access_slot_label(const std::vector<std::uint8_t>& id,
                             std::uint64_t slot_id, const std::string& name) const {
    checked(vault_directory_remember_access_slot_label(
        handle_, id.data(), id.size(), slot_id, name.data(), name.size()));
  }
  bindings::AccessSlotLabelList list_access_slot_labels(
      const std::vector<std::uint8_t>& id) const {
    return take_message<bindings::AccessSlotLabelList>(
        vault_directory_list_access_slot_labels(handle_, id.data(), id.size()));
  }
  bindings::AccessSlotLabelList find_access_slot_labels(
      const std::vector<std::uint8_t>& id, const std::string& name) const {
    return take_message<bindings::AccessSlotLabelList>(vault_directory_find_access_slot_labels(
        handle_, id.data(), id.size(), name.data(), name.size()));
  }
  void forget_access_slot_label(const std::vector<std::uint8_t>& id,
                           std::uint64_t slot_id) const {
    checked(vault_directory_forget_access_slot_label(handle_, id.data(), id.size(), slot_id));
  }
  bindings::FormDefinition define_form(const std::string& alias,
                                       const std::string& name,
                                       const std::string& description,
                                       const bindings::FormFieldList& fields) const {
    const std::string encoded = fields.SerializeAsString();
    return take_message<bindings::FormDefinition>(vault_directory_define_form(
        handle_, alias.data(), alias.size(), name.data(), name.size(),
        description.data(), description.size(),
        reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()));
  }
  bindings::FormDefinition resolve_form(const std::string& reference) const {
    return take_message<bindings::FormDefinition>(
        vault_directory_resolve_form(handle_, reference.data(), reference.size()));
  }
  bindings::FormDefinitionList list_forms() const {
    return take_message<bindings::FormDefinitionList>(vault_directory_list_forms(handle_));
  }
  bindings::FormDefinitionList list_form_revisions(const std::string& type_id) const {
    return take_message<bindings::FormDefinitionList>(
        vault_directory_list_form_revisions(handle_, type_id.data(), type_id.size()));
  }
  std::size_t seed_forms() const { return vault_directory_seed_forms(handle_); }
  void remember_password(const std::vector<std::uint8_t>& id,
                         const std::string& password) const {
    checked(vault_directory_remember_password(
        handle_, id.data(), id.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  std::vector<std::uint8_t> remembered_password(
      const std::vector<std::uint8_t>& id) const {
    return take_bytes(vault_directory_remembered_password(handle_, id.data(), id.size()));
  }
  void* native_handle() const { return handle_; }

 private:
  explicit VaultDirectory(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static VaultDirectory adopt(void* handle) { return VaultDirectory(handle); }
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

class ReadOnlyVaultDirectory {
 public:
  ReadOnlyVaultDirectory(const std::string& root, const std::string& password)
      : handle_(vault_read_only_open(root.data(), root.size(),
          reinterpret_cast<const std::uint8_t*>(password.data()), password.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static ReadOnlyVaultDirectory open_default(const std::string& password) {
    return ReadOnlyVaultDirectory(vault_read_only_open_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  ReadOnlyVaultDirectory(const ReadOnlyVaultDirectory&) = delete;
  ReadOnlyVaultDirectory(ReadOnlyVaultDirectory&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~ReadOnlyVaultDirectory() { if (handle_) vault_read_only_free(handle_); }
  bindings::StringList list_profile_names() const { return take_message<bindings::StringList>(vault_read_only_list_profile_names(handle_)); }
  bindings::StringList list_contact_names() const { return take_message<bindings::StringList>(vault_read_only_list_contact_names(handle_)); }
  bindings::StringList list_form_aliases() const { return take_message<bindings::StringList>(vault_read_only_list_form_aliases(handle_)); }
  bindings::KnownLockboxList list_known_lockboxes() const { return take_message<bindings::KnownLockboxList>(vault_read_only_list_known_lockboxes(handle_)); }
 private:
  explicit ReadOnlyVaultDirectory(void* handle) : handle_(handle) { if (!handle_) throw std::runtime_error(buffer_last_error()); }
  void* handle_{};
};

class KeyFormat {
 public:
  static std::vector<std::uint8_t> fingerprint(const ContactPublicKey& key) {
    return key.fingerprint();
  }
  static std::string hex(const std::vector<std::uint8_t>& bytes) {
    return take_string(vault_key_format_hex(bytes.data(), bytes.size()));
  }
  static std::vector<std::uint8_t> decode_hex(const std::string& text) {
    return take_bytes(vault_key_decode_hex(text.data(), text.size()));
  }
  static std::string crockford(const std::vector<std::uint8_t>& bytes) {
    return take_string(vault_key_format_crockford(bytes.data(), bytes.size()));
  }
  static std::string crockford_reading(const std::string& code) {
    return take_string(vault_key_format_crockford_reading(code.data(), code.size()));
  }
  static std::vector<std::uint8_t> decode_crockford(const std::string& code) {
    return take_bytes(vault_key_decode_crockford(code.data(), code.size()));
  }
  static std::string hex_encode(const std::vector<std::uint8_t>& bytes) {
    return take_string(vault_key_hex_encode(bytes.data(), bytes.size()));
  }
  static std::vector<std::uint8_t> hex_decode(const std::string& text) {
    return take_bytes(vault_key_hex_decode(text.data(), text.size()));
  }
};

class AgentActivity {
 public:
  AgentActivity(const AgentActivity&) = delete;
  AgentActivity& operator=(const AgentActivity&) = delete;
  AgentActivity(AgentActivity&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~AgentActivity() { if (handle_) vault_agent_end_activity(handle_); }
  void end() {
    if (handle_) {
      vault_agent_end_activity(handle_);
      handle_ = nullptr;
    }
  }
 private:
  friend class Agent;
  explicit AgentActivity(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

class Agent {
 public:
  static void start() { checked(vault_agent_start()); }
  static bool is_running() { return vault_is_running(); }
  static void serve() { checked(vault_agent_serve()); }
  static void verify_transport() { checked(vault_agent_verify_transport()); }
  static void forget_all() { checked(vault_forget_all()); }
  static void stop() { checked(vault_agent_stop()); }
  static void put(const std::vector<std::uint8_t>& id,
                  const std::vector<std::uint8_t>& key) {
    checked(vault_agent_put(id.data(), id.size(), key.data(), key.size()));
  }
  static std::vector<std::uint8_t> get(const std::vector<std::uint8_t>& id) {
    return take_bytes(vault_agent_get(id.data(), id.size()));
  }
  static void forget(const std::vector<std::uint8_t>& id) {
    checked(vault_agent_forget(id.data(), id.size()));
  }
  static bindings::AgentEntryList list() {
    return take_message<bindings::AgentEntryList>(vault_agent_list());
  }
  static bindings::SleepSupport sleep_support() {
    return take_message<bindings::SleepSupport>(vault_agent_sleep_support());
  }
  static std::string log_path() { return take_string(vault_agent_log_path()); }
  static std::string log_destination() { return take_string(vault_agent_log_destination()); }
  static void put_vault_unlock_key(const std::string& profile,
                            const std::vector<std::uint8_t>& key,
                            std::uint64_t ttl_seconds) {
    checked(vault_agent_put_vault_unlock_key(profile.data(), profile.size(), key.data(),
                                      key.size(), ttl_seconds));
  }
  static std::vector<std::uint8_t> get_vault_unlock_key(const std::string& profile) {
    return take_bytes(vault_agent_get_vault_unlock_key(profile.data(), profile.size()));
  }
  static void forget_vault_unlock_key(const std::string& profile) {
    checked(vault_agent_forget_vault_unlock_key(profile.data(), profile.size()));
  }
  static void put_owner_signing_key(const std::string& vault_id, const std::string& profile,
                            const SigningKeyPair& key, std::uint64_t ttl_seconds) {
    checked(vault_agent_put_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size(),
        key.native_handle(), ttl_seconds));
  }
  static SigningKeyPair get_owner_signing_key(const std::string& vault_id,
                                      const std::string& profile) {
    return SigningKeyPair(vault_agent_get_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size()));
  }
  static void forget_owner_signing_key(const std::string& vault_id,
                               const std::string& profile) {
    checked(vault_agent_forget_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size()));
  }
  static AgentActivity begin_activity(const std::string& kind) {
    return AgentActivity(vault_agent_begin_activity(kind.data(), kind.size()));
  }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
};

class PlatformSecretStore {
 public:
  static bindings::PlatformStatus status() {
    return take_message<bindings::PlatformStatus>(vault_platform_status());
  }
  static void set_scope(const std::string& scope) {
    checked(vault_platform_set_scope(scope.data(), scope.size()));
  }
  static void enable() { checked(vault_platform_enable()); }
  static void disable() { checked(vault_platform_disable()); }
  static bool disabled() { return vault_platform_disabled(); }
  static void put_password(const std::string& password) {
    checked(vault_platform_put_password(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  static std::string get_password() { return take_string(vault_platform_get_password()); }
  static void forget_password() { checked(vault_platform_forget_password()); }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
};

class LocalVault {
 public:
  LocalVault() : handle_(vault_local()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  LocalVault(const LocalVault&) = delete;
  LocalVault& operator=(const LocalVault&) = delete;
  LocalVault(LocalVault&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  ~LocalVault() { if (handle_) vault_free(handle_); }
  Lockbox create_with_password(const std::string& path,
                               const std::string& password) const {
    return Lockbox::adopt(vault_create_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  Lockbox open_with_password(const std::string& path,
                             const std::string& password) const {
    return Lockbox::adopt(vault_open_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  Lockbox create_with_content_key(const std::string& path,
                                  const std::vector<std::uint8_t>& key,
                                  const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_create_lockbox_content_key(
        handle_, path.data(), path.size(), key.data(), key.size(),
        signing_key.native_handle()));
  }
  Lockbox open_with_content_key(const std::string& path,
                                const std::vector<std::uint8_t>& key,
                                const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_open_lockbox_content_key(
        handle_, path.data(), path.size(), key.data(), key.size(),
        signing_key.native_handle()));
  }
  Lockbox create_for_contact(const std::string& path,
                             const ContactPublicKey& contact,
                             const std::string& name,
                             const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_create_lockbox_contact(
        handle_, path.data(), path.size(), contact.native_handle(), name.data(),
        name.size(), signing_key.native_handle()));
  }
  void cache_password(const std::string& path, const std::string& password,
                      std::uint64_t ttl_seconds) const {
    checked(vault_cache_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size(),
        ttl_seconds));
  }
  void close(const std::string& path) const {
    checked(vault_close_lockbox(handle_, path.data(), path.size()));
  }
  void close_all() const { checked(vault_close_all(handle_)); }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

}  // namespace revault
