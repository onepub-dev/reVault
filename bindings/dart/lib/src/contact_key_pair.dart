import 'dart:ffi' as ffi;
import 'dart:typed_data';
import 'package:revault_api/src/contact_public_key.dart';
import 'package:revault_api/src/key_export_format.dart';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/revault.dart';
import 'package:revault_api/src/secret_bytes.dart';
import 'package:revault_api/src/wrapped_contact_key.dart';

/// A profile's private and public contact-encryption identity.
///
/// Generate, import, or load this key pair for a profile; distribute its public
/// half to contacts and retain the private half to decrypt content keys they
/// address to that profile.
///
/// Example:
/// ```dart
/// final identity = revault.generateContactKeyPair();
/// try {
///   vault.storePrivateKey('alice', identity);
/// } finally {
///   identity.dispose();
/// }
/// ```
final class ContactKeyPair extends Owned {
  /// @nodoc
  ContactKeyPair.internal(super.runtime, super.handle);

  /// Returns the algorithm-specific public bytes paired with this identity.
  ///
  /// Use [publicKey] for normal API work; use these bytes for compact binary
  /// storage paired with [Revault.contactPublicKeyFromBytes].
  ///
  /// Example:
  /// ```dart
  /// final publicBytes = identity.publicBytes();
  /// final publicKey = revault.contactPublicKeyFromBytes(publicBytes);
  /// ```
  Uint8List publicBytes() => runtime.operations.keyContactPublic(handle);

  /// Returns the native private-key record for controlled binary backup.
  ///
  /// Anyone holding this value can assume the identity. Use [export] when a
  /// portable, self-describing representation is required.
  ///
  /// Example:
  /// ```dart
  /// final record = identity.privateRecord();
  /// final restored = revault.contactKeyPairFromPrivate(record);
  /// ```
  Uint8List privateRecord() => runtime.operations.keyContactPrivate(handle);

  /// Creates an independently owned public half of this identity.
  ///
  /// Dispose both handles separately.
  ///
  /// Example:
  /// ```dart
  /// final publicKey = identity.publicKey();
  /// try {
  ///   vault.storeContact('alice', publicKey);
  /// } finally {
  ///   publicKey.dispose();
  /// }
  /// ```
  ContactPublicKey publicKey() =>
      runtime.contactPublicKeyFromBytes(publicBytes());

  /// Exports this private key using [format].
  ///
  /// Anyone holding the result can open lockboxes granted to this identity.
  /// Pass it to [Revault.importContactKeyPair] to import it again.
  ///
  /// Example:
  /// ```dart
  /// final pem = identity.export(KeyExportFormat.lockboxPem);
  /// await secureBackup.writeAsBytes(pem);
  /// ```
  Uint8List export(KeyExportFormat format) =>
      runtime.operations.vaultKeyExportPrivate(handle, format.nativeName);

  /// Decrypts a content key addressed to this identity in [wrapped].
  ///
  /// Use this when an application stores and processes the contact-key envelope
  /// itself. [Lockbox.open] accepts a contact key pair and processes the
  /// archive's access envelope as part of opening the Lockbox.
  ///
  /// Example:
  /// ```dart
  /// final contentKey = identity.decrypt(wrapped);
  /// try {
  ///   final box = Lockbox.open(path, contentKey: contentKey);
  /// } finally {
  ///   contentKey.fillRange(0, contentKey.length, 0);
  /// }
  /// ```
  SecretBytes decrypt(WrappedContactKey wrapped) => SecretBytes.take(
    runtime.operations.keyContactDecrypt(handle, wrapped.handle),
  );

  /// Wipes and releases the native private-key handle.
  ///
  /// This does not delete a copy previously stored in a Vault.
  ///
  /// Example:
  /// ```dart
  /// final identity = vault.loadPrivateKey('alice');
  /// try {
  ///   final box = Lockbox.open(path, contact: identity);
  /// } finally {
  ///   identity.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.keyContactFree(handle);
      handle = ffi.nullptr;
    }
  }
}
