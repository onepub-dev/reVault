#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const pid = 848889;
const evidence = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/retained-page-integration-2026-10-12/raw/exact-overlap-diagnostic';

String? value(String status, String name) {
  for (final line in status.split('\n')) {
    if (line.startsWith('$name:')) return line.substring(name.length + 1).trim();
  }
  return null;
}

Future<void> main() async {
  final samples = File('$evidence/proc-samples-supplemental.jsonl').openWrite();
  final started = DateTime.now().toUtc();
  var count = 0;
  var present = true;
  while (count < 340 && DateTime.now().toUtc().difference(started) < const Duration(minutes: 28)) {
    String? status;
    String? limits;
    try {
      status = File('/proc/$pid/status').readAsStringSync();
      limits = File('/proc/$pid/limits').readAsStringSync();
    } on FileSystemException {
      present = false;
      break;
    }
    samples.writeln(jsonEncode({
      'sample': count,
      'timestamp_utc': DateTime.now().toUtc().toIso8601String(),
      'pid': pid,
      'VmLck': value(status, 'VmLck'),
      'VmRSS': value(status, 'VmRSS'),
      'limits': limits.trim().split('\n'),
    }));
    await samples.flush();
    count++;
    await Future<void>.delayed(const Duration(seconds: 5));
  }
  await samples.flush();
  await samples.close();
  final result = {
    'pid': pid,
    'started_utc': started.toIso8601String(),
    'sample_count': count,
    'pid_exited': !present,
    'max_runtime_seconds': 1680,
  };
  await File('$evidence/supplemental-observer-result.json').writeAsString('${const JsonEncoder.withIndent('  ').convert(result)}\n');
  stdout.writeln(green('SUPPLEMENTAL_OBSERVER_PID=$pid SAMPLES=$count PID_EXITED=${!present}'));
}
