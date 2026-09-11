import 'dart:io';
import 'dart:typed_data';

import 'package:revault_api/revault_api.dart';

Future<void> main(List<String> arguments) async {
  final runtime = await Revault.load(
    nativeLibraryPath: arguments.isEmpty ? null : arguments.single,
  );
  final root = Directory.systemTemp.createTempSync('revault-override-');
  final key = SecretBytes.copyOf(Uint8List(32)..fillRange(0, 32, 75));
  final signing = runtime.generateProfileSigningKeyPair();
  final payload = Uint8List.fromList([0, 255, 128, 10, 64]);
  final path = '${root.path}/test.lbox';
  try {
    // Creating a lockbox resolves the native finalizer address.
    final box = Lockbox.create(path, contentKey: key, signingKey: signing);
    try {
      box.addFile('/payload', payload);
      box.commit();
    } finally {
      box.close();
    }
    final reopened = Lockbox.open(path, contentKey: key);
    try {
      final actual = reopened.getFile('/payload');
      if (actual.length != payload.length ||
          List.generate(
            payload.length,
            (i) => actual[i] == payload[i],
          ).contains(false)) {
        throw StateError('Persisted content differs');
      }
    } finally {
      reopened.close();
    }
    // An absent vault must produce a translated native error, not an FFI
    // resolution error. An invalid bus avoids accessing real credentials.
    try {
      runtime.operations.vaultPlatformGetPasswordFor(
        '${root.path}/absent-vault',
        'unix:path=${root.path}/absent-bus',
      );
      throw StateError('Expected absent vault credential lookup to fail');
    } on RevaultException {
      // Reaching the Rust error confirms the extension resolved successfully.
    }
  } finally {
    key.close();
    root.deleteSync(recursive: true);
  }
}
