#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:dcli/dcli.dart';

const corpusRoot = '/home/bsutton/.cache/revault-protected-20261011';
const oldRoot = '/home/bsutton/.cache/revault-post-security-workers-20261011';
const scaleRoot = '$corpusRoot/scale-v1';
const parallelRoot = '$corpusRoot/parallel-v3';
const evidenceRoot = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/post-security-workers-2026-10-11/raw';
const rustRoot = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust';
const artifactCap = 24 * 1024 * 1024 * 1024;
const probe = 'file_format::candidate_files::resource_probe::candidate_file_resource_probe';
const affinityCpus = '0,1,2,3,4';
const variants = <String>['typed-tree-1', 'typed-tree-4'];

class CaseSpec {
  const CaseSpec(this.name, this.files, this.bytes, this.unit, this.root,
      this.corpus, this.compressed, this.mode);
  final String name;
  final int files;
  final int bytes;
  final int unit;
  final String root;
  final String corpus;
  final bool compressed;
  final String mode;
  int get payloadBytes => files * bytes;
  String sourceName(int index) => files == 1
      ? 'file-000000.bin'
      : 'file-${index.toString().padLeft(6, '0')}.bin';
}

class CapturedRun {
  const CapturedRun(this.exitCode, this.stdout, this.stderr);
  final int exitCode;
  final String stdout;
  final String stderr;
}

late List<CaseSpec> allCases;
late List<CaseSpec> cases;
late Map<String, dynamic> calibration;
Future<void> main(List<String> args) async {
 if(args.isEmpty || args[0]=='help') { print('prepare | create TEST_BINARY | calibrate | batch TEST_BINARY'); return; }
 if(args[0]=='prepare') { await prepareCases(); return; }
 allCases = _readCases();
 cases = allCases;
 if(args[0]=='create' && args.length==2) { for(final spec in allCases) await createTree(File(args[1]).absolute.path,spec); return; }
 if(args[0]=='calibrate') { await calibrateCases(); return; }
 if(args[0]=='batch' && args.length==2) { calibration=jsonDecode(File('$evidenceRoot/physical-read-calibration.json').readAsStringSync()) as Map<String,dynamic>; await runColdBatch(File(args[1]).absolute.path,12); return; }
 throw ArgumentError('Unknown phase or arguments');
}
int allocatedBytes(String path) {
 final stat=Process.runSync('stat',['--format=%b',path]);
 if(stat.exitCode!=0)throw StateError('Cannot inspect allocated blocks');
 return int.parse('${stat.stdout}'.trim())*512;
}
int calibratedBytes(String path) => (calibration[path] as Map)['minimum_physical_read_bytes'] as int;
int readGate(CaseSpec spec, bool isTree) => (calibratedBytes('${spec.root}/tree.lbox')+1)~/2;

List<CaseSpec> _readCases() {
  final rows = jsonDecode(File('$evidenceRoot/cases.json').readAsStringSync()) as List;
  return rows.map((row) => CaseSpec(row['name'] as String, 1,
      row['bytes'] as int, row['unit'] as int, row['root'] as String,
      row['corpus'] as String, row['compressed'] as bool,
      row['mode'] as String)).toList();
}

Future<void> prepareCases() async {
  final source = '$corpusRoot/source-bundle.bin';
  final random = '$corpusRoot/random-1gib-raw-encrypted/source/file-000000.bin';
  final randomInventory = jsonDecode(File('$corpusRoot/random-1gib-raw-encrypted/inventory.json').readAsStringSync()) as List;
  final expectedRandom = (randomInventory.single as Map)['sha256'];
  final ready = jsonDecode(File('$corpusRoot-evidence/corpus-ready.json').readAsStringSync()) as Map;
  if (_sha(source) != ready['bundle_sha256'] ||
      _sha('$corpusRoot/source-expanded-1gib.bin') != ready['expanded_sha256'] ||
      File(random).lengthSync() != 1073741824 || _sha(random) != expectedRandom) {
    throw StateError('Pinned source/random corpus identity validation failed');
  }
  if (Directory(oldRoot).existsSync()) throw StateError('Refusing existing experiment root: $oldRoot');
  Directory(oldRoot).createSync(recursive: true);
  final rows = <Map<String, Object>>[];
  final sizes = [1, 16, 64, 256, 1024].map((mib) => mib * 1024 * 1024).toList();
  final workloads = [
    ('source-raw-plain', 'source', false, 'plain'),
    ('source-zstd-plain', 'source', true, 'plain'),
    ('source-zstd-encrypted', 'source', true, 'encrypted'),
    ('random-zstd-fallback-plain', 'random', true, 'plain'),
  ];
  final projected = 2 * sizes.fold<int>(0, (sum, size) => sum + size) * workloads.length;
  final available = _availableBytes();
  if (projected > artifactCap || available < projected) {
    throw StateError('Insufficient budget for fixtures and candidate trees: need $projected bytes, have $available');
  }
  for (final size in sizes) {
    for (final workload in workloads) {
      final name = '${workload.$1}-${size ~/ (1024 * 1024)}mib';
      final root = '$oldRoot/fixtures/$name';
      Directory(root).createSync(recursive: true);
      final sourceDir = Directory('$root/source')..createSync();
      final sourceFile = File('${sourceDir.path}/file-000000.bin');
      _writePrefix(sourceFile, File(workload.$2 == 'source' ? source : random), size,
          repeat: workload.$2 == 'source');
      final descriptor = <String, Object>{'files': 1, 'bytes': size,
        'corpus': workload.$2 == 'source' ? 'source-bundle' : 'random',
        'compressed': workload.$3, 'mode': workload.$4, 'unpadded': false};
      _writeJson('$root/case.json', descriptor);
      final sourceHash = _sha(sourceFile.path);
      _writeJson('$root/inventory.json', [{'path': 'file-000000.bin', 'bytes': size, 'sha256': sourceHash}]);
      rows.add({'name': name, 'files': 1, 'bytes': size, 'unit': 65536,
        'root': root, 'corpus': descriptor['corpus']!, 'compressed': workload.$3,
        'mode': workload.$4, 'source_sha256': sourceHash,
        'source_path': sourceFile.path});
    }
  }
  _writeJson('$evidenceRoot/cases.json', rows);
  _writeJson('$evidenceRoot/source-corpus-identity.json', {
    'source_bundle_path': source, 'source_bundle_sha256': _sha(source),
    'expanded_1gib_path': '$corpusRoot/source-expanded-1gib.bin',
    'expanded_1gib_sha256': _sha('$corpusRoot/source-expanded-1gib.bin'),
    'random_1gib_path': random, 'random_1gib_sha256': _sha(random),
    'cases': rows, 'case_count': rows.length,
    'generation': 'exact prefixes of prior immutable source-expanded 1-GiB stream and deterministic random corpus; source workload repeats pinned bundle bytes from offset zero'});
  if (rows.length != 20) throw StateError('Expected 20 frozen fixtures');
  print(green('Prepared 20 source files and case manifests; no archive creation or timing ran.'));
}

