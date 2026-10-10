#! /usr/bin/env dcli

import 'dart:convert';
import 'dart:io';

import 'package:dcli/dcli.dart' as dcli;

const shellNames = <String>['bash', 'zsh', 'fish', 'powershell', 'elvish'];

// These tokens cover names allowed by the alias grammar and shell-safe tokens
// the product validator must reject. This driver checks argv preservation only.
const aliases = <({String name, String value, String policy})>[
  (name: 'ascii', value: 'profile_1', policy: 'valid'),
  (name: 'composed', value: 'café', policy: 'valid'),
  (name: 'decomposed', value: 'cafe\u0301', policy: 'valid'),
  (name: 'devanagari_mark', value: 'क़', policy: 'valid'),
  (name: 'hebrew', value: 'שלום', policy: 'valid'),
  (name: 'arabic_mark', value: 'مُستخدم', policy: 'valid'),
  (name: 'cjk', value: '日本語', policy: 'valid'),
  (name: 'cyrillic', value: 'проект', policy: 'valid'),
  (name: 'ascii_numeric_start', value: '123', policy: 'valid'),
  (name: 'arabic_numeric_start', value: '١٢٣', policy: 'valid'),
  (name: 'fullwidth_numeric_start', value: '１２３', policy: 'valid'),
  (name: 'letter_number', value: 'Ⅻ', policy: 'valid'),
  (name: 'other_number', value: '²', policy: 'valid'),
  (name: 'titlecase_letter', value: 'ǅ', policy: 'valid'),
  (name: 'modifier_letter', value: 'ʰ', policy: 'valid'),
  (name: 'spacing_mark', value: 'का', policy: 'valid'),
  (name: 'enclosing_mark', value: 'a\u20dd', policy: 'valid'),
  (name: 'underscore_start', value: '_name', policy: 'valid'),
  (name: 'dot_start', value: '.hidden', policy: 'invalid'),
  (name: 'trailing_dot', value: 'name.', policy: 'valid'),
  (name: 'internal_punctuation', value: 'foo_bar-2.v1', policy: 'valid'),
  (name: 'leading_mark', value: '\u0301leading', policy: 'invalid'),
  (name: 'mark_after_underscore', value: '_\u0301mark', policy: 'invalid'),
  (name: 'mark_after_dot', value: 'a.\u0301mark', policy: 'invalid'),
  (name: 'legacy_leading_hyphen', value: '-legacy', policy: 'legacy_read_only'),
];

Future<void> main() async {
  final requested =
      (Platform.environment['REVAULT_UNICODE_SHELLS'] ?? shellNames.join(','))
          .split(',')
          .map((name) => name.trim().toLowerCase())
          .where((name) => name.isNotEmpty)
          .toList();
  final unknown =
      requested.where((name) => !shellNames.contains(name)).toList();
  if (unknown.isNotEmpty) {
    stderr.writeln(
        'Unknown shells in REVAULT_UNICODE_SHELLS: ${unknown.join(', ')}');
    exitCode = 2;
    return;
  }

  final reportPath = Platform.environment['REVAULT_UNICODE_ARGV_REPORT'] ??
      'target/unicode-shell-argv-results.json';
  final tempHome =
      Directory.systemTemp.createTempSync('revault-unicode-shell-');
  final results = <Map<String, Object?>>[];
  try {
    for (final shell in requested) {
      results.add(await checkShell(shell, tempHome.path));
    }
  } finally {
    tempHome.deleteSync(recursive: true);
  }

  final report = <String, Object?>{
    'driver': 'unicode_shell_argv.dart',
    'platform': Platform.operatingSystem,
    'platformVersion': Platform.operatingSystemVersion,
    'locale': Platform.localeName,
    'environmentLocale': Platform.environment['LC_ALL'] ??
        Platform.environment['LC_CTYPE'] ??
        Platform.environment['LANG'],
    'requiredShells': requested,
    'argvShape': 'unquoted NAME, --alias NAME, and a@NAME tokens',
    'shellSpecificSyntax': <String, String>{
      'U+200D':
          'Elvish rejects ZERO WIDTH JOINER while parsing command source; product alias validation rejects it before shell parsing.',
    },
    'cases': aliases
        .map((entry) => <String, Object?>{
              'name': entry.name,
              'alias': entry.value,
              'policyExpectation': entry.policy,
              'codePoints': entry.value.runes
                  .map((rune) =>
                      'U+${rune.toRadixString(16).toUpperCase().padLeft(4, '0')}')
                  .toList(),
            })
        .toList(),
    'shellResults': results,
  };

  final file = File(reportPath);
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(
      '${const JsonEncoder.withIndent('  ').convert(report)}\n');
  dcli.echo('Wrote Unicode shell argv report to ${file.absolute.path}');
  stdout.writeln();

  final failures = results.where((result) => result['ok'] != true).toList();
  if (failures.isNotEmpty) {
    for (final failure in failures) {
      stderr.writeln(
          '${failure['shell']}: ${failure['error'] ?? 'argument mismatch'}');
    }
    exitCode = 1;
  }
}

