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
  const Capture(this.code, this.stdout, this.stderr, {this.timedOut = false});
  final int code;
  final String stdout;
  final String stderr;
  final bool timedOut;
}

late Map<String, dynamic> corpusRecord;
late List<Map<String, dynamic>> cases;

Future<void> main(List<String> args) async {
  if (args.isEmpty || args.first == 'help') {
    print('prepare | create | calibrate | freeze');
    return;
  }
  switch (args.first) {
    case 'prepare':
      await prepare();
    case 'create':
      await createAll();
    case 'calibrate':
      await calibrateAll();
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
        'source_bytes': target.lengthSync(),
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
  if (result.code != 0 || (acceptStdoutMarker != null && !result.stdout.contains(acceptStdoutMarker))) {
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
  final sameBytes = actual.existsSync() && actual.lengthSync() == expected.lengthSync() && _filesEqual(expected, actual);
  final verified = names.length == 1 && names.single == 'file-000000.bin' && sameBytes &&
      _sha(expected.path) == _sha(actual.path) && File('$root/archive.lbox').existsSync();
  _writeJson('$output/${spec['name']}/$variant-public-extract-verification.json', {
    'command': command,
    'environment': environment,
    'destination': destination,
    'extraction_mode': 'public CLI archive extract --to; ExtractMany, single-file content ReadMostly',
    'expected_bytes': expected.lengthSync(),
    'actual_bytes': actual.existsSync() ? actual.lengthSync() : null,
    'expected_sha256': _sha(expected.path),
    'actual_sha256': actual.existsSync() ? _sha(actual.path) : null,
    'directory_entries': names,
    'bytewise_compare_passed': sameBytes,
    'verified': verified,
  });
  if (!verified) throw StateError('Public CLI persisted extraction verification failed for ${spec['name']}/$variant; output retained.');
  Directory(destination).deleteSync(recursive: true);
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

void freezeIdentities() {
  _loadCases();
  final sourceHashes = <String, String>{};
  for (final base in ['$rustRoot/revault_lockbox_api/src', '$rustRoot/revault_lockbox_api/benches', '$rustRoot/revault_cli/src']) {
    for (final item in Directory(base).listSync(recursive: true, followLinks: false)) {
      if (item is File && item.path.endsWith('.rs')) sourceHashes[item.path.substring(worktree.length + 1)] = _sha(item.path);
    }
  }
  for (final path in ['$rustRoot/Cargo.lock', '$rustRoot/Cargo.toml', '$rustRoot/rust-toolchain.toml',
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
    'A_B_CLI_extract': 'public CLI archive extract --to NEW_DIRECTORY; ExtractMany, single-file content path ReadMostly; explicit creation one-file add after archive precreation',
    'A_B_create': 'encryption none, signing none, default padding, explicit zstd level 3 for source, compression none for raw; one source file per add invocation',
    'C1': 'candidate typed-tree, REVAULT_READ_WORKERS=1, actual worker_threads expected 0 for all four cases',
    'Cauto': 'candidate typed-tree, REVAULT_READ_WORKERS=0; actual worker_threads expected 0 for raw and 4 for compressed 64MiB/1GiB cases',
    'ZIP': 'current-source locked zip crate, Deflated source and Stored random/raw; use the same 64KiB output sink and read through EOF/CRC',
    'native_worker_observation': 'A/B normal API defaults, worker counts are configured defaults, not claimed observed counts',
  };
  final head = Process.runSync('git', ['rev-parse', 'HEAD'], workingDirectory: worktree);
  final manifest = {
    'base_revision': _revision('f6f4417323d4ed19c9591f01b94a26ed3ab26482'),
    'source_revision': '${head.stdout}'.trim(),
    'captured_utc': DateTime.now().toUtc().toIso8601String(),
    'source_hashes': sourceHashes,
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
    'component_settings': settings,
    'build_commands': File('$evidenceRoot/build-commands.txt').readAsStringSync(),
    'fixture_roots': fixtureRoot,
    'artifact_cap_bytes': artifactCap,
    'no_timing_has_run': true,
  };
  _writeJson('$evidenceRoot/pre-timing-identities.json', manifest);
  print(green('Pinned current source, binaries, fixtures, calibration, protocol, runner and settings; no timed samples run.'));
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
  final result = Process.runSync('git', ['rev-parse', requested], workingDirectory: worktree);
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
  final stdout = process.stdout.transform(utf8.decoder).join();
  final stderr = process.stderr.transform(utf8.decoder).join();
  var code = -1;
  var timedOut = false;
  try {
    code = await process.exitCode.timeout(Duration(seconds: timeoutSeconds));
  } on TimeoutException {
    timedOut = true;
    process.kill(ProcessSignal.sigterm);
    try { code = await process.exitCode.timeout(const Duration(seconds: 10)); }
    on TimeoutException { process.kill(ProcessSignal.sigkill); }
  }
  final capturedOut = await stdout.timeout(const Duration(seconds: 10), onTimeout: () => '[stdout pipe remained open after process timeout]');
  final capturedErr = await stderr.timeout(const Duration(seconds: 10), onTimeout: () => '[stderr pipe remained open after process timeout]');
  return Capture(code, capturedOut, capturedErr, timedOut: timedOut);
}

void _writeJson(String path, Object value) {
  final file = File(path);
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(const JsonEncoder.withIndent('  ').convert(value), flush: true);
}
