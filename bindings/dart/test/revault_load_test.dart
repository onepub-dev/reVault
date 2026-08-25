import 'package:revault_api/revault_api.dart';
import 'package:test/test.dart';

void main() {
  test('rejects an empty explicit native library path', () async {
    await expectLater(
      Revault.load(nativeLibraryPath: ''),
      throwsA(isA<ArgumentError>()),
    );
  });
}
