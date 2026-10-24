#! /usr/bin/env dcli

import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

String run(List<String> command, {bool capture = false}) {
  final result = Process.runSync(command.first, command.skip(1).toList());
  if (!capture) stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0) {
    throw StateError('${command.first} failed: ${result.exitCode}');
  }
  return result.stdout as String;
}

void main(List<String> args) {
  if (args.length != 1) {
    throw ArgumentError('Usage: ./tools/build_wasm.dart OUTPUT_DIRECTORY');
  }
  final output = Directory(args.single).absolute;
  final lock = File('rust/Cargo.lock').readAsStringSync();
  final version =
      RegExp(r'^name = "wasm-bindgen"\r?\nversion = "([^"]+)"', multiLine: true)
          .firstMatch(lock)
          ?.group(1);
  if (version == null) throw StateError('Missing locked wasm-bindgen version');
  final metadata = jsonDecode(run([
    'cargo',
    'metadata',
    '--locked',
    '--no-deps',
    '--format-version',
    '1',
    '--manifest-path',
    'rust/Cargo.toml',
  ], capture: true)) as Map<String, dynamic>;
  final target = metadata['target_directory'] as String;
  final tools = '$target/wasm-bindgen-tools-$version';
  final generator =
      '$tools/bin/wasm-bindgen${Platform.isWindows ? '.exe' : ''}';
  if (!File(generator).existsSync()) {
    run([
      'cargo',
      'install',
      'wasm-bindgen-cli',
      '--version',
      version,
      '--locked',
      '--root',
      tools
    ]);
  }
  if (run([generator, '--version'], capture: true).trim() !=
      'wasm-bindgen $version') {
    throw StateError(
        'Generator does not match the locked wasm-bindgen version');
  }
  run([
    'cargo',
    'build',
    '--locked',
    '--release',
    '--manifest-path',
    'rust/Cargo.toml',
    '-p',
    'revault_wasm_bindings',
    '--target',
    'wasm32-unknown-unknown'
  ]);
  output.createSync(recursive: true);
  run([
    generator,
    '$target/wasm32-unknown-unknown/release/revault_wasm_bindings.wasm',
    '--target',
    'nodejs',
    '--out-dir',
    output.path
  ]);
  final commonJs = File('${output.path}/revault_wasm_bindings.cjs');
  if (commonJs.existsSync()) commonJs.deleteSync();
  File('${output.path}/revault_wasm_bindings.js').renameSync(commonJs.path);
  File('${output.path}/build-provenance.json').writeAsStringSync(
    '${jsonEncode({
          'source_sha': run(['git', 'rev-parse', 'HEAD'], capture: true).trim(),
          'wasm_bindgen': version
        })}\n',
  );
  print(green('Built WebAssembly from the current source into ${output.path}'));
}
