#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';

const worktree = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
final inputs = jsonDecode(File("/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/tools/protected-write-baseline-draft/build-inputs.json").readAsStringSync()) as Map<String, dynamic>;
int size(String path) {
  final entity = FileSystemEntity.typeSync(path, followLinks: false);
  if (entity == FileSystemEntityType.notFound) return 0;
  if (entity == FileSystemEntityType.file) return File(path).lengthSync();
  if (entity != FileSystemEntityType.directory) return 0;
  var bytes = 0;
  for (final child in Directory(path).listSync(recursive: false, followLinks: false)) {
    bytes += size(child.path);
  }
  return bytes;
}

void main() {
  final owned = inputs['owned_root'] as String;
  final cli = inputs['cli_path'] as String;
  final evidence = inputs['evidence_root'] as String;
  final roots = <String>[
    owned,
    File(cli).parent.path,
    ...(inputs['additional_owned_roots'] as List).cast<String>(),
    '/home/bsutton/.cache/rv-zipcmp-20261012',
    '/home/bsutton/.cache/rv-pagelife-20261012',
    '$worktree/rust/target/public-zip-comparison-20261012/bin',
    '/home/bsutton/.cache/rv-pagebody-20261012',
    '$worktree/rust/target/page-body-borrow-20261012/bin',
    '/home/bsutton/.cache/rv-pageown-20261012',
    '$worktree/rust/target/page-flush-ownership-20261012/bin',
    '/home/bsutton/.cache/rv-addcache8-20261012',
    '$worktree/rust/target/single-file-add-cache8-20261012/bin',
    '/home/bsutton/.cache/rv-rawown-20261012',
    '$worktree/rust/target/raw-frame-ownership-20261012/bin',
    '/home/bsutton/.cache/rv-addcache-20261012',
    '$worktree/rust/target/single-file-add-cache-20261012/bin',
    '/home/bsutton/.cache/rv-win-20261012',
    '$worktree/rust/target/import-window-20261012/bin',
    '/home/bsutton/.cache/rv-enc-20261012',
    '$worktree/rust/target/page-body-lifetime-20261012/bin',
    '/home/bsutton/.cache/rv-add1-20261012',
    '$worktree/rust/target/parallel-encoder-reuse-202612/bin',
    '/home/bsutton/.cache/rv-add-20261012',
    '/home/bsutton/.cache/rv-single-20261012',
    '$worktree/rust/target/explicit-file-extraction-20261012/bin',
    '/home/bsutton/.cache/rv-many-20261012',
    '/home/bsutton/.cache/rv-many-zstd-20261012',
    '/home/bsutton/.cache/revault-native-public-20261011',
    '/home/bsutton/.cache/rv-credential-20261012',
    '$worktree/rust/target/credential-open-20261012/bin',
    '$worktree/rust/target/parallel-extraction-202612/bin',
    '$worktree/rust/target/native-public-20261011/bin',
    '$worktree/rust/target/native-extract-cache-budget-20261011/bin',
    '$worktree/rust/target/encrypted-final-page-20261012/bin',
    '$worktree/docs/evidence/encrypted-final-page-2026-10-12',
    evidence,
  ];
  var total = 0;
  for (final path in roots) {
    final n = size(path);
    total += n;
    stdout.writeln('$n\t${FileSystemEntity.typeSync(path, followLinks: false)}\t$path');
  }
  final nested = <String>[];
  for (var i = 0; i < roots.length; i++) {
    for (var j = 0; j < roots.length; j++) {
      if (i != j && roots[i].startsWith('${roots[j]}/')) nested.add('${roots[j]} contains ${roots[i]}');
    }
  }
  stdout.writeln('TOTAL_BYTES=$total');
  stdout.writeln('CAP_BYTES=${24 * 1024 * 1024 * 1024}');
  stdout.writeln('NESTED_PAIRS=${nested.length}');
  for (final pair in nested) stdout.writeln('NESTED\t$pair');
}
