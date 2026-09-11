import 'dart:io';

import 'package:test/test.dart';

void main() {
  test(
    'relocated executable uses explicit and inherited native overrides',
    () async {
      final root = Directory.systemTemp.createTempSync('revault-bundle-test-');
      final environment = Map<String, String>.of(Platform.environment)
        ..remove('REVAULT_LIBRARY');
      Future<void> run(
        String executable,
        List<String> arguments, {
        Map<String, String>? env,
      }) async {
        final result = await Process.run(
          executable,
          arguments,
          environment: env ?? environment,
          includeParentEnvironment: false,
        );
        expect(
          result.exitCode,
          0,
          reason: '${result.stdout}\n${result.stderr}',
        );
      }

      try {
        await run(Platform.resolvedExecutable, [
          'build',
          'cli',
          '--target',
          'test/fixtures/native_override.dart',
          '--output',
          root.path,
        ]);
        final suffix = Platform.isWindows ? '.exe' : '';
        final executable = '${root.path}/bundle/bin/native_override$suffix';
        await run(executable, []);
        final library = Directory(
          '${root.path}/bundle/lib',
        ).listSync().whereType<File>().single;
        // Move the executable away from ../lib, reproducing the user's install.
        final moved = File(executable).renameSync('${root.path}/probe$suffix');
        await run(
          moved.path,
          [library.path],
          env: {
            ...environment,
            'REVAULT_LIBRARY': '${root.path}/missing-library',
          },
        );
        await run(
          moved.path,
          [],
          env: {...environment, 'REVAULT_LIBRARY': library.path},
        );
      } finally {
        root.deleteSync(recursive: true);
      }
    },
    timeout: const Timeout(Duration(minutes: 3)),
  );
}
