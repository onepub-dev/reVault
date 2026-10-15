#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const executable = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust/target/release/deps/revault_lockbox_api-4fb36609fe9bd9c0';
const evidence = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/retained-page-integration-2026-10-12/raw/exact-overlap-diagnostic';
const testA = 'file_format::candidate_files::tests::tree_tests::variables::typed_tree_variables_guarded_transaction_and_recovery_faults';
const testB = 'file_format::candidate_files::tests::tree_tests::forms::selected_source::selected_source_replacement_guarded_atomic_recovery_faults';

String? field(String content, String name) {
  final match = RegExp(r'^${name}:\s*(.*)$', multiLine: true).firstMatch(content);
  return match?.group(1)?.trim();
}

Map<int, String> procStatus() {
  final rows = <int, String>{};
  for (final entry in Directory('/proc').listSync(followLinks: false)) {
    final parts = entry.uri.pathSegments.where((s) => s.isNotEmpty).toList();
    final pid = int.tryParse(parts.isEmpty ? '' : parts.last);
    if (pid == null) continue;
    try {
      rows[pid] = File('/proc/$pid/status').readAsStringSync();
    } on FileSystemException {
      // A process can exit between listing /proc and reading its status.
    }
  }
  return rows;
}

Set<int> descendants(int root, Map<int, String> rows) {
  final parents = <int, int>{};
  for (final entry in rows.entries) {
    final parent = field(entry.value, 'PPid');
    if (parent != null) parents[entry.key] = int.tryParse(parent) ?? -1;
  }
  final result = <int>{root};
  var changed = true;
  while (changed) {
    changed = false;
    for (final entry in parents.entries) {
      if (result.contains(entry.value) && result.add(entry.key)) changed = true;
    }
  }
  return result;
}

Future<void> main() async {
  final output = File('$evidence/overlap-test.log').openWrite();
  final samples = File('$evidence/proc-samples.jsonl').openWrite();
  final started = DateTime.now().toUtc();
  final stopwatch = Stopwatch()..start();
  final process = await Process.start(executable, [testA, testB, '--exact', '--nocapture', '--test-threads=2'], runInShell: false);
  await File('$evidence/test-pid.txt').writeAsString('${process.pid}\n');
  stdout.writeln(green('OVERLAP_TEST_PID=${process.pid}'));
  await stdout.flush();
  final stdoutDone = () async {
    await for (final chunk in process.stdout.transform(utf8.decoder)) {
      output.write(chunk);
    }
  }();
  final stderrDone = () async {
    await for (final chunk in process.stderr.transform(utf8.decoder)) {
      output.write(chunk);
    }
  }();
  final exitFuture = process.exitCode;
  final processFinished = Completer<void>();
  exitFuture.then((_) { if (!processFinished.isCompleted) processFinished.complete(); });
  var sample = 0;
  var running = true;
  var timedOut = false;
  final monitorFuture = () async {
    while (running && sample < 370) {
      final procs = procStatus();
      final pids = descendants(process.pid, procs).toList()..sort();
      for (final pid in pids) {
        final content = procs[pid];
        if (content == null) continue;
        String? limits;
        try {
          limits = File('/proc/$pid/limits').readAsStringSync();
        } on FileSystemException {
          limits = null;
        }
        samples.writeln(jsonEncode({'sample': sample, 'timestamp_utc': DateTime.now().toUtc().toIso8601String(), 'pid': pid, 'root_test_pid': process.pid, 'VmLck': field(content, 'VmLck'), 'VmRSS': field(content, 'VmRSS'), 'limits': limits?.trim().split('\n')}));
      }
      sample++;
      await samples.flush();
      if (running && sample < 370) {
        await Future.any<void>([
          Future<void>.delayed(const Duration(seconds: 5)),
          processFinished.future,
        ]);
      }
    }
  }();
  final cap = Completer<int>();
  final capTimer = Timer(const Duration(seconds: 1800), () { if (!cap.isCompleted) cap.complete(-999); });
  var exitCode = await Future.any<int>([exitFuture, cap.future]);
  if (exitCode == -999) {
    timedOut = true;
    process.kill(ProcessSignal.sigterm);
    exitCode = await Future.any<int>([
      exitFuture,
      Future<int>.delayed(const Duration(seconds: 30), () => -998),
    ]);
    if (exitCode == -998) {
      process.kill(ProcessSignal.sigkill);
      exitCode = await exitFuture;
    }
  }
  capTimer.cancel();
  running = false;
  if (!processFinished.isCompleted) processFinished.complete();
  await monitorFuture;
  await stdoutDone;
  await stderrDone;
  await output.flush();
  await output.close();
  stopwatch.stop();
  await samples.flush();
  await samples.close();
  final report = {
    'command': [executable, testA, testB, '--exact', '--nocapture', '--test-threads=2'],
    'test_pid': process.pid,
    'started_utc': started.toIso8601String(),
    'elapsed_seconds': stopwatch.elapsedMilliseconds / 1000,
    'exit_status': exitCode,
    'timed_out': timedOut,
    'observer_samples': sample,
    'test_filters': [testA, testB],
  };
  await File('$evidence/run-result.json').writeAsString('${const JsonEncoder.withIndent('  ').convert(report)}\n');
  stdout.writeln(green('OVERLAP_TEST_EXIT=$exitCode TIMED_OUT=$timedOut OBSERVER_SAMPLES=$sample ELAPSED_SECONDS=${stopwatch.elapsedMilliseconds / 1000}'));
  if (timedOut) exitCode = 124;
  exit(exitCode);
}
