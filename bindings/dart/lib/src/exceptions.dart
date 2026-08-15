import 'domain_models.dart';

/// A failure reported by the native reVault runtime.
///
/// [category] is stable enough for programmatic handling. [message] explains
/// the failure, while [guidance] contains an actionable recovery suggestion
/// when the native runtime supplied one.
///
/// Example:
/// ```dart
/// try {
///   final vault = Vault.open();
/// } on RevaultException catch (error) {
///   log('${error.category}: ${error.message}');
///   showRecoveryAdvice(error.guidance);
/// }
/// ```
class RevaultException implements Exception {
  /// Creates an exception from a native error message and structured details.
  ///
  /// Binding internals use this constructor when converting a native failure;
  /// application code normally catches, rather than constructs, this type.
  ///
  /// Example:
  /// ```dart
  /// const error = RevaultException('Archive is unreadable');
  /// ```
  const RevaultException(this.message, {this.details});

  /// Human-readable description of the failed operation.
  final String message;

  /// Structured native details, when the failure supplied them.
  final ErrorDetails? details;

  /// Stable native error category, or `unknown` when none was supplied.
  ///
  /// Example:
  /// ```dart
  /// if (error.category == 'unsupported_version') requestUpgrade();
  /// ```
  String get category => details?.category ?? 'unknown';

  /// Suggested corrective action, or an empty string when none was supplied.
  ///
  /// Example:
  /// ```dart
  /// if (error.guidance.isNotEmpty) show(error.guidance);
  /// ```
  String get guidance => details?.guidance ?? '';

  @override
  /// Formats the category, message, and any recovery guidance for diagnostics.
  ///
  /// Example:
  /// ```dart
  /// log(error.toString());
  /// ```
  String toString() {
    final suffix = guidance.isEmpty ? '' : ' $guidance';
    return 'RevaultException($category): $message$suffix';
  }
}

/// The default vault could not be opened because no platform passphrase exists.
///
/// Catch this separately when an application wants to prompt for a Vault
/// passphrase and retry with `Vault.open(passphrase: suppliedPassphrase)`.
///
/// Example:
/// ```dart
/// try {
///   return Vault.open();
/// } on VaultPassphraseUnavailableException {
///   return Vault.open(passphrase: await promptForVaultPassphrase());
/// }
/// ```
final class VaultPassphraseUnavailableException extends RevaultException {
  /// Creates the exception.
  ///
  /// The Dart facade throws this when platform storage has no default Vault
  /// passphrase; applications rarely need to construct it directly.
  ///
  /// Example:
  /// ```dart
  /// const error = VaultPassphraseUnavailableException();
  /// ```
  const VaultPassphraseUnavailableException()
    : super(
        'No vault passphrase is remembered; supply passphrase to Vault.open().',
      );

  @override
  /// Formats this missing-platform-credential failure for diagnostics.
  ///
  /// Example:
  /// ```dart
  /// log(error.toString());
  /// ```
  String toString() => 'VaultPassphraseUnavailableException: $message';
}

/// No supplied, remembered, or profile credential could open a lockbox.
///
/// This means the caller must explicitly supply a password, Profile key, or
/// another supported access credential. It does not imply file corruption.
///
/// Example:
/// ```dart
/// try {
///   return Lockbox.open(path);
/// } on LockboxCredentialUnavailableException {
///   return Lockbox.open(path, password: await promptForLockboxPassword());
/// }
/// ```
final class LockboxCredentialUnavailableException extends RevaultException {
  /// Creates an exception for [path].
  ///
  /// Example:
  /// ```dart
  /// const error = LockboxCredentialUnavailableException('/secrets/team.lbox');
  /// ```
  const LockboxCredentialUnavailableException(this.path)
    : super('No supplied or remembered credential can open $path.');

  /// Host path of the lockbox that could not be opened.
  final String path;

  @override
  /// Formats this missing-Lockbox-credential failure for diagnostics.
  ///
  /// Example:
  /// ```dart
  /// log(error.toString());
  /// ```
  String toString() => 'LockboxCredentialUnavailableException: $message';
}
