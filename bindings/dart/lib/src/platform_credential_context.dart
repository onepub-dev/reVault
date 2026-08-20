/// Selects the operating-system credential-store session used by a Vault.
///
/// Context is passed directly to the native credential-store client. It does
/// not alter the process environment.
final class PlatformCredentialContext {
  /// Uses the Linux Secret Service available on [sessionBusAddress].
  ///
  /// This is useful after a sudo-launched process drops back to the invoking
  /// user's effective identity but still has root's environment.
  ///
  /// Example:
  /// ```dart
  /// final context = PlatformCredentialContext.linux(
  ///   sessionBusAddress: 'unix:path=/run/user/1000/bus',
  /// );
  /// ```
  const PlatformCredentialContext.linux({required this.sessionBusAddress});

  /// The explicit Linux D-Bus session address.
  final String sessionBusAddress;
}
