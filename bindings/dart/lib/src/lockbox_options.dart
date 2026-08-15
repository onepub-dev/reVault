import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/lockbox_cache_mode.dart';
import 'package:revault_api/src/lockbox_worker.dart';
import 'package:revault_api/src/lockbox_workload.dart';

/// Memory and CPU settings applied when [Lockbox] is created or opened.
///
/// The defaults suit interactive applications. Increase the cache or select a
/// parallel worker policy for bulk operations after measuring the host's
/// available memory and CPU capacity.
///
/// Example:
/// ```dart
/// final options = LockboxOptions(
///   cacheBytes: 128 << 20,
///   workload: LockboxWorkload.bulkImport,
///   worker: LockboxWorker.threads,
///   jobs: 4,
/// );
/// final box = Lockbox.open(path, contentKey: key, options: options);
/// ```
final class LockboxOptions {
  /// Creates runtime options for one [Lockbox.create] or [Lockbox.open] call.
  ///
  /// Start with the defaults and override only settings justified by workload
  /// measurements; a larger cache retains more decrypted page data in memory.
  ///
  /// Example:
  /// ```dart
  /// const options = LockboxOptions(
  ///   cacheMode: LockboxCacheMode.disabled,
  ///   worker: LockboxWorker.single,
  /// );
  /// ```
  const LockboxOptions({
    this.cacheMode = LockboxCacheMode.bytes,
    this.cacheBytes = 64 << 20,
    this.workload = LockboxWorkload.interactive,
    this.worker = LockboxWorker.auto,
    this.jobs = 0,
  });

  /// Decoded-page cache policy.
  final LockboxCacheMode cacheMode;

  /// Maximum decoded-page cache capacity in bytes when [cacheMode] is `bytes`.
  final int cacheBytes;

  /// Expected access pattern used to tune the native runtime.
  final LockboxWorkload workload;

  /// Native worker scheduling policy.
  final LockboxWorker worker;

  /// Worker count for policies that accept it; zero requests an automatic count.
  final int jobs;
}
