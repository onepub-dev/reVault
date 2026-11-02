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
      print('Android acceptance: loading native library');
      final runtime = await Revault.load();
      print('Android acceptance: native library loaded');
      final root = Directory.systemTemp.createTempSync('revault-android-');
      final password = SecretString.fromString('Android lockbox fixture');
      final passphrase = SecretString.fromString('Android vault fixture');
      final payload = Uint8List.fromList([0, 1, 128, 255, 0, 42]);
      final path = '${root.path}/archive.lbox';
      final vaultPath = '${root.path}/vault';
      print('Android acceptance: generating signer');
      final signer = runtime.generateProfileSigningKeyPair();
      print('Android acceptance: signer generated');
      try {
        print('Android acceptance: creating Vault');
        var vault = Vault.openOrCreate(
          pathTo: vaultPath,
          passphrase: passphrase,
        );
        try {
          print('Android acceptance: creating Lockbox');
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
        print('Android acceptance: reopening persisted Vault');
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
        print('Android acceptance: releasing native credentials');
        signer.dispose();
        password.close();
        passphrase.close();
        root.deleteSync(recursive: true);
      }
    },
  );
}
