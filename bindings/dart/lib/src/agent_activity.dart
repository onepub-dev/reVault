import 'dart:ffi' as ffi;
import 'package:revault_api/src/owned.dart';

/// A lifetime token for an operation that currently needs cached secrets.
///
/// Keep the token returned by the agent activity API alive for the duration of
/// the operation, then call [dispose] so the Session Agent can expire secrets
/// when no other activity needs them.
///
/// Example:
/// ```dart
/// final activity = AgentSession.instance.beginActivity(AgentActivityKind.form);
/// try {
///   editForm();
/// } finally {
///   activity.dispose();
/// }
/// ```
final class AgentActivity extends Owned {
  /// @nodoc
  AgentActivity.internal(super.runtime, super.handle);

  /// Ends the registered activity; repeated calls have no effect.
  ///
  /// Example:
  /// ```dart
  /// final activity = agent.beginActivity(AgentActivityKind.open);
  /// try {
  ///   openSelectedLockbox();
  /// } finally {
  ///   activity.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.vaultAgentEndActivity(handle);
      handle = ffi.nullptr;
    }
  }
}
