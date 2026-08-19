import 'dart:io';

Future<void> main(List<String> arguments) async {
  final options = _options(arguments);
  final repository = Directory.current;
  final packages = Directory(options['packages']!);
  final work = Directory(options['work']!);

  final tree = Directory('${work.path}/dart-tree');
  final packageRoot = Directory('${tree.path}/bindings/dart');
  final conformanceRoot = Directory('${tree.path}/bindings/e2e/dart');
  await _copyTree(Directory('${packages.path}/dart'), packageRoot);
  await _copyTree(
    Directory('${repository.path}/bindings/e2e/dart'),
    conformanceRoot,
  );

  await _run('dart', ['pub', 'get'], conformanceRoot);

  final install = Directory('${work.path}/dart-install');
  await install.create(recursive: true);
  final executable = File(
    '${install.absolute.path}/bundle/bin/conformance${Platform.isWindows ? '.exe' : ''}',
  );

  await _run(
    'dart',
    ['build', 'cli', '--target', 'conformance.dart', '-o', install.path],
    conformanceRoot,
  );

  final artifacts = Directory('${work.path}/artifacts');
  await artifacts.create(recursive: true);
  await _run(
    executable.path,
    const [],
    conformanceRoot,
    environment: {'REVAULT_E2E_ARTIFACT_DIR': artifacts.path},
  );
}

Map<String, String> _options(List<String> arguments) {
  final options = <String, String>{};
  for (var index = 0; index < arguments.length; index += 2) {
    if (index + 1 >= arguments.length || !arguments[index].startsWith('--')) {
      throw ArgumentError('Expected --name value arguments.');
    }
    options[arguments[index].substring(2)] = arguments[index + 1];
  }
  for (final name in ['packages', 'work']) {
    if (!options.containsKey(name)) {
      throw ArgumentError('--$name is required.');
    }
  }
  return options;
}

Future<void> _copyTree(Directory source, Directory destination) async {
  if (!source.existsSync()) {
    throw StateError('Missing source directory: ${source.path}');
  }
  await for (final entity in source.list(recursive: false)) {
    final target = '${destination.path}/${_basename(entity.path)}';
    if (entity is Directory) {
      final targetDirectory = Directory(target);
      await targetDirectory.create(recursive: true);
      await _copyTree(entity, targetDirectory);
    } else if (entity is File) {
      await File(target).parent.create(recursive: true);
      await entity.copy(target);
    }
  }
}

String _basename(String path) => path.split(RegExp(r'[/\\]')).last;

Future<void> _run(
  String executable,
  List<String> arguments,
  Directory workingDirectory, {
  Map<String, String>? environment,
}) async {
  final result = await Process.run(
    executable,
    arguments,
    workingDirectory: workingDirectory.path,
    environment: environment,
    runInShell: Platform.isWindows,
  );
  stdout.write(result.stdout);
  stderr.write(result.stderr);
  if (result.exitCode != 0) {
    throw ProcessException(
        executable, arguments, 'Command failed', result.exitCode);
  }
}
