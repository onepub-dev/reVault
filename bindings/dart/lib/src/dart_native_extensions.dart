// Hand-maintained native operations shared by both bundled and explicitly
// loaded carriers. Keep signatures aligned with revault_api.h.

import 'dart:ffi' as ffi;

import 'revault.dart';
import 'revault_native.dart';

const _assetId = 'package:revault_api/src/revault_native.dart';

typedef _LockboxFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);

typedef _ReadOnlyKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Char>,
      ffi.Size,
    );
typedef _ReadOnlyKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Char>,
      int,
    );
typedef _ReadOnlyGenerationNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Char>,
      ffi.Size,
      ffi.Uint16,
    );
typedef _ReadOnlyGenerationDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Char>,
      int,
      int,
    );
typedef _ReadOnlyRememberedNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );
typedef _ReadOnlyRememberedDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );

@ffi.Native<_ReadOnlyKeyNative>(
  symbol: 'vault_read_only_load_private_key',
  assetId: _assetId,
)
external ffi.Pointer<ffi.Void> _readOnlyPrivateKey(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Char> name,
  int length,
);
@ffi.Native<_ReadOnlyGenerationNative>(
  symbol: 'vault_read_only_load_private_key_generation',
  assetId: _assetId,
)
external ffi.Pointer<ffi.Void> _readOnlyPrivateKeyGeneration(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Char> name,
  int length,
  int index,
);
@ffi.Native<_ReadOnlyKeyNative>(
  symbol: 'vault_read_only_load_owner_signing_key',
  assetId: _assetId,
)
external ffi.Pointer<ffi.Void> _readOnlySigningKey(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Char> name,
  int length,
);
@ffi.Native<_ReadOnlyGenerationNative>(
  symbol: 'vault_read_only_load_owner_signing_key_generation',
  assetId: _assetId,
)
external ffi.Pointer<ffi.Void> _readOnlySigningKeyGeneration(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Char> name,
  int length,
  int index,
);
@ffi.Native<_ReadOnlyKeyNative>(
  symbol: 'vault_read_only_load_profile_password',
  assetId: _assetId,
)
external ffi.Pointer<ffi.Void> _readOnlyProfilePassword(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Char> name,
  int length,
);
@ffi.Native<_ReadOnlyRememberedNative>(
  symbol: 'vault_read_only_remembered_password',
  assetId: _assetId,
)
external bool _readOnlyRememberedPassword(
  ffi.Pointer<ffi.Void> handle,
  ffi.Pointer<ffi.Uint8> id,
  int length,
  ffi.Pointer<ffi.Pointer<ffi.Void>> output,
);

@ffi.Native<_LockboxFreeNative>(symbol: 'lockbox_free', assetId: _assetId)
external void _lockboxFree(ffi.Pointer<ffi.Void> handle);

/// @nodoc
ffi.Pointer<ffi.NativeFunction<_LockboxFreeNative>> get lockboxFreeAddress =>
    Revault.runtime.nativeExtensions.lockboxFreeAddress;

typedef _VaultPlatformGetPasswordForNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );

@ffi.Native<_VaultPlatformGetPasswordForNative>(
  symbol: 'dart_vault_platform_get_password_for',
  assetId: _assetId,
)
/// @nodoc
external RevaultBuffer _vaultPlatformGetPasswordForNative(
  ffi.Pointer<ffi.Uint8> pathTo,
  int pathToLength,
  ffi.Pointer<ffi.Uint8> sessionBusAddress,
  int sessionBusAddressLength,
);

/// @nodoc
RevaultBuffer vaultPlatformGetPasswordForNative(
  ffi.Pointer<ffi.Uint8> pathTo,
  int pathToLength,
  ffi.Pointer<ffi.Uint8> sessionBusAddress,
  int sessionBusAddressLength,
) => Revault.runtime.nativeExtensions.vaultPlatformGetPasswordFor(
  pathTo,
  pathToLength,
  sessionBusAddress,
  sessionBusAddressLength,
);

typedef _VaultPlatformGetPasswordForDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );

/// Native extensions share the carrier selected for the generated bindings.
/// @nodoc
final class DartNativeExtensions {
  /// @nodoc
  DartNativeExtensions(this._library);

  final ffi.DynamicLibrary? _library;

  /// @nodoc
  late final _ReadOnlyKeyDart readOnlyPrivateKey = _library == null
      ? _readOnlyPrivateKey
      : _library.lookupFunction<_ReadOnlyKeyNative, _ReadOnlyKeyDart>(
          'vault_read_only_load_private_key',
        );

  /// @nodoc
  late final _ReadOnlyGenerationDart readOnlyPrivateKeyGeneration =
      _library == null
      ? _readOnlyPrivateKeyGeneration
      : _library
            .lookupFunction<_ReadOnlyGenerationNative, _ReadOnlyGenerationDart>(
              'vault_read_only_load_private_key_generation',
            );

  /// @nodoc
  late final _ReadOnlyKeyDart readOnlySigningKey = _library == null
      ? _readOnlySigningKey
      : _library.lookupFunction<_ReadOnlyKeyNative, _ReadOnlyKeyDart>(
          'vault_read_only_load_owner_signing_key',
        );

  /// @nodoc
  late final _ReadOnlyGenerationDart readOnlySigningKeyGeneration =
      _library == null
      ? _readOnlySigningKeyGeneration
      : _library
            .lookupFunction<_ReadOnlyGenerationNative, _ReadOnlyGenerationDart>(
              'vault_read_only_load_owner_signing_key_generation',
            );

  /// @nodoc
  late final _ReadOnlyKeyDart readOnlyProfilePassword = _library == null
      ? _readOnlyProfilePassword
      : _library.lookupFunction<_ReadOnlyKeyNative, _ReadOnlyKeyDart>(
          'vault_read_only_load_profile_password',
        );

  /// @nodoc
  late final _ReadOnlyRememberedDart readOnlyRememberedPassword =
      _library == null
      ? _readOnlyRememberedPassword
      : _library
            .lookupFunction<_ReadOnlyRememberedNative, _ReadOnlyRememberedDart>(
              'vault_read_only_remembered_password',
            );

  /// @nodoc
  late final ffi.Pointer<ffi.NativeFunction<_LockboxFreeNative>>
  lockboxFreeAddress = _library == null
      ? ffi.Native.addressOf(_lockboxFree)
      : _library.lookup<ffi.NativeFunction<_LockboxFreeNative>>('lockbox_free');

  /// @nodoc
  late final _VaultPlatformGetPasswordForDart vaultPlatformGetPasswordFor =
      _library == null
      ? _vaultPlatformGetPasswordForNative
      : _library.lookupFunction<
          _VaultPlatformGetPasswordForNative,
          _VaultPlatformGetPasswordForDart
        >('dart_vault_platform_get_password_for');
}
