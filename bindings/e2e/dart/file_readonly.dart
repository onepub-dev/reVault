import 'dart:io';
import 'dart:typed_data';
import 'package:revault_api/revault_api.dart';

// Standalone executable for testing the public facade on a real read-only mount.
Future<void> main() async {
  await Revault.load();
  final key = SecretBytes.copyOf(Uint8List.fromList(List.filled(32, 75)));
  final path = Platform.environment['REVAULT_READONLY_ARCHIVE']!;
  final box = Lockbox.open(path, contentKey: key);
  try {
    if (String.fromCharCodes(box.getFile('/hello')) != 'read only') {
      throw StateError('read-only content mismatch');
    }
  } finally {
    box.close();
    key.close();
  }
  stdout.writeln('PASS\tdart\tlockbox_file_readonly_mount\t1');
}
