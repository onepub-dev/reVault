/// Workload profiles used to tune an open Lockbox.
///
/// Example:
/// ```dart
/// lockbox.setWorkloadProfile(LockboxWorkload.readMostly);
/// ```
enum LockboxWorkload {
  /// Balanced behavior for interactive applications.
  interactive._('interactive'),

  /// Optimize a large import operation.
  bulkImport._('bulk-import'),

  /// Optimize repeated reads from an established lockbox.
  readMostly._('read-mostly');

  const LockboxWorkload._(this.nativeName);

  /// @nodoc
  final String nativeName;
}
