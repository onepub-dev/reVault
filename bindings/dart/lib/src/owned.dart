import 'dart:ffi' as ffi;
import 'package:revault_api/src/revault.dart';

/// @nodoc
abstract base class Owned {
  /// @nodoc
  Owned(this.runtime, this.handle);

  /// @nodoc
  final Revault runtime;

  /// @nodoc
  ffi.Pointer<ffi.Void> handle;

  /// @nodoc
  bool get disposed => handle == ffi.nullptr;
}
