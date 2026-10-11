#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:dcli/dcli.dart';

const worktree = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const rustRoot = '$worktree/rust';
const evidenceRoot = '$worktree/docs/evidence/native-public-extraction-2026-10-11/raw';
const corpusRoot = '/home/bsutton/.cache/revault-protected-20261011';
const corpusEvidence = '/home/bsutton/.cache/revault-protected-20261011-evidence/corpus-ready.json';
const previousFixtureRoot = '/home/bsutton/.cache/revault-native-public-20261011/fixtures';
const fixtureRoot = '/home/bsutton/.cache/revault-native-public-20261011/fixtures-v2';
const binRoot = '$rustRoot/target/native-public-20261011/bin';
const artifactCap = 24 * 1024 * 1024 * 1024;
const unitBytes = 65536;
const workloads = <String>['source-zstd-64mib', 'source-zstd-1gib', 'random-raw-64mib', 'random-raw-1gib'];
const variantDirs = <String>['A', 'B', 'C', 'ZIP'];

class Capture {
  const Capture(this.code, this.stdout, this.stderr, {this.timedOut = false, this.incomplete = false});
  final int code;
  final String stdout;
  final String stderr;
  final bool timedOut;
  final bool incomplete;
}

late Map<String, dynamic> corpusRecord;
late List<Map<String, dynamic>> cases;

Future<void> main(List<String> args) async {
  if (args.isEmpty || args.first == 'help') {
    print('prepare | create | calibrate | permission-audit | component-batch | cli-batch | capture-smoke | freeze');
    return;
  }
  switch (args.first) {
    case 'prepare':
      await prepare();
    case 'create':
      await createAll();
    case 'calibrate':
      await calibrateAll();
    case 'permission-audit':
      await auditPermissions();
    case 'component-batch':
      await componentBatch();
    case 'cli-batch':
      await cliBatch();
    case 'capture-smoke':
      await captureSmoke();
    case 'freeze':
      freezeIdentities();
    default:
      throw ArgumentError('Unknown phase: ${args.first}');
  }
}

void _loadCases() {
  cases = (jsonDecode(File('$evidenceRoot/cases.json').readAsStringSync()) as List)
      .cast<Map<String, dynamic>>();
  if (cases.map((row) => row['name']).toList().join('|') != workloads.join('|')) {
    throw StateError('Fixture matrix does not match frozen four-workload order');
  }
}

Future<void> prepare() async {
  final root = Directory(fixtureRoot);
  if (root.existsSync()) throw StateError('Refusing existing fixture root: $fixtureRoot');
  final previousCases = (jsonDecode(File('$evidenceRoot/cases.json').readAsStringSync()) as List)
      .cast<Map<String, dynamic>>();
  if (previousCases.map((row) => row['name']).toList().join('|') != workloads.join('|')) {
    throw StateError('Preserved source prefixes do not match the frozen workload matrix');
  }
  final corpus = jsonDecode(File(corpusEvidence).readAsStringSync()) as Map<String, dynamic>;
  final bundle = '$corpusRoot/source-bundle.bin';
  final expanded = '$corpusRoot/source-expanded-1gib.bin';
  final random = '$corpusRoot/random-1gib-raw-encrypted/source/file-000000.bin';
  final randomInventory = (jsonDecode(File('$corpusRoot/random-1gib-raw-encrypted/inventory.json').readAsStringSync()) as List).single as Map;
  if (_sha(bundle) != corpus['bundle_sha256'] ||
      _sha(expanded) != corpus['expanded_sha256'] ||
      File(expanded).lengthSync() != 1024 * 1024 * 1024 ||
      File(random).lengthSync() != 1024 * 1024 * 1024 ||
      _sha(random) != randomInventory['sha256']) {
    throw StateError('Pinned corpus full hashes/lengths do not match retained provenance');
  }
  final totalLogical = 64 * 1024 * 1024 + 1024 * 1024 * 1024;
  final preservedPartialBytes = _apparentBytes(Directory(previousFixtureRoot));
  final archiveWorstCase = totalLogical * 2 * variantDirs.length;
  final candidateStaging = totalLogical * 2;
  final maximumOutput = 1024 * 1024 * 1024;
  final binaryBytes = Directory(binRoot).listSync().whereType<File>()
      .fold<int>(0, (sum, file) => sum + file.lengthSync());
  final projected = preservedPartialBytes + archiveWorstCase + candidateStaging + maximumOutput + binaryBytes;
  final available = _availableBytes();
  if (projected > artifactCap || available < projected) {
    throw StateError('24 GiB fixture/output budget failed: projected=$projected available=$available');
  }
  root.createSync(recursive: true);
  final specs = [
    _spec('source-zstd-64mib', 64 * 1024 * 1024, expanded, 'source-expanded-1gib-prefix', true),
    _spec('source-zstd-1gib', 1024 * 1024 * 1024, expanded, 'source-expanded-1gib', true),
    _spec('random-raw-64mib', 64 * 1024 * 1024, random, 'random-1gib-prefix', false),
    _spec('random-raw-1gib', 1024 * 1024 * 1024, random, 'random-1gib', false),
  ];
  cases = [];
  for (final spec in specs) {
    final caseDir = Directory('$fixtureRoot/${spec['name']}')..createSync();
    final oldSpec = previousCases.singleWhere((row) => row['name'] == spec['name']);
    final rows = <Map<String, dynamic>>[];
    for (final variant in variantDirs) {
      final variantRoot = Directory('${caseDir.path}/$variant')..createSync();
      final oldVariant = (oldSpec['variants'] as List).cast<Map<String, dynamic>>()
          .singleWhere((row) => row['variant'] == variant);
      final oldSource = File(oldVariant['source_path'] as String);
      if (oldSource.lengthSync() != spec['bytes'] || _sha(oldSource.path) != oldVariant['source_sha256']) {
        throw StateError('Preserved source attempt does not match its recorded hash: ${oldSource.path}');
      }
      Link('${variantRoot.path}/source').createSync(oldSource.parent.path);
      final targetPath = '${variantRoot.path}/source/file-000000.bin';
      final sourceHash = _sha(targetPath);
      final descriptor = <String, Object>{
        'files': 1,
        'bytes': spec['bytes'] as int,
        'corpus': spec['corpus'] as String,
        'compressed': spec['compressed'] as bool,
        'mode': 'plain',
        'unpadded': false,
        'unit': unitBytes,
        'source_corpus_sha256': spec['corpus_sha256'] as String,
      };
      _writeJson('${variantRoot.path}/case.json', descriptor);
      _writeJson('${variantRoot.path}/inventory.json', [
        {'path': 'file-000000.bin', 'bytes': spec['bytes'], 'sha256': sourceHash},
      ]);
      rows.add({
        'variant': variant,
        'root': variantRoot.path,
        'source_path': targetPath,
        'source_sha256': sourceHash,
        'source_bytes': File(targetPath).lengthSync(),
        'case_json_sha256': _sha('${variantRoot.path}/case.json'),
      });
    }
    final row = <String, dynamic>{
      ...spec,
      'files': 1,
      'mode': 'plain',
      'unpadded': false,
      'unit': unitBytes,
      'variants': rows,
    };
    cases.add(row);
    _writeJson('${caseDir.path}/case-manifest.json', row);
  }
  _writeJson('$evidenceRoot/cases.json', cases);
  corpusRecord = {
    'source_bundle_path': bundle,
    'source_bundle_sha256': _sha(bundle),
    'source_bundle_bytes': File(bundle).lengthSync(),
    'expanded_source_path': expanded,
    'expanded_source_sha256': _sha(expanded),
    'expanded_source_bytes': File(expanded).lengthSync(),
    'random_source_path': random,
    'random_source_sha256': _sha(random),
    'random_source_bytes': File(random).lengthSync(),
    'random_original_inventory_sha256': randomInventory['sha256'],
    'case_prefixes': cases.map((row) => {
      'name': row['name'],
      'input_path': row['input'],
      'input_sha256': row['input_sha256'],
      'source_bytes': row['bytes'],
      'variant_source_hashes': (row['variants'] as List).map((v) => (v as Map)['source_sha256']).toSet().toList(),
    }).toList(),
  };
  _writeJson('$evidenceRoot/source-corpus-identity.json', corpusRecord);
  _writeJson('$evidenceRoot/fixture-budget.json', {
    'artifact_cap_bytes': artifactCap,
    'preserved_partial_fixture_bytes_plus_projected_archives_candidate_staging_max_output_and_binaries': projected,
    'preserved_partial_fixture_bytes': preservedPartialBytes,
    'available_bytes_before': available,
    'assertion_passed': projected <= artifactCap && available >= projected,
  });
  print(green('Prepared and hashed four source/random prefixes for A/B/C/ZIP; no archives or timings created.'));
}

