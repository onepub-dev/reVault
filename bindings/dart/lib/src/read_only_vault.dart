import 'dart:ffi' as ffi;
import 'package:revault_api/src/domain_models.dart';
import 'package:revault_api/src/owned.dart';

/// A restricted view of a local metadata store for discovery and listing.
///
/// Use it when an application needs profile names, contacts, forms, or known
/// Lockbox paths without gaining access to profile signing keys or mutation
/// APIs.
///
/// Example:
/// ```dart
/// final view = Vault.openReadOnly(passphrase: passphrase);
/// try {
///   print(view.listProfileNames());
/// } finally {
///   view.close();
/// }
/// ```
final class ReadOnlyVault extends Owned {
  /// @nodoc
  ReadOnlyVault.internal(super.runtime, super.handle);

  /// Lists profile names without loading private or signing keys.
  ///
  /// Example:
  /// ```dart
  /// final profiles = view.listProfileNames();
  /// ```
  List<String> listProfileNames() =>
      runtime.operations.vaultReadOnlyListProfileNames(handle);

  /// Lists contact names without loading contact key material.
  ///
  /// Example:
  /// ```dart
  /// for (final name in view.listContactNames()) print(name);
  /// ```
  List<String> listContactNames() =>
      runtime.operations.vaultReadOnlyListContactNames(handle);

  /// Lists Vault-wide form aliases without loading form secrets.
  ///
  /// Example:
  /// ```dart
  /// final aliases = view.listFormAliases();
  /// ```
  List<String> listFormAliases() =>
      runtime.operations.vaultReadOnlyListFormAliases(handle);

  /// Lists remembered Lockbox identifiers and host paths.
  ///
  /// Example:
  /// ```dart
  /// final known = view.listKnownLockboxes();
  /// ```
  List<KnownLockbox> listKnownLockboxes() =>
      runtime.operations.vaultReadOnlyListKnownLockboxes(handle);

  /// Wipes decrypted metadata and releases the read-only handle.
  ///
  /// Example:
  /// ```dart
  /// try {
  ///   showProfiles(view.listProfileNames());
  /// } finally {
  ///   view.close();
  /// }
  /// ```
  void close() {
    if (!disposed) {
      runtime.operations.vaultReadOnlyFree(handle);
      handle = ffi.nullptr;
    }
  }

  /// Deprecated alias for [close].
  ///
  /// Example:
  /// ```dart
  /// view.dispose(); // New code uses view.close().
  /// ```
  @Deprecated('Use close().')
  void dispose() => close();
}