void _writePrefix(File target, File input, int length, {required bool repeat}) {
  final source = input.openSync();
  if (target.existsSync()) throw StateError('Refusing existing source file: ${target.path}');
  final output = target.openSync(mode: FileMode.write);
  final buffer = Uint8List(65536);
  var written = 0;
  try {
    while (written < length) {
      if (repeat && source.positionSync() == input.lengthSync()) source.setPositionSync(0);
      final count = source.readIntoSync(buffer, 0,
          (length - written).clamp(0, buffer.length).toInt());
      if (count == 0) throw StateError('Input ended before requested prefix: ${input.path}');
      output.writeFromSync(buffer, 0, count);
      written += count;
    }
    output.flushSync();
  } finally {
    source.closeSync();
    output.closeSync();
  }
  if (target.lengthSync() != length) throw StateError('Wrong generated source length: ${target.path}');
}

Future<void> calibrateCases() async {
  final dest = File('$evidenceRoot/physical-read-calibration.json');
  if (dest.existsSync()) throw StateError('Refusing existing calibration');
  final result = <String, dynamic>{};
  final buffer = Uint8List(65536);
  for (final spec in allCases) {
    final path = '${spec.root}/tree.lbox';
    final scans = <Map<String, dynamic>>[];
    for (var index = 0; index < 2; index++) {
      final sync = Process.runSync('/usr/bin/sync', ['-d', '--', path]);
      final advise = Process.runSync('/usr/bin/fadvise', ['--advice', 'dontneed', path]);
      final core = Process.runSync('/usr/bin/fincore', ['--json', '--bytes', '--output', 'SIZE,RES,DIRTY,WRITEBACK,FILE', path]);
      dynamic decoded;
      try { decoded = jsonDecode(core.stdout as String); } catch (_) {}
      final rows = decoded is Map ? decoded['fincore'] : decoded;
      final row = rows is List && rows.length == 1 && rows.single is Map ? rows.single as Map : null;
      final resident = row?['res'] is num ? (row!['res'] as num).toInt() : int.tryParse('${row?['res']}');
      if (sync.exitCode != 0 || advise.exitCode != 0 || core.exitCode != 0 || resident != 0) {
        throw StateError('Calibration cache gate failed for $path scan $index');
      }
      final before = _selfReadBytes();
      final input = File(path).openSync();
      var logical = 0;
      try { while (true) { final n = input.readIntoSync(buffer); if (n == 0) break; logical += n; } }
      finally { input.closeSync(); }
      final physical = _selfReadBytes() - before;
      scans.add({'sync_exit_code': sync.exitCode, 'fadvise_exit_code': advise.exitCode,
        'fincore_exit_code': core.exitCode, 'resident_bytes_before': resident,
        'physical_read_bytes': physical, 'logical_bytes': logical,
        'buffer_bytes': buffer.length});
      result[path] = {'scans': scans, 'completed_scans': scans.length};
      _writeJson(dest.path, result);
      if (physical <= 0 || logical != File(path).lengthSync()) throw StateError('Calibration traffic/length invalid for $path');
    }
    final a = scans[0]['physical_read_bytes'] as int, b = scans[1]['physical_read_bytes'] as int;
    final min = a < b ? a : b, max = a > b ? a : b;
    if (max > min * 1.1) throw StateError('Calibration scans disagree by more than 10%: $path');
    result[path] = {'scans': scans, 'minimum_physical_read_bytes': min,
      'maximum_physical_read_bytes': max, 'required_extraction_read_bytes': (min + 1) ~/ 2,
      'fraction': 0.5, 'basis': 'two independently evicted full-archive synchronous 64KiB reads using this process read_bytes counter'};
    _writeJson(dest.path, result);
  }
  print(green('Calibrated all 20 candidate archives with two cold reads each.'));
}

int _selfReadBytes() => int.parse(File('/proc/self/io').readAsLinesSync()
    .singleWhere((line) => line.startsWith('read_bytes:')).split(':').last.trim());

