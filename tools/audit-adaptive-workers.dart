#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';

import 'package:dcli/dcli.dart';

void require(bool condition, String message) {
  if (!condition) throw StateError(message);
}

Map<String, dynamic> object(Object? value, String label) {
  require(value is Map<String, dynamic>, '$label must be an object');
  return value as Map<String, dynamic>;
}

Object? canonical(Object? value) {
  if (value is Map) {
    final keys = value.keys.cast<String>().toList()..sort();
    return {for (final key in keys) key: canonical(value[key])};
  }
  if (value is List) return value.map(canonical).toList();
  return value;
}

bool same(Object? left, Object? right) =>
    jsonEncode(canonical(left)) == jsonEncode(canonical(right));

Map<String, dynamic> readObject(String path) =>
    object(jsonDecode(File(path).readAsStringSync()), path);

String sha(String path) {
  final result = Process.runSync('sha256sum', ['--', path]);
  require(result.exitCode == 0, 'Cannot hash $path');
  return '${result.stdout}'.split(RegExp(r'\s+')).first;
}

bool autoPoolExpected(String caseName) => const {
  'source-zstd-plain-17mib',
  'source-zstd-plain-64mib',
  'source-zstd-plain-256mib',
  'source-zstd-encrypted-64mib',
  'mixed-raw-first-source-second-64mib',
  'mixed-source-first-raw-second-64mib',
  'mixed-source-first-17mib-64mib',
}.contains(caseName);

