#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const worktree = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const rust = '$worktree/rust';
String get variant => Platform.environment['INTEGRATION_VARIANT'] ?? 'default';
String get evidence => '$worktree/docs/evidence/secure-arena-reclamation-2026-10-12/raw/integration/$variant';
String get cargoOutput => '$evidence/build.stdout.jsonl';

Never fail(String message) => throw StateError(message);
void require(bool condition, String message) {
  if (!condition) fail(message);
}

String sha256(String path) {
  final result = Process.runSync('/usr/bin/sha256sum', ['--', path]);
  require(result.exitCode == 0, 'sha256sum failed for $path');
  return (result.stdout as String).split(RegExp(r'\s+')).first;
}

ProcessResult checked(String executable, List<String> arguments, String cwd) {
  final result = Process.runSync(executable, arguments,
      workingDirectory: cwd,
      environment: {
        'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011',
        'NO_COLOR': '1',
        'TERM': 'dumb',
      },
      includeParentEnvironment: true);
  require(result.exitCode == 0,
      'command failed: $executable ${arguments.join(' ')}');
  return result;
}

void writeNew(String path, String value) {
  require(!File(path).existsSync(), 'refusing to overwrite $path');
  File(path).writeAsStringSync(value, flush: true);
}

void main() {
  Directory(evidence).createSync(recursive: true);
  require(!File('$evidence/integration-pins.json').existsSync(),
      'integration pins already exist');
  final buildExit = File('$evidence/build.exit').readAsStringSync().trim();
  require(buildExit == '0', 'test binary build exit was $buildExit');

  String? executable;
  for (final line in File(cargoOutput).readAsLinesSync()) {
    if (line.isEmpty) continue;
    final event = jsonDecode(line);
    if (event is! Map<String, dynamic> || event['reason'] != 'compiler-artifact') continue;
    final packageId = event['package_id'] as String? ?? '';
    final target = event['target'] as Map<String, dynamic>?;
    final profile = event['profile'] as Map<String, dynamic>?;
    final features = (event['features'] as List<dynamic>? ?? const []).cast<String>();
    final candidate = event['executable'] as String?;
    if (packageId.contains('revault_lockbox_api#0.0.41') &&
        target?['name'] == 'revault_lockbox_api' &&
        target?['test'] == true && profile?['test'] == true &&
        (variant != 'native-block-layout' || features.contains('native-block-layout')) &&
        candidate != null) {
      require(executable == null, 'multiple default API test executables found');
      executable = candidate;
    }
  }
  require(executable != null, 'Cargo JSON had no default API test executable');
  require(File(executable!).existsSync(), 'test executable is missing');

  final head = (checked('/usr/bin/git', ['rev-parse', 'HEAD'], worktree).stdout as String).trim();
  final sourcePathsResult = checked('/usr/bin/git',
      ['ls-files', '*.rs', 'Cargo.toml', 'Cargo.lock'], rust);
  final sourcePaths = (sourcePathsResult.stdout as String)
      .split('\n')
      .where((path) => path.isNotEmpty)
      .toList()
    ..sort();
  require(sourcePaths.isNotEmpty, 'tracked Rust source manifest is empty');
  final manifest = sourcePaths
      .map((path) => '${sha256('$rust/$path')}  $path\n')
      .join();
  final manifestPath = '$evidence/source-manifest-pre.sha256';
  writeNew(manifestPath, manifest);

  final inventoryResult = checked(executable, ['--list'], rust);
  final inventory = inventoryResult.stdout as String;
  final names = <String>{};
  for (final line in inventory.split('\n')) {
    final match = RegExp(r'^(.+): test$').firstMatch(line);
    if (match != null) {
      require(names.add(match.group(1)!), 'duplicate test listed: ${match.group(1)}');
    }
  }
  require(names.isNotEmpty, 'test inventory is empty');
  final listPath = '$evidence/default-inventory.list';
  writeNew(listPath, inventory);
  writeNew('$evidence/default-inventory.stderr', inventoryResult.stderr as String);

  final pins = {
    'head': head,
    'executable': executable,
    'executable_sha256': sha256(executable),
    'inventory_count': names.length,
    'inventory_list_path': listPath,
    'inventory_sha256': sha256(listPath),
    'source_manifest_path': manifestPath,
    'source_manifest_sha256': sha256(manifestPath),
    'source_manifest_entries': sourcePaths.length,
    'build_command': [
      'cargo',
      '+1.88.0',
      'test',
      '--locked',
      '--offline',
      '--release',
      '-p',
      'revault_lockbox_api@0.0.41',
      '--lib',
      '--no-run',
      '--message-format=json',
    ]..addAll(variant == 'native-block-layout' ? ['--features', 'native-block-layout'] : []),
    'inventory_command': [executable, '--list'],
    'test_command': [executable, '--nocapture', '--test-threads=2'],
    'filters': <String>[],
    'additional_ignored_tests': false,
    'captured_utc': DateTime.now().toUtc().toIso8601String(),
    'variant': variant,
  };
  writeNew('$evidence/integration-pins.json',
      '${const JsonEncoder.withIndent('  ').convert(pins)}\n');
  writeNew('$evidence/build-default.command.json',
      '${const JsonEncoder.withIndent('  ').convert({
        'cwd': rust,
        'argv': pins['build_command'],
      'stdout': cargoOutput,
        'stderr': '$evidence/build.stderr.log',
        'exit_file': '$evidence/build.exit',
      })}\n');
  stdout.writeln(green(
      'frozen default API test executable=$executable sha256=${pins['executable_sha256']} inventory=${names.length} source_entries=${sourcePaths.length} HEAD=$head'));
}
