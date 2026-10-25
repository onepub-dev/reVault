#! /usr/bin/env dcli
import 'dart:io';
import 'package:dcli/dcli.dart';

void run(String command, List<String> args, {String? cwd}) {
  final result = Process.runSync(command, args, workingDirectory: cwd);
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0) throw StateError('$command failed');
}

void main() {
  final head = Process.runSync('git', ['rev-parse', 'HEAD']).stdout.toString().trim();
  if (head != Platform.environment['EXPECTED_SOURCE_SHA'] ||
      Platform.environment['GITHUB_REF'] != 'refs/heads/main') {
    throw StateError('Publication requires the approved main revision');
  }
  // This one-off exception is restricted to the already-reviewed hostname patch.
  run('git', ['diff', '--exit-code', '0518ab5e89c2d443fce87d3a5c6ec6e092759074', 'HEAD', '--', 'rust/revault_cli']);
  run('cargo', ['info', 'revault_cli@0.6.0', '--registry', 'crates-io'], cwd: 'rust');
  final cargoHome = Platform.environment['CARGO_HOME'] ?? '${Platform.environment['HOME']}/.cargo';
  final prior = Directory('$cargoHome/registry/src').listSync().whereType<Directory>()
      .map((d) => File('${d.path}/revault_cli-0.6.0/Cargo.toml'))
      .where((f) => f.existsSync()).single.readAsStringSync();
  if (!prior.contains('name = "revault_cli"') || !prior.contains('version = "0.6.0"')) {
    throw StateError('Unexpected published CLI manifest');
  }
  for (final version in ['0.0.57', '0.0.51', '0.0.58', '0.0.55']) {
    if (!prior.contains('version = "$version"')) throw StateError('Unexpected dependencies');
  }
  File('rust/revault_cli/Cargo.toml').writeAsStringSync(
      prior.replaceFirst('version = "0.6.0"', 'version = "0.6.1"'));
  print(green('Publishing only revault_cli 0.6.1; no tests or GitHub binaries.'));
  run('cargo', ['publish', '--allow-dirty', '-p', 'revault_cli@0.6.1'], cwd: 'rust');
}
