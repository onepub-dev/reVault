import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:test/test.dart';

import '../hook/build.dart' as hook;

void main() {
  late Directory package;
  late Directory previous;

  setUp(() {
    previous = Directory.current;
    package = Directory.systemTemp.createTempSync('revault-hook-');
    File(
      '${package.path}/pubspec.yaml',
    ).writeAsStringSync('name: revault_api\n');
    Directory.current = package;
  });
  tearDown(() {
    Directory.current = previous;
    package.deleteSync(recursive: true);
  });

  for (final (architecture, target) in [
    (Architecture.arm64, 'android-aarch64'),
    (Architecture.x64, 'android-x86_64'),
  ]) {
    test('bundles the $target carrier with the FFI asset identity', () async {
      final carrier = File(
        '${package.path}/lib/src/native/$target/librevault_api.so',
      );
      carrier.parent.createSync(recursive: true);
      carrier.writeAsBytesSync([0]);
      await testCodeBuildHook(
        mainMethod: hook.main,
        targetOS: OS.android,
        targetArchitecture: architecture,
        targetAndroidNdkApi: 23,
        check: (input, output) {
          final asset = output.assets.code.single;
          expect(asset.file, carrier.uri);
          expect(asset.id, 'package:revault_api/src/revault_native.dart');
          expect(asset.linkMode, isA<DynamicLoadingBundled>());
        },
      );
    });
  }

  test('rejects ARM32 rather than using a 64-bit carrier', () async {
    await expectLater(
      testCodeBuildHook(
        mainMethod: hook.main,
        targetOS: OS.android,
        targetArchitecture: Architecture.arm,
        targetAndroidNdkApi: 23,
        check: (_, _) {},
      ),
      throwsA(
        isA<UnsupportedError>().having(
          (error) => error.message,
          'message',
          contains('--target-platform android-arm64,android-x64'),
        ),
      ),
    );
  });

  test('rejects Android versions below the carrier API floor', () async {
    await expectLater(
      testCodeBuildHook(
        mainMethod: hook.main,
        targetOS: OS.android,
        targetArchitecture: Architecture.arm64,
        targetAndroidNdkApi: 22,
        check: (_, _) {},
      ),
      throwsA(isA<UnsupportedError>()),
    );
  });

  test('fails when the published Android carrier is missing', () async {
    await expectLater(
      testCodeBuildHook(
        mainMethod: hook.main,
        targetOS: OS.android,
        targetArchitecture: Architecture.arm64,
        targetAndroidNdkApi: 23,
        check: (_, _) {},
      ),
      throwsA(isA<StateError>()),
    );
  });
}
