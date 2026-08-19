import 'dart:io';

const _targets = <String, String>{
  'linux-x86_64-gnu': 'librevault_api.so',
  'linux-aarch64-gnu': 'librevault_api.so',
  'macos-x86_64': 'librevault_api.dylib',
  'macos-aarch64': 'librevault_api.dylib',
  'windows-x86_64-msvc': 'revault_api.dll',
  'windows-aarch64-msvc': 'revault_api.dll',
};

/// Copies the six prebuilt Revault carriers into the Dart publication tree.
///
/// The native-assets hook only declares the files. It cannot build or download
/// them, so this tool must run before `dart pub publish` or `pub_release`.
void main(List<String> arguments) {
  final options = _Options.parse(arguments);
  if (options.help) {
    _printUsage();
    return;
  }

  final source = _findSource(options.source);
  final destination = Directory.fromUri(
    Directory.current.uri.resolve('lib/src/native/'),
  );
  destination.createSync(recursive: true);

  final missing = <String>[];
  for (final entry in _targets.entries) {
    final sourceFile = File.fromUri(
      source.uri.resolve('${entry.key}/${entry.value}'),
    );
    if (!sourceFile.existsSync()) {
      missing.add(sourceFile.path);
      continue;
    }
    if (options.dryRun) {
      stdout.writeln('Would stage ${entry.key}/${entry.value}');
      continue;
    }
    final destinationDirectory = Directory.fromUri(
      destination.uri.resolve('${entry.key}/'),
    )..createSync(recursive: true);
    sourceFile.copySync(
      File.fromUri(destinationDirectory.uri.resolve(entry.value)).path,
    );
    stdout.writeln('Staged ${entry.key}/${entry.value}');
  }

  if (missing.isNotEmpty) {
    stderr.writeln('Native carrier staging failed; missing:');
    for (final path in missing) {
      stderr.writeln('  $path');
    }
    stderr.writeln(
      'Provide --source or REVAULT_DART_NATIVE_SOURCE pointing to a '
      'directory containing all six target directories.',
    );
    exitCode = 1;
  }
}

Directory _findSource(String? configured) {
  final candidates = <Directory>[];
  if (configured != null) {
    candidates.add(Directory(configured));
  }
  final environment = Platform.environment['REVAULT_DART_NATIVE_SOURCE'];
  if (environment != null && environment.isNotEmpty) {
    candidates.add(Directory(environment));
  }
  candidates.addAll([
    Directory.fromUri(
      Directory.current.uri.resolve('../../packages/dart/lib/src/native/'),
    ),
    Directory.fromUri(
      Directory.current.uri.resolve('../../ecosystem/dart/lib/src/native/'),
    ),
    Directory('packages/dart/lib/src/native'),
    Directory('ecosystem/dart/lib/src/native'),
    Directory('lib/src/native'),
  ]);

  for (final candidate in candidates) {
    if (_hasAllCarriers(candidate)) return candidate;
  }
  throw StateError(
    'No complete Revault native layout was found. Checked: '
    '${candidates.map((candidate) => candidate.path).join(', ')}',
  );
}

bool _hasAllCarriers(Directory source) => _targets.entries.every(
  (entry) => File.fromUri(
    source.uri.resolve('${entry.key}/${entry.value}'),
  ).existsSync(),
);

void _printUsage() {
  stdout.writeln('''
Stage Revault native carriers for Dart publication.

Usage:
  dart tool/stage_native_assets.dart [--source <native-layout>]

pub_release invokes the pre-release hook with:
  stage_native_assets.dart [--dry-run] <version>

The source directory must contain:
  <target>/librevault_api.so|librevault_api.dylib|revault_api.dll

The destination is lib/src/native in the current package.
REVAULT_DART_NATIVE_SOURCE may be used instead of --source.
''');
}

final class _Options {
  const _Options({this.source, this.help = false, this.dryRun = false});

  final String? source;
  final bool help;
  final bool dryRun;

  static _Options parse(List<String> arguments) {
    String? source;
    var help = false;
    var dryRun = false;
    for (var index = 0; index < arguments.length; index++) {
      switch (arguments[index]) {
        case '--help' || '-h':
          help = true;
        case '--dry-run':
          dryRun = true;
        case '--source':
          if (++index >= arguments.length) {
            throw FormatException('--source requires a directory');
          }
          source = arguments[index];
        default:
          // pub_release passes the new package version as a positional
          // argument. The staging layout is selected independently.
          if (!arguments[index].startsWith('-') &&
              RegExp(r'^\d+\.\d+\.\d+').hasMatch(arguments[index])) {
            continue;
          }
          throw FormatException('Unknown argument: ${arguments[index]}');
      }
    }
    return _Options(source: source, help: help, dryRun: dryRun);
  }
}
