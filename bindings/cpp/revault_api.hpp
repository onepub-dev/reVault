#pragma once

/**
 * @file revault_api.hpp
 * @brief RAII C++ API for encrypted reVault lockboxes and local vaults.
 *
 * Use the functions and classes in `revault` to create or open lockboxes,
 * manage cryptographic keys, and open the local metadata vault. Objects that
 * retain sensitive state release it when destroyed; secret values are exposed
 * only to callback-scoped byte spans.
 *
 * See the [repository README](https://github.com/onepub-dev/reVault#readme)
 * for installation, the security model, and complete examples.
 */

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
#include "domain_models.hpp"

/** High-level, ownership-safe wrappers around the stable reVault C ABI. */
namespace revault {

class VaultDirectory;
class ReadOnlyVaultDirectory;
class Agent;
class LocalVault;
class WrappedContactKey;

namespace detail {

inline void require_compatible_abi() {
  if (api_abi_version() != 3)
    throw std::runtime_error("revault-api native ABI mismatch; expected 3");
}

class Buffer {
 public:
  /** Returns the buffer. */
  Buffer() = default;
  /** Returns the buffer. */
  explicit Buffer(RevaultBuffer value) : value_(value) {}
  /** Returns the buffer. */
  Buffer(const Buffer&) = delete;
  /** Returns the operator. */
  Buffer& operator=(const Buffer&) = delete;
  /** Returns the buffer. */
  Buffer(Buffer&& other) noexcept : value_(other.value_) { other.value_ = {}; }
  /** Returns the operator. */
  Buffer& operator=(Buffer&& other) noexcept {
    if (this != &other) { reset(); value_ = other.value_; other.value_ = {}; }
    return *this;
  }
  /** Returns the buffer. */
  ~Buffer() { reset(); }
  /** Returns the first byte in the owned native buffer. */
  const std::uint8_t* data() const { return value_.ptr; }
  /** Returns the number of bytes in the owned native buffer. */
  std::size_t size() const { return value_.len; }
  /** Returns the bytes. */
  std::vector<std::uint8_t> bytes() const { return {data(), data() + size()}; }
  /** Returns the operator bool. */
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

/**
 * Invokes `callback` with a temporary plaintext span, then wipes the transfer
 * buffer and releases the native secret handle. The callback must not retain
 * the span.
 */
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
  return decode<Message>({owned.data(), owned.size()});
}

inline bindings::ErrorDetails last_error_details() {
  return take_message<bindings::ErrorDetails>(buffer_last_error_details());
}

}  // namespace detail

/** Throws when the loaded native library does not implement this facade's ABI. */
inline void require_compatible_abi() { detail::require_compatible_abi(); }

/** Returns structured details for the most recent failed native call. */
inline bindings::ErrorDetails last_error_details() {
  return detail::last_error_details();
}

/** A recipient's shareable encryption identity, used when granting that
 * recipient lockbox access and containing no private key material. */
class ContactPublicKey {
 public:
  /** Returns the contact public key. */
  explicit ContactPublicKey(const std::vector<std::uint8_t>& bytes)
      : handle_(key_contact_public_from_bytes(bytes.data(), bytes.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the contact public key. */
  ContactPublicKey(const ContactPublicKey&) = delete;
  /** Returns the operator. */
  ContactPublicKey& operator=(const ContactPublicKey&) = delete;
  /** Returns the contact public key. */
  ContactPublicKey(ContactPublicKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the contact public key. */
  ~ContactPublicKey() { if (handle_) key_contact_public_free(handle_); }
  /** Imports import. */
  static ContactPublicKey import(const std::vector<std::uint8_t>& bytes) {
    return ContactPublicKey(vault_key_import_public(bytes.data(), bytes.size()));
  }
  /** Exports as. */
  std::vector<std::uint8_t> export_as(const std::string& format) const {
    return detail::take_bytes(vault_key_export_public(handle_, format.data(), format.size()));
  }
  /** Returns the stable fingerprint of this key. */
  std::vector<std::uint8_t> fingerprint() const {
    return detail::take_bytes(vault_key_fingerprint(handle_));
  }
  /** Encrypts a content key for the selected contact. */
  WrappedContactKey encrypt(const std::vector<std::uint8_t>& content_key) const;
  /** Returns the native handle. */
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit ContactPublicKey(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

/** A content key encrypted for one contact, stored with an access record and
 * recoverable only by the matching ContactKeyPair. */
class WrappedContactKey {
 public:
  /** Returns the wrapped contact key. */
  WrappedContactKey(const WrappedContactKey&) = delete;
  /** Returns the operator. */
  WrappedContactKey& operator=(const WrappedContactKey&) = delete;
  /** Returns the wrapped contact key. */
  WrappedContactKey(WrappedContactKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the wrapped contact key. */
  ~WrappedContactKey() { if (handle_) key_contact_wrapped_free(handle_); }
  /** Returns the public bytes. */
  std::vector<std::uint8_t> public_bytes() const { return detail::take_bytes(key_contact_wrapped_public(handle_)); }
  /** Returns the ciphertext. */
  std::vector<std::uint8_t> ciphertext() const { return detail::take_bytes(key_contact_wrapped_ciphertext(handle_)); }
  /** Returns the encrypted bytes. */
  std::vector<std::uint8_t> encrypted_bytes() const { return detail::take_bytes(key_contact_wrapped_encrypted(handle_)); }
  /** Returns the native handle. */
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

/** A profile's contact-encryption identity; distribute its public half and
 * retain the private half to decrypt content keys addressed to the profile. */
class ContactKeyPair {
 public:
  /** Returns the contact key pair. */
  ContactKeyPair() : handle_(key_contact_generate()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the from private record. */
  static ContactKeyPair from_private_record(const std::vector<std::uint8_t>& bytes) {
    return ContactKeyPair(key_contact_from_private(bytes.data(), bytes.size()));
  }
  /** Imports import. */
  static ContactKeyPair import(const std::vector<std::uint8_t>& bytes) {
    return ContactKeyPair(vault_key_import_private(bytes.data(), bytes.size()));
  }
  /** Returns the contact key pair. */
  ContactKeyPair(const ContactKeyPair&) = delete;
  /** Returns the operator. */
  ContactKeyPair& operator=(const ContactKeyPair&) = delete;
  /** Returns the contact key pair. */
  ContactKeyPair(ContactKeyPair&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the contact key pair. */
  ~ContactKeyPair() { if (handle_) key_contact_free(handle_); }
  /** Returns the public bytes. */
  std::vector<std::uint8_t> public_bytes() const { return detail::take_bytes(key_contact_public(handle_)); }
  /** Returns the private record. */
  std::vector<std::uint8_t> private_record() const { return detail::take_bytes(key_contact_private(handle_)); }
  /** Returns the public key. */
  ContactPublicKey public_key() const { return ContactPublicKey(public_bytes()); }
  /** Exports as. */
  std::vector<std::uint8_t> export_as(const std::string& format) const {
    return detail::take_bytes(vault_key_export_private(handle_, format.data(), format.size()));
  }
  /** Decrypts a wrapped content key for this contact. */
  std::vector<std::uint8_t> decrypt(const WrappedContactKey& wrapped) const {
    return detail::take_bytes(key_contact_decrypt(handle_, wrapped.native_handle()));
  }
  /** Returns the native handle. */
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit ContactKeyPair(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

/** The shareable half of a lockbox owner's signing identity, used by readers to
 * verify owner-authorized revisions. */
class SigningPublicKey {
 public:
  /** Returns the signing public key. */
  explicit SigningPublicKey(const std::vector<std::uint8_t>& bytes)
      : handle_(key_signing_public_from_bytes(bytes.data(), bytes.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the signing public key. */
  SigningPublicKey(const SigningPublicKey&) = delete;
  /** Returns the operator. */
  SigningPublicKey& operator=(const SigningPublicKey&) = delete;
  /** Returns the signing public key. */
  SigningPublicKey(SigningPublicKey&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the signing public key. */
  ~SigningPublicKey() { if (handle_) key_signing_public_free(handle_); }
  /** Returns the native handle. */
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  explicit SigningPublicKey(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

/** A lockbox owner's signing identity, supplied when creating or committing a
 * mutable lockbox so readers can authenticate its revisions. */
class SigningKeyPair {
 public:
  /** Returns the signing key pair. */
  SigningKeyPair() : handle_(key_signing_generate()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the from private record. */
  static SigningKeyPair from_private_record(const std::vector<std::uint8_t>& bytes) {
    return SigningKeyPair(key_signing_from_private(bytes.data(), bytes.size()));
  }
  /** Returns the signing key pair. */
  SigningKeyPair(const SigningKeyPair&) = delete;
  /** Returns the operator. */
  SigningKeyPair& operator=(const SigningKeyPair&) = delete;
  /** Returns the signing key pair. */
  SigningKeyPair(SigningKeyPair&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the signing key pair. */
  ~SigningKeyPair() { if (handle_) key_signing_free(handle_); }
  /** Returns the public bytes. */
  std::vector<std::uint8_t> public_bytes() const { return detail::take_bytes(key_signing_public(handle_)); }
  /** Returns the private record. */
  std::vector<std::uint8_t> private_record() const { return detail::take_bytes(key_signing_private(handle_)); }
  /** Returns the public key. */
  SigningPublicKey public_key() const { return SigningPublicKey(public_bytes()); }
  /** Returns the native handle. */
  void* native_handle() const { return handle_; }
 private:
  friend class VaultDirectory;
  friend class Agent;
  explicit SigningKeyPair(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

/** Memory and CPU settings applied when creating or opening a Lockbox; the
 * defaults suit interactive applications. */
struct LockboxOptions {
  /** Cache strategy, such as `bytes`. */
  std::string cache_mode{"bytes"};
  /** Maximum cache capacity in bytes. */
  std::uint64_t cache_bytes{64 * 1024 * 1024};
  /** Workload profile, such as `interactive`. */
  std::string workload{"interactive"};
  /** Worker-selection policy, such as `auto`. */
  std::string worker{"auto"};
  /** Worker count; zero lets the library select it. */
  std::size_t jobs{0};
};

/** An open encrypted archive containing files, variables, secrets, and forms;
 * commit mutations and release the object when finished with decrypted data. */
class Lockbox {
 public:
  /** Returns the supported lockbox format version. */
  static std::uint16_t format_version() { return lockbox_format_version(); }
  /** Determines format version without fully opening it. */
  static std::uint16_t probe_format_version(const std::vector<std::uint8_t>& bytes) {
    const auto value = lockbox_probe_format_version(bytes.data(), bytes.size());
    if (!value) throw std::runtime_error(buffer_last_error());
    return value;
  }
  /** Returns the lockbox. */
  explicit Lockbox(const std::vector<std::uint8_t>& key) : handle_(lockbox_create(key.data(), key.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the lockbox. */
  Lockbox(const Lockbox&) = delete;
  /** Returns the operator. */
  Lockbox& operator=(const Lockbox&) = delete;
  /** Returns the lockbox. */
  Lockbox(Lockbox&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the operator. */
  Lockbox& operator=(Lockbox&& other) noexcept {
    if (this != &other) {
      if (handle_) lockbox_free(handle_);
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }
  /** Returns the lockbox. */
  ~Lockbox() { if (handle_) lockbox_free(handle_); }

  /** Creates with password. */
  static Lockbox create_with_password(const std::string& password) {
    return adopt(lockbox_create_password(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Creates with options. */
  static Lockbox create_with_options(const std::vector<std::uint8_t>& key,
                                     const LockboxOptions& options) {
    return adopt(lockbox_create_with_options(
        key.data(), key.size(), options.cache_mode.data(), options.cache_mode.size(),
        options.cache_bytes, options.workload.data(), options.workload.size(),
        options.worker.data(), options.worker.size(), options.jobs));
  }
  /** Creates for contact. */
  static Lockbox create_for_contact(const ContactPublicKey& contact) {
    return adopt(lockbox_create_contact(contact.native_handle()));
  }
  /** Creates signed. */
  static Lockbox create_signed(const std::vector<std::uint8_t>& key,
                               const SigningKeyPair& signing_key) {
    return adopt(lockbox_create_with_signing_key(
        key.data(), key.size(), signing_key.native_handle()));
  }
  /** Opens an existing lockbox. */
  static Lockbox open(const std::vector<std::uint8_t>& archive,
                      const std::vector<std::uint8_t>& key) {
    return adopt(lockbox_open(archive.data(), archive.size(), key.data(), key.size()));
  }
  /** Opens with password. */
  static Lockbox open_with_password(const std::vector<std::uint8_t>& archive,
                                    const std::string& password) {
    return adopt(lockbox_open_password(
        archive.data(), archive.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Opens with options. */
  static Lockbox open_with_options(const std::vector<std::uint8_t>& archive,
                                   const std::vector<std::uint8_t>& key,
                                   const LockboxOptions& options) {
    return adopt(lockbox_open_with_options(
        archive.data(), archive.size(), key.data(), key.size(),
        options.cache_mode.data(), options.cache_mode.size(), options.cache_bytes,
        options.workload.data(), options.workload.size(), options.worker.data(),
        options.worker.size(), options.jobs));
  }
  /** Opens with contact. */
  static Lockbox open_with_contact(const std::vector<std::uint8_t>& archive,
                                   const ContactKeyPair& contact) {
    return adopt(lockbox_open_contact(archive.data(), archive.size(), contact.native_handle()));
  }

  /** Adds file. */
  void add_file(const std::string& path, const std::vector<std::uint8_t>& data, bool replace = false) {
    if (!lockbox_add_file(handle_, path.data(), path.size(), data.data(), data.size(), replace))
      throw std::runtime_error(buffer_last_error());
  }
  /** Authenticates and publishes the staged changes. */
  void commit() {
    if (!lockbox_commit(handle_)) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the storage length. */
  std::uint64_t storage_length() const { return lockbox_storage_len(handle_); }
  /** Returns the id. */
  std::vector<std::uint8_t> id() const { return detail::take_bytes(lockbox_id(handle_)); }
  /** Inspects file. */
  static bindings::FileInspection inspect_file(const std::string& path) {
    return detail::take_message<bindings::FileInspection>(
        lockbox_inspect_file(path.data(), path.size()));
  }
  /** Returns the render recovery report. */
  std::string render_recovery_report(bool verbose = false,
                                     std::size_t max_entries = 100) const {
    return detail::take_string(lockbox_recovery_report_render(handle_, verbose, max_entries));
  }
  /** Scans path. */
  static bindings::RecoveryReport scan_path(const std::string& path,
                                            const std::vector<std::uint8_t>& key) {
    return detail::take_message<bindings::RecoveryReport>(lockbox_recovery_scan_path(
        path.data(), path.size(), key.data(), key.size()));
  }
  /** Scans scan. */
  static bindings::RecoveryReport scan(const std::vector<std::uint8_t>& archive,
                                       const std::vector<std::uint8_t>& key) {
    return detail::take_message<bindings::RecoveryReport>(lockbox_recovery_scan(
        archive.data(), archive.size(), key.data(), key.size()));
  }
  /** Salvages salvage. */
  static Lockbox salvage(const std::vector<std::uint8_t>& archive,
                         const std::vector<std::uint8_t>& key,
                         const SigningKeyPair* signing_key = nullptr) {
    return adopt(lockbox_recovery_salvage(
        archive.data(), archive.size(), key.data(), key.size(),
        signing_key ? signing_key->native_handle() : nullptr));
  }
  /** Returns file. */
  std::vector<std::uint8_t> get_file(const std::string& path) const {
    auto result = lockbox_get_file(handle_, path.data(), path.size());
    if (!result.ptr) throw std::runtime_error(buffer_last_error());
    std::vector<std::uint8_t> value(result.ptr, result.ptr + result.len);
    buffer_free(result);
    return value;
  }
  /** Adds file with permissions. */
  void add_file_with_permissions(const std::string& path, const std::vector<std::uint8_t>& data,
                                 std::uint32_t permissions, bool replace = false) {
    if (!lockbox_add_file_with_permissions(handle_, path.data(), path.size(), data.data(), data.size(), permissions, replace))
      throw std::runtime_error(buffer_last_error());
  }
  /** Creates dir. */
  void create_dir(const std::string& path, bool create_parents = true) {
    if (!lockbox_create_dir(handle_, path.data(), path.size(), create_parents)) throw std::runtime_error(buffer_last_error());
  }
  /** Creates parent dirs. */
  void create_parent_dirs(const std::string& path) {
    if (!lockbox_create_parent_dirs(handle_, path.data(), path.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Extracts file. */
  void extract_file(const std::string& source, const std::string& destination,
                    bool replace = false) const {
    if (!lockbox_extract_file(handle_, source.data(), source.size(), destination.data(),
                              destination.size(), replace))
      throw std::runtime_error(buffer_last_error());
  }
  /** Extracts directory. */
  void extract_directory(const std::string& destination, std::uint64_t max_file_bytes,
                         std::uint64_t max_total_bytes, std::size_t max_files,
                         bool restore_symlinks, bool restore_permissions,
                         bool overwrite) const {
    if (!lockbox_extract_directory(handle_, destination.data(), destination.size(),
                                   max_file_bytes, max_total_bytes, max_files,
                                   restore_symlinks, restore_permissions, overwrite))
      throw std::runtime_error(buffer_last_error());
  }
  /** Removes dir. */
  void remove_dir(const std::string& path, bool recursive = false) {
    if (!lockbox_remove_dir(handle_, path.data(), path.size(), recursive)) throw std::runtime_error(buffer_last_error());
  }
  /** Removes remove. */
  void remove(const std::string& path) {
    if (!lockbox_delete(handle_, path.data(), path.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Updates rename. */
  void rename(const std::string& from, const std::string& to) {
    if (!lockbox_rename(handle_, from.data(), from.size(), to.data(), to.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Adds symlink. */
  void add_symlink(const std::string& path, const std::string& target, bool replace = false) {
    if (!lockbox_add_symlink(handle_, path.data(), path.size(), target.data(), target.size(), replace)) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the symlink target. */
  std::vector<std::uint8_t> symlink_target(const std::string& path) const {
    return detail::take_bytes(lockbox_get_symlink_target(handle_, path.data(), path.size()));
  }
  /** Returns range. */
  std::vector<std::uint8_t> read_range(const std::string& path, std::uint64_t offset, std::uint64_t length) const {
    return detail::take_bytes(lockbox_read_range(handle_, path.data(), path.size(), offset, length));
  }
  /** Lists list. */
  bindings::LockboxEntryList list(const std::string& path = "/", bool recursive = false) const {
    return decoded<bindings::LockboxEntryList>(lockbox_list(handle_, path.data(), path.size(), recursive));
  }
  /** Returns metadata for the selected lockbox entry. */
  bindings::OptionalLockboxEntry stat(const std::string& path) const {
    return decoded<bindings::OptionalLockboxEntry>(lockbox_stat(handle_, path.data(), path.size()));
  }
  /** Lists with options. */
  bindings::LockboxEntryList list_with_options(
      const std::string& path, const std::string& glob, bool recursive,
      bool include_files, bool include_symlinks, bool include_directories,
      std::size_t limit) const {
    return decoded<bindings::LockboxEntryList>(lockbox_list_with_options(
        handle_, path.data(), path.size(), glob.data(), glob.size(), recursive,
        include_files, include_symlinks, include_directories, limit));
  }
  /** Reports whether exists. */
  bool exists(const std::string& path) const { return lockbox_exists(handle_, path.data(), path.size()); }
  /** Reports whether dir. */
  bool is_dir(const std::string& path) const { return lockbox_is_dir(handle_, path.data(), path.size()); }
  /** Returns the permissions. */
  std::uint32_t permissions(const std::string& path) const { return lockbox_permissions(handle_, path.data(), path.size()); }
  /** Sets permissions. */
  void set_permissions(const std::string& path, std::uint32_t value) {
    if (!lockbox_set_permissions(handle_, path.data(), path.size(), value)) throw std::runtime_error(buffer_last_error());
  }
  /** Sets variable. */
  void set_variable(const std::string& name, const std::string& value) {
    if (!lockbox_set_variable(handle_, name.data(), name.size(), value.data(), value.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Sets secret variable. */
  void set_secret_variable(const std::string& name, std::span<const std::uint8_t> value) {
    if (!lockbox_set_secret_variable(handle_, name.data(), name.size(), value.data(), value.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Returns variable. */
  std::optional<std::string> get_variable(const std::string& name) const {
    return decoded<bindings::OptionalString>(
        lockbox_get_variable(handle_, name.data(), name.size()));
  }
  /** Returns the with secret variable. */
  bool with_secret_variable(const std::string& name,
      const std::function<void(std::span<const std::uint8_t>)>& callback) const {
    void* secret{};
    if (!lockbox_get_secret_variable(handle_, name.data(), name.size(), &secret))
      throw std::runtime_error(buffer_last_error());
    return detail::with_secret_handle(secret, callback);
  }
  /** Removes variable. */
  void delete_variable(const std::string& name) {
    if (!lockbox_delete_variable(handle_, name.data(), name.size())) throw std::runtime_error(buffer_last_error());
  }
  /** Updates variables. */
  void move_variables(const bindings::PathMoveList& moves) {
    const auto encoded = detail::encode_moves(moves);
    if (!lockbox_move_variables(handle_, reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Lists variables. */
  bindings::VariableList list_variables() const { return decoded<bindings::VariableList>(lockbox_list_variables(handle_)); }
  /** Returns the variable sensitivity. */
  bindings::OptionalString variable_sensitivity(const std::string& name) const {
    return decoded<bindings::OptionalString>(
        lockbox_variable_sensitivity(handle_, name.data(), name.size()));
  }
  /** Updates import stats. */
  void reset_import_stats() const {
    if (!lockbox_reset_import_stats(handle_)) throw std::runtime_error(buffer_last_error());
  }
  /** Returns cache statistics for this lockbox. */
  bindings::CacheStats cache_stats() const { return decoded<bindings::CacheStats>(lockbox_cache_stats(handle_)); }
  /** Returns import statistics for this lockbox. */
  bindings::ImportStats import_stats() const { return decoded<bindings::ImportStats>(lockbox_import_stats(handle_)); }
  /** Lists key slots. */
  bindings::KeySlotList list_key_slots() const { return decoded<bindings::KeySlotList>(lockbox_list_key_slots(handle_)); }
  /** Adds password. */
  std::uint64_t add_password(const std::string& password) {
    auto id = lockbox_add_password(handle_, reinterpret_cast<const std::uint8_t*>(password.data()), password.size());
    if (id == UINT64_MAX) throw std::runtime_error(buffer_last_error());
    return id;
  }
  /** Adds contact. */
  std::uint64_t add_contact(const ContactPublicKey& contact, const std::string& name) {
    auto id = lockbox_add_contact(handle_, contact.native_handle(), name.data(), name.size());
    if (id == UINT64_MAX) throw std::runtime_error(buffer_last_error());
    return id;
  }
  /** Removes key. */
  void delete_key(std::uint64_t id) {
    if (!lockbox_delete_key(handle_, id)) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the owner inspection. */
  bindings::OwnerInspection owner_inspection() const { return decoded<bindings::OwnerInspection>(lockbox_owner_inspection(handle_)); }
  /** Sets owner signing key. */
  void set_owner_signing_key(const SigningKeyPair& key) {
    if (!lockbox_set_owner_signing_key(handle_, key.native_handle()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Returns the stream content. */
  bindings::StreamChunkList stream_content(bool physical = false) const { return decoded<bindings::StreamChunkList>(lockbox_stream_content(handle_, physical)); }
  /** Returns the page inspection. */
  bindings::PageInspectionList page_inspection() const { return decoded<bindings::PageInspectionList>(lockbox_page_inspection(handle_)); }
  /** Returns the recovery report. */
  bindings::RecoveryReport recovery_report() const { return decoded<bindings::RecoveryReport>(lockbox_recovery_report(handle_)); }
  /** Returns the runtime options. */
  bindings::RuntimeOptions runtime_options() const { return decoded<bindings::RuntimeOptions>(lockbox_runtime_options(handle_)); }
  /** Returns the to bytes. */
  std::vector<std::uint8_t> to_bytes() const { return detail::take_bytes(lockbox_to_bytes(handle_)); }

  /** Returns the define form. */
  bindings::FormDefinition define_form(const std::string& alias,
                                       const std::string& name,
                                       const std::string& description,
                                       const bindings::FormFieldList& fields) {
    const std::string encoded = detail::encode_fields(fields);
    return decoded<bindings::FormDefinition>(lockbox_define_form(
        handle_, alias.data(), alias.size(), name.data(), name.size(),
        description.data(), description.size(),
        reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()));
  }
  /** Lists form definitions. */
  bindings::FormDefinitionList list_form_definitions() const {
    return decoded<bindings::FormDefinitionList>(lockbox_list_form_definitions(handle_));
  }
  /** Returns the resolve form. */
  bindings::FormDefinition resolve_form(const std::string& reference) const {
    return decoded<bindings::FormDefinition>(
        lockbox_resolve_form(handle_, reference.data(), reference.size()));
  }
  /** Lists form revisions. */
  bindings::FormDefinitionList list_form_revisions(const std::string& type_id) const {
    return decoded<bindings::FormDefinitionList>(
        lockbox_list_form_revisions(handle_, type_id.data(), type_id.size()));
  }
  /** Creates form record. */
  bindings::FormRecord create_form_record(const std::string& path,
                                          const std::string& type_reference,
                                          const std::string& name) {
    return decoded<bindings::FormRecord>(lockbox_create_form_record(
        handle_, path.data(), path.size(), type_reference.data(), type_reference.size(),
        name.data(), name.size()));
  }
  /** Sets form field. */
  void set_form_field(const std::string& path, const std::string& field,
                      const std::string& value) {
    if (!lockbox_set_form_field(handle_, path.data(), path.size(), field.data(), field.size(),
                                value.data(), value.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Sets secret form field. */
  void set_secret_form_field(const std::string& path, const std::string& field,
                             std::span<const std::uint8_t> value) {
    if (!lockbox_set_secret_form_field(handle_, path.data(), path.size(), field.data(), field.size(),
                                       value.data(), value.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Lists form records. */
  bindings::FormRecordList list_form_records() const {
    return decoded<bindings::FormRecordList>(lockbox_list_form_records(handle_));
  }
  /** Returns form record. */
  bindings::OptionalFormRecord get_form_record(const std::string& path) const {
    return decoded<bindings::OptionalFormRecord>(
        lockbox_get_form_record(handle_, path.data(), path.size()));
  }
  /** Removes form record. */
  void delete_form_record(const std::string& path) {
    if (!lockbox_delete_form_record(handle_, path.data(), path.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Updates form records. */
  void move_form_records(const bindings::PathMoveList& moves) {
    const auto encoded = detail::encode_moves(moves);
    if (!lockbox_move_form_records(handle_, reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Returns form field. */
  bindings::OptionalFormValue get_form_field(const std::string& path,
                                             const std::string& field) const {
    return decoded<bindings::OptionalFormValue>(lockbox_get_form_field(
        handle_, path.data(), path.size(), field.data(), field.size()));
  }
  /** Returns the with secret form field. */
  bool with_secret_form_field(const std::string& path, const std::string& field,
      const std::function<void(std::span<const std::uint8_t>)>& callback) const {
    void* secret{};
    if (!lockbox_get_secret_form_field(handle_, path.data(), path.size(), field.data(), field.size(), &secret))
      throw std::runtime_error(buffer_last_error());
    return detail::with_secret_handle(secret, callback);
  }
  /** Sets workload profile. */
  void set_workload_profile(const std::string& profile) {
    if (!lockbox_set_workload_profile(handle_, profile.data(), profile.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Sets worker policy. */
  void set_worker_policy(const std::string& mode, std::size_t jobs = 0) {
    if (!lockbox_set_worker_policy(handle_, mode.data(), mode.size(), jobs))
      throw std::runtime_error(buffer_last_error());
  }
  /** Returns the native handle. */
  void* native_handle() const { return handle_; }

 private:
  friend class LocalVault;
  explicit Lockbox(void* handle) : handle_(handle) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  static Lockbox adopt(void* handle) { return Lockbox(handle); }
  static detail::Buffer checked(RevaultBuffer result) {
    if (!result.ptr) throw std::runtime_error(buffer_last_error());
    return detail::Buffer(result);
  }
  template <typename Message>
  static Message decoded(RevaultBuffer result) {
    detail::Buffer owned = checked(result);
    return detail::decode<Message>({owned.data(), owned.size()});
  }
  void* handle_;
};

/** A writable, password-protected store for profile keys, contacts, forms,
 * backups, and remembered lockbox paths; lockbox contents remain separate. */
class VaultDirectory {
 public:
  /** Returns the current structure version. */
  static std::uint32_t current_structure_version() { return vault_structure_version_current(); }
  /** Determines structure version without fully opening it. */
  static std::uint32_t probe_structure_version(const std::string& root, const std::string& password) {
    const auto value = vault_directory_probe_structure_version(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()), password.size());
    if (!value) throw std::runtime_error(buffer_last_error());
    return value;
  }
  /** Opens an existing lockbox. */
  static VaultDirectory open(const std::string& root, const std::string& password) {
    return adopt(vault_directory_open(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  /** Opens or create. */
  static VaultDirectory open_or_create(const std::string& root,
                                       const std::string& password) {
    return adopt(vault_directory_open_or_create(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  /** Updates replace. */
  static VaultDirectory replace(const std::string& root, const std::string& password) {
    return adopt(vault_directory_replace(
        root.data(), root.size(), reinterpret_cast<const std::uint8_t*>(password.data()),
        password.size()));
  }
  /** Opens or create default. */
  static VaultDirectory open_or_create_default(const std::string& password) {
    return adopt(vault_directory_open_or_create_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Updates default. */
  static VaultDirectory replace_default(const std::string& password) {
    return adopt(vault_directory_replace_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Updates password. */
  static void change_password(const std::string& root, const std::string& old_password,
                              const std::string& new_password) {
    if (!vault_directory_change_password(
            root.data(), root.size(),
            reinterpret_cast<const std::uint8_t*>(old_password.data()), old_password.size(),
            reinterpret_cast<const std::uint8_t*>(new_password.data()), new_password.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Updates default password. */
  static void change_default_password(const std::string& old_password,
                                      const std::string& new_password) {
    if (!vault_directory_change_default_password(
            reinterpret_cast<const std::uint8_t*>(old_password.data()), old_password.size(),
            reinterpret_cast<const std::uint8_t*>(new_password.data()), new_password.size()))
      throw std::runtime_error(buffer_last_error());
  }
  /** Returns the default directory. */
  static std::string default_directory() { return detail::take_string(vault_default_directory()); }
  /** Returns the default path. */
  static std::string default_path() { return detail::take_string(vault_default_path()); }
  /** Returns the backup default. */
  static bindings::VaultBackupManifest backup_default(const std::string& path,
                                                       bool overwrite = false) {
    return detail::take_message<bindings::VaultBackupManifest>(
        vault_backup_default(path.data(), path.size(), overwrite));
  }
  /** Returns the restore default. */
  static bindings::VaultBackupManifest restore_default(const std::string& path,
                                                        bool overwrite = false) {
    return detail::take_message<bindings::VaultBackupManifest>(
        vault_restore_default(path.data(), path.size(), overwrite));
  }

  /** Returns the vault directory. */
  VaultDirectory(const VaultDirectory&) = delete;
  /** Returns the operator. */
  VaultDirectory& operator=(const VaultDirectory&) = delete;
  /** Returns the vault directory. */
  VaultDirectory(VaultDirectory&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the operator. */
  VaultDirectory& operator=(VaultDirectory&& other) noexcept {
    if (this != &other) {
      if (handle_) vault_directory_free(handle_);
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }
  /** Returns the vault directory. */
  ~VaultDirectory() { if (handle_) vault_directory_free(handle_); }

  /** Returns the root. */
  std::string root() const { return detail::take_string(vault_directory_root(handle_)); }
  /** Returns the structure version. */
  std::uint32_t structure_version() const { return vault_directory_structure_version(handle_); }
  /** Lists private keys. */
  bindings::StringList list_private_keys() const {
    return detail::take_message<bindings::StringList>(vault_directory_list_private_keys(handle_));
  }
  /** Lists private key names. */
  bindings::StringList list_private_key_names() const {
    return detail::take_message<bindings::StringList>(vault_directory_list_private_key_names(handle_));
  }
  /** Lists contact names. */
  bindings::StringList list_contact_names() const {
    return detail::take_message<bindings::StringList>(vault_directory_list_contact_names(handle_));
  }
  /** Lists form aliases. */
  bindings::StringList list_form_aliases() const {
    return detail::take_message<bindings::StringList>(vault_directory_list_form_aliases(handle_));
  }
  /** Returns the private key exists. */
  bool private_key_exists(const std::string& name) const {
    return vault_directory_private_key_exists(handle_, name.data(), name.size());
  }
  /** Removes private key. */
  void delete_private_key(const std::string& name) const {
    checked(vault_directory_delete_private_key(handle_, name.data(), name.size()));
  }
  /** Stores private key. */
  void store_private_key(const std::string& name, const ContactKeyPair& key) const {
    checked(vault_directory_store_private_key(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  /** Loads private key. */
  ContactKeyPair load_private_key(const std::string& name) const {
    return ContactKeyPair(vault_directory_load_private_key(handle_, name.data(), name.size()));
  }
  /** Loads private key generation. */
  ContactKeyPair load_private_key_generation(const std::string& name,
                                             std::uint16_t index) const {
    return ContactKeyPair(vault_directory_load_private_key_generation(
        handle_, name.data(), name.size(), index));
  }
  /** Stores contact. */
  void store_contact(const std::string& name, const ContactPublicKey& key) const {
    checked(vault_directory_store_contact(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  /** Loads contact. */
  ContactPublicKey load_contact(const std::string& name) const {
    return ContactPublicKey(vault_directory_load_contact(handle_, name.data(), name.size()));
  }
  /** Returns the contact exists. */
  bool contact_exists(const std::string& name) const {
    return vault_directory_contact_exists(handle_, name.data(), name.size());
  }
  /** Removes contact. */
  void delete_contact(const std::string& name) const {
    checked(vault_directory_delete_contact(handle_, name.data(), name.size()));
  }
  /** Lists contacts. */
  bindings::ContactList list_contacts() const {
    return detail::take_message<bindings::ContactList>(vault_directory_list_contacts(handle_));
  }
  /** Stores profile email. */
  void store_profile_email(const std::string& name, const std::string& email) const {
    checked(vault_directory_store_profile_email(
        handle_, name.data(), name.size(), email.data(), email.size()));
  }
  /** Returns the profile email. */
  bindings::OptionalString profile_email(const std::string& name) const {
    return detail::take_message<bindings::OptionalString>(
        vault_directory_profile_email(handle_, name.data(), name.size()));
  }
  /** Stores backup. */
  void store_backup(const std::vector<std::uint8_t>& id,
                    const std::vector<std::uint8_t>& bytes) const {
    checked(vault_directory_store_backup(
        handle_, id.data(), id.size(), bytes.data(), bytes.size()));
  }
  /** Loads backup. */
  std::vector<std::uint8_t> load_backup(const std::vector<std::uint8_t>& id) const {
    return detail::take_bytes(vault_directory_load_backup(handle_, id.data(), id.size()));
  }
  /** Returns the backup count. */
  std::uint64_t backup_count() const { return vault_directory_backup_count(handle_); }
  /** Returns the restore private key. */
  void restore_private_key(const std::string& name, const ContactKeyPair& key,
                           const SigningKeyPair& signing_key, bool overwrite = false) const {
    checked(vault_directory_restore_private_key(
        handle_, name.data(), name.size(), key.native_handle(),
        signing_key.native_handle(), overwrite));
  }
  /** Loads owner signing key. */
  SigningKeyPair load_owner_signing_key(const std::string& name) const {
    return SigningKeyPair(vault_directory_load_owner_signing_key(
        handle_, name.data(), name.size()));
  }
  /** Loads owner signing key generation. */
  SigningKeyPair load_owner_signing_key_generation(const std::string& name,
                                                   std::uint16_t index) const {
    return SigningKeyPair(vault_directory_load_owner_signing_key_generation(
        handle_, name.data(), name.size(), index));
  }
  /** Stores contact signing key. */
  void store_contact_signing_key(const std::string& name,
                                 const SigningPublicKey& key) const {
    checked(vault_directory_store_contact_signing_key(
        handle_, name.data(), name.size(), key.native_handle()));
  }
  /** Loads contact signing key. */
  SigningPublicKey load_contact_signing_key(const std::string& name) const {
    return SigningPublicKey(vault_directory_load_contact_signing_key(
        handle_, name.data(), name.size()));
  }
  /** Lists profile generations. */
  bindings::ProfileHistory list_profile_generations(const std::string& name) const {
    return detail::take_message<bindings::ProfileHistory>(
        vault_directory_list_profile_generations(handle_, name.data(), name.size()));
  }
  /** Updates private key. */
  bindings::ProfileHistory rotate_private_key(const std::string& name) const {
    return detail::take_message<bindings::ProfileHistory>(
        vault_directory_rotate_private_key(handle_, name.data(), name.size()));
  }
  /** Stores lockbox. */
  void remember_lockbox(const std::vector<std::uint8_t>& id,
                        const std::string& path) const {
    checked(vault_directory_remember_lockbox(
        handle_, id.data(), id.size(), path.data(), path.size()));
  }
  /** Lists known lockboxes. */
  bindings::KnownLockboxList list_known_lockboxes() const {
    return detail::take_message<bindings::KnownLockboxList>(
        vault_directory_list_known_lockboxes(handle_));
  }
  /** Removes lockbox. */
  void forget_lockbox(const std::string& path) const {
    checked(vault_directory_forget_lockbox(handle_, path.data(), path.size()));
  }
  /** Stores access slot label. */
  void remember_access_slot_label(const std::vector<std::uint8_t>& id,
                             std::uint64_t slot_id, const std::string& name) const {
    checked(vault_directory_remember_access_slot_label(
        handle_, id.data(), id.size(), slot_id, name.data(), name.size()));
  }
  /** Lists access slot labels. */
  bindings::AccessSlotLabelList list_access_slot_labels(
      const std::vector<std::uint8_t>& id) const {
    return detail::take_message<bindings::AccessSlotLabelList>(
        vault_directory_list_access_slot_labels(handle_, id.data(), id.size()));
  }
  /** Returns the find access slot labels. */
  bindings::AccessSlotLabelList find_access_slot_labels(
      const std::vector<std::uint8_t>& id, const std::string& name) const {
    return detail::take_message<bindings::AccessSlotLabelList>(vault_directory_find_access_slot_labels(
        handle_, id.data(), id.size(), name.data(), name.size()));
  }
  /** Removes access slot label. */
  void forget_access_slot_label(const std::vector<std::uint8_t>& id,
                           std::uint64_t slot_id) const {
    checked(vault_directory_forget_access_slot_label(handle_, id.data(), id.size(), slot_id));
  }
  /** Returns the define form. */
  bindings::FormDefinition define_form(const std::string& alias,
                                       const std::string& name,
                                       const std::string& description,
                                       const bindings::FormFieldList& fields) const {
    const std::string encoded = detail::encode_fields(fields);
    return detail::take_message<bindings::FormDefinition>(vault_directory_define_form(
        handle_, alias.data(), alias.size(), name.data(), name.size(),
        description.data(), description.size(),
        reinterpret_cast<const std::uint8_t*>(encoded.data()), encoded.size()));
  }
  /** Returns the resolve form. */
  bindings::FormDefinition resolve_form(const std::string& reference) const {
    return detail::take_message<bindings::FormDefinition>(
        vault_directory_resolve_form(handle_, reference.data(), reference.size()));
  }
  /** Lists forms. */
  bindings::FormDefinitionList list_forms() const {
    return detail::take_message<bindings::FormDefinitionList>(vault_directory_list_forms(handle_));
  }
  /** Lists form revisions. */
  bindings::FormDefinitionList list_form_revisions(const std::string& type_id) const {
    return detail::take_message<bindings::FormDefinitionList>(
        vault_directory_list_form_revisions(handle_, type_id.data(), type_id.size()));
  }
  /** Returns the seed forms. */
  std::size_t seed_forms() const { return vault_directory_seed_forms(handle_); }
  /** Stores password. */
  void remember_password(const std::vector<std::uint8_t>& id,
                         const std::string& password) const {
    checked(vault_directory_remember_password(
        handle_, id.data(), id.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Returns the remembered password. */
  std::vector<std::uint8_t> remembered_password(
      const std::vector<std::uint8_t>& id) const {
    return detail::take_bytes(vault_directory_remembered_password(handle_, id.data(), id.size()));
  }
  /** Returns the native handle. */
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

/** A restricted metadata view for discovery or diagnostics without loading an
 * owner signing key or exposing mutation operations. */
class ReadOnlyVaultDirectory {
 public:
  /** Returns only vault directory. */
  ReadOnlyVaultDirectory(const std::string& root, const std::string& password)
      : handle_(vault_read_only_open(root.data(), root.size(),
          reinterpret_cast<const std::uint8_t*>(password.data()), password.size())) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Opens default. */
  static ReadOnlyVaultDirectory open_default(const std::string& password) {
    return ReadOnlyVaultDirectory(vault_read_only_open_default(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Returns only vault directory. */
  ReadOnlyVaultDirectory(const ReadOnlyVaultDirectory&) = delete;
  /** Returns only vault directory. */
  ReadOnlyVaultDirectory(ReadOnlyVaultDirectory&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns only vault directory. */
  ~ReadOnlyVaultDirectory() { if (handle_) vault_read_only_free(handle_); }
  /** Lists profile names. */
  bindings::StringList list_profile_names() const { return detail::take_message<bindings::StringList>(vault_read_only_list_profile_names(handle_)); }
  /** Lists contact names. */
  bindings::StringList list_contact_names() const { return detail::take_message<bindings::StringList>(vault_read_only_list_contact_names(handle_)); }
  /** Lists form aliases. */
  bindings::StringList list_form_aliases() const { return detail::take_message<bindings::StringList>(vault_read_only_list_form_aliases(handle_)); }
  /** Lists known lockboxes. */
  bindings::KnownLockboxList list_known_lockboxes() const { return detail::take_message<bindings::KnownLockboxList>(vault_read_only_list_known_lockboxes(handle_)); }
 private:
  explicit ReadOnlyVaultDirectory(void* handle) : handle_(handle) { if (!handle_) throw std::runtime_error(buffer_last_error()); }
  void* handle_{};
};

/** Canonical formatting and parsing helpers for public key material. */
class KeyFormat {
 public:
  /** Returns the stable fingerprint of this key. */
  static std::vector<std::uint8_t> fingerprint(const ContactPublicKey& key) {
    return key.fingerprint();
  }
  /** Returns the hex. */
  static std::string hex(const std::vector<std::uint8_t>& bytes) {
    return detail::take_string(vault_key_format_hex(bytes.data(), bytes.size()));
  }
  /** Decodes hex. */
  static std::vector<std::uint8_t> decode_hex(const std::string& text) {
    return detail::take_bytes(vault_key_decode_hex(text.data(), text.size()));
  }
  /** Returns the crockford. */
  static std::string crockford(const std::vector<std::uint8_t>& bytes) {
    return detail::take_string(vault_key_format_crockford(bytes.data(), bytes.size()));
  }
  /** Returns the crockford reading. */
  static std::string crockford_reading(const std::string& code) {
    return detail::take_string(vault_key_format_crockford_reading(code.data(), code.size()));
  }
  /** Decodes crockford. */
  static std::vector<std::uint8_t> decode_crockford(const std::string& code) {
    return detail::take_bytes(vault_key_decode_crockford(code.data(), code.size()));
  }
  /** Returns the hex encode. */
  static std::string hex_encode(const std::vector<std::uint8_t>& bytes) {
    return detail::take_string(vault_key_hex_encode(bytes.data(), bytes.size()));
  }
  /** Returns the hex decode. */
  static std::vector<std::uint8_t> hex_decode(const std::string& text) {
    return detail::take_bytes(vault_key_hex_decode(text.data(), text.size()));
  }
};

/** A lifetime token kept alive while an operation needs cached secrets; destroy
 * it afterward so the session agent can expire unused secrets. */
class AgentActivity {
 public:
  /** Returns the agent activity. */
  AgentActivity(const AgentActivity&) = delete;
  /** Returns the operator. */
  AgentActivity& operator=(const AgentActivity&) = delete;
  /** Returns the agent activity. */
  AgentActivity(AgentActivity&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the agent activity. */
  ~AgentActivity() { if (handle_) vault_agent_end_activity(handle_); }
  /** Stops end. */
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

/** Client for the local session service that temporarily caches vault unlock
 * and owner signing keys across application operations. */
class Agent {
 public:
  /** Starts start. */
  static void start() { checked(vault_agent_start()); }
  /** Reports whether running. */
  static bool is_running() { return vault_is_running(); }
  /** Returns the serve. */
  static void serve() { checked(vault_agent_serve()); }
  /** Verifies transport. */
  static void verify_transport() { checked(vault_agent_verify_transport()); }
  /** Removes all. */
  static void forget_all() { checked(vault_forget_all()); }
  /** Stops stop. */
  static void stop() { checked(vault_agent_stop()); }
  /** Stores put. */
  static void put(const std::vector<std::uint8_t>& id,
                  const std::vector<std::uint8_t>& key) {
    checked(vault_agent_put(id.data(), id.size(), key.data(), key.size()));
  }
  /** Returns get. */
  static std::vector<std::uint8_t> get(const std::vector<std::uint8_t>& id) {
    return detail::take_bytes(vault_agent_get(id.data(), id.size()));
  }
  /** Removes forget. */
  static void forget(const std::vector<std::uint8_t>& id) {
    checked(vault_agent_forget(id.data(), id.size()));
  }
  /** Lists list. */
  static bindings::AgentEntryList list() {
    return detail::take_message<bindings::AgentEntryList>(vault_agent_list());
  }
  /** Returns the sleep support. */
  static bindings::SleepSupport sleep_support() {
    return detail::take_message<bindings::SleepSupport>(vault_agent_sleep_support());
  }
  /** Returns the log path. */
  static std::string log_path() { return detail::take_string(vault_agent_log_path()); }
  /** Returns the log destination. */
  static std::string log_destination() { return detail::take_string(vault_agent_log_destination()); }
  /** Stores vault unlock key. */
  static void put_vault_unlock_key(const std::string& profile,
                            const std::vector<std::uint8_t>& key,
                            std::uint64_t ttl_seconds) {
    checked(vault_agent_put_vault_unlock_key(profile.data(), profile.size(), key.data(),
                                      key.size(), ttl_seconds));
  }
  /** Returns vault unlock key. */
  static std::vector<std::uint8_t> get_vault_unlock_key(const std::string& profile) {
    return detail::take_bytes(vault_agent_get_vault_unlock_key(profile.data(), profile.size()));
  }
  /** Removes vault unlock key. */
  static void forget_vault_unlock_key(const std::string& profile) {
    checked(vault_agent_forget_vault_unlock_key(profile.data(), profile.size()));
  }
  /** Stores owner signing key. */
  static void put_owner_signing_key(const std::string& vault_id, const std::string& profile,
                            const SigningKeyPair& key, std::uint64_t ttl_seconds) {
    checked(vault_agent_put_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size(),
        key.native_handle(), ttl_seconds));
  }
  /** Returns owner signing key. */
  static SigningKeyPair get_owner_signing_key(const std::string& vault_id,
                                      const std::string& profile) {
    return SigningKeyPair(vault_agent_get_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size()));
  }
  /** Removes owner signing key. */
  static void forget_owner_signing_key(const std::string& vault_id,
                               const std::string& profile) {
    checked(vault_agent_forget_owner_signing_key(
        vault_id.data(), vault_id.size(), profile.data(), profile.size()));
  }
  /** Starts activity. */
  static AgentActivity begin_activity(const std::string& kind) {
    return AgentActivity(vault_agent_begin_activity(kind.data(), kind.size()));
  }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
};

/** Access to operating-system credential storage for a scoped vault password. */
class PlatformSecretStore {
 public:
  /** Returns the status. */
  static bindings::PlatformStatus status() {
    return detail::take_message<bindings::PlatformStatus>(vault_platform_status());
  }
  /** Sets scope. */
  static void set_scope(const std::string& scope) {
    checked(vault_platform_set_scope(scope.data(), scope.size()));
  }
  /** Returns the enable. */
  static void enable() { checked(vault_platform_enable()); }
  /** Returns the disable. */
  static void disable() { checked(vault_platform_disable()); }
  /** Returns the disabled. */
  static bool disabled() { return vault_platform_disabled(); }
  /** Stores password. */
  static void put_password(const std::string& password) {
    checked(vault_platform_put_password(
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Returns password. */
  static std::string get_password() { return detail::take_string(vault_platform_get_password()); }
  /** Removes password. */
  static void forget_password() { checked(vault_platform_forget_password()); }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
};

/** A session for creating or opening lockboxes by host path, caching short-lived
 * passwords, and committing and closing files used by a local application. */
class LocalVault {
 public:
  /** Returns the local vault. */
  LocalVault() : handle_(vault_local()) {
    if (!handle_) throw std::runtime_error(buffer_last_error());
  }
  /** Returns the local vault. */
  LocalVault(const LocalVault&) = delete;
  /** Returns the operator. */
  LocalVault& operator=(const LocalVault&) = delete;
  /** Returns the local vault. */
  LocalVault(LocalVault&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
  /** Returns the local vault. */
  ~LocalVault() { if (handle_) vault_free(handle_); }
  /** Creates with password. */
  Lockbox create_with_password(const std::string& path,
                               const std::string& password) const {
    return Lockbox::adopt(vault_create_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Opens with password. */
  Lockbox open_with_password(const std::string& path,
                             const std::string& password) const {
    return Lockbox::adopt(vault_open_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size()));
  }
  /** Creates with content key. */
  Lockbox create_with_content_key(const std::string& path,
                                  const std::vector<std::uint8_t>& key,
                                  const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_create_lockbox_content_key(
        handle_, path.data(), path.size(), key.data(), key.size(),
        signing_key.native_handle()));
  }
  /** Opens with content key. */
  Lockbox open_with_content_key(const std::string& path,
                                const std::vector<std::uint8_t>& key,
                                const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_open_lockbox_content_key(
        handle_, path.data(), path.size(), key.data(), key.size(),
        signing_key.native_handle()));
  }
  /** Creates for contact. */
  Lockbox create_for_contact(const std::string& path,
                             const ContactPublicKey& contact,
                             const std::string& name,
                             const SigningKeyPair& signing_key) const {
    return Lockbox::adopt(vault_create_lockbox_contact(
        handle_, path.data(), path.size(), contact.native_handle(), name.data(),
        name.size(), signing_key.native_handle()));
  }
  /** Stores password. */
  void cache_password(const std::string& path, const std::string& password,
                      std::uint64_t ttl_seconds) const {
    checked(vault_cache_lockbox_password(
        handle_, path.data(), path.size(),
        reinterpret_cast<const std::uint8_t*>(password.data()), password.size(),
        ttl_seconds));
  }
  /** Releases the native resources held by close. */
  void close(const std::string& path) const {
    checked(vault_close_lockbox(handle_, path.data(), path.size()));
  }
  /** Releases the native resources held by all. */
  void close_all() const { checked(vault_close_all(handle_)); }
 private:
  static void checked(bool result) {
    if (!result) throw std::runtime_error(buffer_last_error());
  }
  void* handle_{};
};

}  // namespace revault
