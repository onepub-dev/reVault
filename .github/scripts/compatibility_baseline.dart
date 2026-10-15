#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

// Run from rust/. Metadata and the public compatibility line share one authority.
void main() {
  final policy = jsonDecode(File('release-compatibility.json').readAsStringSync())
      as Map<String, dynamic>;
  String value(String key) => policy[key] as String;
  final version = value('baseline_cli_version');
  final tag = value('baseline_tag');
  if (tag != 'revault_cli-v$version') throw StateError('Baseline tag mismatch');
  final root = Directory.systemTemp.createTempSync('revault-line-baseline-');
  const repository = 'onepub-dev/reVault';
  final asset = value('baseline_linux_asset');
  final archive = '${root.path}/$asset';
  String execute(String program, List<String> args, {String? workingDirectory}) {
    final result = Process.runSync(program, args, workingDirectory: workingDirectory);
    if (result.exitCode != 0) {
      stderr.write(result.stderr);
      throw StateError('$program failed (${result.exitCode})');
    }
    return result.stdout as String;
  }
  execute('gh', ['release', 'download', tag, '--repo', repository,
    '--pattern', asset, '--pattern', '$asset.sha256', '--dir', root.path]);
  final checksum = execute('sha256sum', [archive]).split(' ').first;
  if (checksum != value('baseline_linux_sha256')) {
    throw StateError('Baseline archive differs from the pinned release');
  }
  execute('sha256sum', ['--check', '$asset.sha256'], workingDirectory: root.path);
  execute('gh', ['attestation', 'verify', archive, '--repo', repository,
    '--source-digest', value('baseline_source_sha'),
    '--signer-workflow', '$repository/.github/workflows/revault_cli-v-release.yml']);
  execute('tar', ['-xzf', archive, '-C', root.path]);
  final binary = '${root.path}/${asset.replaceFirst('.tar.gz', '')}/lockbox';
  final actualVersion = execute(binary, ['--version']).trim();
  if (!actualVersion.split(RegExp(r'\s+')).contains(version)) {
    throw StateError('Unexpected baseline version: $actualVersion');
  }
  final environment = Platform.environment['GITHUB_ENV'];
  if (environment == null) throw StateError('This acquisition step requires GITHUB_ENV');
  File(environment).writeAsStringSync('REVAULT_COMPAT_BASELINE_BIN=$binary\n', mode: FileMode.append);
  print(green('Verified published compatibility baseline $tag ($checksum)'));
}