Map<String, dynamic> _spec(String name, int bytes, String input, String corpus, bool compressed) => {
  'name': name,
  'bytes': bytes,
  'input': input,
  'input_sha256': _sha(input),
  'corpus_sha256': _sha(input),
  'corpus': corpus,
  'compressed': compressed,
};

void _copyPrefix(File input, File target, int bytes) {
  if (target.existsSync()) throw StateError('Refusing existing source: ${target.path}');
  if (input.lengthSync() < bytes) throw StateError('Pinned source too short: ${input.path}');
  final source = input.openSync();
  final output = target.openSync(mode: FileMode.write);
  final buffer = Uint8List(unitBytes);
  var copied = 0;
  try {
    while (copied < bytes) {
      final count = source.readIntoSync(buffer, 0, (bytes - copied).clamp(0, buffer.length));
      if (count == 0) throw StateError('Unexpected end of pinned input: ${input.path}');
      output.writeFromSync(buffer, 0, count);
      copied += count;
    }
    output.flushSync();
  } finally {
    source.closeSync();
    output.closeSync();
  }
  if (target.lengthSync() != bytes) throw StateError('Wrong copied prefix length: ${target.path}');
}

Future<void> createAll() async {
  _loadCases();
  _validateBinary('lockbox-A-default');
  _validateBinary('lockbox-B-native');
  _validateBinary('evaluator-A-default');
  _validateBinary('evaluator-B-native');
  _validateBinary('revault_lockbox_api-C-external-source-test');
  final creationRoot = Directory('$evidenceRoot/fixture-creation-v2');
  if (creationRoot.existsSync()) throw StateError('Refusing existing fixture creation evidence: ${creationRoot.path}');
  creationRoot.createSync(recursive: true);
  for (final spec in cases) {
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    for (final variant in variantDirs) _validateSource(spec, roots[variant]!);
    final compressed = spec['compressed'] as bool;
    final cliCompression = compressed ? ['--compression', 'zstd', '--compression-level', '3'] : ['--compression', 'none'];
    for (final variant in ['A', 'B']) {
      final root = roots[variant]!;
      final cli = '$binRoot/lockbox-${variant == 'A' ? 'A-default' : 'B-native'}';
      final home = '$root/private-home';
      final environment = _privateEnv(root, home);
      final createCommand = [cli, '$root/archive.lbox', 'create', '--encryption', 'none', '--signing', 'none', ...cliCompression];
      await _recordCommand(creationRoot.path, spec['name'] as String, '$variant-create',
          ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', ...createCommand], environment, 1900);
      final addCommand = [cli, '$root/archive.lbox', 'add', '${root}/source/file-000000.bin', '--to', '/file-000000.bin'];
      await _recordCommand(creationRoot.path, spec['name'] as String, '$variant-add',
          ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', ...addCommand], environment, 1900);
      _assertNonEmpty('$root/archive.lbox');
      await _verifyPublicExtraction(creationRoot.path, spec, variant, cli, root, environment);
    }
    final candidateRoot = roots['C']!;
    final candidate = '$binRoot/revault_lockbox_api-C-external-source-test';
    final candidateEnv = <String, String>{
      ..._privateEnv(candidateRoot, '${candidateRoot}/private-home'),
      'REVAULT_CANDIDATE_ROOT': candidateRoot,
      'REVAULT_CANDIDATE_PHASE': 'tree-create',
      'REVAULT_CANDIDATE_UNIT': '$unitBytes',
    };
    final candidateCommand = [
      'taskset', '-c', '0-4', '/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', candidate,
      '--exact', 'file_format::candidate_files::resource_probe::candidate_file_resource_probe',
      '--ignored', '--nocapture',
    ];
    final candidateCapture = await _recordCommand(creationRoot.path, spec['name'] as String, 'C-create', candidateCommand, candidateEnv, 1900,
        acceptStdoutMarker: 'CANDIDATE_SAMPLE ');
    final cRows = _records(candidateCapture.stdout, 'CANDIDATE_SAMPLE ');
    if (cRows.length != 1 || cRows.single['verified'] != true) throw StateError('C creation verification missing for ${spec['name']}');
    final census = cRows.single['codec_census'] as Map?;
    final chunks = (spec['bytes'] as int) ~/ unitBytes;
    if (census == null || census['logical_bytes'] != spec['bytes'] ||
        census['raw_chunks'] is! int || census['zstd_chunks'] is! int ||
        (census['raw_chunks'] as int) + (census['zstd_chunks'] as int) != chunks ||
        (!compressed && census['zstd_chunks'] != 0) ||
        (compressed && census['zstd_chunks'] == 0)) {
      throw StateError('Candidate authenticated codec census mismatch for ${spec['name']}: $census');
    }
    _assertNonEmpty('$candidateRoot/tree.lbox');
    _assertNonEmpty('$candidateRoot/tree.public');
    final zipRoot = roots['ZIP']!;
    final zipCommand = ['$binRoot/evaluator-A-default', 'create', zipRoot, 'zip'];
    await _recordCommand(creationRoot.path, spec['name'] as String, 'ZIP-create',
        ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', ...zipCommand], _privateEnv(zipRoot, '${zipRoot}/private-home'), 1900);
    _assertNonEmpty('$zipRoot/archive.zip');
    final record = {
      'case': spec['name'],
      'actual_candidate_codec_census': census,
      'candidate_tree_sha256': _sha('$candidateRoot/tree.lbox'),
      'candidate_public_sha256': _sha('$candidateRoot/tree.public'),
      'A_archive_sha256': _sha('${roots['A']}/archive.lbox'),
      'B_archive_sha256': _sha('${roots['B']}/archive.lbox'),
      'ZIP_archive_sha256': _sha('$zipRoot/archive.zip'),
      'archive_sizes': {
        'A': File('${roots['A']}/archive.lbox').lengthSync(),
        'B': File('${roots['B']}/archive.lbox').lengthSync(),
        'C': File('$candidateRoot/tree.lbox').lengthSync(),
        'ZIP': File('$zipRoot/archive.zip').lengthSync(),
      },
      'source_hashes_identical': (spec['variants'] as List).map((row) => (row as Map)['source_sha256']).toSet().length == 1,
      'verified': true,
    };
    _writeJson('${creationRoot.path}/${spec['name']}.json', record);
    print(green('Created A/B/C/ZIP fixtures and verified source/codec census for ${spec['name']}'));
  }
}

Future<Capture> _recordCommand(String output, String caseName, String label, List<String> command,
    Map<String, String> environment, int timeoutSeconds, {String? acceptStdoutMarker}) async {
  final result = await _capture(command.first, command.skip(1).toList(), environment, timeoutSeconds);
  final path = '$output/$caseName/$label';
  Directory('$output/$caseName').createSync(recursive: true);
  File('$path.stdout').writeAsStringSync(result.stdout, flush: true);
  File('$path.stderr').writeAsStringSync(result.stderr, flush: true);
  _writeJson('$path.json', {
    'command': command,
    'environment': environment,
    'exit_code': result.code,
    'outer_runner_timeout': result.timedOut,
    'stdout_sha256': _shaText(result.stdout),
    'stderr_sha256': _shaText(result.stderr),
    'stdout_marker_required': acceptStdoutMarker,
    'stdout_marker_present': acceptStdoutMarker == null || result.stdout.contains(acceptStdoutMarker),
  });
  if (result.timedOut || result.incomplete || result.code != 0 || (acceptStdoutMarker != null && !result.stdout.contains(acceptStdoutMarker))) {
    throw StateError('Fixture command failed ($label, exit=${result.code}); logs retained at $path');
  }
  return result;
}

Future<void> _verifyPublicExtraction(String output, Map<String, dynamic> spec,
    String variant, String cli, String root, Map<String, String> environment) async {
  final destination = '$root/verified-extract';
  if (Directory(destination).existsSync() || FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.notFound) {
    throw StateError('Refusing existing CLI verification destination: $destination');
  }
  final command = [cli, '$root/archive.lbox', 'extract', '--to', destination];
  await _recordCommand(output, spec['name'] as String, '$variant-public-extract',
      ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', ...command], environment, 1900);
  final expected = File('$root/source/file-000000.bin');
  final actual = File('$destination/file-000000.bin');
  final names = Directory(destination).listSync(followLinks: false).map((entity) => _basename(entity.path)).toList()..sort();
  final actualType = FileSystemEntity.typeSync(actual.path, followLinks: false);
  final actualMode = actualType == FileSystemEntityType.file ? actual.statSync().mode & 0x1ff : null;
  final expectedMode = expected.statSync().mode & 0x1ff;
  final sameBytes = actualType == FileSystemEntityType.file && actual.lengthSync() == expected.lengthSync() && _filesEqual(expected, actual);
  final verified = names.length == 1 && names.single == 'file-000000.bin' && sameBytes &&
      _sha(expected.path) == _sha(actual.path) && File('$root/archive.lbox').existsSync();
  _writeJson('$output/${spec['name']}/$variant-public-extract-verification.json', {
    'command': command,
    'environment': environment,
    'destination': destination,
    'extraction_mode': 'public CLI archive extract --to; ExtractMany throughout',
    'expected_bytes': expected.lengthSync(),
    'actual_bytes': actual.existsSync() ? actual.lengthSync() : null,
    'source_permissions_octal': expectedMode.toRadixString(8),
    'output_permissions_octal': actualMode?.toRadixString(8),
    'output_nofollow_type': actualType == FileSystemEntityType.file ? 'regular_file' : actualType.toString(),
    'expected_sha256': _sha(expected.path),
    'actual_sha256': actual.existsSync() ? _sha(actual.path) : null,
    'directory_entries': names,
    'bytewise_compare_passed': sameBytes,
    'verified': verified,
  });
  if (!verified) throw StateError('Public CLI persisted extraction verification failed for ${spec['name']}/$variant; output retained.');
  Directory(destination).deleteSync(recursive: true);
}

Future<void> auditPermissions() async {
  _loadCases();
  final output = Directory('$evidenceRoot/permission-audit-v3');
  if (output.existsSync()) throw StateError('Refusing existing permission audit output');
  for (final spec in cases) {
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    for (final variant in ['A', 'B']) {
      final root = roots[variant]!;
      final cli = '$binRoot/lockbox-${variant == 'A' ? 'A-default' : 'B-native'}';
      final env = _privateEnv(root, '$root/private-home');
      final destination = '$root/permission-audit-v3-output';
      if (FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.notFound) {
        throw StateError('Refusing existing permission audit destination: $destination');
      }
      final command = [cli, '$root/archive.lbox', 'extract', '--to', destination, '--restore-permissions'];
      await _recordCommand(output.path, spec['name'] as String, '$variant-restore-permissions',
          ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '1800', ...command], env, 1900);
      final expected = File('$root/source/file-000000.bin');
      final actual = File('$destination/file-000000.bin');
      final actualType = FileSystemEntity.typeSync(actual.path, followLinks: false);
      final expectedType = FileSystemEntity.typeSync(expected.path, followLinks: false);
      final hostSourceMode = expected.statSync().mode & 0x1ff;
      // Public file import records Lockbox's documented default mode rather
      // than the incidental host mode of the source file.
      const expectedMode = 0x180; // 0600
      final actualMode = actualType == FileSystemEntityType.file ? actual.statSync().mode & 0x1ff : -1;
      final bytesMatch = actualType == FileSystemEntityType.file &&
          actual.lengthSync() == expected.lengthSync() && _filesEqual(expected, actual);
      final verified = expectedType == FileSystemEntityType.file && actualType == FileSystemEntityType.file &&
          expectedMode == actualMode && bytesMatch;
      _writeJson('${output.path}/${spec['name']}/$variant-permissions.json', {
        'command': command,
        'expected_regular_file_nofollow': expectedType == FileSystemEntityType.file,
        'actual_regular_file_nofollow': actualType == FileSystemEntityType.file,
        'host_source_permissions_octal': hostSourceMode.toRadixString(8),
        'stored_default_permissions_octal': expectedMode.toRadixString(8),
        'restored_permissions_octal': actualMode < 0 ? null : actualMode.toRadixString(8),
        'permission_bits_match': expectedMode == actualMode,
        'bytes_match': bytesMatch,
        'verified': verified,
      });
      if (!verified) throw StateError('Stored/restored permissions failed for ${spec['name']}/$variant; output retained.');
      Directory(destination).deleteSync(recursive: true);
    }
  }
  print(green('Public CLI --restore-permissions checks verified regular files, bytes and permission bits for A/B.'));
}

bool _filesEqual(File expected, File actual) {
  final left = expected.openSync();
  final right = actual.openSync();
  final a = Uint8List(unitBytes), b = Uint8List(unitBytes);
  try {
    while (true) {
      final an = left.readIntoSync(a);
      final bn = right.readIntoSync(b);
      if (an != bn) return false;
      if (an == 0) return true;
      for (var index = 0; index < an; index++) {
        if (a[index] != b[index]) return false;
      }
    }
  } finally {
    left.closeSync();
    right.closeSync();
  }
}

Map<String, String> _privateEnv(String root, String home) {
  final paths = <String, String>{
    'HOME': home,
    'XDG_CONFIG_HOME': '$root/private-config',
    'XDG_CACHE_HOME': '$root/private-cache',
    'XDG_STATE_HOME': '$root/private-state',
    'XDG_DATA_HOME': '$root/private-data',
    'LOCKBOX_SESSION_AGENT_DIR': '$root/private-agent',
    'LOCKBOX_VAULT_DIR': '$root/private-vault',
    'LOCKBOX_PLATFORM_SECRET_STORE': 'disabled',
    'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011',
    'PATH': '/usr/bin:/bin',
  };
  for (final directory in paths.entries.where((entry) => !['PATH', 'LOCKBOX_PLATFORM_SECRET_STORE', 'TMPDIR'].contains(entry.key))) {
    Directory(directory.value).createSync(recursive: true);
  }
  Directory('$home/.revault').createSync(recursive: true);
  Directory(paths['LOCKBOX_VAULT_DIR']!).createSync(recursive: true);
  return paths;
}

void _validateSource(Map<String, dynamic> spec, String root) {
  final expected = (spec['variants'] as List).cast<Map<String, dynamic>>()
      .singleWhere((row) => row['root'] == root)['source_sha256'];
  final source = File('$root/source/file-000000.bin');
  if (source.lengthSync() != spec['bytes'] || _sha(source.path) != expected) {
    throw StateError('External source changed or mismatched at ${source.path}');
  }
}

Future<void> calibrateAll() async {
  _loadCases();
  final destination = File('$evidenceRoot/physical-read-calibration.json');
  if (destination.existsSync()) throw StateError('Refusing existing calibration evidence');
  final result = <String, dynamic>{};
  final buffer = Uint8List(unitBytes);
  for (final spec in cases) {
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    final archives = <String, String>{
      'A': '${roots['A']}/archive.lbox',
      'B': '${roots['B']}/archive.lbox',
      'C': '${roots['C']}/tree.lbox',
      'ZIP': '${roots['ZIP']}/archive.zip',
    };
    for (final entry in archives.entries) {
      final path = entry.value;
      _assertNonEmpty(path);
      final scans = <Map<String, dynamic>>[];
      result['${spec['name']}/${entry.key}'] = {'path': path, 'scans': scans, 'complete': false};
      _writeJson(destination.path, result);
      for (var scan = 0; scan < 2; scan++) {
        final sync = Process.runSync('/usr/bin/sync', ['-d', '--', path]);
        final advise = Process.runSync('/usr/bin/fadvise', ['--advice', 'dontneed', path]);
        final fincore = Process.runSync('/usr/bin/fincore', ['--json', '--bytes', '--output', 'SIZE,RES,DIRTY,WRITEBACK,FILE', path]);
        dynamic parsed;
        try { parsed = jsonDecode(fincore.stdout as String); } catch (_) {}
        final rows = parsed is Map ? parsed['fincore'] : parsed;
        final item = rows is List && rows.length == 1 && rows.single is Map ? rows.single as Map : null;
        final residentRaw = item?['res'];
        final resident = residentRaw is num ? residentRaw.toInt() : int.tryParse('$residentRaw');
        if (sync.exitCode != 0 || advise.exitCode != 0 || fincore.exitCode != 0 || resident != 0) {
          throw StateError('Cold calibration eviction failed for $path scan $scan; partial evidence retained');
        }
        final before = _selfReadBytes();
        final input = File(path).openSync();
        var logical = 0;
        try { while (true) { final count = input.readIntoSync(buffer); if (count == 0) break; logical += count; } }
        finally { input.closeSync(); }
        final physical = _selfReadBytes() - before;
        scans.add({
          'scan': scan,
          'sync_exit_code': sync.exitCode,
          'fadvise_exit_code': advise.exitCode,
          'fincore_exit_code': fincore.exitCode,
          'resident_bytes_before': resident,
          'physical_read_bytes': physical,
          'logical_bytes': logical,
          'archive_bytes': File(path).lengthSync(),
          'buffer_bytes': buffer.length,
        });
        _writeJson(destination.path, result);
        if (physical <= 0 || logical != File(path).lengthSync()) throw StateError('Cold calibration scan incomplete for $path');
      }
      final first = scans[0]['physical_read_bytes'] as int;
      final second = scans[1]['physical_read_bytes'] as int;
      final minRead = first < second ? first : second;
      final maxRead = first > second ? first : second;
      if (maxRead > minRead * 1.10) throw StateError('Independent cold reads disagree >10% for $path');
      result['${spec['name']}/${entry.key}'] = {
        'path': path,
        'archive_sha256': _sha(path),
        'scans': scans,
        'minimum_physical_read_bytes': minRead,
        'maximum_physical_read_bytes': maxRead,
        'required_sample_read_bytes': (minRead + 1) ~/ 2,
        'agreement_fraction': 0.10,
        'complete': true,
      };
      _writeJson(destination.path, result);
    }
  }
  print(green('Calibrated cold full-file reads for 16 current A/B/C/ZIP archives.'));
}

const componentVariants = <String>['A', 'B', 'C1', 'Cauto', 'ZIP'];

Future<void> componentBatch() async {
  _loadCases();
  _verifyFrozenIdentities();
  final calibration = jsonDecode(File('$evidenceRoot/physical-read-calibration.json').readAsStringSync()) as Map;
  final output = Directory('$evidenceRoot/component-batch-v1');
  if (output.existsSync()) throw StateError('Refusing existing component batch: ${output.path}');
  output.createSync(recursive: true);
  final started = DateTime.now().toUtc();
  final wall = Stopwatch()..start();
  var accepted = 0;
  for (var caseIndex = 0; caseIndex < cases.length; caseIndex++) {
    final spec = cases[caseIndex];
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    final archives = <String, String>{
      'A': '${roots['A']}/archive.lbox', 'B': '${roots['B']}/archive.lbox',
      'C1': '${roots['C']}/tree.lbox', 'Cauto': '${roots['C']}/tree.lbox', 'ZIP': '${roots['ZIP']}/archive.zip',
    };
    for (var round = 0; round < 24; round++) {
      if (wall.elapsed > const Duration(minutes: 90)) throw StateError('Component batch reached 90-minute cap; partial rows retained.');
      final warmup = round < 4;
      for (var position = 0; position < componentVariants.length; position++) {
        if (wall.elapsed > const Duration(minutes: 90)) throw StateError('Component batch reached 90-minute cap; partial rows retained.');
        final variant = componentVariants[(position + round + caseIndex) % componentVariants.length];
        final root = variant.startsWith('C') ? roots['C']! : roots[variant]!;
        final archive = archives[variant]!;
        final archiveBytes = File(archive).lengthSync();
        final gate = await _boundedColdGate(archive, wall, const Duration(minutes: 90));
        if (gate['passed'] != true) throw StateError('Cold archive admission failed: ${spec['name']}/$variant/$round/$position: $gate');
        final sampleId = '${spec['name']}-${variant}-r${round.toString().padLeft(2, '0')}-p$position';
        Directory('$root/component-output').createSync(recursive: true);
        _checkArtifactBudget(spec['bytes'] as int);
        final destination = '${root}/component-output/$sampleId';
        if (FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.notFound) {
          throw StateError('Refusing existing component output $destination');
        }
        final env = _privateEnv(root, '$root/private-home');
        env['REVAULT_EXTRACT_TID_PARITY'] = '${round % 2}';
        env['REVAULT_EXTRACT_PID_NAMESPACE'] = 'pid:[4026531836]';
        env['REVAULT_EXTRACT_BLOCK_DEVICES'] = 'nvme0n1,nvme1n1';
        env['REVAULT_EXTRACT_DURABLE'] = '0';
        final bin = <String>[];
        if (variant == 'A' || variant == 'B') {
          final evaluator = '$binRoot/evaluator-${variant == 'A' ? 'A-default' : 'B-native'}';
          bin.addAll([evaluator, 'extract-lockbox', root, destination]);
        } else if (variant == 'ZIP') {
          bin.addAll(['$binRoot/evaluator-A-default', 'extract-zip', root, destination]);
        } else {
          env['REVAULT_CANDIDATE_ROOT'] = root;
          env['REVAULT_CANDIDATE_PHASE'] = 'tree-extract';
          env['REVAULT_CANDIDATE_UNIT'] = '$unitBytes';
          env['REVAULT_READ_WORKERS'] = variant == 'C1' ? '1' : '0';
          env['REVAULT_EXTRACT_OUTPUT'] = destination;
          bin.addAll([
            '$binRoot/revault_lockbox_api-C-external-source-test',
            '--exact', 'file_format::candidate_files::resource_probe::candidate_file_resource_probe',
            '--ignored', '--nocapture', '--test-threads=1',
          ]);
        }
        final command = ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '180', 'taskset', '-c', '0-4', ...bin];
        Capture? capture;
        Map<String, dynamic>? record;
        final attemptEvidence = <Map<String, dynamic>>[];
        for (var attempt = 0; attempt < 16; attempt++) {
          final currentGate = attempt == 0 ? gate : await _boundedColdGate(archive, wall, const Duration(minutes: 90));
          final attemptPath = '${output.path}/$sampleId-attempt-$attempt';
          if (currentGate['passed'] != true) {
            attemptEvidence.add({'attempt': attempt, 'gate': currentGate, 'accepted': false, 'reason': 'archive_not_cold'});
            _appendJsonl('${output.path}/admission-rejections.jsonl', {
              'case': spec['name'], 'backend': variant, 'round': round, 'warmup': warmup,
              'attempt': attempt, 'gate': currentGate, 'reason': 'archive_not_cold'});
            continue;
          }
          final seconds = _remainingCommandSeconds(wall, const Duration(minutes: 90));
          command[4] = '$seconds';
          capture = await _capture(command.first, command.skip(1).toList(), env, seconds + 20);
          File('$attemptPath.stdout').writeAsStringSync(capture.stdout, flush: true);
          File('$attemptPath.stderr').writeAsStringSync(capture.stderr, flush: true);
          if (capture.timedOut || capture.incomplete || capture.code != 0) {
            _writeJson('$attemptPath.json', {'case': spec['name'], 'backend': variant, 'round': round, 'position': position,
              'warmup': warmup, 'sample': sampleId, 'command': command, 'environment': env, 'gate': currentGate,
              'exit_code': capture.code, 'outer_runner_timeout': capture.timedOut, 'capture_incomplete': capture.incomplete, 'accepted': false});
            throw StateError('Component sample failed; raw output retained at $attemptPath');
          }
          record = variant.startsWith('C')
              ? (() { final rows = _records(capture!.stdout, 'CANDIDATE_SAMPLE '); return rows.length == 1 ? rows.single : null; })()
              : _firstJsonLine(capture.stdout);
          if (record == null) throw StateError('No unique component sample JSON: $sampleId');
          if (record['kind'] == 'extraction_admission_skip') {
            if (record['reason'] != 'reader_tid_parity_mismatch' || record['archive_opened'] != false ||
                record['output_created'] != false || record['requested_reader_tid_parity'] != round % 2 ||
                FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.notFound) {
              throw StateError('Invalid parity admission skip; raw logs retained');
            }
            if (attempt.isOdd) {
              final nudge = await _capture('/usr/bin/true', [], env, 5);
              if (nudge.code != 0 || nudge.incomplete || nudge.timedOut) throw StateError('Parity retry nudge failed');
            }
            final rejection = {'attempt': attempt, 'gate': currentGate, 'record': record, 'accepted': false};
            attemptEvidence.add(rejection);
            _appendJsonl('${output.path}/admission-rejections.jsonl', {
              'case': spec['name'], 'backend': variant, 'round': round, 'warmup': warmup, ...rejection});
            capture = null;
            record = null;
            continue;
          }
          attemptEvidence.add({'attempt': attempt, 'gate': currentGate, 'accepted': true});
          break;
        }
        if (capture == null || record == null) throw StateError('Component admission retries exhausted: $sampleId');
        final expectedHash = (spec['variants'] as List).cast<Map<String, dynamic>>().first['source_sha256'] as String;
        final file = File('$destination/file-000000.bin');
        final type = FileSystemEntity.typeSync(file.path, followLinks: false);
        final outputHash = type == FileSystemEntityType.file ? _sha(file.path) : '';
        final sourceBytes = File('${root}/source/file-000000.bin').lengthSync();
        final extractionVerified = record['verified'] == true && _exactOutputTree(destination) && type == FileSystemEntityType.file && file.lengthSync() == sourceBytes && outputHash == expectedHash;
        final reads = _readBytesFromRecord(record);
        // A/C variants share the same archive census calibration record.
        final calibrationKey = variant.startsWith('C') ? 'C' : variant;
        final threshold = calibration['${spec['name']}/$calibrationKey']?['required_sample_read_bytes'];
        final physicalPassed = reads != null && threshold is int && reads >= threshold;
        final workersOk = variant == 'Cauto'
            ? ((spec['compressed'] == true && record['worker_threads'] == 4) ||
                (spec['compressed'] == false && record['worker_threads'] == 0))
            : (variant == 'C1' ? record['worker_threads'] == 0 : true);
        if (variant.startsWith('C')) _validateCandidateCensus(spec, outputRoot: roots['C']!);
        final metricsValid = _validEndpoint(record, spec, round % 2, variant) &&
            _positive(_timeElapsed(capture.stderr)) && _positive(_timeMaximumRss(capture.stderr));
        final drain = extractionVerified && physicalPassed && workersOk && metricsValid
            ? await _drainOutput(destination, wall, const Duration(minutes: 90))
            : <String, dynamic>{'passed': false, 'not_attempted': true};
        final sampleRecord = <String, dynamic>{
          'kind': 'sample', 'case': spec['name'], 'backend': variant,
          'sample_backend': record['backend'], 'round': round, 'position': position,
          'parity': round % 2, 'warmup': warmup, 'sample_id': sampleId, 'sample': record,
          'command': command, 'environment': env, 'gate': gate, 'admission_attempts': attemptEvidence,
          'archive_path': archive, 'archive_bytes': archiveBytes,
          'archive_sha256': _sha(archive), 'read_bytes': reads,
          'required_read_bytes': threshold, 'physical_read_gate_passed': physicalPassed,
          'elapsed_seconds': record['total_seconds'], 'cpu': record['resources']?['cpu'],
          'peak_rss_kib': record['peak_rss_kib'] ?? record['resources']?['peak_rss_kib'],
          'whole_process_elapsed_seconds': _timeElapsed(capture.stderr),
          'whole_process_peak_rss_kib': _timeMaximumRss(capture.stderr),
          'worker_threads': record['worker_threads'], 'max_queued_chunks': record['max_queued_chunks'],
          'output_path': destination, 'output_sha256': outputHash,
          'output_bytes': type == FileSystemEntityType.file ? file.lengthSync() : null,
          'extraction_verified': extractionVerified, 'postprocess_drain': drain,
          'metrics_valid': metricsValid, 'output_drained_and_verified': extractionVerified && drain['passed'] == true,
          'worker_metadata_valid': workersOk,
          'exit_code': capture.code, 'outer_runner_timeout': capture.timedOut, 'capture_incomplete': capture.incomplete,
          'accepted': extractionVerified && physicalPassed && workersOk && metricsValid && drain['passed'] == true,
        };
        _appendJsonl('${output.path}/samples.jsonl', sampleRecord);
        if (!sampleRecord['accepted']) throw StateError('Component sample admission failed; row/log/output retained: $sampleId');
        accepted++;
        Directory(destination).deleteSync(recursive: true);
        _writeJson('${output.path}/progress.json', {'started_utc': started.toIso8601String(),
          'accepted_sample_count_including_prep': accepted, 'next_case': spec['name'],
          'next_round': round, 'next_variant_position': position + 1, 'wall_seconds': wall.elapsed.inSeconds});
      }
    }
  }
  _verifyFrozenIdentities();
  if (accepted != 480) throw StateError('Incomplete component matrix');
  _writeJson('${output.path}/complete.json', {'accepted': accepted, 'measured': 400, 'prep': 80, 'identities_verified_after': true, 'wall_seconds': wall.elapsed.inMilliseconds / 1000});
  print(green('Completed ${accepted} component samples (includes 80 warmups and 400 measured).'));
}

Future<void> cliBatch() async {
  _loadCases();
  _verifyFrozenIdentities();
  final output = Directory('$evidenceRoot/public-cli-batch-v1');
  if (output.existsSync()) throw StateError('Refusing existing public CLI batch');
  output.createSync(recursive: true);
  final wall = Stopwatch()..start();
  var count = 0;
  for (final spec in cases) {
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    for (var round = 0; round < 8; round++) {
      if (wall.elapsed > const Duration(minutes: 30)) throw StateError('Public CLI batch reached 30-minute cap; partial rows retained.');
      final order = round.isEven ? ['A', 'B'] : ['B', 'A'];
      for (final variant in order) {
        if (wall.elapsed > const Duration(minutes: 30)) throw StateError('Public CLI batch reached 30-minute cap; partial rows retained.');
        final warmup = round < 2;
        final root = roots[variant]!;
        final archive = '$root/archive.lbox';
        final gate = await _boundedColdGate(archive, wall, const Duration(minutes: 30));
        if (gate['passed'] != true) throw StateError('Public CLI cold admission failed: ${spec['name']}/$variant/$round');
        final sampleId = '${spec['name']}-${variant}-r${round.toString().padLeft(2, '0')}';
        _checkArtifactBudget(spec['bytes'] as int);
        final destination = '$root/cli-output-$sampleId';
        if (FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.notFound) throw StateError('Refusing output $destination');
        final env = _privateEnv(root, '$root/private-home');
        final cli = '$binRoot/lockbox-${variant == 'A' ? 'A-default' : 'B-native'}';
        final command = ['/usr/bin/time', '-v', 'timeout', '--kill-after=15s', '180',
          'taskset', '-c', '0-4', cli, archive, 'extract', '--to', destination];
        final seconds = _remainingCommandSeconds(wall, const Duration(minutes: 30));
        command[4] = '$seconds';
        final capture = await _capture(command.first, command.skip(1).toList(), env, seconds + 20);
        final samplePath = '${output.path}/$sampleId';
        File('$samplePath.stdout').writeAsStringSync(capture.stdout, flush: true);
        File('$samplePath.stderr').writeAsStringSync(capture.stderr, flush: true);
        if (capture.timedOut || capture.incomplete || capture.code != 0) {
          _writeJson('$samplePath.json', {'case': spec['name'], 'backend': variant, 'round': round, 'warmup': warmup,
            'command': command, 'environment': env, 'gate': gate, 'exit_code': capture.code, 'timed_out': capture.timedOut,
            'capture_incomplete': capture.incomplete});
          throw StateError('Public CLI extraction failed; output retained: $samplePath');
        }
        final expected = File('$root/source/file-000000.bin');
        final actual = File('$destination/file-000000.bin');
        final fileType = FileSystemEntity.typeSync(actual.path, followLinks: false);
        final expectedMode = _defaultFileMode(root);
        final actualMode = fileType == FileSystemEntityType.file ? actual.statSync().mode & 0x1ff : null;
        final verified = _exactOutputTree(destination) && fileType == FileSystemEntityType.file &&
            actualMode == expectedMode && actual.lengthSync() == expected.lengthSync() && _filesEqual(expected, actual);
        final elapsed = _timeElapsed(capture.stderr);
        final rss = _timeMaximumRss(capture.stderr);
        final cpu = _timeCpu(capture.stderr);
        final metricsValid = _positive(elapsed) && _positive(rss) && cpu != null;
        final drain = verified && metricsValid
            ? await _drainOutput(destination, wall, const Duration(minutes: 30))
            : <String, dynamic>{'passed': false, 'not_attempted': true};
        final result = {'kind': 'public_cli_sample', 'case': spec['name'], 'backend': variant,
          'round': round, 'warmup': warmup, 'sample': sampleId, 'order': order,
          'command': command, 'environment': env, 'gate': gate,
          'elapsed_seconds': elapsed, 'peak_rss_kib': rss, 'cpu': cpu,
          'metrics_valid': metricsValid, 'expected_default_mode': expectedMode, 'actual_mode': actualMode,
          'postprocess_drain': drain, 'output_drained_and_verified': verified && drain['passed'] == true,
          'output_regular_file_nofollow': fileType == FileSystemEntityType.file,
          'output_bytes': fileType == FileSystemEntityType.file ? actual.lengthSync() : null,
          'output_sha256': fileType == FileSystemEntityType.file ? _sha(actual.path) : null,
          'expected_sha256': _sha(expected.path), 'extraction_verified': verified,
          'exit_code': capture.code, 'outer_runner_timeout': capture.timedOut,
          'capture_incomplete': capture.incomplete, 'accepted': verified && metricsValid && drain['passed'] == true && !capture.incomplete};
        _appendJsonl('${output.path}/samples.jsonl', result);
        if (result['accepted'] != true) throw StateError('Public CLI verification failed; row/output retained: $sampleId');
        count++;
        Directory(destination).deleteSync(recursive: true);
      }
    }
  }
  _verifyFrozenIdentities();
  if (count != 64) throw StateError('Incomplete public CLI matrix');
  _writeJson('${output.path}/complete.json', {'accepted': count, 'measured': 48, 'prep': 16, 'identities_verified_after': true, 'wall_seconds': wall.elapsed.inMilliseconds / 1000});
  print(green('Completed $count public CLI samples (includes 16 warmups and 48 measured).'));
}

Map<String, dynamic>? _firstJsonLine(String stdout) {
  final records = <Map<String, dynamic>>[];
  for (final line in stdout.split('\n')) {
    try { final decoded = jsonDecode(line); if (decoded is Map<String, dynamic>) records.add(decoded); } catch (_) {}
  }
  return records.length == 1 ? records.single : null;
}

int? _readBytesFromRecord(Map<String, dynamic> record) {
  final storage = record['storage_io'];
  final raw = storage is Map ? storage['read_bytes'] : null;
  return raw is num ? raw.toInt() : int.tryParse('$raw');
}

double? _timeElapsed(String stderr) {
  final match = RegExp(r'Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s*([^\n]+)').firstMatch(stderr);
  if (match == null) return null;
  final parts = match.group(1)!.trim().split(':').map(double.parse).toList();
  return parts.length == 3 ? parts[0] * 3600 + parts[1] * 60 + parts[2] : parts[0] * 60 + parts[1];
}

int? _timeMaximumRss(String stderr) {
  final match = RegExp(r'Maximum resident set size \(kbytes\):\s*(\d+)').firstMatch(stderr);
  return match == null ? null : int.parse(match.group(1)!);
}

void _appendJsonl(String path, Object value) {
  File(path).parent.createSync(recursive: true);
  File(path).writeAsStringSync('${jsonEncode(value)}\n', mode: FileMode.append, flush: true);
}

void _validateCandidateCensus(Map<String, dynamic> spec, {required String outputRoot}) {
  final creation = jsonDecode(File('$evidenceRoot/fixture-creation-v2/${spec['name']}.json').readAsStringSync()) as Map;
  final census = creation['actual_candidate_codec_census'] as Map;
  final chunks = (spec['bytes'] as int) ~/ unitBytes;
  final expectedRaw = spec['compressed'] == true ? 0 : chunks;
  final expectedZstd = spec['compressed'] == true ? chunks : 0;
  if (census['logical_bytes'] != spec['bytes'] || census['raw_chunks'] != expectedRaw || census['zstd_chunks'] != expectedZstd) {
    throw StateError('Candidate codec census does not match frozen workload profile at $outputRoot: $census');
  }
}

Future<void> captureSmoke() async {
  final environment = <String, String>{'PATH': '/usr/bin:/bin', 'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011'};
  final result = await _capture('/usr/bin/time', ['-v', '/usr/bin/printf', 'capture-ok\n'], environment, 5);
  final timeoutResult = await _capture('/usr/bin/timeout', ['--kill-after=1s', '0.1s', '/usr/bin/sleep', '2'], environment, 5);
  final record = {
    'printf_command': ['/usr/bin/time', '-v', '/usr/bin/printf', 'capture-ok\\n'],
    'printf_exit_code': result.code, 'printf_outer_timeout': result.timedOut,
    'printf_capture_incomplete': result.incomplete, 'printf_stdout': result.stdout,
    'printf_time_stderr_captured': result.stderr.contains('Maximum resident set size'),
    'timeout_command': ['/usr/bin/timeout', '--kill-after=1s', '0.1s', '/usr/bin/sleep', '2'],
    'timeout_exit_code': timeoutResult.code, 'timeout_outer_timeout': timeoutResult.timedOut,
    'timeout_capture_incomplete': timeoutResult.incomplete,
    'timeout_status_retained_and_rejected': timeoutResult.code == 124 && !timeoutResult.timedOut && !timeoutResult.incomplete,
  };
  record['passed'] = result.code == 0 && !result.timedOut && !result.incomplete &&
      result.stdout == 'capture-ok\n' && record['printf_time_stderr_captured'] == true &&
      record['timeout_status_retained_and_rejected'] == true;
  _writeJson('$evidenceRoot/capture-smoke-v2.json', record);
  if (record['passed'] != true) throw StateError('Bounded complete-capture/timeout smoke failed; evidence retained.');
  print(green('Complete stdout/stderr capture and timeout-status smoke passed.'));
}

void freezeIdentities() {
  if (File('$evidenceRoot/pre-timing-identities.json').existsSync()) throw StateError('Refusing to overwrite frozen identities');
  _loadCases();
  final diff = Process.runSync('git', ['diff', '--', 'rust/revault_lockbox_api/benches/evaluation/runner.rs'], workingDirectory: worktree);
  if (diff.exitCode != 0 || (diff.stdout as String).isEmpty) throw StateError('Expected current-source benchmark runner diff is missing.');
  File('$evidenceRoot/benchmark-runner.diff').writeAsStringSync(diff.stdout as String, flush: true);
  final sourceHashes = <String, String>{};
  for (final base in ['$rustRoot/revault_lockbox_api/src', '$rustRoot/revault_lockbox_api/benches', '$rustRoot/revault_cli/src']) {
    for (final item in Directory(base).listSync(recursive: true, followLinks: false)) {
      if (item is File && item.path.endsWith('.rs')) sourceHashes[item.path.substring(worktree.length + 1)] = _sha(item.path);
    }
  }
  for (final path in ['$rustRoot/Cargo.lock', '$rustRoot/Cargo.toml', '$worktree/rust-toolchain.toml',
      '$rustRoot/revault_lockbox_api/Cargo.toml', '$rustRoot/revault_cli/Cargo.toml']) {
    sourceHashes[path.substring(worktree.length + 1)] = _sha(path);
  }
  final binaries = <String, dynamic>{};
  for (final file in Directory(binRoot).listSync().whereType<File>()) {
    binaries[file.uri.pathSegments.last] = {'path': file.path, 'sha256': _sha(file.path), 'bytes': file.lengthSync()};
  }
  final archiveInventory = <Map<String, dynamic>>[];
  for (final spec in cases) {
    final roots = {for (final row in spec['variants'] as List) (row as Map)['variant'] as String: row['root'] as String};
    final paths = {'A': '${roots['A']}/archive.lbox', 'B': '${roots['B']}/archive.lbox',
      'C': '${roots['C']}/tree.lbox', 'ZIP': '${roots['ZIP']}/archive.zip'};
    for (final entry in paths.entries) archiveInventory.add({
      'case': spec['name'], 'variant': entry.key, 'path': entry.value,
      'bytes': File(entry.value).lengthSync(), 'sha256': _sha(entry.value),
    });
  }
  final settings = {
    'A_B_native_component': 'public Lockbox::open(Unencrypted), standard native metadata authentication/cache, Interactive access, WorkerPolicy::Auto, CacheLimit::Auto128MiB; no extra verify_all',
    'A_B_CLI_extract': 'public CLI archive extract --to NEW_DIRECTORY; ExtractMany throughout, default permission restoration behavior; explicit creation one-file add after archive precreation',
    'A_B_create': 'encryption none, signing none, default padding, explicit zstd level 3 for source, compression none for raw; one source file per add invocation',
    'C1': 'candidate typed-tree, REVAULT_READ_WORKERS=1, actual worker_threads expected 0 for all four cases',
    'Cauto': 'candidate typed-tree, REVAULT_READ_WORKERS=0; actual worker_threads expected 0 for raw and 4 for compressed 64MiB/1GiB cases',
    'ZIP': 'current-source locked zip crate, Deflated source and Stored random/raw; use the same 64KiB output sink and read through EOF/CRC',
    'native_worker_observation': 'A/B normal API defaults, worker counts are configured defaults, not claimed observed counts',
  };
  final head = Process.runSync('git', ['rev-parse', 'HEAD'], workingDirectory: worktree);
  final manifest = {
    'base_revision': _revision('f6f44173'),
    'source_revision': '${head.stdout}'.trim(),
    'captured_utc': DateTime.now().toUtc().toIso8601String(),
    'source_hashes': sourceHashes,
    'creation_hashes': {for (final spec in cases) '${spec['name']}': _sha('$evidenceRoot/fixture-creation-v2/${spec['name']}.json')},
    'source_diff': {
      'path': '$evidenceRoot/benchmark-runner.diff',
      'sha256': _sha('$evidenceRoot/benchmark-runner.diff'),
    },
    'protocol_snapshot': {'path': '$evidenceRoot/protocol-before-measurements.md', 'sha256': _sha('$evidenceRoot/protocol-before-measurements.md')},
    'runner': {'path': Platform.script.toFilePath(), 'sha256': _sha(Platform.script.toFilePath())},
    'binaries': binaries,
    'source_corpus_sha256': _sha('$evidenceRoot/source-corpus-identity.json'),
    'fixture_manifest_sha256': _sha('$evidenceRoot/cases.json'),
    'calibration_sha256': _sha('$evidenceRoot/physical-read-calibration.json'),
    'archive_inventory': archiveInventory,
    'candidate_public_keys': [for (final spec in cases) {
      'case': spec['name'], 'path': '${(spec['variants'] as List).cast<Map<String, dynamic>>().singleWhere((row) => row['variant'] == 'C')['root']}/tree.public',
      'sha256': _sha('${(spec['variants'] as List).cast<Map<String, dynamic>>().singleWhere((row) => row['variant'] == 'C')['root']}/tree.public'),
    }],
    'component_settings': settings,
    'build_commands': File('$evidenceRoot/build-commands.txt').readAsStringSync(),
    'fixture_roots': fixtureRoot,
    'artifact_cap_bytes': artifactCap,
    'no_timing_has_run': true,
  };
  _writeJson('$evidenceRoot/pre-timing-identities.json', manifest);
  print(green('Pinned current source, binaries, fixtures, calibration, protocol, runner and settings; no timed samples run.'));
}

void _verifyFrozenIdentities() {
  final path = '$evidenceRoot/pre-timing-identities.json';
  if (!File(path).existsSync()) throw StateError('Frozen pre-timing identity manifest is missing.');
  final manifest = jsonDecode(File(path).readAsStringSync()) as Map<String, dynamic>;
  if (_revision('f6f44173') != manifest['base_revision']) throw StateError('Unexpected base commit');
  final currentHead = Process.runSync('git', ['rev-parse', 'HEAD'], workingDirectory: worktree);
  if (currentHead.exitCode != 0 || '${currentHead.stdout}'.trim() != manifest['source_revision']) throw StateError('Source HEAD changed');
  for (final spec in cases) {
    for (final row in spec['variants'] as List) {
      _validateSource(spec, row['root'] as String);
      if (_sha('${row['root']}/case.json') != row['case_json_sha256']) throw StateError('Case descriptor changed');
    }
    if (_sha('$evidenceRoot/fixture-creation-v2/${spec['name']}.json') != (manifest['creation_hashes'] as Map)[spec['name']]) throw StateError('Codec census evidence changed');
  }
  final runner = Platform.script.toFilePath();
  if (_sha(runner) != (manifest['runner'] as Map)['sha256'] ||
      _sha('$evidenceRoot/protocol-before-measurements.md') != (manifest['protocol_snapshot'] as Map)['sha256'] ||
      _sha('$evidenceRoot/benchmark-runner.diff') != (manifest['source_diff'] as Map)['sha256'] ||
      _sha('$evidenceRoot/source-corpus-identity.json') != manifest['source_corpus_sha256'] ||
      _sha('$evidenceRoot/cases.json') != manifest['fixture_manifest_sha256'] ||
      _sha('$evidenceRoot/physical-read-calibration.json') != manifest['calibration_sha256']) {
    throw StateError('Frozen runner/protocol/diff/source/fixture/calibration identity changed after freeze.');
  }
  for (final entry in (manifest['source_hashes'] as Map).entries) {
    if (_sha('$worktree/${entry.key}') != entry.value) throw StateError('Pinned source changed: ${entry.key}');
  }
  for (final entry in (manifest['binaries'] as Map).entries) {
    final row = entry.value as Map;
    if (_sha(row['path'] as String) != row['sha256']) throw StateError('Pinned executable changed: ${entry.key}');
  }
  for (final row in manifest['archive_inventory'] as List) {
    final archive = row as Map;
    if (_sha(archive['path'] as String) != archive['sha256']) throw StateError('Pinned archive changed: ${archive['case']}/${archive['variant']}');
  }
  for (final row in manifest['candidate_public_keys'] as List) {
    final key = row as Map;
    if (_sha(key['path'] as String) != key['sha256']) throw StateError('Pinned candidate public key changed: ${key['case']}');
  }
}

int _selfReadBytes() => int.parse(File('/proc/self/io').readAsLinesSync()
    .singleWhere((line) => line.startsWith('read_bytes:')).split(':').last.trim());

void _validateBinary(String name) {
  final path = '$binRoot/$name';
  if (!File(path).existsSync() || File(path).lengthSync() == 0) throw StateError('Missing or empty build binary: $path');
}

void _assertNonEmpty(String path) {
  final file = File(path);
  if (!file.existsSync() || file.lengthSync() == 0) throw StateError('Missing/empty artifact: $path');
}

List<Map<String, dynamic>> _records(String text, String marker) => [
  for (final line in text.split('\n').where((line) => line.contains(marker)))
    jsonDecode(line.substring(line.indexOf(marker) + marker.length)) as Map<String, dynamic>,
];

int _availableBytes() {
  final result = Process.runSync('df', ['-Pk', '/home/bsutton/.cache']);
  if (result.exitCode != 0) throw StateError('df failed: ${result.stderr}');
  final parts = '${result.stdout}'.trim().split('\n').last.trim().split(RegExp(r'\s+'));
  return int.parse(parts[3]) * 1024;
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

String _basename(String path) {
  final trimmed = path.replaceAll(RegExp(r'/+$'), '');
  return trimmed.substring(trimmed.lastIndexOf('/') + 1);
}

String _revision(String requested) {
  final result = Process.runSync('git', ['rev-parse', '--verify', '$requested^{commit}'], workingDirectory: worktree);
  if (result.exitCode != 0) throw StateError('Unable to resolve frozen base revision $requested');
  final resolved = '${result.stdout}'.trim();
  if (!resolved.startsWith(requested)) throw StateError('Unexpected base revision: $resolved');
  return resolved;
}

String _sha(String path) {
  final result = Process.runSync('sha256sum', [path]);
  if (result.exitCode != 0) throw StateError('sha256sum failed for $path: ${result.stderr}');
  return '${result.stdout}'.split(RegExp(r'\s+')).first;
}

String _shaText(String text) {
  final path = '$evidenceRoot/.stdout-${DateTime.now().microsecondsSinceEpoch}';
  File(path).writeAsStringSync(text, flush: true);
  final hash = _sha(path);
  File(path).deleteSync();
  return hash;
}

Future<Capture> _capture(String executable, List<String> args, Map<String, String> env, int timeoutSeconds) async {
  final process = await Process.start(executable, args, environment: env, includeParentEnvironment: false);
  final stdoutBuffer = StringBuffer();
  final stderrBuffer = StringBuffer();
  var streamFailed = false;
  final stdoutDone = Completer<void>();
  final stderrDone = Completer<void>();
  final stdoutSubscription = process.stdout.transform(utf8.decoder).listen(
    stdoutBuffer.write, onError: (Object error, StackTrace stack) {
      streamFailed = true;
      stderrBuffer.write('\n[stdout stream error: $error]');
      if (!stdoutDone.isCompleted) stdoutDone.complete();
    }, onDone: () { if (!stdoutDone.isCompleted) stdoutDone.complete(); });
  final stderrSubscription = process.stderr.transform(utf8.decoder).listen(
    stderrBuffer.write, onError: (Object error, StackTrace stack) {
      streamFailed = true;
      stderrBuffer.write('\n[stderr stream error: $error]');
      if (!stderrDone.isCompleted) stderrDone.complete();
    }, onDone: () { if (!stderrDone.isCompleted) stderrDone.complete(); });
  var code = -1;
  var timedOut = false;
  try {
    code = await process.exitCode.timeout(Duration(seconds: timeoutSeconds));
  } on TimeoutException {
    timedOut = true;
    process.kill(ProcessSignal.sigterm);
    try { code = await process.exitCode.timeout(const Duration(seconds: 10)); }
    on TimeoutException {
      process.kill(ProcessSignal.sigkill);
      try { code = await process.exitCode.timeout(const Duration(seconds: 10)); }
      on TimeoutException { code = -1; }
    }
  }
  var incomplete = false;
  try {
    await Future.wait([stdoutDone.future, stderrDone.future]).timeout(const Duration(seconds: 10));
  } on TimeoutException {
    incomplete = true;
    await stdoutSubscription.cancel();
    await stderrSubscription.cancel();
    stdoutBuffer.write('\n[stdout capture incomplete after bounded process timeout]');
    stderrBuffer.write('\n[stderr capture incomplete after bounded process timeout]');
  }
  return Capture(code, stdoutBuffer.toString(), stderrBuffer.toString(), timedOut: timedOut, incomplete: incomplete || streamFailed);
}

void _writeJson(String path, Object value) {
  final file = File(path);
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(const JsonEncoder.withIndent('  ').convert(value), flush: true);
}

bool _positive(dynamic value) => value is num && value.isFinite && value > 0;
bool _nonnegative(dynamic value) => value is num && value.isFinite && value >= 0;

bool _validEndpoint(Map<String, dynamic> sample, Map<String, dynamic> spec, int parity, String variant) {
  final identity = sample['reader_identity'];
  final resources = sample['resources'];
  final cpu = resources is Map ? resources['cpu'] : null;
  final devices = sample['block_device_io'];
  if (sample['kind'] != 'filesystem_extraction' || sample['verified'] != true ||
      sample['files'] != 1 || sample['logical_bytes_written'] != spec['bytes'] ||
      sample['file_syncs'] != 0 || sample['directory_syncs'] != 0 || sample['durability'] is! String ||
      !_positive(sample['total_seconds']) || resources is! Map || !_positive(resources['peak_rss_kib']) ||
      cpu is! Map || !_nonnegative(cpu['user_seconds']) || !_nonnegative(cpu['system_seconds']) ||
      identity is! Map || identity['reader_tid_parity'] != parity ||
      identity['pid_namespace'] != 'pid:[4026531836]' || identity['namespace_tids'] is! List ||
      (identity['namespace_tids'] as List).length != 1 || sample['requested_reader_tid_parity'] != parity ||
      devices is! Map || devices['devices'] is! List || (devices['devices'] as List).length != 2) return false;
  for (var index = 0; index < 2; index++) {
    final device = (devices['devices'] as List)[index];
    if (device is! Map || device['device'] != ['nvme0n1', 'nvme1n1'][index] || device['delta'] is! Map) return false;
    final delta = device['delta'] as Map;
    if (delta['sector_bytes'] != 512 || !_nonnegative(delta['read_sectors']) || !_nonnegative(delta['write_sectors'])) return false;
  }
  if (variant.startsWith('C')) {
    final automatic = variant == 'Cauto';
    final threads = automatic && spec['compressed'] == true ? 4 : 0;
    if (sample['configured_cpu_workers'] != (automatic ? 0 : 1) ||
        sample['automatic_workers'] != automatic || sample['single_chunk_serial_fallback'] != true ||
        sample['worker_threads'] != threads || sample['max_queued_chunks'] != (threads == 0 ? 1 : 32) ||
        sample['parallel_min_requested_bytes'] != (automatic ? 16 * 1024 * 1024 : 128 * 1024) ||
        (automatic && sample['automatic_contiguous_compressed_qualification_bytes'] != 16 * 1024 * 1024)) return false;
  }
  return true;
}

bool _exactOutputTree(String destination) {
  if (FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.directory) return false;
  final children = Directory(destination).listSync(followLinks: false);
  return children.length == 1 && children.single.path == '$destination/file-000000.bin' &&
      FileSystemEntity.typeSync(children.single.path, followLinks: false) == FileSystemEntityType.file;
}

int _defaultFileMode(String root) {
  final probe = File('$root/.default-mode-${DateTime.now().microsecondsSinceEpoch}');
  probe.writeAsStringSync('');
  final mode = probe.statSync().mode & 0x1ff;
  probe.deleteSync();
  return mode;
}

Map<String, double>? _timeCpu(String stderr) {
  double? field(String name) {
    final match = RegExp('$name \\(seconds\\):\\s*([0-9.]+)').firstMatch(stderr);
    return match == null ? null : double.tryParse(match.group(1)!);
  }
  final user = field('User time');
  final system = field('System time');
  return _nonnegative(user) && _nonnegative(system) ? {'user_seconds': user!, 'system_seconds': system!} : null;
}

int _remainingCommandSeconds(Stopwatch wall, Duration cap, {int maximum = 180}) {
  // Reserve child TERM/KILL grace, outer kill fallback and stream draining.
  final available = cap.inSeconds - wall.elapsed.inSeconds - 50;
  if (available < 1) throw StateError('Batch deadline reached; partial evidence retained');
  return available < maximum ? available : maximum;
}

Future<Capture> _boundedUtility(List<String> command, Stopwatch wall, Duration cap) async {
  final seconds = _remainingCommandSeconds(wall, cap, maximum: 30);
  return _capture('/usr/bin/timeout', ['--kill-after=15s', '$seconds', ...command],
      {'PATH': '/usr/bin:/bin', 'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011'}, seconds + 20);
}

Future<Map<String, dynamic>> _boundedColdGate(String archive, Stopwatch wall, Duration cap) async {
  final attempts = <Map<String, dynamic>>[];
  for (var attempt = 0; attempt < 3; attempt++) {
    final captures = <Capture>[];
    final commands = <List<String>>[
      ['/usr/bin/sync', '-d', '--', archive],
      ['/usr/bin/fadvise', '--advice', 'dontneed', archive],
      ['/usr/bin/fincore', '--json', '--bytes', '--output', 'SIZE,RES,DIRTY,WRITEBACK,FILE', archive],
    ];
    for (final command in commands) {
      final capture = await _boundedUtility(command, wall, cap);
      captures.add(capture);
      if (capture.timedOut || capture.incomplete || capture.code != 0) break;
    }
    dynamic parsed;
    try { if (captures.length == 3) parsed = jsonDecode(captures.last.stdout); } catch (_) {}
    final rows = parsed is Map ? parsed['fincore'] : parsed;
    final raw = rows is List && rows.length == 1 && rows.single is Map ? (rows.single as Map)['res'] : null;
    final resident = raw is num ? raw.toInt() : int.tryParse('$raw');
    final passed = captures.length == 3 && captures.every((r) => r.code == 0 && !r.incomplete && !r.timedOut) && resident == 0;
    attempts.add({'attempt': attempt, 'commands': commands, 'results': [for (final r in captures)
      {'exit_code': r.code, 'stdout': r.stdout, 'stderr': r.stderr, 'timed_out': r.timedOut, 'incomplete': r.incomplete}],
      'resident_bytes_before': resident, 'passed': passed});
    if (passed) return {'passed': true, 'attempts': attempts};
  }
  _appendJsonl('$evidenceRoot/cold-admission-failures.jsonl', {'archive': archive, 'attempts': attempts});
  return {'passed': false, 'attempts': attempts};
}

Future<Map<String, dynamic>> _drainOutput(String destination, Stopwatch wall, Duration cap) async {
  final started = Stopwatch()..start();
  final commands = <List<String>>[
    ['/usr/bin/sync', '-d', '--', '$destination/file-000000.bin'],
    ['/usr/bin/sync', '-d', '--', destination, Directory(destination).parent.path],
  ];
  final results = <Map<String, dynamic>>[];
  for (final command in commands) {
    final result = await _boundedUtility(command, wall, cap);
    final passed = result.code == 0 && !result.timedOut && !result.incomplete;
    results.add({'command': command, 'exit_code': result.code, 'stdout': result.stdout, 'stderr': result.stderr,
      'timed_out': result.timedOut, 'incomplete': result.incomplete, 'passed': passed});
    if (!passed) return {'passed': false, 'commands': results, 'seconds': started.elapsed.inMicroseconds / 1e6};
  }
  return {'passed': true, 'commands': results, 'seconds': started.elapsed.inMicroseconds / 1e6, 'outside_measurement': true};
}

void _checkArtifactBudget(int nextOutputBytes) {
  final owned = _apparentBytes(Directory('/home/bsutton/.cache/revault-native-public-20261011')) +
      _apparentBytes(Directory(binRoot)) + _apparentBytes(Directory(evidenceRoot));
  if (owned + nextOutputBytes > artifactCap || _availableBytes() < nextOutputBytes + 2 * 1024 * 1024 * 1024) {
    throw StateError('Owned artifact cap or free-space headroom would be exceeded');
  }
}
