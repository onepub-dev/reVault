#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const worktree = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const toolDir = '$worktree/tools/encrypted-final-page-trial';
const evidence = '$worktree/docs/evidence/encrypted-final-page-2026-10-12';
const owned = '/home/bsutton/.cache/rv-ep-20261012';
const oldCli = '$worktree/rust/target/protected-write-current-20261012/bin/lockbox-current';
const newCli = '$worktree/rust/target/encrypted-final-page-20261012/bin/lockbox-new';
const baselineEvidence = '$worktree/docs/evidence/protected-write-current-2026-10-12/raw/build';
const candidateEvidence = '$evidence/raw/build-new';

void require(bool value, String message) { if (!value) throw StateError(message); }
Map<String, dynamic> readObject(String path) => jsonDecode(File(path).readAsStringSync()) as Map<String, dynamic>;
String sha(String path) {
  final result = Process.runSync('/usr/bin/sha256sum', ['--', path]);
  require(result.exitCode == 0, 'Unable to hash $path');
  return (result.stdout as String).split(' ').first;
}
Map<String, String> parseManifest(String path) {
  final result = <String, String>{};
  for (final line in File(path).readAsLinesSync()) {
    final pieces = line.split(RegExp(r'\s+', multiLine: false));
    require(pieces.length == 2, 'Malformed manifest line: $line');
    result[pieces[1]] = pieces[0];
  }
  return result;
}
void main() {
  final inputs = readObject('$worktree/tools/protected-write-baseline-draft/build-inputs.json');
  require(inputs['head'] == 'b5f82693b0b46dbd9c9c4fb45c5166f0ba3af8ab', 'Unexpected pinned HEAD');
  require(inputs['cli_sha256'] == '9ab29f4cf50054ed9c816ae6e1bc464b8ff053fb0a2e83775a2d8da5a85f9312', 'Baseline executable provenance changed');
  final baselineManifest = parseManifest('$baselineEvidence/runtime-source-manifest.pre-rust.sha256');
  final candidatePre = parseManifest('$candidateEvidence/runtime-source-manifest-pre-rust.sha256');
  final candidatePost = parseManifest('$candidateEvidence/runtime-source-manifest-post-rust.sha256');
  require(baselineManifest.length == 779 && candidatePre.length == 779 && candidatePost.length == 779, 'Expected 779 Rust/Cargo inputs');
  require(candidatePre.keys.toSet().containsAll(baselineManifest.keys) && baselineManifest.keys.toSet().containsAll(candidatePre.keys), 'Rust/Cargo manifest paths differ');
  require(jsonEncode(candidatePre) == jsonEncode(candidatePost), 'Rust/Cargo inputs changed during build');
  final changed = <String>[];
  for (final path in baselineManifest.keys) {
    if (baselineManifest[path] != candidatePre[path]) changed.add(path);
  }
  require(changed.toSet().containsAll({
    'rust/revault_lockbox_api/src/file_format/page.rs',
    'rust/revault_lockbox_api/src/keys/crypto.rs',
  }) && changed.length == 2, 'Candidate source differs outside the reviewed pair: $changed');
  final runtime = Map<String, dynamic>.from(inputs['runtime_source_hashes'] as Map);
  require(runtime.length == 779, 'Unexpected runtime map size ${runtime.length}');
  for (final path in runtime.keys.toList()) {
    final relative = path.replaceFirst('$worktree/', '');
    if (candidatePost.containsKey(relative)) runtime[path] = candidatePost[relative];
  }
  for (final path in runtime.keys) require(path.startsWith('$worktree/rust/'), 'Runtime path outside rust: $path');

  final gitFiles = Process.runSync('/usr/bin/git', ['-C', worktree, 'ls-files', '-z', 'rust']);
  require(gitFiles.exitCode == 0, 'git ls-files failed');
  final supplemental = <String, String>{};
  for (final relative in (gitFiles.stdout as String).split('\u0000').where((v) => v.isNotEmpty)) {
    final basename = relative.split('/').last;
    final isCrateManifest = basename == 'Cargo.toml' && relative != 'rust/Cargo.toml';
    final isToolchain = basename.startsWith('rust-toolchain');
    final isCargoConfig = relative.contains('/.cargo/config') || relative.startsWith('rust/.cargo/config');
    if (!isCrateManifest && !isToolchain && !isCargoConfig) continue;
    final path = '$worktree/$relative';
    final check = Process.runSync('/usr/bin/git', ['-C', worktree, 'diff', '--quiet', 'HEAD', '--', relative]);
    require(check.exitCode == 0, 'Supplemental build config differs from HEAD: $relative');
    supplemental[path] = sha(path);
  }
  require(supplemental.isNotEmpty, 'No supplemental Cargo/toolchain config pins found');

  final oldHash = sha(oldCli), newHash = sha(newCli);
  require(oldHash == inputs['cli_sha256'], 'Preserved old CLI hash mismatch');
  require(newHash == '4142dd043db7ccd608f5eaccdf3d9673b76b5879fb0a729e2527207c0924d4d6', 'Candidate CLI hash mismatch');
  final variants = {'old': oldCli, 'new': newCli};
  final executableHashes = {oldCli: oldHash, newCli: newHash};
  final provenance = <String, String>{};
  for (final relative in [
    'cargo-build.command.txt', 'cargo-build.stdout.log', 'cargo-build.stderr.log', 'cargo-build.exit',
    'build.started-utc.txt', 'build.finished-utc.txt', 'head.pre.txt', 'head.post.txt',
    'runtime-source-manifest-pre-rust.sha256', 'runtime-source-manifest-post-rust.sha256',
    'candidate-vs-baseline-rust-manifest.diff', 'pre-post-rust-manifest.diff',
    'runtime-source-diff.patch', 'changed-inputs.sha256', 'build-artifacts.sha256',
    'runtime-source-manifest.pre.sha256', 'runtime-source-manifest.post.sha256',
  ]) provenance['$candidateEvidence/$relative'] = sha('$candidateEvidence/$relative');
  for (final relative in [
    'cargo-build.command.txt', 'cargo-build.stdout.log', 'cargo-build.stderr.log', 'cargo-build.exit',
    'build.started-utc.txt', 'build.finished-utc.txt', 'head.pre.txt', 'head.post.txt',
    'runtime-source-manifest.pre-rust.sha256', 'runtime-source-manifest.post-rust.sha256',
    'runtime-source-diff.patch', 'cli-artifact-hashes.sha256',
  ]) provenance['$baselineEvidence/$relative'] = sha('$baselineEvidence/$relative');
  provenance.addAll(supplemental);

  final roots = <String>{
    ...(inputs['additional_owned_roots'] as List).cast<String>(),
    inputs['evidence_root'] as String,
    inputs['owned_root'] as String,
    File(oldCli).parent.path,
    '$worktree/docs/evidence/protected-write-current-2026-10-12-attempt2',
    toolDir,
  }.toList();
  final config = <String, dynamic>{
    ...inputs,
    'execution_authorized': true,
    'worktree': worktree,
    'head': 'b5f82693b0b46dbd9c9c4fb45c5166f0ba3af8ab',
    'cli_path': newCli,
    'cli_sha256': newHash,
    'variants': variants,
    'executable_hashes': executableHashes,
    'evidence_root': evidence,
    'owned_root': owned,
    'runtime_source_hashes': runtime,
    'build_source_manifest_verified': true,
    'build_provenance_hashes': provenance,
    'additional_owned_roots': roots,
  };
  require(provenance.length > 15 && runtime.length == 779, 'Incomplete provenance');
  File('$toolDir/PROTOCOL.txt').writeAsStringSync(File('$evidence/PROTOCOL.txt').readAsStringSync());
  File('$toolDir/build-inputs.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(config)}\n', flush: true);
  print(green('Pinned old/new executables; 779 current Rust/Cargo hashes, ${supplemental.length} supplemental config hashes, ${provenance.length} build provenance entries.'));
}
