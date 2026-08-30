/// Categories used to tell the Session Agent why secrets are active.
///
/// Example:
/// ```dart
/// final activity = AgentSession.instance.beginActivity(AgentActivityKind.open);
/// try {
///   final box = AgentSession.instance.acquireOpenLockbox(path);
/// } finally {
///   activity.dispose();
/// }
/// ```
enum AgentActivityKind {
  /// Opening or unlocking a lockbox.
  open._('open'),

  /// Closing a lockbox.
  close._('close'),

  /// Reading or changing variables.
  variables._('variables'),

  /// Reading or changing form records.
  form._('form'),

  /// Recovering a damaged lockbox.
  recovery._('recovery'),

  /// Accessing the persistent vault.
  vault._('vault');

  const AgentActivityKind._(this.nativeName);

  /// @nodoc
  final String nativeName;
}