Future<Map<String, Object?>> checkShell(String shell, String tempHome) async {
  final overrideName = 'REVAULT_UNICODE_${shell.toUpperCase()}';
  final executable = Platform.environment[overrideName] ?? shell;
  final expected = <String>[];
  for (final entry in aliases) {
    expected.addAll(
        <String>[entry.value, '--alias', entry.value, 'a@${entry.value}']);
  }

  final script = switch (shell) {
    'bash' => "set -f; printf '%s\\0' ${expected.join(' ')}",
    'zsh' => "setopt noglob; printf '%s\\0' ${expected.join(' ')}",
    'fish' => "printf '%s\\0' ${expected.join(' ')}",
    'elvish' => 'e:printf "%s\\\\0" ${expected.join(' ')}',
    'powershell' => _powerShellScript(expected),
    _ => throw StateError('Unrecognized shell $shell'),
  };

  final versionArgs = switch (shell) {
    'bash' || 'zsh' || 'fish' => <String>['--version'],
    'elvish' => <String>['-version'],
    'powershell' => <String>[
        '-NoLogo',
        '-NoProfile',
        '-Command',
        r'$PSVersionTable.PSVersion.ToString()',
      ],
    _ => const <String>[],
  };
  final commandArgs = switch (shell) {
    'bash' => <String>['--noprofile', '--norc', '-c', script],
    'zsh' => <String>['-f', '-c', script],
    'fish' => <String>['--no-config', '-c', script],
    'elvish' => <String>['-norc', '-c', script],
    'powershell' => <String>['-NoLogo', '-NoProfile', '-Command', script],
    _ => const <String>[],
  };

  try {
    final versionResult = await Process.run(
      executable,
      versionArgs,
      runInShell: false,
      stdoutEncoding: null,
      stderrEncoding: utf8,
      environment: _environment(tempHome),
    );
    final version = utf8
        .decode(versionResult.stdout as List<int>, allowMalformed: true)
        .trim()
        .split('\n')
        .first;
    final result = await Process.run(
      executable,
      commandArgs,
      runInShell: false,
      stdoutEncoding: null,
      stderrEncoding: utf8,
      environment: _environment(tempHome),
    );
    final outputBytes = result.stdout as List<int>;
    final actual = shell == 'powershell'
        ? _decodePowerShellArgs(outputBytes)
        : _decodeNulSeparatedArgs(outputBytes);
    final matched = _sameArgs(expected, actual);
    return <String, Object?>{
      'shell': shell,
      'executable': executable,
      'version': version,
      'exitCode': result.exitCode,
      'ok': result.exitCode == 0 && matched,
      'expectedArgs': expected,
      'actualArgs': actual,
      'expectedUtf8Hex': expected.map(_utf8Hex).toList(),
      'actualUtf8Hex': actual.map(_utf8Hex).toList(),
      if (result.exitCode != 0) 'stderr': result.stderr,
      if (!matched)
        'error': 'The parsed argv did not match the expected Unicode tokens.',
    };
  } on ProcessException catch (error) {
    return <String, Object?>{
      'shell': shell,
      'executable': executable,
      'version': null,
      'ok': false,
      'expectedArgs': expected,
      'actualArgs': <String>[],
      'expectedUtf8Hex': expected.map(_utf8Hex).toList(),
      'actualUtf8Hex': <String>[],
      'error': error.message,
    };
  } on FormatException catch (error) {
    return <String, Object?>{
      'shell': shell,
      'executable': executable,
      'version': null,
      'ok': false,
      'expectedArgs': expected,
      'actualArgs': <String>[],
      'expectedUtf8Hex': expected.map(_utf8Hex).toList(),
      'actualUtf8Hex': <String>[],
      'error': error.message,
    };
  }
}

String _powerShellScript(List<String> expected) =>
    r'''function argvProbe {
  $encoded = @($args | ForEach-Object {
    [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes([string]$_))
  })
  [Console]::Out.WriteLine((ConvertTo-Json -Compress -InputObject $encoded))
}
argvProbe ''' +
    expected.join(' ');

Map<String, String> _environment(String tempHome) => <String, String>{
      ...Platform.environment,
      if (!Platform.isWindows) 'HOME': tempHome,
    };

List<String> _decodeNulSeparatedArgs(List<int> output) {
  if (output.isEmpty || output.last != 0) {
    throw const FormatException(
        'Shell argv recorder output had no final NUL byte.');
  }
  final strings = <String>[];
  var start = 0;
  for (var index = 0; index < output.length; index++) {
    if (output[index] == 0) {
      strings.add(utf8.decode(output.sublist(start, index)));
      start = index + 1;
    }
  }
  return strings;
}

List<String> _decodePowerShellArgs(List<int> output) {
  final encoded = jsonDecode(utf8.decode(output)) as List<dynamic>;
  return encoded
      .cast<String>()
      .map((value) => utf8.decode(base64.decode(value)))
      .toList();
}

bool _sameArgs(List<String> expected, List<String> actual) {
  if (expected.length != actual.length) return false;
  for (var index = 0; index < expected.length; index++) {
    if (expected[index] != actual[index]) return false;
  }
  return true;
}

String _utf8Hex(String value) => utf8
    .encode(value)
    .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
    .join();
