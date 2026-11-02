#! /usr/bin/env dcli

import 'dart:async';
import 'dart:io';
import 'package:dcli/dcli.dart';

Future<void> run(List<String> command,
    {String? directory, Duration timeout = const Duration(minutes: 20)}) async {
  stdout.writeln('+ ${command.join(' ')}');
  final process = await Process.start(command.first, command.skip(1).toList(),
      workingDirectory: directory, mode: ProcessStartMode.inheritStdio);
  int status;
  try {
    status = await process.exitCode.timeout(timeout);
  } on TimeoutException {
    process.kill(ProcessSignal.sigterm);
    try {
      await process.exitCode.timeout(const Duration(seconds: 10));
    } on TimeoutException {
      process.kill(ProcessSignal.sigkill);
      await process.exitCode;
    }
    throw TimeoutException('${command.first} exceeded $timeout', timeout);
  }
  if (status != 0) throw StateError('${command.first} failed: $status');
}

Future<void> main(List<String> args) async {
  if (args.length != 2)
    throw ArgumentError(
        'Usage: ./tools/test_android_dart.dart DART_PACKAGE DEVICE_ID|build');
  final package = Directory(args[0]).absolute.path;
  final fixture =
      File('bindings/e2e/android_dart/lifecycle_test.dart').absolute;
  final root = Directory.systemTemp.createTempSync('revault-android-flutter-');
  try {
    await run([
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
    await run(['flutter', 'pub', 'get'], directory: root.path);
    if (args[1] == 'build') {
      await run([
        'flutter',
        'build',
        'apk',
        '--debug',
        '--target-platform=android-arm64,android-x64',
        '--target=integration_test/lifecycle_test.dart'
      ], directory: root.path);
    } else {
      await run([
        'flutter',
        'test',
        'integration_test/lifecycle_test.dart',
        '--reporter=expanded',
        '-d',
        args[1]
      ], directory: root.path);
    }
    print(green('Android Flutter native-assets acceptance passed'));
  } catch (error) {
    if (args[1] != 'build') {
      try {
        await run(['adb', '-s', args[1], 'logcat', '-d', '-t', '500'],
            timeout: const Duration(seconds: 30));
      } catch (diagnosticError) {
        stderr.writeln('Could not collect Android logcat: $diagnosticError');
      }
    }
    rethrow;
  } finally {
    try {
      root.deleteSync(recursive: true);
    } catch (error) {
      stderr.writeln('Could not remove Android acceptance fixture: $error');
    }
  }
}