Future<void> createTree(String candidate, CaseSpec spec) async {
  _validateFixture(spec);
  if (!File(candidate).existsSync()) throw StateError('Missing candidate test binary: $candidate');
  for (final name in ['tree.lbox', 'tree.public', 'tree-staging.lbox']) {
    if (FileSystemEntity.typeSync('${spec.root}/$name', followLinks: false) != FileSystemEntityType.notFound) {
      throw StateError('Refusing existing tree creation artifact: ${spec.root}/$name');
    }
  }
  _validateBudget(projectedNewBytes: 2 * spec.payloadBytes);
  final before = _inputIdentity(spec, includeTree: false);
  final candidateBefore = _sha(candidate);
  final command = [
    'taskset', '-c', affinityCpus, '/usr/bin/time', '-v', 'timeout', '900',
    candidate, '--exact', probe, '--ignored', '--nocapture',
  ];
  final env = <String, String>{
    ...Platform.environment,
    'REVAULT_CANDIDATE_ROOT': spec.root,
    'REVAULT_CANDIDATE_PHASE': 'tree-create',
    'REVAULT_CANDIDATE_UNIT': '${spec.unit}',
  };
  final run = await _runCaptured(command.first, command.skip(1).toList(),
      environment: env, timeoutSeconds: 930);
  final records = _records(run.stdout, 'CANDIDATE_SAMPLE ');
  final sample = records.length == 1 ? records.single : null;
  final after = _inputIdentity(spec, includeTree: false);
  final candidateAfter = _sha(candidate);
  final census = sample?['codec_census'];
  final rawChunks = census is Map ? census['raw_chunks'] : null;
  final zstdChunks = census is Map ? census['zstd_chunks'] : null;
  final logicalCensusBytes = census is Map ? census['logical_bytes'] : null;
  final censusValid = census is Map && logicalCensusBytes == spec.payloadBytes &&
      rawChunks is int && rawChunks >= 0 && zstdChunks is int && zstdChunks >= 0 &&
      (spec.corpus == 'random'
          ? rawChunks > 0 && zstdChunks == 0
          : spec.compressed ? zstdChunks > 0 : rawChunks > 0 && zstdChunks == 0);
  final valid = run.exitCode == 0 &&
      sample?['verified'] == true &&
      censusValid &&
      File('${spec.root}/tree.lbox').existsSync() &&
      File('${spec.root}/tree.lbox').lengthSync() > 0 &&
      jsonEncode(before) == jsonEncode(after) &&
      candidateBefore == candidateAfter &&
      _apparentBytes(Directory(oldRoot)) <= artifactCap;
  final outputDir = Directory('$evidenceRoot/tree-creation')..createSync(recursive: true);
  _writeJson('${outputDir.path}/${spec.name}.json', {
    'command': command,
    'environment': {
      'REVAULT_CANDIDATE_ROOT': spec.root,
      'REVAULT_CANDIDATE_PHASE': 'tree-create',
      'REVAULT_CANDIDATE_UNIT': '${spec.unit}',
    },
    'candidate_sha256_before': candidateBefore,
    'candidate_sha256_after': candidateAfter,
    'inputs_before': before,
    'inputs_after': after,
    'sample': sample,
    'codec_census_verified': censusValid,
    'tree_sha256': File('${spec.root}/tree.lbox').existsSync() ? _sha('${spec.root}/tree.lbox') : null,
    'tree_public_sha256': File('${spec.root}/tree.public').existsSync() ? _sha('${spec.root}/tree.public') : null,
    'exit_code': run.exitCode,
    'stdout': run.stdout,
    'stderr': run.stderr,
    'verified': valid,
  });
  if (!valid) throw StateError('Tree creation/verification failed for ${spec.name}; evidence retained.');
  print(green('Created and verified tree archive for ${spec.name}; logs under ${outputDir.path}.'));
}

