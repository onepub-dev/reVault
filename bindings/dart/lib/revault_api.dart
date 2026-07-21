/// Encrypt files, variables, and typed form records in portable reVault
/// lockboxes, and manage keys and local vault metadata.
///
/// Start with [Vault.load], then create or open a [Lockbox]. Native resources
/// implement explicit `dispose` methods; secret variables and secret form
/// fields are exposed only to callback-scoped accessors so plaintext is not
/// retained accidentally.
///
/// See the [repository README](https://github.com/onepub-dev/reVault#readme)
/// for installation, the security model, and complete examples.
library;

export 'src/domain_models.dart' hide DomainDecoders;
export 'vault.dart';
