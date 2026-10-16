#! /usr/bin/env dcli

import 'dart:io';
import 'package:dcli/dcli.dart';

void run(List<String> command, {String? directory}) {
  final result = Process.runSync(command.first, command.skip(1).toList(),
      workingDirectory: directory);
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0)
    throw StateError('${command.first} failed: ${result.exitCode}');
}

void main(List<String> args) {
  if (args.length != 2)
    throw ArgumentError(
        'Usage: ./tools/test_android_dart.dart DART_PACKAGE DEVICE_ID|build');
  final package = Directory(args[0]).absolute.path;
  final fixture =
      File('bindings/e2e/android_dart/lifecycle_test.dart').absolute;
  final root = Directory.systemTemp.createTempSync('revault-android-flutter-');
  try {
    run([
      'flutter',
      'create',
      '--platforms=android',
      '--project-name=revault_android_probe',
      root.path
    ]);
    File('${root.path}/pubspec.yaml').writeAsStringSync('''
name: revault_android_probe
publish_to: none
environment:
  sdk: '>=3.10.0 <4.0.0'
dependencies:
  flutter:
    sdk: flutter
  revault_api:
    path: '$package'
dev_dependencies:
  flutter_test:
    sdk: flutter
  integration_test:
    sdk: flutter
''');
    Directory('${root.path}/integration_test').createSync();
    fixture.copySync('${root.path}/integration_test/lifecycle_test.dart');
    final gradle = File('${root.path}/android/app/build.gradle.kts');
    gradle.writeAsStringSync(gradle.readAsStringSync().replaceAll(
        'minSdk = flutter.minSdkVersion',
        'minSdk = maxOf(23, flutter.minSdkVersion)'));
    run(['flutter', 'pub', 'get'], directory: root.path);
    if (args[1] == 'build') {
      run([
        'flutter',
        'build',
        'apk',
        '--debug',
        '--target-platform=android-arm64,android-x64',
        '--target=integration_test/lifecycle_test.dart'
      ], directory: root.path);
    } else {
      run([
        'flutter',
        'test',
        'integration_test/lifecycle_test.dart',
        '-d',
        args[1]
      ], directory: root.path);
    }
    print(green('Android Flutter native-assets acceptance passed'));
  } finally {
    root.deleteSync(recursive: true);
  }
}
