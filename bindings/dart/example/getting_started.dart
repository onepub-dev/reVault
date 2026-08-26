import 'dart:io';

import 'package:revault_api/revault_api.dart';

Future<void> main() async {
  await Revault.load();

  final workspace = Directory('.revault-example')..createSync();
  final vaultPassphrase = _promptSecret('Vault passphrase: ');
  final lockboxPassword = _promptSecret('Lockbox password: ');
  final vault = Vault.openOrCreate(
    pathTo: '${workspace.path}/vault',
    passphrase: vaultPassphrase,
  );

  try {
    final path = '${workspace.path}/team-secrets.lbox';
    final lockbox = File(path).existsSync()
        ? Lockbox.open(path, password: lockboxPassword)
        : Lockbox.create(path, password: lockboxPassword);
    try {
      lockbox.setDescription('Team deployment credentials');
      lockbox.commit();
      vault.rememberPassword(lockbox.id, lockboxPassword);
      stdout.writeln(
        'Prepared $path and remembered its password in the Vault.',
      );
    } finally {
      lockbox.close();
    }
  } finally {
    vault.close();
    lockboxPassword.close();
    vaultPassphrase.close();
  }
}

SecretString _promptSecret(String label) {
  stdout.write(label);
  final echoMode = stdin.echoMode;
  try {
    stdin.echoMode = false;
    final value = stdin.readLineSync();
    stdout.writeln();
    if (value == null || value.isEmpty) {
      throw StateError('A non-empty value is required.');
    }
    // stdin.readLineSync returns an immutable String. Applications with
    // stronger memory-handling requirements should collect owned UTF-8 bytes
    // and pass them to SecretString.takeUtf8 instead.
    return SecretString.fromString(value);
  } finally {
    stdin.echoMode = echoMode;
  }
}
