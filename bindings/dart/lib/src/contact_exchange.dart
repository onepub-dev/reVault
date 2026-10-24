import 'dart:convert';
import 'dart:ffi' as ffi;

import 'domain_models.dart';
import 'owned.dart';
import 'revault.dart';
import 'secret_string.dart';

/// Full reciprocal exchange session. Retains protected credentials, not a Vault
/// lock. Compare identities and the full fingerprint independently between calls.
final class ContactExchange extends Owned {
  ContactExchange._(super.runtime, super.handle);

  /// Open an existing Vault file for exchanges, without using the Session Agent.
  /// The caller owns [vaultPassphrase]; close it when its input is no longer needed.
  /// The session retains a protected native copy until [close].
  ///
  /// ```dart
  /// final session = ContactExchange.open(vaultFile, vaultPassphrase);
  /// try { print(session.list(offline: true).length); }
  /// finally { session.close(); }
  /// ```
  factory ContactExchange.open(String path, SecretString vaultPassphrase) {
    final runtime = Revault.runtime;
    return ContactExchange._(
      runtime,
      vaultPassphrase.withBytes(
        (bytes) => runtime.operations.contactExchangeOpen(path, bytes),
      ),
    );
  }
  List<ContactExchangeStatus> _execute(Map<String, Object> request) {
    if (disposed) throw StateError('Exchange session is closed');
    return runtime.operations.contactExchangeExecute(
      handle,
      jsonEncode(request),
    );
  }

  /// Create an invitation; relayPending means retry using list.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   final offer = session.invite(relayOrigin, 'default', 'bob@example.com');
  ///   if (offer.relayPending) session.list();
  /// } finally { session.close(); }
  /// ```
  ContactExchangeStatus invite(
    String relay,
    String profile,
    String recipientEmail, {
    int ttlHours = 24,
  }) => _execute({
    'operation': 'invite',
    'relay': relay,
    'profile': profile,
    'recipient_email': recipientEmail,
    'ttl_hours': ttlHours,
  }).single;

  /// Accept with one selected profile's encryption and signing public keys.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   final reply = session.accept(invitationUrl, 'default');
  ///   print(reply.fingerprint);
  /// } finally { session.close(); }
  /// ```
  ContactExchangeStatus accept(String invitation, String profile) => _execute({
    'operation': 'accept',
    'invitation': invitation,
    'profile': profile,
  }).single;

  /// List public status, optionally avoiding relay requests.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   for (final status in session.list(offline: true)) { print(status.id); }
  /// } finally { session.close(); }
  /// ```
  List<ContactExchangeStatus> list({bool offline = false}) =>
      _execute({'operation': 'list', 'offline': offline});

  /// Save both contact keys after independent full-fingerprint comparison.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   // After independent identity and full-fingerprint comparison:
  ///   session.verify(exchangeId, 'bob', comparedFingerprint, ComparisonChannel.inPerson);
  /// } finally { session.close(); }
  /// ```
  ContactExchangeStatus verify(
    String id,
    String contactName,
    String fingerprint,
    ComparisonChannel channel,
  ) => _execute({
    'operation': 'verify',
    'id': id,
    'contact_name': contactName,
    'fingerprint': fingerprint,
    'channel': channel.wireName,
  }).single;

  /// Authenticate an archive against the saved contact signing key.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   session.verifyAuthor('alice', receivedArchivePath);
  /// } finally { session.close(); }
  /// ```
  void verifyAuthor(String contactName, String lockboxPath) {
    _execute({
      'operation': 'verify_author',
      'contact_name': contactName,
      'lockbox_path': lockboxPath,
    });
  }

  /// Cancel an unaccepted invitation owned by this Vault.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   session.cancel(exchangeId);
  /// } finally { session.close(); }
  /// ```
  void cancel(String id) {
    _execute({'operation': 'cancel', 'id': id});
  }

  /// Forget local exchange state without deleting a verified contact.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   session.forget(exchangeId);
  /// } finally { session.close(); }
  /// ```
  void forget(String id) {
    _execute({'operation': 'forget', 'id': id});
  }

  /// Release the protected passphrase without cancelling any exchange.
  /// Example:
  /// ```
  /// final session = ContactExchange.open(vaultPath, vaultPassphrase);
  /// try {
  ///   final saved = session.list(offline: true);
  ///   print(saved.length);
  /// } finally { session.close(); }
  /// ```
  void close() {
    if (!disposed) {
      runtime.operations.contactExchangeFree(handle);
      handle = ffi.nullptr;
    }
  }
}

/// Independently authenticated channel used to compare both identities and fingerprint.
enum ComparisonChannel {
  /// Comparison with the person present.
  inPerson('in-person'),

  /// Call using an independently known number.
  knownPhoneCall('known-phone-call'),

  /// Independently authenticated messenger conversation.
  trustedMessenger('trusted-messenger');

  const ComparisonChannel(this.wireName);

  /// Protocol spelling; applications normally use the enum value.
  final String wireName;
}
