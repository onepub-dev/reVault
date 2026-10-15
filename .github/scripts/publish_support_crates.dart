#! /usr/bin/env dcli

// Explicit support-package publication through the existing trusted workflow.
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const bootstrap = [
  'revault_secure_memory',
  'revault_migration_format',
  'revault_migrate_v1',
  'revault_migrate_v2',
  'revault_migrate_v3',
];

String run(String executable, List<String> arguments, {String? cwd, bool quiet = false}) {
  final result = Process.runSync(executable, arguments, workingDirectory: cwd);
  if (!quiet) stdout.write(result.stdout);
  if (result.exitCode != 0) {
    stderr.write(result.stderr);
    throw StateError('$executable failed (${result.exitCode})');
  }
  return result.stdout as String;
}

Future<bool> published(String package, String version) async {
  final client = HttpClient();
  try {
    final uri = Uri.https('index.crates.io',
        '/${package.substring(0, 2)}/${package.substring(2, 4)}/$package');
    final response = await (await client.getUrl(uri)).close();
    final body = await utf8.decoder.bind(response).join();
    if (response.statusCode == 404) return false;
    if (response.statusCode != 200) {
      throw StateError('Registry lookup failed: ${response.statusCode}');
    }
    for (final line in const LineSplitter().convert(body)) {
      final entry = jsonDecode(line) as Map<String, dynamic>;
      if (entry['vers'] != version) continue;
      if (entry['name'] != package || entry['yanked'] != false) {
        throw StateError('$package@$version is yanked or mismatched');
      }
      return true;
    }
    return false;
  } finally {
    client.close(force: true);
  }
}

Future<void> publish(String package, String version, String manifest,
    bool apply, {bool locked = false}) async {
  if (await published(package, version)) {
    print('Already published: $package@$version');
    return;
  }
  run('cargo', ['publish', '--manifest-path', manifest, '--registry', 'crates-io',
    if (locked) '--locked',
    if (!apply) '--dry-run']);
  if (!apply) return;
  for (var attempt = 0; attempt < 30; attempt++) {
    if (await published(package, version)) return;
    await Future<void>.delayed(const Duration(seconds: 2));
  }
  throw StateError('$package@$version did not become visible');
}

Future<void> main(List<String> args) async {
  if (args.length == 1 && args.single == '--help') {
    print('publish_support_crates.dart bootstrap|retirement [--publish]\n'
        'Defaults to cargo publish dry runs; publication requires the reviewed main CI revision.');
    return;
  }
  if (args.isEmpty || !['bootstrap', 'retirement'].contains(args.first) ||
      args.skip(1).any((arg) => arg != '--publish')) {
    throw ArgumentError('publish_support_crates.dart bootstrap|retirement [--publish]');
  }
  final apply = args.contains('--publish');
  final root = run('git', ['rev-parse', '--show-toplevel']).trim();
  String git(List<String> arguments) => run('git', arguments, cwd: root).trim();
  if (git(['branch', '--show-current']) != 'main' ||
      git(['status', '--porcelain']).isNotEmpty) {
    throw StateError('Support publication requires clean committed main');
  }
  final sha = git(['rev-parse', 'HEAD']);
  if (apply && (Platform.environment['GITHUB_REF'] != 'refs/heads/main' ||
      Platform.environment['GITHUB_SHA'] != sha ||
      Platform.environment['EXPECTED_SOURCE_SHA'] != sha)) {
    throw StateError('Publication requires the explicitly selected main workflow revision');
  }
  final remote = run('gh', ['api', 'repos/onepub-dev/reVault/compare/$sha...main',
    '--jq', '.merge_base_commit.sha'], cwd: root).trim();
  if (remote != sha) throw StateError('Source revision is not on remote main');
  final metadata = jsonDecode(run('cargo', ['metadata', '--locked', '--no-deps',
    '--format-version', '1', '--manifest-path', '$root/rust/Cargo.toml'], quiet: true))
      as Map<String, dynamic>;
  final packages = <String, Map<String, dynamic>>{
    for (final package in (metadata['packages'] as List).cast<Map<String, dynamic>>())
      package['name'] as String: package,
  };
  String version(String name) => packages[name]!['version'] as String;
  if (args.first == 'bootstrap') {
    for (final name in bootstrap) {
      await publish(name, version(name), packages[name]!['manifest_path'] as String, apply, locked: true);
    }
  } else {
    // Terminal metadata releases follow the new CLI release and every replacement.
    final cliVersion = version('revault_cli');
    final release = jsonDecode(run('gh', ['release', 'view', 'revault_cli-v$cliVersion',
      '--repo', 'onepub-dev/reVault', '--json', 'isDraft,isPrerelease,tagName'])) as Map<String, dynamic>;
    if (release['isDraft'] != false || release['isPrerelease'] != false ||
        !await published('revault_cli', cliVersion)) {
      throw StateError('Publish the stable CLI release before retiring old packages');
    }
    final recipe = jsonDecode(File('$root/rust/retired-packages.json').readAsStringSync())
        as Map<String, dynamic>;
    if (recipe['schema'] != 1) throw StateError('Unsupported retirement allowlist');
    final entries = (recipe['packages'] as List).cast<Map<String, dynamic>>();
    for (final replacement in entries.map((entry) => entry['replacement'] as String).toSet()) {
      if (!await published(replacement, version(replacement))) {
        throw StateError('Replacement $replacement@${version(replacement)} is not published');
      }
    }
    final staging = '/tmp/revault-retirement-${sha.substring(0, 12)}';
    run('$root/.github/scripts/stage_retired_crates.dart',
      ['--revision', sha, '--output', staging], cwd: root);
    for (final entry in entries) {
      final name = entry['package'] as String;
      await publish(name, entry['terminal_version'] as String, '$staging/$name/Cargo.toml', apply);
    }
  }
  print(green('${args.first} ${apply ? 'publication' : 'dry run'} completed'));
}