Future<Map<String, dynamic>> _extractSample(
  CaseSpec spec,
  String backend,
  String executable,
  int round,
  bool warmup,
  String evidenceDir, {
  int? workerCount,
  required bool durable,
  required int minimumStorageReadBytes,
  int admissionAttempt = 0,
}) async {
  _validateBudget(projectedNewBytes: spec.payloadBytes);
  _validateFixture(spec);
  final requestedParity = round % 2;
  final label = '${warmup ? 'warmup' : 'sample'}-${round.toString().padLeft(2, '0')}-$backend-attempt${admissionAttempt.toString().padLeft(2, '0')}';
  final output = '${spec.root}/extract-output/protected-recovery/$label';
  if (FileSystemEntity.typeSync(output, followLinks: false) != FileSystemEntityType.notFound) {
    throw StateError('Refusing existing extraction target: $output');
  }
  Directory('${spec.root}/extract-output/protected-recovery').createSync(recursive: true);
  final isTree = backend.startsWith('typed-tree-');
  final effectiveWorkers = isTree ? (workerCount ?? 1) : null;
  if (isTree && ![1, 2, 4].contains(effectiveWorkers)) {
    throw StateError('Unsupported typed-tree worker count: $effectiveWorkers');
  }
  final inputPath = '${spec.root}/tree.lbox';
  final outputDir = Directory('$evidenceDir/logs')..createSync(recursive: true);
  final base = {
    'case': spec.name,
    'round': round,
    'warmup': warmup,
    'backend': backend,
    'worker_count': effectiveWorkers,
    'input_path': inputPath,
    'archive_apparent_bytes': File(inputPath).lengthSync(),
    'archive_allocated_bytes': allocatedBytes(inputPath),
    'calibrated_archive_physical_read_bytes': calibratedBytes(inputPath),
    'read_gate_policy': 'at least 50% independent cold full-archive calibrated physical read bytes; zero fincore residency required',
    'output_path': output,
    'affinity': affinityCpus,
    'durable': durable,
    'requested_reader_tid_parity': requestedParity,
    'admission_attempt': admissionAttempt,
  };
  final evictionAttempts = <Map<String, dynamic>>[];
  Map<String, dynamic> cacheGate = {};
  for (var attempt = 0; attempt < 3; attempt++) {
    cacheGate = _evictAndInspect(inputPath);
    evictionAttempts.add({'attempt': attempt, ...cacheGate});
    if (cacheGate['success'] == true) break;
    if (attempt < 2) await Future<void>.delayed(const Duration(milliseconds: 100));
  }
  cacheGate = {...cacheGate, 'attempts': evictionAttempts,
    'maximum_attempts': 3, 'inter_attempt_delay_ms': 100};
  _writeJson('${outputDir.path}/$label.cache-gate.json', {
    ...base,
    ...cacheGate,
  });
  if (cacheGate['success'] != true) {
    _writeJson('${outputDir.path}/$label.failed-cold-gate.json', {
      ...base,
      ...cacheGate,
      'candidate_started': false,
    });
    throw StateError('Cold cache gate failed for ${spec.name}/$backend; retained gate evidence.');
  }
  final binaryHash = _sha(executable);
  final command = <String>[
    'taskset', '-c', affinityCpus, '/usr/bin/time', '-v', 'timeout', '900',
    executable, '--exact', probe, '--ignored', '--nocapture',
  ];
  final env = <String, String>{
    ...Platform.environment,
    'REVAULT_EXTRACT_DURABLE': durable ? '1' : '0',
    'REVAULT_EXTRACT_TID_PARITY': '$requestedParity',
    'REVAULT_EXTRACT_PID_NAMESPACE': 'pid:[4026531836]',
    'REVAULT_EXTRACT_BLOCK_DEVICES': 'nvme0n1,nvme1n1',
    if (isTree) 'REVAULT_CANDIDATE_ROOT': spec.root,
    if (isTree) 'REVAULT_CANDIDATE_PHASE': 'tree-extract',
    if (isTree) 'REVAULT_CANDIDATE_UNIT': '${spec.unit}',
    if (isTree) 'REVAULT_EXTRACT_OUTPUT': output,
    if (isTree) 'REVAULT_READ_WORKERS': '$effectiveWorkers',
  };
  final commandInfo = {
    ...base,
    'command': command,
    'environment': {
      'REVAULT_EXTRACT_DURABLE': durable ? '1' : '0',
      'REVAULT_EXTRACT_TID_PARITY': '$requestedParity',
      'REVAULT_EXTRACT_PID_NAMESPACE': 'pid:[4026531836]',
      'REVAULT_EXTRACT_BLOCK_DEVICES': 'nvme0n1,nvme1n1',
      'REVAULT_CANDIDATE_ROOT': spec.root,
      'REVAULT_CANDIDATE_PHASE': 'tree-extract',
      'REVAULT_CANDIDATE_UNIT': '${spec.unit}',
      'REVAULT_EXTRACT_OUTPUT': output,
      'REVAULT_READ_WORKERS': '$effectiveWorkers',
    },
    'executable': executable,
    'executable_sha256_before': binaryHash,
    'cache_gate': cacheGate,
  };
  _writeJson('${outputDir.path}/$label.command.json', commandInfo);
  final run = await _runCaptured(command.first, command.skip(1).toList(),
      environment: env, timeoutSeconds: 930);
  Map<String, dynamic>? sample;
  try {
    if (isTree) {
      final rows = _records(run.stdout, 'CANDIDATE_SAMPLE ');
      if (rows.length == 1) sample = rows.single;
    } else {
      sample = jsonDecode(run.stdout.trim()) as Map<String, dynamic>;
    }
  } catch (_) {
    sample = null;
  }
  if (sample?['kind'] == 'extraction_admission_skip') {
    final skip = {...commandInfo, 'sample': sample, 'exit_code': run.exitCode,
      'executable_sha256_after': _sha(executable), 'excluded_from_timing': true};
    _writeJson('${outputDir.path}/$label.admission-skip.json', skip);
    File('${outputDir.path}/$label.stdout').writeAsStringSync(run.stdout, flush: true);
    File('${outputDir.path}/$label.stderr').writeAsStringSync(run.stderr, flush: true);
    if (run.exitCode != 0 || skip['executable_sha256_after'] != binaryHash ||
        sample?['reason'] != 'reader_tid_parity_mismatch' ||
        sample?['archive_opened'] != false || sample?['output_created'] != false ||
        sample?['requested_reader_tid_parity'] != requestedParity ||
        FileSystemEntity.typeSync(output, followLinks: false) != FileSystemEntityType.notFound) {
      throw StateError('Unexpected admission failure; logs retained');
    }
    if (admissionAttempt >= 15) throw StateError('Reader parity admission exhausted 16 attempts; logs retained');
    // A fixed subprocess count can otherwise preserve the wrong parity forever.
    // Alternate zero/one no-op child across retry transitions. This depends only
    // on skipped identity, never elapsed time, and preserves the sixteen-attempt cap.
    final nudgeCount = admissionAttempt % 2;
    final nudges = <Map<String, dynamic>>[];
    for (var index = 0; index < nudgeCount; index++) {
      final nudge = await _runCaptured('/usr/bin/true', [], timeoutSeconds: 10);
      nudges.add({'command': ['/usr/bin/true'], 'exit_code': nudge.exitCode,
        'stdout': nudge.stdout, 'stderr': nudge.stderr});
    }
    skip['retry_pid_allocation'] = {
      'policy': 'alternate zero/one harmless child per skipped-attempt transition; identity-based only',
      'requested_child_count': nudgeCount, 'children': nudges,
    };
    _writeJson('${outputDir.path}/$label.admission-skip.json', skip);
    if (nudges.any((nudge) => nudge['exit_code'] != 0)) {
      throw StateError('Harmless parity retry child failed; skip evidence retained');
    }
    return _extractSample(spec, backend, executable, round, warmup, evidenceDir,
      workerCount: workerCount, durable: durable, minimumStorageReadBytes: minimumStorageReadBytes,
      admissionAttempt: admissionAttempt + 1);
  }
  final parsedTime = _parseTime(run.stderr);
  final extracted = Directory(output);
  final inventory = extracted.existsSync() ? _directoryInventory(extracted) : <String, int>{};
  final bytes = inventory.values.fold<int>(0, (sum, length) => sum + length);
  final resources = sample?['resources'];
  final cpu = resources is Map ? resources['cpu'] : null;
  final rss = resources is Map ? resources['peak_rss_kib'] : null;
  final io = sample?['storage_io'];
  final readBytes = io is Map ? io['read_bytes'] : null;
  final durabilityDescription = sample?['durability'];
  final syncCountsValid = sample != null &&
      (durable
          ? sample['file_syncs'] == spec.files && sample['directory_syncs'] == 2
          : sample['file_syncs'] == 0 && sample['directory_syncs'] == 0);
  final verified = run.exitCode == 0 &&
      sample?['reader_identity'] is Map &&
      (sample!['reader_identity'] as Map)['reader_tid_parity'] == requestedParity &&
      (sample['reader_identity'] as Map)['pid_namespace'] == 'pid:[4026531836]' &&
      (sample['reader_identity'] as Map)['namespace_tids'] is List &&
      ((sample['reader_identity'] as Map)['namespace_tids'] as List).length == 1 &&
      sample['requested_reader_tid_parity'] == requestedParity &&
      _validBlockCounters(sample['block_device_io']) &&
      sample?['verified'] == true &&
      sample?['files'] == spec.files &&
      sample?['logical_bytes_written'] == spec.payloadBytes &&
      cpu is Map &&
      rss is num &&
      io is Map &&
      readBytes is num && readBytes >= minimumStorageReadBytes &&
      durabilityDescription is String &&
      syncCountsValid &&
      inventory.length == spec.files &&
      bytes == spec.payloadBytes;
  final executableHashAfter = _sha(executable);
  final drain = verified && executableHashAfter == binaryHash
      ? await _drainOwnedOutput(spec, extracted, inventory)
      : <String, dynamic>{'passed': false, 'seconds': 0.0, 'not_attempted': true};
  final fullyVerified = verified && executableHashAfter == binaryHash && drain['passed'] == true;
  final result = <String, dynamic>{
    ...commandInfo,
    'exit_code': run.exitCode,
    'sample': sample,
    'time_v': parsedTime,
    'process_wall_seconds': parsedTime['elapsed_seconds'],
    'process_max_rss_kib': parsedTime['maximum_resident_set_kib'],
    'process_user_seconds': parsedTime['user_seconds'],
    'process_system_seconds': parsedTime['system_seconds'],
    'output_file_count': inventory.length,
    'output_total_bytes': bytes,
    'source_and_output_byte_verification': sample?['verified'] == true,
    'cold_disk_gate_passed': cacheGate['success'] == true,
    'observed_process_storage_read_bytes': readBytes,
    'minimum_storage_read_bytes': minimumStorageReadBytes,
    'storage_read_bytes_per_payload_byte': readBytes is num ? readBytes / spec.payloadBytes : null,
    'durability_description': durabilityDescription,
    'executable_sha256_after': executableHashAfter,
    'executable_stable_for_sample': executableHashAfter == binaryHash,
    'postprocess_drain': drain,
    'postprocess_drain_passed': drain['passed'] == true,
    'postprocess_drain_seconds': drain['seconds'],
    'verified': fullyVerified,
    'output_retained_on_failure': !fullyVerified,
  };
  File('${outputDir.path}/$label.stdout').writeAsStringSync(run.stdout, flush: true);
  File('${outputDir.path}/$label.stderr').writeAsStringSync(run.stderr, flush: true);
  File('${outputDir.path}/$label.result.json').writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert(result), flush: true);
  final samplesFile = File('$evidenceDir/samples.jsonl');
  samplesFile.parent.createSync(recursive: true);
  samplesFile.writeAsStringSync('${jsonEncode(result)}\n',
      mode: FileMode.append, flush: true);
  if (!result['verified']) {
    throw StateError('Extraction failed verification for ${spec.name}/$backend; output/logs retained at $output.');
  }
  if (!output.startsWith('${spec.root}/extract-output/protected-recovery/')) {
    throw StateError('Refusing cleanup outside runner-owned output path: $output');
  }
  extracted.deleteSync(recursive: true);
  result['successful_output_removed_after_verification'] = true;
  return result;
}

