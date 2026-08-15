import 'dart:ffi' as ffi;
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/owned.dart';

/// The shareable half of a Vault Profile's signing identity.
///
/// A Profile can become a Lockbox owner by assigning its signing key with
/// [Lockbox.setOwnerSigningKey]. Contacts may carry this public key so readers
/// can verify that owner's revisions, but a Contact cannot sign. This object
/// contains no private signing material.
///
/// Example:
/// ```dart
/// final owner = signingKey.publicKey();
/// vault.storeContactSigningKey('alice', owner);
/// owner.dispose();
/// ```
final class ProfileSigningPublicKey extends Owned {
  /// @nodoc
  ProfileSigningPublicKey.internal(super.runtime, super.handle);

  /// Releases the native verification-key handle.
  ///
  /// Example:
  /// ```dart
  /// final owner = vault.loadContactSigningKey('alice');
  /// try {
  ///   // Use owner for verification metadata.
  /// } finally {
  ///   owner.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.keySigningPublicFree(handle);
      handle = ffi.nullptr;
    }
  }
}
