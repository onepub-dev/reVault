// Package-private native-library discovery.
// ignore_for_file: public_member_api_docs

import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:isolate';

Future<ffi.DynamicLibrary> loadNativeLibrary({
  String? nativeLibraryPath,
}) async {
  if (nativeLibraryPath != null) {
    if (nativeLibraryPath.isEmpty) {
      throw ArgumentError.value(
        nativeLibraryPath,
        'nativeLibraryPath',
        'must not be empty',
      );
    }
    return ffi.DynamicLibrary.open(nativeLibraryPath);
  }

  final override = Platform.environment['REVAULT_LIBRARY'];
  if (override != null && override.isNotEmpty) {
    return ffi.DynamicLibrary.open(override);
  }

  final target = _target();
  final library = _libraryName();
  Uri? uri;
  try {
    uri = await Isolate.resolvePackageUri(
      Uri.parse('package:revault_api/src/native/$target/$library'),
    );
  } catch (error) {
    // Flutter desktop does not provide package-URI resolution. Continue to the
    // executable-relative carrier lookup, or let the final error explain how
    // to supply an explicit path.
    if (error is! UnsupportedError && error is! UnimplementedError) {
      rethrow;
    }
  }
  if (uri != null && uri.scheme == 'file') {
    final bundled = File.fromUri(uri);
    if (bundled.existsSync()) {
      return ffi.DynamicLibrary.open(bundled.path);
    }
  }
  // AOT executables do not retain a pub package URI map. Package installers
  // therefore place the carrier beside the executable in native/<target>.
  final executableCarrier = File(
    '${File(Platform.resolvedExecutable).parent.path}'
    '${Platform.pathSeparator}native${Platform.pathSeparator}$target'
    '${Platform.pathSeparator}$library',
  );
  if (executableCarrier.existsSync()) {
    return ffi.DynamicLibrary.open(executableCarrier.path);
  }
  throw StateError(
    'revault-api native carrier is missing for $target; '
    'pass nativeLibraryPath to Revault.load() or set REVAULT_LIBRARY',
  );
}

String _target() {
  final architecture = switch (ffi.Abi.current()) {
    ffi.Abi.linuxX64 || ffi.Abi.macosX64 || ffi.Abi.windowsX64 => 'x86_64',
    ffi.Abi.linuxArm64 ||
    ffi.Abi.macosArm64 ||
    ffi.Abi.windowsArm64 => 'aarch64',
    _ => throw UnsupportedError('Unsupported native ABI: ${ffi.Abi.current()}'),
  };
  return switch (Platform.operatingSystem) {
    'linux' => 'linux-$architecture-gnu',
    'macos' => 'macos-$architecture',
    'windows' => 'windows-$architecture-msvc',
    _ => throw UnsupportedError(
      'Unsupported operating system: ${Platform.operatingSystem}',
    ),
  };
}

String _libraryName() => switch (Platform.operatingSystem) {
  'linux' => 'librevault_api.so',
  'macos' => 'librevault_api.dylib',
  'windows' => 'revault_api.dll',
  _ => throw UnsupportedError(
    'Unsupported operating system: ${Platform.operatingSystem}',
  ),
};