bool _validBlockCounters(dynamic value) {
  if (value is! Map || value['devices'] is! List) return false;
  final devices = value['devices'] as List;
  if (devices.length != 2) return false;
  for (var index = 0; index < devices.length; index++) {
    final row = devices[index];
    if (row is! Map || row['device'] != ['nvme0n1', 'nvme1n1'][index] || row['delta'] is! Map) return false;
    final delta = row['delta'] as Map;
    if (delta['sector_bytes'] != 512 || delta['read_sectors'] is! int || delta['write_sectors'] is! int ||
        (delta['read_sectors'] as int) < 0 || (delta['write_sectors'] as int) < 0) return false;
  }
  return true;
}

Future<Map<String, dynamic>> _drainOwnedOutput(
    CaseSpec spec, Directory output, Map<String, int> inventory) async {
  final watch = Stopwatch()..start();
  final commands = <Map<String, dynamic>>[];
  var passed = false;
  String? failure;
  try {
    if (!output.path.startsWith('${spec.root}/extract-output/protected-recovery/')) {
      throw StateError('Drain outside runner-owned output refused');
    }
    final paths = <String>[];
    for (var index = 0; index < spec.files; index++) {
      final name = spec.sourceName(index);
      final path = '${output.path}/$name';
      if (inventory[name] != spec.bytes ||
          FileSystemEntity.typeSync(path, followLinks: false) != FileSystemEntityType.file) {
        throw StateError('Drain requires expected synthetic regular files');
      }
      paths.add(path);
    }
    Future<void> syncPaths(List<String> ownedPaths) async {
      final result = await _runCaptured('/usr/bin/sync', ['--', ...ownedPaths], timeoutSeconds: 930);
      commands.add({'executable': '/usr/bin/sync', 'flags': ['--'],
        'path_count': ownedPaths.length, 'first_owned_path': ownedPaths.first,
        'last_owned_path': ownedPaths.last,
        'exit_code': result.exitCode, 'stdout': result.stdout, 'stderr': result.stderr});
      if (result.exitCode != 0) throw StateError('Scoped output sync failed');
    }
    for (var start = 0; start < paths.length; start += 500) {
      final end = start + 500 < paths.length ? start + 500 : paths.length;
      await syncPaths(paths.sublist(start, end));
    }
    await syncPaths([output.path, output.parent.path]);
    passed = true;
  } catch (error) {
    failure = error.toString();
  }
  watch.stop();
  return {'passed': passed, 'seconds': watch.elapsedMicroseconds / 1000000,
    'scope': 'owned output files, output directory and parent after process verification; outside extraction timer',
    'file_count': spec.files, 'directory_count': 2, 'commands': commands, 'failure': failure};
}

