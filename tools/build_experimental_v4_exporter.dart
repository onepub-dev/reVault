#! /usr/bin/env dcli
// Build only the isolated exporter for the unreleased pre-integrity v4 checkpoint.
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const revision = 'a7a9f4d85b0fa716bf9e402de5f29ff5a6f61e9b';
void run(String executable, List<String> args, {String? directory}) {
  final result = Process.runSync(executable, args, workingDirectory: directory);
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0) throw StateError('$executable failed (${result.exitCode})');
}
String hash(String path) {
  final result = Process.runSync('sha256sum', [path]);
  if (result.exitCode != 0) throw StateError('Cannot hash $path');
  return (result.stdout as String).split(' ').first;
}
void main(List<String> args) {
  if (args.length != 1) throw ArgumentError('Usage: ./build_experimental_v4_exporter.dart NEW_OUTPUT_DIRECTORY');
  final repository = File(Platform.script.toFilePath()).parent.parent.absolute.path;
  final output = Directory(args.single).absolute;
  if (output.existsSync()) throw StateError('Output must be new; preserve prior evidence');
  output.createSync(recursive: true);
  final source = Directory('${output.path}/source')..createSync();
  final tar = '${output.path}/source.tar';
  run('git', ['-C', repository, 'archive', '--format=tar', '--output=$tar', revision]);
  run('tar', ['-xf', tar, '-C', source.path]);
  final adapter = Directory('${output.path}/adapter')..createSync();
  final bin = Directory('${adapter.path}/src/bin')..createSync(recursive: true);
  final hashes = <String, String>{};
  for (final kind in ['archive', 'vault']) {
    final original = '$repository/tools/experimental-v4-exporter/$kind.rs';
    File(original).copySync('${bin.path}/revault-migrate-$kind-experimental-v4.rs');
    hashes[kind] = hash(original);
  }
  String dependency(String package, {String? features}) =>
      '$package = { path = ${jsonEncode('${source.path}/rust/$package')}${features == null ? '' : ', features = $features'} }';
  File('${adapter.path}/Cargo.toml').writeAsStringSync('''
[package]
name = "revault_experimental_v4_exporter"
version = "0.0.0"
edition = "2021"
publish = false
[workspace]
[dependencies]
${dependency('revault_lockbox_api', features: '["migration", "vault-integration"]')}
${dependency('revault_vault_api')}
${dependency('revault_migration')}
${dependency('revault_migration_format')}
getrandom = "0.4"
sha2 = "0.11"
zeroize = "1.9"
tempfile = "3"
[patch.crates-io]
zstd-complete = { path = ${jsonEncode('${source.path}/rust/vendor/zstd-complete')} }
''');
  File('${source.path}/rust/Cargo.lock').copySync('${adapter.path}/Cargo.lock');
  // Retain archived dependency versions while adding the standalone adapter.
  // generate-lockfile would discard them and select a newer dependency graph.
  run('cargo', ['build', '--release', '--offline', '--target-dir', '${adapter.path}/target', '--manifest-path', '${adapter.path}/Cargo.toml'], directory: '${source.path}/rust');
  run('cargo', ['build', '--release', '--offline', '--locked', '--target-dir', '${adapter.path}/target', '--manifest-path', '${adapter.path}/Cargo.toml'], directory: '${source.path}/rust');
  final binaries = <String, String>{};
  for (final kind in ['archive', 'vault']) {
    final name = 'revault-migrate-$kind-experimental-v4';
    final destination = '${output.path}/$name';
    File('${adapter.path}/target/release/$name').copySync(destination);
    run('chmod', ['+x', destination]);
    run(destination, ['capabilities']);
    binaries[name] = hash(destination);
  }
  File('${output.path}/provenance.json').writeAsStringSync(const JsonEncoder.withIndent('  ').convert({
    'source_revision': revision,
    'adapter_sha256': hashes,
    'cargo_lock_sha256': hash('${adapter.path}/Cargo.lock'),
    'binary_sha256': binaries,
  }));
  print(green('Isolated historical exporters and provenance: ${output.path}'));
}
