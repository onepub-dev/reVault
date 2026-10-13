#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

void main() {
  const root = '/tmp/rv-security-cli-isolated-20261013';
  const sourceRoot = '$root/validation';
  const taskRoot = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
  const destination = '$taskRoot/docs/evidence/security-review-2026-10-13/raw/isolated-validation-copy';
  const oldSanitizedPublish = '$taskRoot/docs/evidence/security-review-2026-10-13/raw/post-default-corrected-cli/publish_two_servers.log';

  String sha(String path) {
    final result = Process.runSync('sha256sum', [path]);
    if (result.exitCode != 0) throw StateError('sha256sum failed for $path');
    return (result.stdout as String).split(RegExp(r'\s+')).first;
  }

  Map<String, Object?> copyOne(String source, String relative) {
    final sourceFile = File(source);
    if (FileSystemEntity.typeSync(source, followLinks: false) != FileSystemEntityType.file) {
      throw StateError('Refusing non-regular evidence file: $source');
    }
    final kind = Process.runSync('file', ['-b', source]);
    if (kind.exitCode != 0 || (kind.stdout as String).toLowerCase().contains('binary')) {
      throw StateError('Refusing binary evidence file: $source');
    }
    final bytes = sourceFile.lengthSync();
    final digest = sha(source);
    final target = File('$destination/$relative');
    target.parent.createSync(recursive: true);
    sourceFile.copySync(target.path);
    target.setLastModifiedSync(sourceFile.lastModifiedSync());
    Process.runSync('chmod', ['${(sourceFile.statSync().mode & 0x1ff).toRadixString(8)}', target.path]);
    if (target.lengthSync() != bytes || sha(target.path) != digest) {
      throw StateError('Copied artifact changed: $relative');
    }
    return {
      'source': source,
      'destination': relative,
      'bytes': bytes,
      'mode_octal': (sourceFile.statSync().mode & 0x1ff).toRadixString(8),
      'sha256': digest,
      'file_type': (kind.stdout as String).trim(),
    };
  }

  if (Directory(destination).existsSync() || File(destination).existsSync()) {
    throw StateError('Destination already exists; refusing overwrite: $destination');
  }

  final markerArgs = [
    '-l', '-i', '--no-messages', '--glob', '!*.wasm', '--glob', '!*.rlib',
    '--glob', '!*.rmeta', '--glob', '!*.so', '--glob', '!*.a', '--glob', '!*.d',
    '-e', '-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----',
    '-e', '-----BEGIN [A-Z ]+ PRIVATE KEY-----', '-e', 'vault.?backup',
    '-e', 'remembered_passwords', '-e', 'private_key', sourceRoot,
  ];
  final scan = Process.runSync('rg', markerArgs);
  if (scan.exitCode > 1) throw StateError('Marker scan command failed before copy.');
  final hits = (scan.stdout as String).trim().isEmpty
      ? <String>[]
      : (scan.stdout as String).trim().split('\n').toSet().toList()..sort();
  final expectedHits = <String>{
    '$sourceRoot/add-syntax-correction/before-1.txt',
    '$sourceRoot/add-syntax-correction/before-2.txt',
    '$sourceRoot/migration_cli-current-overlay.diff',
  };
  if (hits.toSet().difference(expectedHits).isNotEmpty || expectedHits.difference(hits.toSet()).isNotEmpty) {
    throw StateError('Marker scan differs from reviewed 3-path exclusion list; no copy performed.');
  }

  final publishFiles = [
    oldSanitizedPublish,
    '$sourceRoot/publish_two_servers.log',
    '$sourceRoot/publish_failover.log',
    '$sourceRoot/publish_two_servers-suppressed.log',
    '$sourceRoot/publish_two_servers-suppressed-ignored.log',
    '$sourceRoot/publish_failover-suppressed-ignored.log',
  ];
  final publishScan = Process.runSync('rg', [
    '-l', '-i', '--no-messages',
    '-e', '-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----',
    '-e', '-----BEGIN [A-Z ]+ PRIVATE KEY-----', '-e', 'vault.?backup',
    '-e', 'remembered_passwords', '-e', 'private_key', ...publishFiles,
  ]);
  if (publishScan.exitCode != 1 || (publishScan.stdout as String).trim().isNotEmpty) {
    throw StateError('Sanitized publish logs had a marker hit or scan error; no copy performed.');
  }

  Directory(destination).createSync(recursive: true);
  final excluded = <Map<String, Object?>>[];
  final copied = <Map<String, Object?>>[];
  final excludedSet = expectedHits;
  final files = Directory(sourceRoot).listSync(recursive: true, followLinks: false)
      .whereType<File>().toList()..sort((a, b) => a.path.compareTo(b.path));
  for (final file in files) {
    if (excludedSet.contains(file.path)) {
      excluded.add({
        'source': file.path,
        'bytes': file.lengthSync(),
        'sha256': sha(file.path),
        'disposition': 'excluded because broad marker scan matched; marker requires classification; not asserted to be secret',
      });
      continue;
    }
    final relative = file.path.substring(sourceRoot.length + 1);
    copied.add(copyOne(file.path, 'isolated/$relative'));
  }
  for (final publishFile in publishFiles) {
    final name = publishFile.split('/').last;
    final relative = publishFile == oldSanitizedPublish
        ? 'redacted-publish/pre-correction/$name'
        : 'redacted-publish/corrected/$name';
    copied.add(copyOne(publishFile, relative));
  }
  copied.add(copyOne('/tmp/rv-security-cli-isolated-20261013/copy-validation-evidence.dart', 'copy-validation-evidence.dart'));
  final manifest = <String, Object?>{
    'kind': 'sanitized_isolated_validation_evidence_copy',
    'source_root': sourceRoot,
    'destination_root': destination,
    'source_validation_file_count': files.length,
    'copied_file_count': copied.length,
    'excluded_marker_hit_file_count': excluded.length,
    'excluded_marker_hits': excluded,
    'marker_scan': {
      'patterns': ['PEM private-key begin markers', 'vault backup marker', 'remembered_passwords', 'private_key'],
      'hit_count': hits.length,
      'hit_paths_only': hits.map((p) => p.replaceFirst('$sourceRoot/', '')).toList(),
      'sanitized_publish_log_hit_count': 0,
      'note': 'Filename/count-only result manifest; this pattern scan is not proof that all copied artifacts contain no sensitive material.',
    },
    'redaction_record': {
      'path': 'isolated/log-redaction-disposition.json',
      'note': 'Copied unchanged. Original pre-redaction transcript was already present in tool output and is not recoverable; original hashes were unavailable. Copied publish logs are the sanitized current files only.',
    },
    'files': copied,
  };
  final manifestFile = File('$destination/copy-manifest.json');
  manifestFile.writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(manifest)}\n');
  print('Copied ${copied.length} text/metadata files; excluded ${excluded.length} broad-marker-hit files.');
}