Map<String, dynamic> _evictAndInspect(String inputPath) {
  final sync = Process.runSync('/usr/bin/sync', ['-d', '--', inputPath]);
  final advise = Process.runSync('/usr/bin/fadvise', ['--advice', 'dontneed', inputPath]);
  final core = Process.runSync('/usr/bin/fincore', [
    '--json', '--bytes', '--output', 'SIZE,RES,DIRTY,WRITEBACK,FILE', inputPath,
  ]);
  dynamic decoded;
  try {
    decoded = jsonDecode(core.stdout as String);
  } catch (_) {}
  final rows = decoded is Map ? decoded['fincore'] : decoded;
  final row = rows is List && rows.length == 1 && rows.single is Map
      ? rows.single as Map
      : null;
  final resident = row?['res'];
  final residentBytes = resident is num ? resident.toInt() : int.tryParse('$resident');
  final success = sync.exitCode == 0 &&
      advise.exitCode == 0 &&
      core.exitCode == 0 &&
      residentBytes == 0;
  return {
    'sync_exit_code': sync.exitCode,
    'sync_stderr': sync.stderr,
    'fadvise_exit_code': advise.exitCode,
    'fadvise_stdout': advise.stdout,
    'fadvise_stderr': advise.stderr,
    'fincore_exit_code': core.exitCode,
    'fincore_stdout': core.stdout,
    'fincore_stderr': core.stderr,
    'resident_bytes_after_dontneed': residentBytes,
    'required_resident_bytes': 0,
    'success': success,
  };
}

Future<CapturedRun> _runCaptured(String executable, List<String> args,
    {Map<String, String>? environment, required int timeoutSeconds}) async {
  final command = [executable, ...args];
  final wrapped = <String>[
    '/usr/bin/time', '-v', 'timeout', '$timeoutSeconds', ...command,
  ];
  final process = await Process.start(wrapped.first, wrapped.skip(1).toList(),
      environment: environment);
  final stdout = process.stdout.transform(utf8.decoder).join();
  final stderr = process.stderr.transform(utf8.decoder).join();
  final exitCode = await process.exitCode;
  return CapturedRun(exitCode, await stdout, await stderr);
}

Future<void> runColdBatch(
    String treeExecutable, int measuredRounds) async {
  if (!File(treeExecutable).existsSync()) {
    throw StateError('Missing extraction executable');
  }
  for (final spec in cases) _validateReady(spec);
  _validateBudget(projectedNewBytes: cases.map((spec)=>spec.payloadBytes).reduce((a,b)=>a>b?a:b));
  final batch = Directory('$evidenceRoot/worker-crossover-batch');
  if (batch.existsSync()) throw StateError('Refusing existing batch: ${batch.path}');
  batch.createSync(recursive: true);
  final before = _batchIdentity(treeExecutable);
  _writeJson('${batch.path}/identity-before.json', before);
  _writeJson('${batch.path}/capacity-before.json', {
    'apparent_bytes_before': _apparentBytes(Directory(oldRoot)),
    'projected_new_bytes': cases.map((spec)=>spec.payloadBytes).reduce((a,b)=>a>b?a:b),
    'artifact_cap_bytes': artifactCap,
    'available_bytes': _availableBytes(),
    'assertion_passed': true,
  });
  final caseAudits = <Map<String, dynamic>>[];
  var totalRows = 0;
  Object? failure;
  StackTrace? failureStack;
  try {
    for (var caseIndex = 0; caseIndex < cases.length; caseIndex++) {
      final spec = cases[caseIndex];
      final inputsBefore = _inputIdentity(spec);
      var roundsComplete = 0;
      final counts = <String, int>{for (final variant in variants) variant: 0};
      Object? caseFailure;
      StackTrace? caseFailureStack;
      try {
        for (var round = 0; round < measuredRounds + 4; round++) {
          final warmup = round < 4;
          final rotation = (round ~/ 2 + caseIndex) % variants.length;
          final order = [
            for (var offset = 0; offset < variants.length; offset++)
              variants[(rotation + offset) % variants.length],
          ];
          final rows = <Map<String, dynamic>>[];
          for (final variant in order) {
            final isTree = true;
            final workers = int.parse(variant.split('-').last);
            final row = await _extractSample(
              spec,
              variant,
              treeExecutable,
              round,
              warmup,
              '${batch.path}/${spec.name}/$variant',
              workerCount: workers,
              durable: false,
              minimumStorageReadBytes: readGate(spec,isTree),
            );
            rows.add(row);
            counts[variant] = counts[variant]! + 1;
            totalRows++;
          }
          roundsComplete++;
          _writeJson('${batch.path}/${spec.name}/round-${round.toString().padLeft(2, '0')}.json', {
            'case': spec.name,
            'round': round,
            'warmup': warmup,
            'rotated_order': order,
            'results': rows,
          });
        }
      } catch (error, stack) {
        caseFailure = error;
        caseFailureStack = stack;
      }
      final inputsAfter = _inputIdentity(spec);
      final stable = jsonEncode(inputsBefore) == jsonEncode(inputsAfter);
      final audit = {
        'case': spec.name,
        'input_identity_before': inputsBefore,
        'input_identity_after': inputsAfter,
        'stable': stable,
        'rounds_complete': roundsComplete,
        'warmup_rounds_planned': 4,
        'measured_rounds_planned': measuredRounds,
        'sample_counts': counts,
        'failure': caseFailure?.toString(),
      };
      caseAudits.add(audit);
      _writeJson('${batch.path}/${spec.name}/case-audit.json', audit);
      if (!stable) throw StateError('Inputs changed during ${spec.name}; evidence retained.');
      if (caseFailure != null) Error.throwWithStackTrace(caseFailure, caseFailureStack!);
    }
  } catch (error, stack) {
    failure = error;
    failureStack = stack;
  } finally {
    final after = _batchIdentity(treeExecutable);
    _writeJson('${batch.path}/identity-after.json', after);
    if (!_stableBatchIdentity(before, after)) {
      failure ??= StateError('Source/executable identity changed during cold batch.');
      failureStack ??= StackTrace.current;
    }
  }
  if (failure != null) Error.throwWithStackTrace(failure, failureStack!);
  _writeJson('${batch.path}/complete.json', {
    'complete': true,
    'measured_rounds_per_case': measuredRounds,
    'warmup_rounds_per_case': 4,
    'reader_tid_parity_policy': 'round modulo 2; six measured and two preparatory samples per parity per variant',
    'admission_max_attempts': 16,
    'expected_pid_namespace': 'pid:[4026531836]',
    'device_counter_scope': 'global counters; corroboration only, parity does not imply identical physical mirror',
    'sample_rows': totalRows,
    'cases': caseAudits,
    'variants': variants,
    'affinity_mask_for_every_sample': affinityCpus,
    'compared_workers': [1, 4],
    'cold_gate': 'sync -d, fadvise DONTNEED, fincore RES == 0; sample must report process storage_io.read_bytes >= 50% of independent cold full-archive calibrated physical read bytes',
    'durable': false,
    'artifact_cap_bytes': artifactCap,
    'apparent_bytes_after': _apparentBytes(Directory(oldRoot)),
  });
  print(green('Completed cold buffered extraction batch: $totalRows verified fresh-process samples.'));
}

