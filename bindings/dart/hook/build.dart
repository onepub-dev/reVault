import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

const _assetName = 'src/revault_native.dart';

Future<void> main(List<String> arguments) async {
  await build(arguments, (input, output) async {
    if (!input.config.buildCodeAssets) return;

    final target = _target(input.config.code);
    final carrier = input.packageRoot.resolve(
      'lib/src/native/$target/${_libraryName(input.config.code.targetOS)}',
    );
    if (!File.fromUri(carrier).existsSync()) {
      throw StateError(
        'Revault native carrier is missing: $carrier. '
        'The published package must include its target carrier under '
        'lib/src/native/$target.',
      );
    }
    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: _assetName,
        file: carrier,
        linkMode: DynamicLoadingBundled(),
      ),
    );
  });
}

String _target(CodeConfig code) {
  if (code.targetOS == OS.android && code.android.targetNdkApi < 23) {
    throw UnsupportedError('Revault requires Android API 23 or newer.');
  }
  final architecture = switch (code.targetArchitecture) {
    Architecture.x64 => 'x86_64',
    Architecture.arm64 => 'aarch64',
    final value => throw UnsupportedError(
      code.targetOS == OS.android
          ? 'Unsupported Revault Android architecture: ${value.name}. '
                'Android supports arm64 and x64 only. Build with '
                'flutter build apk --target-platform android-arm64,android-x64.'
          : 'Unsupported Revault architecture: ${value.name}',
    ),
  };
  return switch (code.targetOS) {
    OS.android => 'android-$architecture',
    OS.linux => 'linux-$architecture-gnu',
    OS.macOS => 'macos-$architecture',
    OS.windows => 'windows-$architecture-msvc',
    final value => throw UnsupportedError(
      'Unsupported Revault operating system: ${value.name}',
    ),
  };
}

String _libraryName(OS os) => switch (os) {
  OS.linux || OS.android => 'librevault_api.so',
  OS.macOS => 'librevault_api.dylib',
  OS.windows => 'revault_api.dll',
  final value => throw UnsupportedError(
    'Unsupported Revault operating system: ${value.name}',
  ),
};
