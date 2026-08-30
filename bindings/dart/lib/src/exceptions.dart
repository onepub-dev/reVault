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
        'No Vault passphrase is remembered; supply passphrase to Vault.open().',
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

/// The remembered Vault passphrase could not be read from the platform
/// credential store.
///
/// This differs from [VaultPassphraseUnavailableException]: a credential may
/// exist, but the current process cannot retrieve it. Common causes include
/// running as a user who cannot access the platform credential store containing
/// the Vault passphrase. Callers may prompt for the Vault passphrase and retry
/// with `Vault.open(passphrase: suppliedPassphrase)`.
///
/// Example:
/// ```dart
/// try {
///   return Vault.open();
/// } on VaultPassphraseAccessException catch (error) {
///   log(error.toString());
///   return Vault.open(passphrase: await promptForVaultPassphrase());
/// }
/// ```
final class VaultPassphraseAccessException extends RevaultException {
  /// Creates an actionable exception from the platform credential store
  /// [cause].
  ///
  /// The Dart facade constructs this when `Vault.open()` cannot retrieve a
  /// remembered passphrase. Application code normally catches this exception
  /// rather than constructing it.
  ///
  /// Example:
  /// ```dart
  /// final error = VaultPassphraseAccessException(
  ///   const RevaultException('Permission denied'),
  /// );
  /// log(error.toString());
  /// ```
  VaultPassphraseAccessException(this.cause)
    : super(
        'The remembered Vault passphrase could not be read from the '
        'platform credential store.',
        details: ErrorDetails(
          category: 'platform_credential_store',
          message: cause.message,
          guidance:
              'Run the application as the user with access to the '
              'platform credential store containing the Vault passphrase, or '
              'supply the Vault passphrase to '
              'Vault.open(passphrase: ...).',
        ),
      );

  /// The original native platform credential store failure.
  final RevaultException cause;

  @override
  /// Formats the access failure, its native cause, and recovery guidance.
  ///
  /// Example:
  /// ```dart
  /// log(error.toString());
  /// ```
  String toString() {
    return 'VaultPassphraseAccessException: $message '
        'Native cause: ${cause.message}. $guidance';
  }
}

/// The requested Lockbox does not currently have a key in the Session Agent.
///
/// This represents an ordinary cache miss. Agent transport, permission, and
/// protocol failures continue to surface as [RevaultException].
///
/// Example:
/// ```dart
/// try {
///   return AgentSession.instance.acquireOpenLockbox(path);
/// } on AgentLockboxNotOpenException {
///   return Lockbox.open(path);
/// }
/// ```
final class AgentLockboxNotOpenException implements Exception {
  /// Creates a cache-miss exception for [path].
  const AgentLockboxNotOpenException(this.path);

  /// The Lockbox path that was not open in the agent.
  final String path;

  @override
  String toString() =>
      'AgentLockboxNotOpenException: The lockbox is not open in the session '
      'agent: $path';
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