void _validateFixture(CaseSpec spec) {
  for (final name in ['case.json', 'inventory.json', 'source']) {
    final type = FileSystemEntity.typeSync('${spec.root}/$name', followLinks: true);
    if (type == FileSystemEntityType.notFound) throw StateError('Missing ${spec.root}/$name');
  }
  final caseData = jsonDecode(File('${spec.root}/case.json').readAsStringSync()) as Map<String, dynamic>;
  if (caseData['files'] != spec.files || caseData['bytes'] != spec.bytes ||
      caseData['mode'] != spec.mode || caseData['compressed'] != spec.compressed) {
    throw StateError('Unexpected fixture descriptor at ${spec.root}: $caseData');
  }
  if (!File('${spec.root}/source/${spec.sourceName(0)}').existsSync()) {
    throw StateError('Source fixture missing for ${spec.name}');
  }
}

void _validateReady(CaseSpec spec) {
  _validateFixture(spec);
  for (final name in ['tree.lbox', 'tree.public']) {
    final file = File('${spec.root}/$name');
    if (!file.existsSync() || file.lengthSync() == 0) {
      throw StateError('Tree fixture is not ready: ${file.path}');
    }
  }
}

Map<String, dynamic> _inputIdentity(CaseSpec spec, {bool includeTree = true}) => {
      'source': _sourceIdentity(spec),
      'case_json_sha256': _sha('${spec.root}/case.json'),
      'inventory_sha256': _sha('${spec.root}/inventory.json'),
      if (includeTree) 'tree_sha256': _sha('${spec.root}/tree.lbox'),
      if (includeTree) 'tree_public_sha256': _sha('${spec.root}/tree.public'),
    };

Map<String, dynamic> _sourceIdentity(CaseSpec spec) {
  final inventoryPath = '${spec.root}/inventory.json';
  final inventory = jsonDecode(File(inventoryPath).readAsStringSync()) as List<dynamic>;
  if (inventory.length != spec.files) throw StateError('Inventory count mismatch for ${spec.name}');
  final expected = <String, String>{};
  for (var index = 0; index < inventory.length; index++) {
    final row = inventory[index] as Map<String, dynamic>;
    final path = '${spec.root}/source/${row['path']}';
    if (row['bytes'] != spec.bytes) throw StateError('Unexpected source length for $path');
    expected[path] = row['sha256'] as String;
  }
  final actual = <String, String>{};
  final allPaths = expected.keys.toList();
  for (var start = 0; start < allPaths.length; start += 500) {
    final end = (start + 500).clamp(0, allPaths.length);
    final batch = allPaths.sublist(start, end);
    final result = Process.runSync('sha256sum', batch);
    if (result.exitCode != 0) throw StateError('Source hashing failed: ${result.stderr}');
    for (final line in (result.stdout as String).split('\n').where((line) => line.isNotEmpty)) {
      final split = line.indexOf('  ');
      if (split < 0) throw StateError('Unexpected sha256sum output: $line');
      actual[line.substring(split + 2)] = line.substring(0, split);
    }
  }
  if (actual.length != expected.length || expected.entries.any((entry) => actual[entry.key] != entry.value)) {
    throw StateError('Source files do not match inventory for ${spec.name}');
  }
  return {
    'inventory_sha256': _sha(inventoryPath),
    'file_count': actual.length,
    'unique_file_hashes': actual.values.toSet().length,
    'aggregate_manifest_sha256': _hashManifest(actual),
  };
}

String _hashManifest(Map<String, String> hashes) {
  final lines = hashes.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
  final temp = File('$evidenceRoot/.manifest-${DateTime.now().microsecondsSinceEpoch}');
  temp.writeAsStringSync(lines.map((entry) => '${entry.value}  ${entry.key}\n').join(), flush: true);
  final hash = _sha(temp.path);
  temp.deleteSync();
  return hash;
}

Map<String, dynamic> _batchIdentity(String treeExecutable) {
  final sourceHashes = <String, String>{};
  for (final directory in [
    Directory('$rustRoot/revault_lockbox_api/src'),
    Directory('$rustRoot/revault_lockbox_api/benches'),
  ]) {
    for (final entity in directory.listSync(recursive: true, followLinks: false)) {
      if (entity is File && entity.path.endsWith('.rs')) {
        sourceHashes[entity.path.substring(rustRoot.length + 1)] = _sha(entity.path);
      }
    }
  }
  for (final path in [
    '$rustRoot/Cargo.lock',
    '$rustRoot/Cargo.toml',
    '$rustRoot/rust-toolchain.toml',
    '$rustRoot/revault_lockbox_api/Cargo.toml',
    '$rustRoot/revault_lockbox_api/build.rs',
  ]) {
    if (File(path).existsSync()) sourceHashes[path.substring(rustRoot.length + 1)] = _sha(path);
  }
  final head = Process.runSync('git', ['rev-parse', 'HEAD'], workingDirectory: '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust');
  final findmnt = Process.runSync('findmnt', [
    '-T', oldRoot, '-o', 'TARGET,SOURCE,FSTYPE,OPTIONS', '--json',
  ]);
  final lscpu = Process.runSync('lscpu', ['-e=CPU,CORE,SOCKET,NODE,ONLINE']);
  return {
    'source_revision': '${head.stdout}'.trim(),
    'source_base_revision': 'a191020aabd20387818a307f0ec5cd9e2a841966',
    'build_source_epoch': 'current issue-310 worktree source; separately built Rust 1.88.0 locked/offline release with external-source; no tests or timings in the build command',
    'source_hashes': sourceHashes,
    'source_diff_sha256': _sha('$evidenceRoot/source-diff-a191020a.patch'),
    'source_hash_manifest_sha256': _sha('$evidenceRoot/source-hashes-before-measurements.txt'),
    'runner_sha256': _sha(Platform.script.toFilePath()),
    'calibration_sha256': _sha('$evidenceRoot/physical-read-calibration.json'),
    'protocol_sha256': _sha('$evidenceRoot/protocol-before-measurements.md'),
    'tree_test_executable': treeExecutable,
    'tree_test_executable_sha256': _sha(treeExecutable),
    'fixture_manifest_sha256': _sha('$evidenceRoot/cases.json'),
    'source_corpus_identity_sha256': _sha('$evidenceRoot/source-corpus-identity.json'),
    'captured_utc': DateTime.now().toUtc().toIso8601String(),
    'cpu_model': File('/proc/cpuinfo').readAsLinesSync().firstWhere(
        (line) => line.startsWith('model name'), orElse: () => 'model name: unknown'),
    'cpu_topology': '${lscpu.stdout}',
    'allowed_cpu_list': File('/proc/self/status').readAsLinesSync().firstWhere(
        (line) => line.startsWith('Cpus_allowed_list:'), orElse: () => 'unknown'),
    'shared_affinity_mask': affinityCpus,
    'filesystem': '${findmnt.stdout}',
    'load_average': File('/proc/loadavg').readAsStringSync().trim(),
    'meminfo': File('/proc/meminfo').readAsStringSync(),
    'fadvise_path': '/usr/bin/fadvise',
    'fincore_path': '/usr/bin/fincore',
  };
}

