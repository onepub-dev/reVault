import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/contact_public_key.dart';
import 'package:revault_api/src/owned.dart';

/// An encrypted content key addressed to one contact.
///
/// [ContactPublicKey.encrypt] creates this value for storage or transfer with a
/// lockbox access record. Only the matching [ContactKeyPair] can decrypt it.
///
/// Example:
/// ```dart
/// final wrapped = recipient.encrypt(contentKey);
/// final recovered = recipientPrivateKey.decrypt(wrapped);
/// wrapped.dispose();
/// ```
final class WrappedContactKey extends Owned {
  /// @nodoc
  WrappedContactKey.internal(super.runtime, super.handle);

  /// Returns the envelope's ephemeral public encapsulation material.
  ///
  /// Use this only when an external protocol stores the envelope components
  /// separately; [encryptedBytes] produces a self-contained encrypted value.
  ///
  /// Example:
  /// ```dart
  /// final ephemeralPublic = wrapped.publicBytes();
  /// ```
  Uint8List publicBytes() => runtime.operations.keyContactWrappedPublic(handle);

  /// Returns the encrypted content-key portion of this envelope.
  ///
  /// It is meaningful only with [publicBytes] and the matching private key.
  ///
  /// Example:
  /// ```dart
  /// final ciphertext = wrapped.ciphertext();
  /// ```
  Uint8List ciphertext() =>
      runtime.operations.keyContactWrappedCiphertext(handle);

  /// Returns the complete serialized envelope for storage or transport.
  ///
  /// Example:
  /// ```dart
  /// await File('alice.key-envelope').writeAsBytes(wrapped.encryptedBytes());
  /// ```
  Uint8List encryptedBytes() =>
      runtime.operations.keyContactWrappedEncrypted(handle);

  /// Wipes and releases the native envelope handle.
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
  void dispose() {
    if (!disposed) {
      runtime.operations.keyContactWrappedFree(handle);
      handle = ffi.nullptr;
    }
  }
}
