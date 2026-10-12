#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const repo = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const root = '$repo/rust/target/secure-arena-reclamation-20261012';
const raw = '$repo/docs/evidence/secure-arena-reclamation-2026-10-12/raw/hot-reuse-methodology-review';
const oldBinary = '$root/old/build-target/release/examples/securevec_hot_reuse';
const newBinary = '$root/new/build-target/release/examples/securevec_hot_reuse';
const oldHotSha = '129d3a37948a29a1ce620eaafbaa475539b7d600055297c38f671b13d2049ae4';
const newHotSha = 'cea3324ff4122224431a7ca0320fc591482ae757b6a43ef711214ed51de5406e';
const hotSourceSha = 'e96a48dc7891e439d3d97b7025aec88855bdc39b05aac8586d2160eb99c7dc4e'; // Exact reviewed source hash.
const oldSource = '$root/old/rust/revault_page_api/examples/securevec_hot_reuse.rs';
const newSource = '$root/new/rust/revault_page_api/examples/securevec_hot_reuse.rs';
const oldManifest = '$raw/hot-old-isolated-manifest.sha256';
const newManifest = '$raw/hot-new-isolated-manifest.sha256';
const lockedBytes = 8 * 1024 * 1024;
const timeout = '/usr/bin/timeout';
const taskset = '/usr/bin/taskset';
const executeAfterReview = true;

void require(bool ok, String why) { if (!ok) throw StateError(why); }
String sha(String path) {
  final result = Process.runSync('/usr/bin/sha256sum', [path]);
  require(result.exitCode == 0, 'sha256sum failed for $path');
  return (result.stdout as String).split(RegExp(r'\s+')).first;
}
void verifyManifest(String path) {
  for (final line in File(path).readAsLinesSync()) {
    if (line.trim().isEmpty) continue;
    final m = RegExp(r'^([0-9a-f]{64})  (.+)$').firstMatch(line);
    require(m != null, 'bad manifest row in $path');
    require(sha('$root/${m!.group(2)}') == m.group(1), 'source hash mismatch: ${m.group(2)}');
  }
}
Map<String, Object> identities() {
  verifyManifest(oldManifest);
  verifyManifest(newManifest);
  return {
    'old_binary': {'path': oldBinary, 'sha256': sha(oldBinary)},
    'new_binary': {'path': newBinary, 'sha256': sha(newBinary)},
    'old_hot_source': {'path': oldSource, 'sha256': sha(oldSource)},
    'new_hot_source': {'path': newSource, 'sha256': sha(newSource)},
    'reviewed_source_snapshot': {'path': '$raw/securevec_hot_reuse.rs.txt', 'sha256': sha('$raw/securevec_hot_reuse.rs.txt')},
    'old_manifest': {'path': oldManifest, 'sha256': sha(oldManifest)},
    'new_manifest': {'path': newManifest, 'sha256': sha(newManifest)},
    'old_lock': {'path': '$root/old/rust/revault_page_api/Cargo.lock', 'sha256': sha('$root/old/rust/revault_page_api/Cargo.lock')},
    'new_lock': {'path': '$root/new/rust/revault_page_api/Cargo.lock', 'sha256': sha('$root/new/rust/revault_page_api/Cargo.lock')},
    'old_manifest_toml': {'path': '$root/old/rust/revault_page_api/Cargo.toml', 'sha256': sha('$root/old/rust/revault_page_api/Cargo.toml')},
    'new_manifest_toml': {'path': '$root/new/rust/revault_page_api/Cargo.toml', 'sha256': sha('$root/new/rust/revault_page_api/Cargo.toml')},
    'runner': {'path': '$raw/run-hot-reuse.dart', 'sha256': sha('$raw/run-hot-reuse.dart')},
    'analyzer': {'path': '$raw/analyze-hot-reuse.dart', 'sha256': sha('$raw/analyze-hot-reuse.dart')},
    'protocol': {'path': '$raw/protocol.txt', 'sha256': sha('$raw/protocol.txt')},
    'execution_mode': {'label': 'hot-reuse-fixed-56-v1'},
  };
}
List<Map<String, Object>> schedule() {
  final rows = <Map<String, Object>>[];
  for (var round = 0; round < 14; round++) {
    for (final size in [64, 65536]) {
      final order = round.isEven ? ['old', 'new'] : ['new', 'old'];
      for (final variant in order) {
        rows.add({'round': round, 'size_bytes': size, 'iterations': size == 64 ? 100000 : 10000, 'variant': variant, 'preparation_round': round < 2});
      }
    }
  }
  return rows;
}
Map<String, dynamic> oneJson(String text) {
  final lines = text.split('\n').where((line) => line.trim().isNotEmpty).toList();
  require(lines.length == 1, 'expected one hot result JSON record');
  final value = jsonDecode(lines.single);
  require(value is Map<String, dynamic>, 'hot result is not a JSON object');
  return value as Map<String, dynamic>;
}
double positive(Map<String, dynamic> row, String key) {
  final value = row[key];
  require(value is num, '$key must be numeric');
  final n = (value as num).toDouble();
  require(n.isFinite && n > 0, '$key must be finite and positive');
  return n;
}
void admit(Map<String, dynamic> row, String variant, int size, int iterations) {
  require(row['kind'] == 'hot_reuse' && row['variant'] == variant, 'wrong kind/variant');
  require(row['size_bytes'] == size && row['iterations'] == iterations, 'wrong size/iterations');
  require(row['byte_verification_ok'] == true && row['weak_policy'] == false, 'byte or policy gate failed');
  require(row['memlock_soft_bytes'] == lockedBytes && row['memlock_hard_bytes'] == lockedBytes, 'memlock limit changed');
  require(row['baseline_VmLck_bytes'] == 0, 'fresh-process baseline VmLck must be zero');
  final vmValues = <int>[];
  for (final key in ['prewarm_live_VmLck_bytes', 'before_inner_VmLck_bytes', 'after_inner_VmLck_bytes']) {
    final value = row[key];
    require(value is num && value > 0 && value <= lockedBytes, '$key must be positive and within unchanged limit');
    vmValues.add((value as num).toInt());
  }
  require(vmValues.toSet().length == 1, 'VmLck must remain equal across prewarm/drop/reuse');
  positive(row, 'inner_elapsed_ns');
}

