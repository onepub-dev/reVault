import 'dart:convert';
import 'dart:typed_data';
import 'package:revault_api/src/secret_bytes.dart';

/// An owning UTF-8 secret used for Vault passphrases and Lockbox passwords.
///
/// Use [takeUtf8] when input is already available as bytes because it transfers
/// ownership of storage that this object can wipe. [fromString] cannot erase
/// the immutable source [String] held by the Dart VM. Call [close] as soon as
/// the passphrase or password is no longer needed.
///
/// Example:
/// ```dart
/// final passphrase = SecretString.fromString(promptedValue);
/// try {
///   final vault = Vault.open(passphrase: passphrase);
/// } finally {
///   passphrase.close();
/// }
/// ```
final class SecretString {
  /// UTF-8 encodes [plaintext] into a wipeable owned buffer.
  ///
  /// The original Dart String cannot be wiped. Use [takeUtf8] when a password
  /// input component can supply owned bytes directly.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString('correct horse battery staple');
  /// ```
  SecretString.fromString(String plaintext)
    : _secret = SecretBytes.take(Uint8List.fromList(utf8.encode(plaintext)));

  /// Copies UTF-8 [bytes] into a new wipeable buffer.
  ///
  /// The caller must wipe its original byte list separately.
  ///
  /// Example:
  /// ```dart
  /// final passphrase = SecretString.fromUtf8(inputBytes);
  /// inputBytes.fillRange(0, inputBytes.length, 0);
  /// ```
  SecretString.fromUtf8(Uint8List bytes) : _secret = SecretBytes.copyOf(bytes);

  /// Takes ownership of UTF-8 [bytes] without making another Dart copy.
  ///
  /// Do not access [bytes] afterward; [close] will overwrite it.
  ///
  /// Example:
  /// ```dart
  /// final passphrase = SecretString.takeUtf8(await passwordPrompt.readUtf8());
  /// ```
  SecretString.takeUtf8(Uint8List bytes) : _secret = SecretBytes.take(bytes);

  final SecretBytes _secret;

  /// Whether [close] has already wiped this secret.
  ///
  /// Example:
  /// ```dart
  /// passphrase.close();
  /// assert(passphrase.isClosed);
  /// ```
  bool get isClosed => _secret.isClosed;

  /// Number of UTF-8 bytes, or zero after [close].
  ///
  /// Example:
  /// ```dart
  /// if (password.length == 0) throw ArgumentError('Password is empty');
  /// ```
  int get length => _secret.length;

  /// Provides scoped access to the UTF-8 bytes for an external integration.
  ///
  /// reVault methods accept [SecretString] directly. Do not retain or modify
  /// the supplied list.
  ///
  /// Example:
  /// ```dart
  /// password.withBytes((bytes) => externalPasswordApi(bytes));
  /// ```
  T withBytes<T>(T Function(Uint8List bytes) action) =>
      _secret.withBytes(action);

  /// Overwrites the UTF-8 buffer and makes this object unusable.
  ///
  /// Example:
  /// ```dart
  /// try {
  ///   Vault.open(passphrase: passphrase).close();
  /// } finally {
  ///   passphrase.close();
  /// }
  /// ```
  void close() => _secret.close();

  @override
  /// Returns lifecycle metadata without revealing the secret text.
  ///
  /// Example:
  /// ```dart
  /// print(password); // SecretString(redacted)
  /// ```
  String toString() =>
      isClosed ? 'SecretString(closed)' : 'SecretString(redacted)';
}
