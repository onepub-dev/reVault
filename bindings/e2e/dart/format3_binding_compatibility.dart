#! /usr/bin/env dcli

import 'dart:io';
import 'dart:typed_data';

import 'package:dcli/dcli.dart';
import 'package:revault_api/revault_api.dart';

// Run this same harness in a project pinned to the published revault_api 0.4.1
// package and in bindings/e2e/dart with the restored native carrier. Pass the
// carrier path and restored CLI path explicitly; neither may be substituted
// silently. The harness prints evidence only after all assertions succeed.
Future<void> main(List<String> args) async {
  if (args.length < 2 || args.length > 3) {
    throw ArgumentError(
      'Usage: ./format3_binding_compatibility.dart NATIVE_LIBRARY CLI [ALIAS_CAPABLE_CLI]',
    );
  }
  final library = File(args[0]).absolute.path;
  final cli = File(args[1]).absolute.path;
  final aliasCli = args.length == 3 ? File(args[2]).absolute.path : cli;
  final api = await Revault.load(nativeLibraryPath: library);
  check(api.lockboxFormatVersion == 3, 'carrier must write format 3');
  check(
    api.currentVaultStructureVersion == 3,
    'carrier must write Vault structure 3',
  );
  final root = Directory.systemTemp.createTempSync('revault-format3-binding-');
  final environment =
      <String, String>{
          ...Platform.environment,
          'LOCKBOX_VAULT_DIR': '${root.path}/vault',
          'LOCKBOX_SESSION_AGENT_DIR': '${root.path}/agent',
          'LOCKBOX_PLATFORM_SECRET_STORE': 'disabled',
          'LOCKBOX_VAULT_PASSWORD': 'synthetic-binding-vault-password',
          'LOCKBOX_KEY': 'synthetic-binding-content-key',
        }
        ..remove('LOCKBOX_PASSWORD')
        ..remove('COMPLETE');
  List<int> run(String executable, List<String> arguments) {
    final result = Process.runSync(
      executable,
      arguments,
      workingDirectory: root.path,
      environment: environment,
      includeParentEnvironment: false,
      stdoutEncoding: null,
      stderrEncoding: null,
    );
    check(result.exitCode == 0, 'CLI $arguments failed: ${result.stderr}');
    return result.stdout as List<int>;
  }

  List<int> command(List<String> arguments) => run(cli, arguments);
  List<int> aliasCommand(List<String> arguments) => run(aliasCli, arguments);
  final passphrase = SecretString.fromString(
    'synthetic-binding-vault-password',
  );
  final key = SecretBytes.fromString('synthetic-binding-content-key');
  try {
    command(['vault', 'init']);
    for (final creator in ['cli', 'dart']) {
      final name = '$creator.lbox';
      final path = '${root.path}/$name';
      final original = Uint8List.fromList(
        List.generate(131073, (i) => (i * 37) % 251),
      );
      final replacement = Uint8List.fromList(
        List.generate(262149, (i) => (i * 11 + 7) % 256),
      );
      final vault = Vault.open(
        pathTo: '${root.path}/vault',
        passphrase: passphrase,
      );
      final signer = vault.loadProfileSigningKey('default');
      vault.close();
      try {
        if (creator == 'cli') {
          command([name, 'create']);
          File('${root.path}/source').writeAsBytesSync(original);
          command([name, 'add', 'source', '--to', '/payload']);
          command([name, 'variable', 'set', 'VALUE', 'original']);
        } else {
          final box = Lockbox.create(path, contentKey: key, signingKey: signer);
          try {
            box.addFile('/payload', original);
            box.setVariable('VALUE', 'original');
            box.commit();
          } finally {
            box.close();
          }
        }
        var box = Lockbox.open(path, contentKey: key, signingKey: signer);
        try {
          check(
            equal(box.getFile('/payload'), original),
            'binding reads creator payload',
          );
          check(
            box.getVariable('VALUE') == 'original',
            'binding reads creator variable',
          );
          box.setVariable('BINDING', 'persisted');
          box.commit();
        } finally {
          box.close();
        }
        check(
          equal(command([name, 'cat', '/payload']), original),
          'CLI reads creator payload',
        );
        check(
          String.fromCharCodes(command([name, 'variable', 'get', 'BINDING'])) ==
              'persisted\n',
          'CLI reads binding write',
        );
        File('${root.path}/source').writeAsBytesSync(replacement);
        command([name, 'add', '--overwrite', 'source', '--to', '/payload']);
        command([name, 'variable', 'set', 'VALUE', 'replacement']);
        box = Lockbox.open(path, contentKey: key, signingKey: signer);
        try {
          check(
            equal(box.getFile('/payload'), replacement),
            'binding reads CLI replacement',
          );
          check(
            box.getVariable('VALUE') == 'replacement',
            'binding reads CLI updated variable',
          );
          box.addFile('/binding-added', original);
          box.commit();
        } finally {
          box.close();
        }
        check(
          equal(command([name, 'cat', '/binding-added']), original),
          'CLI reads binding addition',
        );
        aliasCommand(['vault', 'lockbox', 'alias', 'set', creator, name]);
        final oldVault = Vault.open(
          pathTo: '${root.path}/vault',
          passphrase: passphrase,
        );
        try {
          oldVault.storeProfileEmail(
            'default',
            'compatibility@example.invalid',
          );
        } finally {
          oldVault.close();
        }
        check(
          equal(aliasCommand(['a@$creator', 'cat', '/payload']), replacement),
          'alias survives binding Vault write',
        );
      } finally {
        signer.dispose();
      }
    }
    print(
      green(
        'PASS format3 binding/CLI mutual creation, reads, updates and Vault alias preservation; carrier=$library; CLI=$cli; aliasCLI=$aliasCli',
      ),
    );
  } finally {
    key.close();
    passphrase.close();
    root.deleteSync(recursive: true);
  }
}

void check(bool condition, String message) {
  if (!condition) throw StateError(message);
}

bool equal(List<int> left, List<int> right) {
  if (left.length != right.length) return false;
  for (var i = 0; i < left.length; i++) {
    if (left[i] != right[i]) return false;
  }
  return true;
}
