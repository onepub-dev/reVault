#! /usr/bin/env dcli

import 'dart:io';

import 'package:dcli/dcli.dart';

// API 23 is the native carrier floor; Flutter may require a higher minimum.
void main(List<String> arguments) {
  if (arguments.length != 1 ||
      !['aarch64', 'x86_64'].contains(arguments.single)) {
    throw ArgumentError(
        'Usage: ./tools/build_android_native.dart aarch64|x86_64');
  }
  final arch = arguments.single;
  final target = '$arch-linux-android';
  final sdk = Platform.environment['ANDROID_HOME'] ??
      Platform.environment['ANDROID_SDK_ROOT'];
  final ndk =
      Platform.environment['ANDROID_NDK_HOME'] ?? '$sdk/ndk/28.2.13676358';
  final bin = '$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin';
  final compiler = '$bin/${target}23-clang';
  if (!File(compiler).existsSync()) {
    throw StateError(
        'Install Android NDK 28.2.13676358 or set ANDROID_NDK_HOME');
  }
  final environment = {
    'CC_${target.replaceAll('-', '_')}': compiler,
    'AR_${target.replaceAll('-', '_')}': '$bin/llvm-ar',
    'CARGO_TARGET_${target.replaceAll('-', '_').toUpperCase()}_LINKER':
        compiler,
    'CARGO_TARGET_${target.replaceAll('-', '_').toUpperCase()}_RUSTFLAGS':
        '-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-soname,librevault_api.so -C link-arg=-Wl,--no-undefined',
  };
  final result = Process.runSync(
      'cargo',
      [
        'build',
        '--locked',
        '--release',
        '--target',
        target,
        '-p',
        'revault_bindings',
      ],
      workingDirectory: 'rust',
      environment: environment);
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0) exit(result.exitCode);
  final library = 'rust/target/$target/release/librevault_api.so';
  final headers = Process.runSync('$bin/llvm-readelf', ['-lW', library]);
  if (headers.exitCode != 0) throw StateError('${headers.stderr}');
  final loads = '${headers.stdout}'
      .split('\n')
      .where((line) => line.trimLeft().startsWith('LOAD'));
  if (loads.isEmpty ||
      loads.any((line) =>
          int.parse(line.trim().split(RegExp(r'\s+')).last.substring(2),
              radix: 16) <
          16384)) {
    throw StateError(
        'Android carrier does not have 16 KB aligned ELF segments');
  }
  final dynamicSection = Process.runSync('$bin/llvm-readelf', ['-d', library]);
  if (dynamicSection.exitCode != 0 ||
      !'${dynamicSection.stdout}'
          .contains('Library soname: [librevault_api.so]')) {
    throw StateError('Android carrier is missing its relocatable SONAME');
  }
  print(green('Built API 23, 16 KB aligned Android carrier: $library'));
}