bool _stableBatchIdentity(Map<String, dynamic> before, Map<String, dynamic> after) {
  const keys = [
    'source_revision', 'source_base_revision', 'source_hashes', 'source_diff_sha256',
    'source_hash_manifest_sha256', 'runner_sha256', 'calibration_sha256', 'protocol_sha256',
    'fixture_manifest_sha256', 'source_corpus_identity_sha256',
    'tree_test_executable', 'tree_test_executable_sha256',
  ];
  return keys.every((key) => jsonEncode(before[key]) == jsonEncode(after[key]));
}

Map<String, dynamic> _parseTime(String text) {
  double? number(String pattern) {
    final match = RegExp(pattern, multiLine: true).firstMatch(text);
    return match == null ? null : double.tryParse(match.group(1)!);
  }
  final elapsed = RegExp(r'Elapsed \(wall clock\) time \(h:mm:ss or m:ss\): ([0-9:.]+)')
      .firstMatch(text)?.group(1);
  final rss = RegExp(r'Maximum resident set size \(kbytes\): (\d+)')
      .firstMatch(text)?.group(1);
  return {
    'user_seconds': number(r'User time \(seconds\): ([0-9.]+)'),
    'system_seconds': number(r'System time \(seconds\): ([0-9.]+)'),
    'elapsed_seconds': elapsed == null ? null : _parseClock(elapsed),
    'elapsed_raw': elapsed,
    'maximum_resident_set_kib': rss == null ? null : int.parse(rss),
  };
}

double _parseClock(String value) {
  final fields = value.split(':');
  if (fields.length == 2) return int.parse(fields[0]) * 60 + double.parse(fields[1]);
  if (fields.length == 3) return int.parse(fields[0]) * 3600 + int.parse(fields[1]) * 60 + double.parse(fields[2]);
  throw FormatException('Unrecognized elapsed time: $value');
}

List<Map<String, dynamic>> _records(String text, String marker) => [
      for (final line in text.split('\n').where((line) => line.contains(marker)))
        jsonDecode(line.substring(line.indexOf(marker) + marker.length)) as Map<String, dynamic>,
    ];

Map<String, int> _directoryInventory(Directory directory) {
  final result = <String, int>{};
  void visit(Directory current, String relative) {
    for (final entity in current.listSync(followLinks: false)) {
      final name = relative.isEmpty ? _basename(entity.path) : '$relative/${_basename(entity.path)}';
      final type = FileSystemEntity.typeSync(entity.path, followLinks: false);
      if (type == FileSystemEntityType.file) {
        result[name] = File(entity.path).lengthSync();
      } else if (type == FileSystemEntityType.directory) {
        visit(Directory(entity.path), name);
      }
    }
  }
  visit(directory, '');
  return result;
}

int _apparentBytes(Directory directory) {
  if (!directory.existsSync()) return 0;
  var total = 0;
  for (final entity in directory.listSync(followLinks: false)) {
    final type = FileSystemEntity.typeSync(entity.path, followLinks: false);
    if (type == FileSystemEntityType.file) {
      total += File(entity.path).lengthSync();
    } else if (type == FileSystemEntityType.directory) {
      total += _apparentBytes(Directory(entity.path));
    }
  }
  return total;
}

int _availableBytes() {
  final result = Process.runSync('df', ['-Pk', '/home/bsutton/.cache']);
  if (result.exitCode != 0) throw StateError('df failed: ${result.stderr}');
  final fields = '${result.stdout}'.trim().split('\n').last.trim().split(RegExp(r'\s+'));
  return int.parse(fields[3]) * 1024;
}

void _validateBudget({required int projectedNewBytes}) {
  final existing = _apparentBytes(Directory(oldRoot));
  final projectedTotal = existing + projectedNewBytes;
  final available = _availableBytes();
  if (projectedTotal > artifactCap) {
    throw StateError('Refusing workload: apparent bytes $existing + projected $projectedNewBytes exceeds 24 GiB cap');
  }
  if (available < projectedNewBytes) {
    throw StateError('Insufficient free space: $available bytes available for $projectedNewBytes projected bytes');
  }
}

String _sha(String path) {
  final result = Process.runSync('sha256sum', [path]);
  if (result.exitCode != 0) throw StateError('sha256sum failed for $path: ${result.stderr}');
  return '${result.stdout}'.split(RegExp(r'\s+')).first;
}

String _basename(String path) {
  final trimmed = path.replaceAll(RegExp(r'/+$'), '');
  return trimmed.substring(trimmed.lastIndexOf('/') + 1);
}

void _writeJson(String path, Object value) {
  final file = File(path);
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(const JsonEncoder.withIndent('  ').convert(value), flush: true);
}
