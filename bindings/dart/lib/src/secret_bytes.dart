import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

/// An owning, wipeable buffer for secret binary values such as content keys.
///
/// Use this instead of a bare [Uint8List] when bytes must have an explicit
/// lifetime. [close] overwrites the owned buffer and is safe to call more than
/// once. This is best-effort protection: the Dart VM, operating system, and
/// native calls may still create transient copies outside Dart's control.
///
/// Example:
/// ```dart
/// final contentKey = SecretBytes.random(32);
/// try {
///   final box = Lockbox.open(path, contentKey: contentKey);
/// } finally {
///   contentKey.close();
/// }
/// ```
final class SecretBytes {
  /// Copies [bytes] into a new independently owned secret buffer.
  ///
  /// Choose this when the caller must retain [bytes]. The caller remains
  /// responsible for wiping its original copy.
  ///
  /// Example:
  /// ```dart
  /// final secret = SecretBytes.copyOf(sourceBytes);
  /// sourceBytes.fillRange(0, sourceBytes.length, 0);
  /// ```
  SecretBytes.copyOf(Uint8List bytes) : _bytes = Uint8List.fromList(bytes);

  /// UTF-8 encodes [plaintext] into a new owned secret buffer.
  ///
  /// Use this for textual secret variables or form fields.
  ///
  /// **Security limitation:** Dart strings are immutable. Closing this
  /// [SecretBytes] wipes the UTF-8 buffer created by this constructor, but it
  /// cannot wipe [plaintext]. The Dart VM may retain that source string in
  /// memory for an indeterminate period. For password prompts and other
  /// sensitive input, obtain an owned UTF-8 [Uint8List] and pass it to [take]
  /// so the same input buffer can be wiped by [close].
  ///
  /// Example:
  /// ```dart
  /// final token = SecretBytes.fromString('secret');
  /// try {
  ///   lockbox.setSecretVariable('token', token);
  /// } finally {
  ///   token.close();
  /// }
  /// ```
  SecretBytes.fromString(String plaintext)
    : _bytes = Uint8List.fromList(utf8.encode(plaintext));

  /// Takes ownership of [bytes] without copying it.
  ///
  /// Do not read or modify [bytes] after this call. Closing the returned object
  /// overwrites that same list.
  ///
  /// Example:
  /// ```dart
  /// final secret = SecretBytes.take(await readKeyBytes());
  /// try {
  ///   useContentKey(secret);
  /// } finally {
  ///   secret.close();
  /// }
  /// ```
  SecretBytes.take(Uint8List bytes) : _bytes = bytes;

  /// Creates [length] cryptographically random bytes.
  ///
  /// Use `SecretBytes.random(32)` for a new 256-bit Lockbox content key.
  ///
  /// Example:
  /// ```dart
  /// final contentKey = SecretBytes.random(32);
  /// ```
  factory SecretBytes.random(int length) {
    if (length <= 0) throw ArgumentError.value(length, 'length');
    final random = Random.secure();
    return SecretBytes.take(
      Uint8List.fromList(List.generate(length, (_) => random.nextInt(256))),
    );
  }

  Uint8List? _bytes;

  /// Whether [close] has already wiped this secret.
  ///
  /// Example:
  /// ```dart
  /// secret.close();
  /// assert(secret.isClosed);
  /// ```
  bool get isClosed => _bytes == null;

  /// Number of bytes in this secret, or zero after [close].
  ///
  /// Example:
  /// ```dart
  /// if (contentKey.length != 32) throw ArgumentError('Expected 32 bytes');
  /// ```
  int get length => _bytes?.length ?? 0;

  /// Provides temporary access to the owned bytes while running [action].
  ///
  /// The list must not be retained or modified. This is primarily an
  /// integration escape hatch; reVault methods accept [SecretBytes] directly.
  /// Calling it after [close] throws [StateError].
  ///
  /// Example:
  /// ```dart
  /// final digest = secret.withBytes((bytes) => sha256.convert(bytes));
  /// ```
  T withBytes<T>(T Function(Uint8List bytes) action) {
    final bytes = _bytes;
    if (bytes == null) throw StateError('SecretBytes has been closed.');
    return action(bytes);
  }

  /// Overwrites the owned bytes and makes this object unusable.
  ///
  /// Call it in `finally`; repeated calls are harmless.
  ///
  /// Example:
  /// ```dart
  /// final key = SecretBytes.random(32);
  /// try {
  ///   Lockbox.create(path, contentKey: key).close();
  /// } finally {
  ///   key.close();
  /// }
  /// ```
  void close() {
    final bytes = _bytes;
    if (bytes == null) return;
    bytes.fillRange(0, bytes.length, 0);
    _bytes = null;
  }

  @override
  /// Returns only lifecycle and length metadata; it never reveals key bytes.
  ///
  /// Example:
  /// ```dart
  /// print(secret); // SecretBytes(32 bytes)
  /// ```
  String toString() =>
      isClosed ? 'SecretBytes(closed)' : 'SecretBytes($length bytes)';
}
