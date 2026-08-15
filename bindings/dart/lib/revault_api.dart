/// Encrypt files, variables, and typed form records in portable reVault
/// lockboxes, and manage keys and local vault metadata.
///
/// Start by awaiting [Revault.load], then open a persistent [Vault] or create or
/// open a [Lockbox]. Ordinary lockboxes retain their content key only in the
/// current process. [AgentSession] is an explicit optional service for
/// time-limited cross-process access.
///
/// Native vault and lockbox resources implement `close`; key handles implement
/// `dispose`. Secret variables and secret form fields are exposed only to
/// callback-scoped accessors so plaintext is not retained accidentally.
///
/// See the [repository README](https://github.com/onepub-dev/reVault#readme)
/// for installation, the security model, and complete examples.
library;

export 'src/domain_models.dart' hide DomainDecoders;
export 'src/exceptions.dart';
export 'src/agent_activity.dart' show AgentActivity;
export 'src/agent_activity_kind.dart' show AgentActivityKind;
export 'src/agent_session.dart' show AgentSession;
export 'src/contact_key_pair.dart' show ContactKeyPair;
export 'src/contact_public_key.dart' show ContactPublicKey;
export 'src/key_export_format.dart' show KeyExportFormat;
export 'src/lockbox.dart' show Lockbox;
export 'src/lockbox_cache_mode.dart' show LockboxCacheMode;
export 'src/lockbox_options.dart' show LockboxOptions;
export 'src/lockbox_worker.dart' show LockboxWorker;
export 'src/lockbox_workload.dart' show LockboxWorkload;
export 'src/profile_signing_key_pair.dart' show ProfileSigningKeyPair;
export 'src/profile_signing_public_key.dart' show ProfileSigningPublicKey;
export 'src/read_only_vault.dart' show ReadOnlyVault;
export 'src/revault.dart' show Revault;
export 'src/secret_bytes.dart' show SecretBytes;
export 'src/secret_string.dart' show SecretString;
export 'src/vault.dart' show Vault;
export 'src/wrapped_contact_key.dart' show WrappedContactKey;
