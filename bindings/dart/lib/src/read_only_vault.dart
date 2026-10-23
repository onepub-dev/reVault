import 'dart:ffi' as ffi;
import 'dart:convert';
import 'dart:typed_data';
import 'package:ffi/ffi.dart';
import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/domain_models.dart';
import 'package:revault_api/src/exceptions.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/profile_signing_key_pair.dart';
import 'package:revault_api/src/secret_string.dart';

/// A read-only view of a local Vault, including its stored credentials.
///
/// Supplying the Vault passphrase grants access to stored private keys and
/// passwords. The handle keeps a shared file lock until [close]; other readers
/// may open concurrently, while writers wait.
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

  T _withName<T>(String name, T Function(ffi.Pointer<ffi.Char>, int) action) {
    final bytes = Uint8List.fromList(utf8.encode(name));
    final pointer = calloc<ffi.Uint8>(bytes.length);
    try {
      pointer.asTypedList(bytes.length).setAll(0, bytes);
      return action(pointer.cast<ffi.Char>(), bytes.length);
    } finally {
      pointer.asTypedList(bytes.length).fillRange(0, bytes.length, 0);
      bytes.fillRange(0, bytes.length, 0);
      calloc.free(pointer);
    }
  }

  Never _throwLastError() {
    final message = runtime.operations.lastErrorMessage();
    ErrorDetails? details;
    try {
      details = runtime.operations.bufferLastErrorDetails();
    } on Object {
      // Preserve the native message if structured diagnostics are unavailable.
    }
    throw RevaultException(message, details: details);
  }

  ffi.Pointer<ffi.Void> _requireHandle(ffi.Pointer<ffi.Void> result) =>
      result == ffi.nullptr ? _throwLastError() : result;

  SecretString _takeSecret(ffi.Pointer<ffi.Void> secret) {
    final native = runtime.operations.native;
    final length = calloc<ffi.Size>();
    try {
      if (!native.secret_len(secret, length)) _throwLastError();
      final bytes = calloc<ffi.Uint8>(length.value);
      try {
        if (!native.secret_copy(secret, bytes, length.value)) _throwLastError();
        return SecretString.takeUtf8(
          Uint8List.fromList(bytes.asTypedList(length.value)),
        );
      } finally {
        bytes.asTypedList(length.value).fillRange(0, length.value, 0);
        calloc.free(bytes);
      }
    } finally {
      calloc.free(length);
      native.secret_free(secret);
    }
  }

  /// Loads an existing Profile private key; dispose the returned key after use.
  ///
  /// ```dart
  /// final view = Vault.openReadOnly(passphrase: passphrase);
  /// try { final key = view.loadPrivateKey('alice'); key.dispose(); }
  /// finally { view.close(); }
  /// ```
  ContactKeyPair loadPrivateKey(String name) => ContactKeyPair.internal(
    runtime,
    _withName(
      name,
      (pointer, length) => _requireHandle(
        runtime.nativeExtensions.readOnlyPrivateKey(handle, pointer, length),
      ),
    ),
  );

  /// Loads an existing historical Profile private-key generation.
  ///
  /// ```dart
  /// final key = view.loadPrivateKeyGeneration('alice', 1);
  /// try { useKey(key); } finally { key.dispose(); }
  /// ```
  ContactKeyPair loadPrivateKeyGeneration(String name, int index) =>
      ContactKeyPair.internal(
        runtime,
        _withName(
          name,
          (pointer, length) => _requireHandle(
            runtime.nativeExtensions.readOnlyPrivateKeyGeneration(
              handle,
              pointer,
              length,
              index,
            ),
          ),
        ),
      );

  /// Loads an existing Profile signing key. A missing key throws `NotFound`.
  /// The read-only Vault never creates or caches a key.
  ///
  /// ```dart
  /// final signingKey = view.loadProfileSigningKey('alice');
  /// try { lockbox.setOwnerSigningKey(signingKey); }
  /// finally { signingKey.dispose(); }
  /// ```
  ProfileSigningKeyPair loadProfileSigningKey(String name) =>
      ProfileSigningKeyPair.internal(
        runtime,
        _withName(
          name,
          (pointer, length) => _requireHandle(
            runtime.nativeExtensions.readOnlySigningKey(
              handle,
              pointer,
              length,
            ),
          ),
        ),
      );

  /// Loads an existing historical Profile signing-key generation.
  ///
  /// ```dart
  /// final signingKey = view.loadProfileSigningKeyGeneration('alice', 1);
  /// try { verifyGeneration(signingKey); } finally { signingKey.dispose(); }
  /// ```
  ProfileSigningKeyPair loadProfileSigningKeyGeneration(
    String name,
    int index,
  ) => ProfileSigningKeyPair.internal(
    runtime,
    _withName(
      name,
      (pointer, length) => _requireHandle(
        runtime.nativeExtensions.readOnlySigningKeyGeneration(
          handle,
          pointer,
          length,
          index,
        ),
      ),
    ),
  );

  /// Loads an existing password Profile. Close the returned secret after use.
  ///
  /// ```dart
  /// final password = view.loadProfilePassword('server');
  /// try { password.withBytes(sendPassword); } finally { password.close(); }
  /// ```
  SecretString loadProfilePassword(String name) => _takeSecret(
    _withName(
      name,
      (pointer, length) => _requireHandle(
        runtime.nativeExtensions.readOnlyProfilePassword(
          handle,
          pointer,
          length,
        ),
      ),
    ),
  );

  /// Loads a remembered Lockbox password for a 16-byte [lockboxId], if present.
  /// Close the returned secret after use.
  ///
  /// ```dart
  /// final password = view.rememberedPassword(lockboxId);
  /// try { usePassword(password); } finally { password?.close(); }
  /// ```
  SecretString? rememberedPassword(Uint8List lockboxId) {
    if (lockboxId.length != 16) {
      throw ArgumentError.value(
        lockboxId.length,
        'lockboxId',
        'must be 16 bytes',
      );
    }
    final id = calloc<ffi.Uint8>(16);
    final output = calloc<ffi.Pointer<ffi.Void>>();
    try {
      id.asTypedList(16).setAll(0, lockboxId);
      if (!runtime.nativeExtensions.readOnlyRememberedPassword(
        handle,
        id,
        16,
        output,
      )) {
        _throwLastError();
      }
      return output.value == ffi.nullptr ? null : _takeSecret(output.value);
    } finally {
      id.asTypedList(16).fillRange(0, 16, 0);
      calloc.free(id);
      calloc.free(output);
    }
  }

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

  /// Lists aliases without opening their target archives.
  ///
  /// Example:
  /// ```dart
  /// final aliases = vault.listLockboxAliases();
  /// ```
  List<LockboxAlias> listLockboxAliases() =>
      runtime.operations.vaultReadOnlyListLockboxAliases(handle);

  /// Resolves metadata; verify target identity before opening its path.
  ///
  /// Example:
  /// ```dart
  /// final target = vault.resolveLockboxAlias('team');
  /// ```
  KnownLockbox resolveLockboxAlias(String name) =>
      runtime.operations.vaultReadOnlyResolveLockboxAlias(handle, name);

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
