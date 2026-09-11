// Dart-specific native operations that are intentionally outside the shared,
// generated C ABI surface.

import 'dart:ffi' as ffi;

import 'revault.dart';
import 'revault_native.dart';

const _assetId = 'package:revault_api/src/revault_native.dart';

typedef _LockboxFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);

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
