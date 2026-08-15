import 'package:revault_api/src/lockbox_options.dart';

/// Decoded-page cache policies used by [LockboxOptions].
///
/// Example:
/// ```dart
/// const lowMemory = LockboxOptions(cacheMode: LockboxCacheMode.disabled);
/// ```
enum LockboxCacheMode {
  /// Cache decoded pages up to [LockboxOptions.cacheBytes].
  bytes._('bytes'),

  /// Do not retain decoded pages.
  disabled._('disabled'),

  /// Let the native runtime choose the cache policy.
  automatic._('automatic');

  const LockboxCacheMode._(this.nativeName);

  /// @nodoc
  final String nativeName;
}
