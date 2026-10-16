import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:revault_api/revault_api.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'bundled Android carrier persists lockboxes and vault credentials',
    (tester) async {
      final runtime = await Revault.load();
      final root = Directory.systemTemp.createTempSync('revault-android-');
      final password = SecretString.fromString('Android lockbox fixture');
      final passphrase = SecretString.fromString('Android vault fixture');
      final payload = Uint8List.fromList([0, 1, 128, 255, 0, 42]);
      final path = '${root.path}/archive.lbox';
      final vaultPath = '${root.path}/vault';
      final signer = runtime.generateProfileSigningKeyPair();
      try {
        var vault = Vault.openOrCreate(
          pathTo: vaultPath,
          passphrase: passphrase,
        );
        try {
          final box = Lockbox.create(path, password: password);
          try {
            box.addFile('/résumé', payload);
            box.commit();
            box.commit();
            vault.rememberPassword(box.id, password);
          } finally {
            box.close();
          }
        } finally {
          vault.close();
        }
        vault = Vault.open(pathTo: vaultPath, passphrase: passphrase);
        try {
          var box = Lockbox.open(path, vault: vault, writable: true);
          try {
            expect(box.getFile('/résumé'), payload);
            box.addFile('/added', payload);
            box.addFile('/résumé', Uint8List.fromList([7, 8]), replace: true);
            box.commit();
          } finally {
            box.close();
          }
          box = Lockbox.open(path, password: password);
          try {
            expect(box.getFile('/résumé'), [7, 8]);
            expect(box.getFile('/added'), payload);
            expect(() => box.commit(), throwsA(isA<RevaultException>()));
          } finally {
            box.close();
          }
          box = Lockbox.open(path, password: password, writable: true);
          try {
            box.delete('/résumé');
            box.commit();
          } finally {
            box.close();
          }
          box = Lockbox.open(path, password: password);
          try {
            expect(box.exists('/résumé'), false);
            expect(box.getFile('/added'), payload);
          } finally {
            box.close();
          }
        } finally {
          vault.close();
        }
      } finally {
        signer.dispose();
        password.close();
        passphrase.close();
        root.deleteSync(recursive: true);
      }
    },
  );
}