void main(List<String> args) {
  require(args.length <= 1, 'Usage: ./audit-adaptive-workers.dart [BATCH_DIR]');
  final repo = Directory.current.absolute.path;
  final batch = Directory(args.isEmpty
          ? '$repo/docs/evidence/adaptive-workers-2026-10-11/raw/adaptive-workers-batch'
          : args.single)
      .absolute
      .path;
  final raw = Directory(batch).parent.path;
  const variants = ['typed-tree-1', 'typed-tree-4', 'typed-tree-auto'];
  const bytesBySize = <String, int>{
    '1mib': 1048576,
    '16mib': 16777216,
    '17mib': 17825792,
    '64mib': 67108864,
    '256mib': 268435456,
  };
  const profiles = <String, Map<String, Object>>{
    'source-zstd-plain-1mib': {'bytes': 1048576, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'plain', 'raw': 0, 'zstd': 16},
    'source-zstd-plain-16mib': {'bytes': 16777216, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'plain', 'raw': 0, 'zstd': 256},
    'source-zstd-plain-17mib': {'bytes': 17825792, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'plain', 'raw': 0, 'zstd': 272},
    'source-zstd-plain-64mib': {'bytes': 67108864, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'plain', 'raw': 0, 'zstd': 1024},
    'source-zstd-plain-256mib': {'bytes': 268435456, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'plain', 'raw': 0, 'zstd': 4096},
    'source-zstd-encrypted-64mib': {'bytes': 67108864, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': true, 'mode': 'encrypted', 'raw': 0, 'zstd': 1024},
    'source-raw-plain-64mib': {'bytes': 67108864, 'corpus': 'source-bundle-repeat', 'shape': 'prefix-repetition', 'compressed': false, 'mode': 'plain', 'raw': 1024, 'zstd': 0},
    'random-zstd-fallback-plain-64mib': {'bytes': 67108864, 'corpus': 'random-prefix', 'shape': 'prefix', 'compressed': true, 'mode': 'plain', 'raw': 1024, 'zstd': 0},
    'mixed-raw-first-source-second-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'raw-first-half', 'compressed': true, 'mode': 'plain', 'raw': 512, 'zstd': 512},
    'mixed-source-first-raw-second-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'source-first-half', 'compressed': true, 'mode': 'plain', 'raw': 512, 'zstd': 512},
    'mixed-alternating-64k-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'alternating-64k', 'compressed': true, 'mode': 'plain', 'raw': 512, 'zstd': 512},
    'mixed-source-1mib-per-16mib-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'periodic-source-window', 'compressed': true, 'mode': 'plain', 'raw': 960, 'zstd': 64},
    'mixed-source-first-17mib-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'source-first-17mib', 'compressed': true, 'mode': 'plain', 'raw': 752, 'zstd': 272},
    'mixed-source-final-1mib-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'source-final-1mib', 'compressed': true, 'mode': 'plain', 'raw': 1008, 'zstd': 16},
    'mixed-source-final-16mib-64mib': {'bytes': 67108864, 'corpus': 'mixed-source-random', 'shape': 'source-final-16mib', 'compressed': true, 'mode': 'plain', 'raw': 768, 'zstd': 256},
  };
  final expectedNames = profiles.keys.toList();
  final manifest = (jsonDecode(File('$raw/cases.json').readAsStringSync()) as List)
      .map((row) => object(row, 'fixture manifest row'))
      .toList();
  require(manifest.length == 15 && same(manifest.map((row) => row['name']).toList(), expectedNames),
      'Case manifest differs from frozen 15-case order');
  for (var index = 0; index < manifest.length; index++) {
    final row = manifest[index];
    final name = expectedNames[index];
    final profile = profiles[name]!;
    require(row['files'] == 1 && row['unit'] == 65536, 'Frozen file/unit profile mismatch for $name');
    require(row['bytes'] == profile['bytes'] && row['corpus'] == profile['corpus'] &&
        row['shape'] == profile['shape'] && row['compressed'] == profile['compressed'] &&
        row['mode'] == profile['mode'] && row['expected_raw_chunks'] == profile['raw'] &&
        row['expected_zstd_chunks'] == profile['zstd'], 'Frozen profile/census mismatch for $name');
    final creation = readObject('$raw/tree-creation/$name.json');
    final census = object(object(creation['sample'], 'creation sample')['codec_census'], 'codec census');
    require(creation['verified'] == true && creation['codec_census_verified'] == true &&
        census['raw_chunks'] == profile['raw'] && census['zstd_chunks'] == profile['zstd'] &&
        census['logical_bytes'] == profile['bytes'], 'Authenticated descriptor census mismatch for $name');
    final sourcePath = row['source_path'] as String;
    require(sha(sourcePath) == row['source_sha256'], 'Fixture source hash mismatch for $name');
  }

  final complete = readObject('$batch/complete.json');
  final before = readObject('$batch/identity-before.json');
  final after = readObject('$batch/identity-after.json');
  require(complete['complete'] == true && complete['durable'] == false &&
      complete['sample_rows'] == 720 && same(complete['variants'], variants) &&
      same(complete['compared_workers'], [1, 4, 0]), 'Batch completion metadata mismatch');
  for (final key in [
    'source_revision', 'source_base_revision', 'source_hashes', 'source_diff_sha256',
    'source_hash_manifest_sha256', 'runner_sha256', 'calibration_sha256', 'protocol_sha256',
    'fixture_manifest_sha256', 'source_corpus_identity_sha256', 'tree_test_executable',
    'tree_test_executable_sha256',
  ]) {
    require(before[key] != null && same(before[key], after[key]), 'Batch identity changed: $key');
  }
  require(before['source_base_revision'] == '1a7242a3e448109155e9fe73d0d4ea6b1b76c157' &&
      before['runner_sha256'] == sha('$raw/adaptive-workers-runner.dart') &&
      before['protocol_sha256'] == sha('$raw/protocol-before-measurements.md') &&
      before['fixture_manifest_sha256'] == sha('$raw/cases.json') &&
      before['calibration_sha256'] == sha('$raw/physical-read-calibration.json') &&
      before['source_corpus_identity_sha256'] == sha('$raw/source-corpus-identity.json') &&
      before['tree_test_executable_sha256'] == sha(before['tree_test_executable'] as String),
      'Frozen source/runner/protocol/executable digest mismatch');
  for (final entry in object(before['source_hashes'], 'source hashes').entries) {
    require(sha('/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust/${entry.key}') == entry.value,
        'Source hash mismatch for ${entry.key}');
  }

  const warmups = 4;
  const rounds = 16;
  final activationCounts = <String, Map<String, int>>{};
  final orderCounts = <String, Map<String, List<List<int>>>>{};
  var checked = 0;
  var maxRssAll = 0;
  var maxRssMeasured = 0;
  for (var caseIndex = 0; caseIndex < expectedNames.length; caseIndex++) {
    final name = expectedNames[caseIndex];
    final caseDir = '$batch/$name';
    final audit = readObject('$caseDir/case-audit.json');
    final completeAudit = (complete['cases'] as List)
        .map((row) => object(row, 'complete case record'))
        .singleWhere((row) => row['case'] == name);
    require(same(audit, completeAudit) && audit['stable'] == true && audit['failure'] == null &&
        audit['rounds_complete'] == rounds && audit['warmup_rounds_planned'] == warmups,
        'Incomplete or unstable case audit for $name');
    final byVariant = <String, Map<int, Map<String, dynamic>>>{};
    for (final variant in variants) {
      final path = '$caseDir/$variant/samples.jsonl';
      final rows = File(path).readAsLinesSync().where((line) => line.trim().isNotEmpty)
          .map((line) => object(jsonDecode(line), path)).toList();
      require(rows.length == rounds, 'Wrong row count in $name/$variant');
      final indexed = <int, Map<String, dynamic>>{};
      for (final row in rows) {
        final round = row['round'];
        require(round is int && !indexed.containsKey(round), 'Duplicate/invalid sample round in $name/$variant');
        indexed[round as int] = row;
        final worker = variant == 'typed-tree-auto' ? 0 : int.parse(variant.split('-').last);
        final shouldActivate = autoPoolExpected(name);
        final expectedThreads = worker == 4 ? 4 : worker == 0 && shouldActivate ? 4 : 0;
        final sample = object(row['sample'], 'sample');
        require(row['verified'] == true && row['worker_count'] == worker &&
            sample['configured_cpu_workers'] == worker &&
            sample['automatic_workers'] == (worker == 0) &&
            sample['worker_threads'] == expectedThreads &&
            sample['max_queued_chunks'] == (expectedThreads == 0 ? 1 : 32) &&
            row['auto_pool_activated'] == (worker == 0 && shouldActivate),
            'Worker/pool activation metadata mismatch in $name/$variant/$round');
        if (variant == 'typed-tree-auto' && round >= warmups && expectedThreads == 4) {
          activationCounts.putIfAbsent(name, () => {'measured': 0, 'all': 0});
          activationCounts[name]!['measured'] = activationCounts[name]!['measured']! + 1;
        }
        if (variant == 'typed-tree-auto' && expectedThreads == 4) {
          activationCounts.putIfAbsent(name, () => {'measured': 0, 'all': 0});
          activationCounts[name]!['all'] = activationCounts[name]!['all']! + 1;
        }
        final rss = object(object(sample['resources'], 'resources')['cpu'] == null
            ? sample['resources'] : sample['resources'], 'resources')['peak_rss_kib'];
        require(rss is num && rss.isFinite && rss > 0, 'Invalid extraction RSS for $name/$variant/$round');
        if (rss > maxRssAll) maxRssAll = (rss as num).toInt();
        if (round >= warmups && rss > maxRssMeasured) maxRssMeasured = (rss as num).toInt();
      }
      require(indexed.length == rounds && List.generate(rounds, indexed.containsKey).every((value) => value),
          'Missing round in $name/$variant');
      byVariant[variant] = indexed;
    }
    final parityPositions = <String, List<List<int>>>{
      '0': List.generate(3, (_) => [0, 0, 0]),
      '1': List.generate(3, (_) => [0, 0, 0]),
    };
    for (var round = 0; round < rounds; round++) {
      final path = '$caseDir/round-${round.toString().padLeft(2, '0')}.json';
      final pair = readObject(path);
      final rotation = (round + caseIndex) % variants.length;
      final expectedOrder = [for (var offset = 0; offset < variants.length; offset++)
        variants[(rotation + offset) % variants.length]];
      require(pair['case'] == name && pair['round'] == round &&
          pair['warmup'] == (round < warmups) && same(pair['rotated_order'], expectedOrder),
          'Wrong rotation in $name round $round');
      final results = (pair['results'] as List).map((row) => object(row, 'round result')).toList();
      require(results.length == variants.length &&
          same(results.map((row) => row['backend']).toList(), expectedOrder),
          'Result ordering mismatch in $name round $round');
      for (var position = 0; position < results.length; position++) {
        final result = Map<String, dynamic>.from(results[position])
          ..remove('successful_output_removed_after_verification');
        require(same(result, byVariant[expectedOrder[position]]![round]),
            'Round JSON differs from matching sample JSONL in $name round $round position $position');
        checked++;
      }
      if (round >= warmups) {
        for (var position = 0; position < expectedOrder.length; position++) {
          parityPositions['${round % 2}']![position][positionFor(expectedOrder[position])]++;
        }
      }
    }
    for (final parity in ['0', '1']) {
      require(same(parityPositions[parity], [[2, 2, 2], [2, 2, 2], [2, 2, 2]]),
          'Each variant position must occur twice per parity in $name');
    }
    orderCounts[name] = parityPositions;
    activationCounts.putIfAbsent(name, () => {'measured': 0, 'all': 0});
  }
  require(checked == 720, 'Expected 720 round results to match sample rows');
  final auditOutput = {
    'complete': true,
    'audited_utc': DateTime.now().toUtc().toIso8601String(),
    'frozen_case_count': expectedNames.length,
    'frozen_case_names': expectedNames,
    'case_census_profiles_verified': true,
    'all_expected_source_hashes_and_identity_hashes_match': true,
    'round_json_results_equal_sample_rows_ignoring_only_cleanup_flag': true,
    'round_result_rows_checked': checked,
    'rotation': 'rotation = (round + caseIndex) modulo 3; sample parity is round modulo 2',
    'measured_position_counts_per_case_and_parity': orderCounts,
    'auto_pool_expected_active_cases': expectedNames.where(autoPoolExpected).toList(),
    'auto_pool_activation_counts': activationCounts,
    'maximum_extraction_peak_rss_kib_all_720_rows': maxRssAll,
    'maximum_extraction_peak_rss_kib_measured_540_rows': maxRssMeasured,
    'source_runner_protocol_executable_hashes': {
      'runner': before['runner_sha256'], 'protocol': before['protocol_sha256'],
      'source_diff': before['source_diff_sha256'], 'source_hash_manifest': before['source_hash_manifest_sha256'],
      'executable': before['tree_test_executable_sha256'],
    },
  };
  final output = File('$raw/adaptive-workers-audit.json');
  require(!output.existsSync(), 'Refusing to replace existing audit ${output.path}');
  output.writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(auditOutput)}\n', flush: true);
  print(green('Audited $checked adaptive round results, exact profiles, rotation, activation and identity.'));
}

int positionFor(String first) => const {
  'typed-tree-1': 0,
  'typed-tree-4': 1,
  'typed-tree-auto': 2,
}[first]!;
