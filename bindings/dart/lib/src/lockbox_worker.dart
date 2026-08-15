/// Worker policies controlling where native Lockbox work executes.
///
/// Example:
/// ```dart
/// lockbox.setWorkerPolicy(LockboxWorker.threads, jobs: 4);
/// ```
enum LockboxWorker {
  /// Let the native runtime choose an implementation.
  auto._('auto'),

  /// Perform work on the calling thread.
  single._('single'),

  /// Use native worker threads.
  threads._('threads');

  const LockboxWorker._(this.nativeName);

  /// @nodoc
  final String nativeName;
}
