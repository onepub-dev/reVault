#! /usr/bin/env dcli

// Stage only: no registry access and no publishing. Run from a main checkout.
import 'dart:convert';
import 'dart:io';

import 'package:dcli/dcli.dart';

String run(String executable, List<String> arguments, {String? cwd}) {
  final result = Process.runSync(executable, arguments, workingDirectory: cwd);
  if (result.exitCode != 0) {
    throw StateError('$executable failed: ${result.stderr}');
  }
  return result.stdout as String;
}

Never fail(String message) => throw StateError(message);

// This deliberately supports the manifest syntax used by the retirement
// allowlist, rejecting ambiguous/unsupported path dependency declarations.
// Split TOML into statements without splitting multiline arrays/inline tables.
List<String> statements(String input) {
  final result = <String>[];
  var start = 0;
  var depth = 0;
  String? quote;
  var escaped = false;
  var comment = false;
  for (var i = 0; i < input.length; i++) {
    final c = input[i];
    if (comment) {
      if (c != '\n') continue;
      comment = false;
    } else if (quote != null) {
      if (escaped) {
        escaped = false;
      } else if (c == r'\' && quote == '"') {
        escaped = true;
      } else if (c == quote) {
        quote = null;
      }
      continue;
    } else if (c == '"' || c == "'") {
      quote = c;
      continue;
    } else if (c == '#') {
      comment = true;
      continue;
    } else if (c == '[' || c == '{') {
      depth++;
    } else if (c == ']' || c == '}') {
      depth--;
    }
    if (depth < 0) fail('Unbalanced manifest');
    if (c == '\n' && depth == 0) {
      result.add(input.substring(start, i + 1));
      start = i + 1;
    }
  }
  if (quote != null || depth != 0) fail('Unsupported or incomplete TOML');
  if (start < input.length) result.add(input.substring(start));
  return result;
}

final scalar = RegExp(r'''^\s*([\w-]+)\s*=\s*(["'])(.*?)\2\s*(?:#.*)?$''');
final pathField = RegExp(r'''\bpath\s*=\s*(["'])([^"'\r\n]*)\1''');
final versionField = RegExp(r'''\bversion\s*=\s*(["'])([^"'\r\n]*)\1''');
final exactVersion = RegExp(
  r'^=?\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$',
);

String pin(String value) {
  if (!exactVersion.hasMatch(value)) {
    fail('Path dependency needs an explicit complete registry version: $value');
  }
  return value.startsWith('=') ? value : '=$value';
}

String sectionName(String statement) {
  final match = RegExp(r'^\s*\[([^\n]+)\]\s*(?:#.*)?$')
      .firstMatch(statement.trim());
  return match?.group(1) ?? '';
}

bool dependencySection(String section) =>
    RegExp(r'(^|\.)(dependencies|dev-dependencies|build-dependencies)(\.|$)')
        .hasMatch(section);
bool dependencyTable(String section) =>
    RegExp(r'(^|\.)(dependencies|dev-dependencies|build-dependencies)\.')
        .hasMatch(section);

