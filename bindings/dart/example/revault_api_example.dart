import 'dart:typed_data';

import 'package:revault_api/revault_api.dart';

/// Creates an in-memory lockbox and stores public and secret values in it.
Future<void> main() async {
  final vault = await Vault.load();
  final lockbox = vault.createLockbox(Uint8List(32));

  try {
    lockbox.addFile(
      '/hello.txt',
      Uint8List.fromList('hello\n'.codeUnits),
      replace: false,
    );
    lockbox.setVariable('owner', 'alice');
    lockbox.setSecretVariable(
      'token',
      Uint8List.fromList('secret'.codeUnits),
    );
    final tokenLength = lockbox.withSecretVariable(
      'token',
      (token) => token.length,
    );
    print('Stored a $tokenLength-byte secret.');
    lockbox.commit();
  } finally {
    lockbox.dispose();
  }
}