Future<void> runBatch() async {
  require(executeAfterReview, 'execution gate is closed until draft review');
  require(oldHotSha.isNotEmpty && newHotSha.isNotEmpty && hotSourceSha.isNotEmpty, 'hot binary/source hashes not pinned');
  final before = identities();
  require((before['old_binary'] as Map)['sha256'] == oldHotSha && (before['new_binary'] as Map)['sha256'] == newHotSha, 'hot binary hash mismatch');
  require((before['old_hot_source'] as Map)['sha256'] == hotSourceSha && (before['new_hot_source'] as Map)['sha256'] == hotSourceSha, 'hot source hashes differ');
  File('$raw/identities-before.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(before)}\n', flush: true);
  final samples = File('$raw/samples.jsonl');
  require(!samples.existsSync(), 'refusing to overwrite samples');
  final wall = Stopwatch()..start();
  var count = 0;
  for (final item in schedule()) {
    final remainMs = const Duration(minutes: 15).inMilliseconds - wall.elapsedMilliseconds - 2000;
    require(remainMs >= 1000, '15-minute batch deadline reached');
    final variant = item['variant'] as String;
    final size = item['size_bytes'] as int;
    final iterations = item['iterations'] as int;
    final binary = variant == 'old' ? oldBinary : newBinary;
    final remainingSeconds = remainMs ~/ 1000;
    final perProcessSeconds = remainingSeconds < 120 ? remainingSeconds : 120;
    final argv = [timeout, '--kill-after=1s', '${perProcessSeconds}s', taskset, '-c', '0-4', binary, variant, '$size', '$iterations'];
    final stem = '$raw/process-${count.toString().padLeft(2, '0')}-round-${item['round']}-size-$size-$variant';
    File('$stem.command.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert({'argv': argv, 'cwd': repo, 'timeout_seconds': perProcessSeconds, 'cpu_affinity': '0-4'})}\n', flush: true);
    final timer = Stopwatch()..start();
    final process = await Process.start(argv.first, argv.skip(1).toList(), workingDirectory: repo, runInShell: false);
    final stdout = StringBuffer(), stderr = StringBuffer();
    final outDone = process.stdout.transform(utf8.decoder).listen(stdout.write).asFuture<void>();
    final errDone = process.stderr.transform(utf8.decoder).listen(stderr.write).asFuture<void>();
    final exit = await process.exitCode;
    await Future.wait([outDone, errDone]);
    timer.stop();
    require(timer.elapsedMicroseconds > 0, 'whole-process timer resolution was not positive');
    File('$stem.stdout.log').writeAsStringSync(stdout.toString(), flush: true);
    File('$stem.stderr.log').writeAsStringSync(stderr.toString(), flush: true);
    File('$stem.status.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert({'exit_code': exit, 'elapsed_us': timer.elapsedMicroseconds, 'timed_out': exit == 124 || exit == 137})}\n', flush: true);
    require(exit == 0, 'process failed/timed out; stopping without retry');
    final result = oneJson(stdout.toString());
    admit(result, variant, size, iterations);
    final row = <String, Object?>{...item, 'execution_mode': 'hot-reuse-fixed-56-v1', 'kind': result['kind'], 'inner_elapsed_ns': positive(result, 'inner_elapsed_ns'), 'whole_process_elapsed_ns': timer.elapsedMicroseconds * 1000, 'baseline_VmLck_bytes': result['baseline_VmLck_bytes'], 'prewarm_live_VmLck_bytes': result['prewarm_live_VmLck_bytes'], 'before_inner_VmLck_bytes': result['before_inner_VmLck_bytes'], 'after_inner_VmLck_bytes': result['after_inner_VmLck_bytes'], 'byte_verification_ok': result['byte_verification_ok'], 'weak_policy': result['weak_policy'], 'memlock_soft_bytes': result['memlock_soft_bytes'], 'memlock_hard_bytes': result['memlock_hard_bytes'], 'exit_code': exit};
    samples.writeAsStringSync('${jsonEncode(row)}\n', mode: FileMode.append, flush: true);
    count++;
  }
  wall.stop();
  require(count == 56 && wall.elapsed < const Duration(minutes: 15), 'batch count/deadline gate failed');
  final after = identities();
  File('$raw/identities-after.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(after)}\n', flush: true);
  require(jsonEncode(before) == jsonEncode(after), 'binary/source/lock identity changed during batch');
}

Future<void> main(List<String> args) async {
  print(green('Execution mode: hot-reuse-fixed-56-v1'));
  print('56 processes; strict equal-positive VmLck gate; one execution only.');
  require(schedule().length == 56, 'schedule must contain exactly 56 rows');
  if (args.contains('--execute-reviewed')) await runBatch();
}
