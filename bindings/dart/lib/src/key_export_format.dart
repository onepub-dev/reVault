/// Portable representations supported by contact-key export operations.
///
/// Example:
/// ```dart
/// final pem = contact.export(KeyExportFormat.lockboxPem);
/// await File('alice-public.pem').writeAsBytes(pem);
/// ```
enum KeyExportFormat {
  /// Lockbox's armored PEM representation, recommended for files and sharing.
  lockboxPem._('lockbox-pem'),

  /// A JSON Web Key object using the Lockbox ML-KEM-1024 profile.
  jwk._('jwk'),

  /// A JSON Web Key Set containing the exported key.
  jwks._('jwks'),

  /// Unarmored lowercase hexadecimal key material.
  rawHex._('raw-hex');

  const KeyExportFormat._(this.nativeName);

  /// @nodoc
  final String nativeName;
}
