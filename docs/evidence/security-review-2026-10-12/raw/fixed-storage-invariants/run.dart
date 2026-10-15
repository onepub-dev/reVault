#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const wt = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const raw = '$wt/docs/evidence/security-review-2026-10-12/raw';
const out = '$raw/fixed-storage-invariants';
const module = '$wt/rust/vendor/zstd-complete/src/fixed_storage.rs';
const testDraft = '/tmp/revault-security-drafts/fixed-storage-invariants.rs.txt';
const compile6 = '$raw/codec-fixed-storage-compile6';

void require(bool ok, String message) {
  if (!ok) throw StateError(message);
}
String sha(String path) {
  final result = Process.runSync('/usr/bin/sha256sum', ['--', path]);
  require(result.exitCode == 0, 'sha256sum failed for $path');
  return (result.stdout as String).split(RegExp(r'\s+')).first;
}
void write(String path, String contents) {
  File(path).writeAsStringSync(contents, flush: true);
}
Map<String, Object?> execute(String label, List<String> argv, int seconds) {
  final actual = [
    '/usr/bin/timeout', '--kill-after=5s', '${seconds}s', ...argv,
  ];
  final result = Process.runSync(actual.first, actual.skip(1).toList(),
      workingDirectory: '$wt/rust',
      environment: {
        ...Platform.environment,
        'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011',
      },
      includeParentEnvironment: false);
  write('$out/$label.stdout', result.stdout as String);
  write('$out/$label.stderr', result.stderr as String);
  write('$out/$label.exit', '${result.exitCode}\n');
  write('$out/$label.command.txt', '${actual.join(' ')}\n');
  return {
    'argv': actual,
    'exit_code': result.exitCode,
    'timed_out': result.exitCode == 124 || result.exitCode == 137,
  };
}

void main() {
  Directory(out).createSync(recursive: true);
  Directory(compile6).createSync(recursive: true);
  final incident = '''Compiler-only check6 incident
Command: cargo +1.88.0 check --locked --offline -p zstd-complete@0.2.0 --lib
Working directory: $wt/rust
Observed exit: 101
Observed diagnostics: 259 unused `Result` errors; 21 warnings.
Capture limitation: the direct exec session returned truncated output (3,252 total lines). No complete compiler log, pre/post source manifest, or compiler output file was captured; this note records only observed terminal facts. No check6 rerun is authorized.
''';
  final incidentPath = '$compile6/capture-limitation.txt';
  if (!File(incidentPath).existsSync()) write(incidentPath, incident);
  File(testDraft).copySync('$out/fixed-storage-invariants.rs.txt');
  final sourceBefore = sha(module);
  final testHash = sha('$out/fixed-storage-invariants.rs.txt');
  final moduleBefore = sourceBefore;
  write('$out/module-hash.before.txt', '$moduleBefore  $module\n');
  write('$out/test-source.sha256', '$testHash  fixed-storage-invariants.rs.txt\n');
  final rustc = Platform.environment['RUSTC'] ?? '/home/bsutton/.cargo/bin/rustup';
  final binary = '$out/fixed-storage-invariants-test';
  final compile = execute('compile', [
    rustc, 'run', '1.88.0', 'rustc', '--crate-name=fixed_storage_invariants', '--edition=2021', '--test',
    '$out/fixed-storage-invariants.rs.txt', '-o', binary,
  ], 120);
  write('$out/compile.result.json', const JsonEncoder.withIndent('  ').convert(compile));
  require(compile['exit_code'] == 0, 'Isolated invariant test compile failed; see compile logs');
  final run = execute('test', [binary, '--test-threads=1'], 60);
  write('$out/test.result.json', const JsonEncoder.withIndent('  ').convert(run));
  final moduleAfter = sha(module);
  write('$out/module-hash.after.txt', '$moduleAfter  $module\n');
  require(moduleAfter == moduleBefore, 'FixedStorage module changed during the test');
  require(run['exit_code'] == 0, 'Isolated invariant tests failed; see test logs');
  write('$out/summary.json', const JsonEncoder.withIndent('  ').convert({
    'test_source': testDraft,
    'test_source_sha256': testHash,
    'module': module,
    'module_sha256_before': moduleBefore,
    'module_sha256_after': moduleAfter,
    'compile_exit': compile['exit_code'],
    'test_exit': run['exit_code'],
    'test_scope': 'only fixed_storage.rs through standalone rustc test module',
  }));
}
