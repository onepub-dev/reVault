import 'dart:io';
import 'dart:typed_data';
import 'package:revault_api/revault_api.dart';
import 'package:test/test.dart';

void main() {
  late Revault runtime;
  setUpAll(() async {
    runtime = await Revault.load();
  });

  test('native file persistence, reader locks and atomic replacement', () {
    final root = Directory.systemTemp.createTempSync('revault-dart-');
    final path = '${root.path}/résumé.lbox';
    final key = SecretBytes.copyOf(Uint8List.fromList(List.filled(32, 75)));
    final signing = runtime.generateProfileSigningKeyPair();
    final payload = Uint8List.fromList([0, 255, 128, 10, 0, 64]);
    Lockbox open({bool write = false}) =>
        Lockbox.open(path, contentKey: key, signingKey: write ? signing : null);
    try {
      var box = Lockbox.create(path, contentKey: key, signingKey: signing);
      try {
        box.addFile('/hello', payload);
        box.commit();
        box.commit();
      } finally {
        box.close();
      }
      final reader = open();
      try {
        box = open();
        try {
          expect(box.getFile('/hello'), payload);
        } finally {
          box.close();
        }
        expect(
          () => reader.setOwnerSigningKey(signing),
          throwsA(isA<RevaultException>()),
        );
        expect(
          () => reader.addFile('/bad', payload),
          throwsA(isA<RevaultException>()),
        );
        expect(() => open(write: true), throwsA(isA<RevaultException>()));
        expect(
          () => Lockbox.create(
            path,
            contentKey: key,
            signingKey: signing,
            overwrite: true,
          ),
          throwsA(isA<RevaultException>()),
        );
      } finally {
        reader.close();
      }
      box = open(write: true);
      try {
        box.addFile('/hello', Uint8List.fromList([1, 2, 3]), replace: true);
        box.addFile('/added', payload);
        box.commit();
      } finally {
        box.close();
      }
      box = open();
      try {
        expect(box.getFile('/hello'), [1, 2, 3]);
        expect(box.getFile('/added'), payload);
      } finally {
        box.close();
      }
      box = open(write: true);
      try {
        box.delete('/hello');
        box.commit();
      } finally {
        box.close();
      }
      box = open();
      try {
        expect(box.exists('/hello'), false);
        expect(box.getFile('/added'), payload);
      } finally {
        box.close();
      }
      expect(
        () => Lockbox.create(path, contentKey: key),
        throwsA(isA<RevaultException>()),
      );
      box = Lockbox.create(
        path,
        contentKey: key,
        signingKey: signing,
        overwrite: true,
      );
      try {
        box.addFile('/new', payload);
        box.commit();
      } finally {
        box.close();
      }
      box = open();
      try {
        expect(box.exists('/added'), false);
        expect(box.getFile('/new'), payload);
      } finally {
        box.close();
      }
      expect(root.listSync().map((f) => f.path), [path]);
    } finally {
      signing.dispose();
      key.close();
      root.deleteSync(recursive: true);
    }
  });

  test('password file reads and writes', () {
    final root = Directory.systemTemp.createTempSync('revault-dart-password-');
    final path = '${root.path}/archive.lbox';
    final password = SecretString.fromString('file test password');
    final signing = runtime.generateProfileSigningKeyPair();
    try {
      var box = Lockbox.create(path, password: password, signingKey: signing);
      try {
        box.addFile('/hello', Uint8List.fromList([0, 1, 255]));
        box.commit();
      } finally {
        box.close();
      }
      box = Lockbox.open(path, password: password);
      try {
        expect(box.getFile('/hello'), [0, 1, 255]);
      } finally {
        box.close();
      }
      box = Lockbox.open(path, password: password, signingKey: signing);
      try {
        box.addFile('/hello', Uint8List.fromList([5, 6]), replace: true);
        box.commit();
      } finally {
        box.close();
      }
      box = Lockbox.open(path, password: password);
      try {
        expect(box.getFile('/hello'), [5, 6]);
      } finally {
        box.close();
      }
    } finally {
      signing.dispose();
      password.close();
      root.deleteSync(recursive: true);
    }
  });

  test(
    'vault credentials are resolved by Rust and lock errors are preserved',
    () {
      final root = Directory.systemTemp.createTempSync('revault-dart-vault-');
      final path = '${root.path}/archive.lbox';
      final password = SecretString.fromString('file password');
      final passphrase = SecretString.fromString('vault passphrase');
      final signer = runtime.generateProfileSigningKeyPair();
      final vault = Vault.openOrCreate(
        pathTo: '${root.path}/vault',
        passphrase: passphrase,
      );
      try {
        var box = Lockbox.create(path, password: password, signingKey: signer);
        try {
          box.addFile('/hello', Uint8List.fromList([0, 42, 255]));
          box.commit();
          vault.rememberPassword(box.id, password);
        } finally {
          box.close();
        }
        final reader = Lockbox.open(path, vault: vault);
        try {
          expect(reader.getFile('/hello'), [0, 42, 255]);
          try {
            final unexpected = Lockbox.open(
              path,
              vault: vault,
              signingKey: signer,
            );
            unexpected.close();
            fail('writer opened while shared reader was live');
          } on RevaultException catch (error) {
            expect(error.message.toLowerCase(), contains('lock'));
          }
        } finally {
          reader.close();
        }
        box = Lockbox.open(path, vault: vault, signingKey: signer);
        try {
          box.addFile('/hello', Uint8List.fromList([9]), replace: true);
          box.commit();
        } finally {
          box.close();
        }
        box = Lockbox.open(path, vault: vault);
        try {
          expect(box.getFile('/hello'), [9]);
        } finally {
          box.close();
        }
      } finally {
        vault.close();
        signer.dispose();
        password.close();
        passphrase.close();
        root.deleteSync(recursive: true);
      }
    },
  );

  test('Rust resolves a matching profile key from the vault', () {
    final root = Directory.systemTemp.createTempSync('revault-dart-contact-');
    final path = '${root.path}/archive.lbox';
    final passphrase = SecretString.fromString('vault passphrase');
    final contact = runtime.generateContactKeyPair();
    final unrelated = runtime.generateContactKeyPair();
    final publicKey = contact.publicKey();
    final signer = runtime.generateProfileSigningKeyPair();
    final vault = Vault.openOrCreate(
      pathTo: '${root.path}/vault',
      passphrase: passphrase,
    );
    try {
      vault.storePrivateKey('aaa-unrelated', unrelated);
      vault.storePrivateKey('reader', contact);
      var box = Lockbox.create(path, contact: publicKey, signingKey: signer);
      try {
        box.addFile('/hello', Uint8List.fromList([0, 42, 255]));
        box.commit();
      } finally {
        box.close();
      }
      box = Lockbox.open(path, vault: vault);
      try {
        expect(box.getFile('/hello'), [0, 42, 255]);
      } finally {
        box.close();
      }
      box = Lockbox.open(path, vault: vault, signingKey: signer);
      try {
        box.addFile('/hello', Uint8List.fromList([7]), replace: true);
        box.commit();
      } finally {
        box.close();
      }
      box = Lockbox.open(path, vault: vault);
      try {
        expect(box.getFile('/hello'), [7]);
      } finally {
        box.close();
      }
    } finally {
      vault.close();
      publicKey.dispose();
      contact.dispose();
      unrelated.dispose();
      signer.dispose();
      passphrase.close();
      root.deleteSync(recursive: true);
    }
  });

  test('read-only bind mount', () {
    final key = SecretBytes.copyOf(Uint8List.fromList(List.filled(32, 75)));
    final box = Lockbox.open(
      Platform.environment['REVAULT_READONLY_ARCHIVE']!,
      contentKey: key,
    );
    try {
      expect(String.fromCharCodes(box.getFile('/hello')), 'read only');
    } finally {
      box.close();
      key.close();
    }
  }, skip: !Platform.environment.containsKey('REVAULT_READONLY_ARCHIVE'));
}
