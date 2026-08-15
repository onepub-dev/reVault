import 'dart:typed_data';

import 'package:revault_api/revault_api.dart';

/// Creates an in-memory lockbox and stores public and secret values in it.
Future<void> main() async {
  await Revault.load();
  final contentKey = SecretBytes.random(32);
  final lockbox = Lockbox.createInMemory(contentKey: contentKey);

  try {
    lockbox.addFile(
      '/hello.txt',
      Uint8List.fromList('hello\n'.codeUnits),
      replace: false,
    );
    lockbox.setVariable('owner', 'alice');
    final token = SecretBytes.fromString('secret');
    try {
      lockbox.setSecretVariable('token', token);
    } finally {
      token.close();
    }
    final tokenLength = lockbox.withSecretVariable(
      'token',
      (token) => token.length,
    );
    print('Stored a $tokenLength-byte secret.');
    lockbox.commit();
  } finally {
    lockbox.close();
    contentKey.close();
  }
}
