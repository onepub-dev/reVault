import 'dart:ffi' as ffi;
import 'dart:typed_data';
import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/key_export_format.dart';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/revault.dart';
import 'package:revault_api/src/secret_bytes.dart';
import 'package:revault_api/src/wrapped_contact_key.dart';

/// A recipient's shareable encryption identity.
///
/// Import or load this key before granting that recipient lockbox access. It
/// contains no private key material and encrypts content keys that only the
/// matching [ContactKeyPair] can recover.
///
/// Example:
/// ```dart
/// final contact = vault.loadContact('alice');
/// try {
///   lockbox.addContact(contact, 'alice');
/// } finally {
///   contact.dispose();
/// }
/// ```
final class ContactPublicKey extends Owned {
  /// @nodoc
  ContactPublicKey.internal(super.runtime, super.handle);

  /// Exports this public key using [format].
  ///
  /// Pass the returned bytes to [Revault.importContactPublicKey].
  /// Choose PEM for sharing as a file and JWK/JWKS for JSON integrations.
  ///
  /// Example:
  /// ```dart
  /// final pem = contact.export(KeyExportFormat.lockboxPem);
  /// await File('alice-public.pem').writeAsBytes(pem);
  /// ```
  Uint8List export(KeyExportFormat format) =>
      runtime.operations.vaultKeyExportPublic(handle, format.nativeName);

  /// Returns stable fingerprint bytes used to verify this public key.
  ///
  /// Compare a formatted fingerprint with the contact over a trusted channel
  /// before granting access.
  ///
  /// Example:
  /// ```dart
  /// print(revault.formatKeyCrockford(contact.fingerprint()));
  /// ```
  Uint8List fingerprint() => runtime.operations.vaultKeyFingerprint(handle);

  /// Wraps a Lockbox content [key] for the matching private contact identity.
  ///
  /// Most applications use [Lockbox.addContact]. Use this lower-level envelope
  /// operation only when constructing or transporting access records yourself.
  ///
  /// Example:
  /// ```dart
  /// final wrapped = contact.encrypt(contentKey);
  /// try {
  ///   sendEnvelope(wrapped.encryptedBytes());
  /// } finally {
  ///   wrapped.dispose();
  /// }
  /// ```
  WrappedContactKey encrypt(SecretBytes key) => key.withBytes(
    (bytes) => WrappedContactKey.internal(
      runtime,
      runtime.operations.keyContactEncrypt(handle, bytes),
    ),
  );

  /// Releases the native public-key handle; repeated calls have no effect.
  ///
  /// Call this when an imported or loaded key is no longer needed. Disposing a
  /// public key does not remove it from a Vault or revoke existing access.
  ///
  /// Example:
  /// ```dart
  /// final contact = vault.loadContact('alice');
  /// try {
  ///   lockbox.addContact(contact, 'alice');
  /// } finally {
  ///   contact.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.keyContactPublicFree(handle);
      handle = ffi.nullptr;
    }
  }
}