String normalize(
  String original,
  Map<String, dynamic> recipe,
  String Function(String) localFile,
) {
  final lines = statements(original);
  var section = '';
  final package = <String, String>{};
  for (final line in lines) {
    final header = sectionName(line);
    if (header.isNotEmpty) section = header;
    if (section == 'package') {
      final m = scalar.firstMatch(line.trim());
      if (m != null) package[m[1]!] = m[3]!;
    }
  }
  if (package['name'] != recipe['package'] ||
      package['version'] != recipe['source_version']) {
    fail('Source package/version differs from recipe: ${recipe['package']}');
  }
  final replacement = recipe['replacement'] as String;
  bool notice(String text) =>
      text.toLowerCase().contains('deprecated') && text.contains(replacement);
  if (!notice(package['description'] ?? '')) fail('Missing description notice');
  final readme = package['readme'];
  if (readme != null &&
      (readme.startsWith('/') || readme.split('/').contains('..'))) {
    fail('README must be inside the source package');
  }
  if (readme == null || !notice(localFile(readme)))
    fail('Missing README notice');
  if (!exactVersion.hasMatch(recipe['terminal_version'] as String) ||
      recipe['terminal_version'] == recipe['source_version']) {
    fail('Invalid terminal version');
  }
  final output = StringBuffer();
  section = '';
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    final header = sectionName(line);
    if (header.isNotEmpty) {
      section = header;
      if (section == 'workspace' || section.startsWith('workspace.')) {
        fail(
          'Source contains a workspace; staging requires an independent crate',
        );
      }
      if (dependencyTable(section)) {
        final body = <String>[];
        var j = i + 1;
        while (j < lines.length && sectionName(lines[j]).isEmpty) {
          body.add(lines[j++]);
        }
        if (body.any(
          (entry) => RegExp(r'\b(workspace|git)\s*=').hasMatch(entry),
        )) {
          fail('Inherited/git dependency is unsupported');
        }
        final paths = body
            .where((l) => RegExp(r'^\s*path\s*=').hasMatch(l))
            .toList();
        if (paths.isNotEmpty) {
          if (paths.length != 1 ||
              scalar.firstMatch(paths.single.trim()) == null)
            fail('Ambiguous path dependency');
          final versions = body
              .where((l) => RegExp(r'^\s*version\s*=').hasMatch(l))
              .toList();
          if (versions.length != 1)
            fail('Unversioned path dependency: $section');
          final v = scalar.firstMatch(versions.single.trim());
          if (v == null) fail('Unsupported dependency version');
          output.write(line);
          for (final entry in body) {
            if (entry == paths.single) continue;
            output.write(
              entry == versions.single ? 'version = "${pin(v[3]!)}"\n' : entry,
            );
          }
          i = j - 1;
          continue;
        }
      }
    }
    if (section == 'package') {
      if (RegExp(r'^\s*publish\s*=').hasMatch(line)) {
        if (!RegExp(r'^\s*publish\s*=\s*false\s*(?:#.*)?$')
            .hasMatch(line.trim()))
          fail('Unexpected publish restriction');
        continue;
      }
      final m = scalar.firstMatch(line.trim());
      if (m?[1] == 'version')
        line = 'version = "${recipe['terminal_version']}"\n';
      if (m?[1] == 'license-file') {
        // Materialize external licenses inside the standalone package.
        localFile(m![3]!);
        if (m[3]!.split('/').contains('..')) {
          line = 'license-file = "${m[3]!.split('/').last}"\n';
        }
      }
    }
    if (dependencySection(section)) {
      if (RegExp(r'\b(workspace|git)\s*=').hasMatch(line))
        fail('Inherited/git dependency is unsupported');
      final paths = pathField.allMatches(line).toList();
      if (paths.isNotEmpty) {
        final versions = versionField.allMatches(line).toList();
        if (paths.length != 1 || versions.length != 1 || !line.contains('{'))
          fail('Unversioned or unsupported path dependency');
        final p = paths.single;
        var before = line.substring(0, p.start);
        var after = line.substring(p.end);
        if (RegExp(r'^\s*,').hasMatch(after)) {
          after = after.replaceFirst(RegExp(r'^\s*,'), '');
        } else if (RegExp(r',\s*$').hasMatch(before)) {
          before = before.replaceFirst(RegExp(r',\s*$'), '');
        } else {
          fail('Unsupported inline path dependency');
        }
        line = '$before$after';
        final v = versionField.firstMatch(line)!;
        line = line.replaceRange(v.start, v.end, 'version = "${pin(v[2]!)}"');
      }
      if (RegExp(r'\bpath\s*=').hasMatch(line))
        fail('Unprocessed dependency path');
    }
    output.write(line);
  }
  return output.toString();
}

