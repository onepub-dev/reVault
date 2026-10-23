import 'dart:io';
import 'dart:typed_data';

import 'package:revault_api/revault_api.dart';
import 'package:test/test.dart';

void main() {
  final nativeLibrary = Platform.environment['REVAULT_LIBRARY'];
  test(
    'read-only Vault loads stored credentials through the public facade',
    () async {
      final root = Directory.systemTemp.createTempSync('revault-readonly-');
      final passphrase = SecretString.fromString('test-vault-passphrase');
      final remembered = SecretString.fromString('test-lockbox-password');
      final lockboxId = Uint8List.fromList(
        List<int>.generate(16, (index) => index),
      );
      try {
        final runtime = await Revault.load(nativeLibraryPath: nativeLibrary);
        final identity = runtime.generateContactKeyPair();
        try {
          final vault = Vault.openOrCreate(
            pathTo: root.path,
            passphrase: passphrase,
          );
          vault.storePrivateKey('alice', identity);
          vault.rememberPassword(lockboxId, remembered);
          vault.close();

          final first = Vault.openReadOnly(
            pathTo: root.path,
            passphrase: passphrase,
          );
          final second = Vault.openReadOnly(
            pathTo: root.path,
            passphrase: passphrase,
          );
          try {
            final loaded = first.loadPrivateKey('alice');
            final historical = second.loadPrivateKeyGeneration('alice', 1);
            final signer = first.loadProfileSigningKey('alice');
            final historicalSigner = second.loadProfileSigningKeyGeneration(
              'alice',
              1,
            );
            final password = second.rememberedPassword(lockboxId);
            try {
              expect(
                () => first.loadProfileSigningKey('missing'),
                throwsA(
                  isA<RevaultException>().having(
                    (error) => error.category,
                    'category',
                    'not_found',
                  ),
                ),
              );
              expect(loaded.privateRecord(), identity.privateRecord());
              expect(historical.privateRecord(), identity.privateRecord());
              expect(historicalSigner.publicBytes(), signer.publicBytes());
              expect(password, isNotNull);
              password!.withBytes(
                (bytes) =>
                    expect(bytes, remembered.withBytes((value) => value)),
              );
            } finally {
              loaded.dispose();
              historical.dispose();
              signer.dispose();
              historicalSigner.dispose();
              password?.close();
            }
          } finally {
            first.close();
            second.close();
          }
        } finally {
          identity.dispose();
        }
      } finally {
        passphrase.close();
        remembered.close();
        root.deleteSync(recursive: true);
      }
    },
    skip: nativeLibrary == null ? 'REVAULT_LIBRARY is required' : false,
  );
}
