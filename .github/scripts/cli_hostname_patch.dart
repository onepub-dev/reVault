#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const version = '0.6.1';
const tag = 'revault_cli-v$version';

Future<String> run(String command, List<String> args, {String? cwd}) async {
  final r = await Process.run(command, args, workingDirectory: cwd, runInShell: Platform.isWindows);
  if (r.exitCode != 0) {
    stderr.write(r.stderr);
    throw StateError('$command failed (${r.exitCode})');
  }
  return r.stdout as String;
}

Future<void> copyTree(Directory source, Directory destination) async {
  destination.createSync(recursive: true);
  for (final entry in source.listSync()) {
    final name = entry.uri.pathSegments.where((s) => s.isNotEmpty).last;
    if (name == '.cargo-ok' || name == '.cargo-checksum.json') continue;
    final path = '${destination.path}/$name';
    if (entry is Directory) {
      await copyTree(entry, Directory(path));
    } else if (entry is File) {
      entry.copySync(path);
    }
  }
}

Future<Directory> registrySource(String selectedVersion) async {
  await run('cargo', ['info', 'revault_cli@$selectedVersion', '--registry', 'crates-io'], cwd: Directory.systemTemp.path);
  final home = Platform.environment['CARGO_HOME'] ?? '${Platform.environment[Platform.isWindows ? 'USERPROFILE' : 'HOME']}/.cargo';
  final sources = Directory('$home/registry/src').listSync().whereType<Directory>();
  final matches = sources.map((d) => Directory('${d.path}/revault_cli-$selectedVersion')).where((d) => d.existsSync()).toList();
  if (matches.length != 1) throw StateError('Expected one downloaded CLI registry source');
  return matches.single;
}

Future<Directory> stagePublished() async {
  final stage = Directory('cli-patch-stage').absolute;
  if (stage.existsSync()) throw StateError('Staging directory already exists');
  await copyTree(await registrySource(version), stage);
  File('${stage.path}/dist-workspace.toml').writeAsStringSync('''
[workspace]
members = ["cargo:."]
[dist]
cargo-dist-version = "0.31.0"
tag-namespace = "revault_cli-v"
allow-dirty = ["ci"]
''');
  // cargo-dist requires repository context even for an isolated registry crate.
  await run('git', ['init'], cwd: stage.path);
  await run('git', ['config', 'user.name', 'reVault release'], cwd: stage.path);
  await run('git', ['config', 'user.email', 'support@onepub.dev'], cwd: stage.path);
  await run('git', ['remote', 'add', 'origin', 'https://github.com/onepub-dev/reVault.git'], cwd: stage.path);
  await run('git', ['add', '.'], cwd: stage.path);
  await run('git', ['commit', '-m', 'Package published CLI $version'], cwd: stage.path);
  return stage;
}

Future<void> main(List<String> args) async {
  if (args.length != 1) throw ArgumentError('Expected integrate, stage, binaries, or publish-binaries');
  final source = Platform.environment['PATCH_SOURCE_SHA'] ?? '';
  if (!RegExp(r'^[0-9a-f]{40}$').hasMatch(source)) throw StateError('Exact source SHA required');
  if ((await run('git', ['rev-parse', 'HEAD'])).trim() != source) throw StateError('Source checkout mismatch');
  await run('git', ['fetch', 'origin', 'main']);
  if (args.single == 'integrate') {
    await run('git', ['merge-base', '--is-ancestor', 'origin/main', source]);
    if (!File('rust/revault_cli/Cargo.toml').readAsStringSync().contains('version = "$version"')) throw StateError('Unexpected CLI version');
    // GITHUB_TOKEN avoids recursive push workflows; no CI-skip commit markers.
    await run('git', ['push', 'origin', 'HEAD:main']);
    print('Integrated reviewed CLI patch source $source into main. No tests requested.');
    return;
  }
  await run('git', ['merge-base', '--is-ancestor', source, 'origin/main']);
  if (args.single == 'stage') {
    await stagePublished();
    return;
  }
  if (args.single == 'binaries') {
    final target = Platform.environment['PATCH_TARGET'] ?? '';
    if (!['x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu', 'x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-pc-windows-msvc'].contains(target)) throw StateError('Unexpected target');
    final stage = await stagePublished();
    final manifest = await run('dist', ['build', '--artifacts=local', '--target=$target', '--tag=$tag', '--output-format=json'], cwd: stage.path);
    File('${stage.path}/target/distrib/$target-dist-manifest.json').writeAsStringSync(manifest);
    print('Built CLI $version for $target without running tests.');
    return;
  }
  if (args.single != 'publish-binaries') throw StateError('Unknown mode');
  final stage = Directory('cli-patch-stage').absolute;
  final manifest = await run('dist', ['build', '--artifacts=global', '--tag=$tag', '--output-format=json'], cwd: stage.path);
  final data = jsonDecode(manifest) as Map<String, dynamic>;
  final uploads = (data['upload_files'] as List).cast<String>().map((p) => File(p).isAbsolute ? p : '${stage.path}/$p').toList();
  if (uploads.isEmpty) throw StateError('No release assets');
  await run('git', ['tag', tag, source]);
  await run('git', ['push', 'origin', 'refs/tags/$tag']);
  await run('gh', ['release', 'create', tag, '--verify-tag', '--title', 'reVault CLI $version', '--notes-file', 'release-notes/cli-0.6.1.txt', ...uploads]);
  print('Published only CLI $version GitHub binaries and installers.');
}
