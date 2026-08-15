import 'dart:ffi' as ffi;
import 'dart:typed_data';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/profile_signing_public_key.dart';

/// A Vault Profile's private and public signing identity.
///
/// Supply it to [Lockbox.setOwnerSigningKey], or as `signingKey` while opening
/// a mutable Lockbox. That Profile then occupies the Lockbox owner role and
/// later readers can authenticate its revisions with the corresponding
/// [ProfileSigningPublicKey].
///
/// Example:
/// ```dart
/// final signingKey = revault.generateProfileSigningKeyPair();
/// final box = Lockbox.create(path, contentKey: key, signingKey: signingKey);
/// ```
final class ProfileSigningKeyPair extends Owned {
  /// @nodoc
  ProfileSigningKeyPair.internal(super.runtime, super.handle);

  /// Returns the canonical public bytes paired with this signing identity.
  ///
  /// Example:
  /// ```dart
  /// final publicKey = revault.profileSigningPublicKeyFromBytes(key.publicBytes());
  /// ```
  Uint8List publicBytes() => runtime.operations.keySigningPublic(handle);

  /// Returns the private signing-key record for secure binary backup.
  ///
  /// Anyone holding this record can authorize revisions as the owner.
  ///
  /// Example:
  /// ```dart
  /// final record = signingKey.privateRecord();
  /// final restored = revault.profileSigningKeyPairFromPrivate(record);
  /// ```
  Uint8List privateRecord() => runtime.operations.keySigningPrivate(handle);

  /// Creates an independently owned public verification-key handle.
  ///
  /// Example:
  /// ```dart
  /// final publicKey = signingKey.publicKey();
  /// try {
  ///   vault.storeContactSigningKey('alice', publicKey);
  /// } finally {
  ///   publicKey.dispose();
  /// }
  /// ```
  ProfileSigningPublicKey publicKey() =>
      runtime.profileSigningPublicKeyFromBytes(publicBytes());

  /// Wipes and releases the native signing-key handle.
  ///
  /// Example:
  /// ```dart
  /// final signingKey = revault.generateProfileSigningKeyPair();
  /// try {
  ///   lockbox.setOwnerSigningKey(signingKey);
  /// } finally {
  ///   signingKey.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.keySigningFree(handle);
      handle = ffi.nullptr;
    }
  }
}
