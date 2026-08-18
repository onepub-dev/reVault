import 'contact_key_pair.dart';

/// Hybrid X25519 and ML-KEM-768 private recipient identity.
///
/// This is the recipient-neutral name for the cryptographic type historically
/// exposed as `ContactKeyPair`. Human contacts, phones, recovery identities,
/// and unattended CI sources can all own one.
typedef RecipientKeyPair = ContactKeyPair;
