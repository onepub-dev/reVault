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

bool same(Object? a, Object? b) => jsonEncode(canonical(a)) == jsonEncode(canonical(b));

Object? canonical(Object? value) {
  if (value is Map) {
    final keys = value.keys.cast<String>().toList()..sort();
    return {for (final key in keys) key: canonical(value[key])};
  }
  if (value is List) return value.map(canonical).toList();
  return value;
}

Map<String, dynamic> readObject(String path) =>
    object(jsonDecode(File(path).readAsStringSync()), path);

void main(List<String> args) {
  require(args.length <= 1,
      'Usage: ./audit-worker-crossover-rounds.dart [BATCH_DIR]');
  final repo = Directory.current.parent.absolute.path;
  final batch = Directory(args.isEmpty
          ? '$repo/docs/evidence/post-security-workers-2026-10-11/raw/worker-crossover-batch'
          : args.single)
      .absolute
      .path;
  final evidence = Directory(batch).parent.path;
  final manifestPath = '$evidence/cases.json';
  final summaryPath = '$batch/metrics-summary-v2.json';
  final complete = readObject('$batch/complete.json');
  final manifest = (jsonDecode(File(manifestPath).readAsStringSync()) as List)
      .map((row) => object(row, 'case manifest row'))
      .toList();
  const sizes = <String, int>{
    '1mib': 1048576,
    '16mib': 16777216,
    '64mib': 67108864,
    '256mib': 268435456,
    '1024mib': 1073741824,
  };
  const profiles = <String, Map<String, Object>>{
    'source-raw-plain': {'corpus': 'source-bundle', 'compressed': false, 'mode': 'plain'},
    'source-zstd-plain': {'corpus': 'source-bundle', 'compressed': true, 'mode': 'plain'},
    'source-zstd-encrypted': {'corpus': 'source-bundle', 'compressed': true, 'mode': 'encrypted'},
    'random-zstd-fallback-plain': {'corpus': 'random', 'compressed': true, 'mode': 'plain'},
  };
  final expectedCases = <String>[];
  for (final size in sizes.entries) {
    for (final profile in profiles.entries) {
      expectedCases.add('${profile.key}-${size.key}');
    }
  }
  require(manifest.length == 20, 'Frozen case manifest must contain 20 rows');
  for (var i = 0; i < manifest.length; i++) {
    final row = manifest[i];
    final expected = expectedCases[i];
    require(row['name'] == expected, 'Unexpected frozen case at index $i: ${row['name']}');
    final sizeName = sizes.keys.singleWhere(expected.endsWith);
    final profileName = profiles.keys.singleWhere(expected.startsWith);
    require(row['bytes'] == sizes[sizeName] && row['files'] == 1 && row['unit'] == 65536,
        'Frozen size/files/unit mismatch in $expected');
    for (final entry in profiles[profileName]!.entries) {
      require(row[entry.key] == entry.value, 'Frozen profile mismatch in $expected: ${entry.key}');
    }
  }
  require(same(complete['variants'], ['typed-tree-1', 'typed-tree-4']) &&
      complete['sample_rows'] == 640 && complete['measured_rounds_per_case'] == 12 &&
      complete['warmup_rounds_per_case'] == 4,
      'Completion metadata does not match the frozen batch design');

  final measuredOrderCounts = <String, Map<String, List<int>>>{};
  var checkedRows = 0;
  var maxRssAll = 0;
  var maxRssMeasured = 0;
  final variants = ['typed-tree-1', 'typed-tree-4'];
  for (var caseIndex = 0; caseIndex < expectedCases.length; caseIndex++) {
    final caseName = expectedCases[caseIndex];
    final roundDirectory = '$batch/$caseName';
    final rowsByVariant = <String, Map<int, Map<String, dynamic>>>{};
    for (final variant in variants) {
      final samplesPath = '$roundDirectory/$variant/samples.jsonl';
      final rows = File(samplesPath)
          .readAsLinesSync()
          .where((line) => line.trim().isNotEmpty)
          .map((line) => object(jsonDecode(line), samplesPath))
          .toList();
      require(rows.length == 16, '$caseName/$variant must retain 16 rows');
      final byRound = <int, Map<String, dynamic>>{};
      for (final row in rows) {
        final round = row['round'];
        require(round is int && !byRound.containsKey(round),
            '$caseName/$variant has invalid or duplicate round $round');
        byRound[round as int] = row;
        require(row['case'] == caseName && row['backend'] == variant,
            'Sample identity mismatch in $caseName/$variant round $round');
        final rss = object(object(row['sample'], 'sample')['resources'], 'resources')['peak_rss_kib'];
        require(rss is num && rss.isFinite && rss > 0, 'Invalid extraction RSS in $caseName/$variant');
        if (rss > maxRssAll) maxRssAll = (rss as num).toInt();
        if (round >= 4 && rss > maxRssMeasured) maxRssMeasured = (rss as num).toInt();
      }
      require(byRound.keys.toSet().length == 16 &&
          List.generate(16, (round) => byRound.containsKey(round)).every((value) => value),
          'Missing retained round in $caseName/$variant');
      rowsByVariant[variant] = byRound;
    }
    final perParity = <String, List<int>>{'0': [0, 0], '1': [0, 0]};
    for (var round = 0; round < 16; round++) {
      final path = '$roundDirectory/round-${round.toString().padLeft(2, '0')}.json';
      final pair = readObject(path);
      final expectedOrder = ((round ~/ 2 + caseIndex) % 2 == 0)
          ? variants
          : variants.reversed.toList();
      require(pair['case'] == caseName && pair['round'] == round &&
          pair['warmup'] == (round < 4) && same(pair['rotated_order'], expectedOrder),
          'Planned pair-order mismatch in $caseName round $round');
      final results = (pair['results'] as List)
          .map((row) => object(row, 'round result'))
          .toList();
      require(results.length == 2, 'Expected two results in $caseName round $round');
      require(same(results.map((row) => row['backend']).toList(), expectedOrder),
          'Result order does not match rotation in $caseName round $round');
      for (var orderIndex = 0; orderIndex < results.length; orderIndex++) {
        final result = results[orderIndex];
        final variant = expectedOrder[orderIndex];
        final sample = rowsByVariant[variant]![round]!;
        final comparableResult = Map<String, dynamic>.from(result)
          ..remove('successful_output_removed_after_verification');
        require(same(comparableResult, sample),
            'Round result differs from retained sample row in $caseName/$variant/$round');
        checkedRows++;
      }
      if (round >= 4) {
        final parity = '${round % 2}';
        perParity[parity]![expectedOrder[0] == variants[0] ? 0 : 1]++;
      }
    }
    for (final parity in ['0', '1']) {
      require(same(perParity[parity], [3, 3]),
          'Expected three orders per parity in $caseName parity $parity');
    }
    measuredOrderCounts[caseName] = perParity;
  }
  require(checkedRows == 640, 'Expected all 640 round results to match sample rows');

  final summary = readObject(summaryPath);
  final summaryCases = object(summary['cases'], 'summary cases');
  final compressedLarge = <MapEntry<String, double>>[];
  for (final caseName in expectedCases) {
    final size = sizes.keys.singleWhere(caseName.endsWith);
    if (caseName.startsWith('source-zstd-') && size != '1mib') {
      final caseSummary = object(summaryCases[caseName], caseName);
      final comparisons = object(caseSummary['paired_comparisons'], 'paired comparisons');
      final comparison = object(comparisons['typed-tree-4_vs_typed-tree-1'], '4/1 comparison');
      final cpu = object(comparison['cpu_seconds'], 'CPU comparison');
      final ratio = cpu['geometric_mean_ratio'];
      require(ratio is num && ratio > 0, 'Invalid compressed-large CPU ratio for $caseName');
      compressedLarge.add(MapEntry(caseName, (ratio as num).toDouble()));
    }
  }
  require(compressedLarge.length == 8, 'Expected eight compressed-source cases at 16 MiB and above');
  compressedLarge.sort((a, b) => a.value.compareTo(b.value));
  final result = {
    'complete': true,
    'audited_utc': DateTime.now().toUtc().toIso8601String(),
    'case_manifest_sha256': sha256(manifestPath),
    'frozen_cases_in_order': expectedCases,
    'round_rotation': '(round ~/ 2 + caseIndex) % 2; expected pair is typed-tree-1 then typed-tree-4 for zero, reversed for one',
    'measured_round_order_counts_by_reader_parity': measuredOrderCounts,
    'round_json_results_equal_samples_jsonl': true,
    'round_result_rows_checked': checkedRows,
    'maximum_extraction_peak_rss_kib_all_640_rows': maxRssAll,
    'maximum_extraction_peak_rss_kib_measured_480_rows': maxRssMeasured,
    'compressed_source_cpu_ratio_4_vs_1_16mib_to_1024mib': {
      'definition': 'paired geometric mean CPU-seconds ratio in validated summary; less than one favors four workers',
      'minimum': {'case': compressedLarge.first.key, 'ratio': compressedLarge.first.value},
      'maximum': {'case': compressedLarge.last.key, 'ratio': compressedLarge.last.value},
      'cases': {for (final row in compressedLarge) row.key: row.value},
    },
  };
  final output = File('$batch/round-order-audit.json');
  require(!output.existsSync(), 'Refusing to replace existing audit ${output.path}');
  output.writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(result)}\n', flush: true);
  print(green('Audited $checkedRows retained round results, frozen cases, order balance, RSS and CPU ratios.'));
}

String sha256(String path) {
  final result = Process.runSync('sha256sum', ['--', path]);
  require(result.exitCode == 0, 'Cannot hash $path');
  return '${result.stdout}'.split(RegExp(r'\s+')).first;
}
