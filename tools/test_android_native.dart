#! /usr/bin/env dcli

import 'dart:io';
import 'package:dcli/dcli.dart';

void run(List<String> command) {
  final result = Process.runSync(command.first, command.skip(1).toList());
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0)
    throw StateError('${command.first} failed: ${result.exitCode}');
}

void main(List<String> args) {
  if ((args.length != 3 && args.length != 4) ||
      !['aarch64', 'x86_64'].contains(args[1])) {
    throw ArgumentError(
        'Usage: ./tools/test_android_native.dart ARCHIVE aarch64|x86_64 DEVICE_ID|build [PAGE_SIZE]');
  }
  final sdk = Platform.environment['ANDROID_HOME'] ??
      Platform.environment['ANDROID_SDK_ROOT'];
  final ndk =
      Platform.environment['ANDROID_NDK_HOME'] ?? '$sdk/ndk/28.2.13676358';
  final repository = Directory.current.path;
  final root = Directory.systemTemp.createTempSync('revault-android-sdk-');
  final prefix = '${root.path}/sdk';
  final remote =
      '/data/local/tmp/${root.uri.pathSegments.where((part) => part.isNotEmpty).last}';
  final abi = args[1] == 'aarch64' ? 'arm64-v8a' : 'x86_64';
  final options = [
    '-DCMAKE_TOOLCHAIN_FILE=$ndk/build/cmake/android.toolchain.cmake',
    '-DANDROID_ABI=$abi',
    '-DANDROID_PLATFORM=android-23',
    '-DANDROID_STL=c++_static',
    '-DCMAKE_PREFIX_PATH=$prefix',
    '-DCMAKE_FIND_ROOT_PATH=$prefix',
    '-DCMAKE_BUILD_TYPE=Release',
  ];
  void adb(List<String> command) => run(['adb', '-s', args[2], ...command]);
  try {
    run([
      'cargo',
      'run',
      '--locked',
      '--manifest-path',
      'rust/Cargo.toml',
      '-p',
      'revault_tooling',
      '--',
      'internal',
      'install-native',
      '--archive',
      args[0],
      '--prefix',
      prefix
    ]);
    run([
      'cmake',
      '-S',
      '$prefix/cpp',
      '-B',
      '${root.path}/cpp',
      ...options,
      '-DCMAKE_INSTALL_PREFIX=$prefix'
    ]);
    run(['cmake', '--build', '${root.path}/cpp', '--parallel', '2']);
    run(['cmake', '--install', '${root.path}/cpp']);
    for (final language in ['c', 'cpp']) {
      run([
        'cmake',
        '-S',
        '$repository/bindings/e2e/$language',
        '-B',
        '${root.path}/$language-test',
        ...options
      ]);
      run([
        'cmake',
        '--build',
        '${root.path}/$language-test',
        '--parallel',
        '2'
      ]);
    }
    if (args[2] != 'build') {
      if (args.length == 4) {
        final pageSize = Process.runSync(
            'adb', ['-s', args[2], 'shell', 'getconf', 'PAGE_SIZE']);
        if (pageSize.exitCode != 0 || '${pageSize.stdout}'.trim() != args[3]) {
          throw StateError(
              'Android page size differs from ${args[3]}: ${pageSize.stdout} ${pageSize.stderr}');
        }
      }
      adb(['shell', 'mkdir', '-p', remote]);
      adb(['push', '$prefix/lib/librevault_api.so', '$remote/']);
      for (final language in ['c', 'cpp']) {
        final binary = 'revault_${language}_conformance';
        adb(['push', '${root.path}/$language-test/$binary', '$remote/']);
        for (final mode in ['--loader-lifecycle', '--android-core']) {
          adb([
            'shell',
            'LD_LIBRARY_PATH=$remote',
            'TMPDIR=$remote',
            'REVAULT_E2E_ARTIFACT_DIR=$remote/artifacts',
            '$remote/$binary',
            mode
          ]);
        }
      }
      for (final (consumer, producer) in [('c', 'cpp'), ('cpp', 'c')]) {
        adb([
          'shell',
          'LD_LIBRARY_PATH=$remote',
          'TMPDIR=$remote',
          'REVAULT_E2E_ARTIFACT_DIR=$remote/artifacts',
          '$remote/revault_${consumer}_conformance',
          '--interop',
          producer
        ]);
      }
    }
    print(green('Android C/C++ installed SDK acceptance passed: $abi'));
  } finally {
    if (args[2] != 'build') adb(['shell', 'rm', '-rf', remote]);
    root.deleteSync(recursive: true);
  }
}
