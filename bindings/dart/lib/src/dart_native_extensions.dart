// Dart-specific native operations that are intentionally outside the shared,
// generated C ABI surface.

import 'dart:ffi' as ffi;

import 'revault_native.dart';

const _assetId = 'package:revault_api/revault_api.dart';

typedef _LockboxFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);

@ffi.Native<_LockboxFreeNative>(symbol: 'lockbox_free', assetId: _assetId)
external void _lockboxFree(ffi.Pointer<ffi.Void> handle);

/// @nodoc
ffi.Pointer<ffi.NativeFunction<_LockboxFreeNative>> get lockboxFreeAddress =>
    ffi.Native.addressOf(_lockboxFree);

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
external RevaultBuffer vaultPlatformGetPasswordForNative(
  ffi.Pointer<ffi.Uint8> pathTo,
  int pathToLength,
  ffi.Pointer<ffi.Uint8> sessionBusAddress,
  int sessionBusAddressLength,
);