void main(List<String> args) {
  try {
    var revision = 'HEAD';
    String? requestedOutput;
    for (var i = 0; i < args.length; i++) {
      if (args[i] == '--help') {
        print(
          'stage_retired_crates.dart [--revision REV] [--output /tmp/NEW_DIRECTORY]\nStages committed main sources only. Does not publish or verify registry availability.',
        );
        return;
      }
      if (i + 1 >= args.length) fail('Missing argument value');
      switch (args[i]) {
        case '--revision':
          revision = args[++i];
        case '--output':
          requestedOutput = args[++i];
        default:
          fail('Unknown option: ${args[i]}');
      }
    }
    final repo = run('git', ['rev-parse', '--show-toplevel']).trim();
    String git(List<String> arguments) => run('git', arguments, cwd: repo);
    if (git(['branch', '--show-current']).trim() != 'main')
      fail('Run from main');
    final sha = git([
      'rev-parse',
      '--verify',
      '--end-of-options',
      '$revision^{commit}',
    ]).trim();
    git(['merge-base', '--is-ancestor', sha, 'refs/heads/main']);
    const recipePath = 'rust/retired-packages.json';
    final recipeText = git(['show', '$sha:$recipePath']);
    final recipe = jsonDecode(recipeText) as Map<String, dynamic>;
    if (recipe['schema'] != 1) fail('Unsupported retirement recipe');
    final entries = (recipe['packages'] as List).cast<Map<String, dynamic>>();
    if (entries.isEmpty) fail('Empty retirement recipe');
    final names = <String>{};
    final root = requestedOutput == null
        ? Directory('/tmp').createTempSync('revault-retirement-')
        : Directory(requestedOutput).absolute;
    if (requestedOutput != null) {
      final parent = root.parent.resolveSymbolicLinksSync();
      if (parent != '/tmp' ||
          FileSystemEntity.typeSync(root.path) != FileSystemEntityType.notFound)
        fail('Output must be a new direct child of /tmp');
      root.createSync();
    }
    final evidence = <Map<String, dynamic>>[];
    for (final entry in entries) {
      final name = entry['package'] as String;
      final source = entry['source'] as String;
      if (!RegExp(r'^[a-zA-Z0-9_-]+$').hasMatch(name) || !names.add(name))
        fail('Invalid/duplicate package');
      if (!RegExp(r'^rust/[a-zA-Z0-9_/-]+$').hasMatch(source) ||
          source.split('/').contains('..'))
        fail('Invalid source');
      final destination = Directory('${root.path}/$name');
      final extracted = Directory.systemTemp.createTempSync(
        'revault-retirement-source-',
      );
      try {
        final tar = '${extracted.path}/source.tar';
        git(['archive', '--format=tar', '--output=$tar', sha, '--', source]);
        run('tar', ['-xf', tar, '-C', extracted.path]);
        run('cp', ['-a', '${extracted.path}/$source', destination.path]);
        String localFile(String relative) {
          if (relative.startsWith('/')) fail('Absolute package file path');
          final uri = Uri(path: '/$source/').resolve(relative);
          final committed = uri.path.substring(1);
          if (uri.hasScheme || uri.hasAuthority || committed.isEmpty)
            fail('Invalid package file');
          final content = git(['show', '$sha:$committed']);
          if (relative.split('/').contains('..')) {
            final filename = relative.split('/').last;
            if (filename.isEmpty ||
                File('${destination.path}/$filename').existsSync())
              fail('External file collision');
            File('${destination.path}/$filename').writeAsStringSync(content);
          }
          return content;
        }

        final manifest = File('${destination.path}/Cargo.toml');
        if (FileSystemEntity.typeSync(manifest.path, followLinks: false) !=
            FileSystemEntityType.file) {
          fail('Package manifest must be a regular committed file');
        }
        final original = git(['show', '$sha:$source/Cargo.toml']);
        manifest.writeAsStringSync(normalize(original, entry, localFile));
        evidence.add({
          ...entry,
          'source_manifest_blob': git(['rev-parse', '$sha:$source/Cargo.toml'])
              .trim(),
          'staged_manifest_sha256': run('sha256sum', [
            manifest.path,
          ]).split(' ').first,
          'staged_directory': destination.path,
        });
      } finally {
        extracted.deleteSync(recursive: true);
      }
    }
    File('${root.path}/retirement-provenance.json').writeAsStringSync(
      '${const JsonEncoder.withIndent('  ').convert({
        'schema': 1,
        'source_commit': sha,
        'recipe_path': recipePath,
        'recipe_blob': git(['rev-parse', '$sha:$recipePath']).trim(),
        'registry_verified': false,
        'published': false,
        'packages': evidence,
      })}\n',
    );
    print(
      green(
        'Staged ${entries.length} retirement crates from $sha at ${root.path}',
      ),
    );
    print(
      'No packages published. Verify replacement availability and package artifacts before publication.',
    );
  } catch (error) {
    stderr.writeln('Retirement staging refused: $error');
    exitCode = 1;
  }
}
