import 'dart:typed_data';

/// Mobile platform hosting an enrolled approval device.
enum ApprovalDevicePlatform {
  /// Apple iOS.
  ios,

  /// Google Android.
  android,
}

/// Lifecycle state retained for device and source audit records.
enum ApprovalEnrollmentState {
  /// The enrollment may authorize requests.
  active,

  /// The enrollment has been administratively revoked.
  revoked,
}

/// Whether a source requires a phone or holds an unattended CI recipient key.
enum ApprovalSourceMode {
  /// Each request requires an interactive phone approval.
  approvalRequired,

  /// CI holds its recipient key without requiring the phone.
  unattended,
}

/// Operation an enrolled source may request.
enum ApprovalAction {
  /// Open a Lockbox for read-only access.
  unlockRead,
}

/// Provider identity constraints verified for each approval request.
sealed class ApprovalSourceIdentity {
  const ApprovalSourceIdentity();

  /// Decodes the provider-specific identity policy returned by Rust.
  factory ApprovalSourceIdentity.fromJson(Map<String, Object?> json) {
    return switch (json['provider']) {
      'local_desktop' => LocalDesktopIdentity(
        requestSigningPublicKey: _bytes(json['request_signing_public_key']),
      ),
      'github_actions' => GitHubActionsIdentity(
        issuer: json['issuer']! as String,
        audiences: _strings(json['audiences']),
        repositoryIds: _strings(json['repository_ids']),
        workflowRefs: _strings(json['workflow_refs']),
        refs: _strings(json['refs']),
        environments: _strings(json['environments']),
      ),
      'gitlab_ci' => GitLabCiIdentity(
        issuer: json['issuer']! as String,
        audiences: _strings(json['audiences']),
        projectIds: _strings(json['project_ids']),
        refs: _strings(json['refs']),
        environments: _strings(json['environments']),
      ),
      'generic_oidc' => GenericOidcIdentity(
        issuer: json['issuer']! as String,
        audiences: _strings(json['audiences']),
        claimRules: (json['claim_rules']! as Map<String, Object?>).map(
          (claim, values) => MapEntry(claim, _strings(values)),
        ),
      ),
      final provider => throw FormatException(
        'Unknown approval source provider: $provider',
      ),
    };
  }

  /// Encodes the policy for storage by the Rust Vault API.
  Map<String, Object?> toJson();
}

/// Identity of an enrolled desktop request signer.
final class LocalDesktopIdentity extends ApprovalSourceIdentity {
  /// Creates a desktop identity from its platform-protected signing public key.
  const LocalDesktopIdentity({required this.requestSigningPublicKey});

  /// Public key authenticating the enrolled machine, not an individual process.
  final Uint8List requestSigningPublicKey;

  @override
  Map<String, Object?> toJson() => {
    'provider': 'local_desktop',
    'request_signing_public_key': requestSigningPublicKey.toList(),
  };
}

/// GitHub Actions OIDC workload policy.
final class GitHubActionsIdentity extends ApprovalSourceIdentity {
  /// Creates a policy using stable repository IDs and allowed workflow context.
  const GitHubActionsIdentity({
    required this.issuer,
    required this.audiences,
    required this.repositoryIds,
    required this.workflowRefs,
    required this.refs,
    required this.environments,
  });

  /// Required token issuer.
  final String issuer;

  /// Allowed audiences.
  final List<String> audiences;

  /// Stable GitHub repository IDs.
  final List<String> repositoryIds;

  /// Allowed reusable or repository workflow references.
  final List<String> workflowRefs;

  /// Allowed Git refs.
  final List<String> refs;

  /// Allowed deployment environments.
  final List<String> environments;

  @override
  Map<String, Object?> toJson() => {
    'provider': 'github_actions',
    'issuer': issuer,
    'audiences': audiences,
    'repository_ids': repositoryIds,
    'workflow_refs': workflowRefs,
    'refs': refs,
    'environments': environments,
  };
}

/// GitLab CI OIDC workload policy.
final class GitLabCiIdentity extends ApprovalSourceIdentity {
  /// Creates a policy using stable project IDs and allowed job context.
  const GitLabCiIdentity({
    required this.issuer,
    required this.audiences,
    required this.projectIds,
    required this.refs,
    required this.environments,
  });

  /// Required token issuer.
  final String issuer;

  /// Allowed audiences.
  final List<String> audiences;

  /// Stable GitLab project IDs.
  final List<String> projectIds;

  /// Allowed Git refs.
  final List<String> refs;

  /// Allowed deployment environments.
  final List<String> environments;

  @override
  Map<String, Object?> toJson() => {
    'provider': 'gitlab_ci',
    'issuer': issuer,
    'audiences': audiences,
    'project_ids': projectIds,
    'refs': refs,
    'environments': environments,
  };
}

/// Generic OIDC workload policy for providers without a built-in adapter.
final class GenericOidcIdentity extends ApprovalSourceIdentity {
  /// Creates a generic issuer, audience, and exact-claim policy.
  const GenericOidcIdentity({
    required this.issuer,
    required this.audiences,
    required this.claimRules,
  });

  /// Required token issuer.
  final String issuer;

  /// Allowed audiences.
  final List<String> audiences;

  /// Allowed exact values keyed by claim name.
  final Map<String, List<String>> claimRules;

  @override
  Map<String, Object?> toJson() => {
    'provider': 'generic_oidc',
    'issuer': issuer,
    'audiences': audiences,
    'claim_rules': claimRules,
  };
}

/// Public enrollment metadata for one approval phone.
final class ApprovalDevice {
  /// Creates a device record received from an authenticated pairing transcript.
  const ApprovalDevice({
    required this.id,
    required this.name,
    required this.recipientPublicKey,
    required this.transportPublicKey,
    required this.responseVerificationKey,
    required this.mailboxId,
    required this.platform,
    required this.capabilities,
    required this.state,
    required this.createdAtUnixMs,
    this.revokedAtUnixMs,
  });

  /// Decodes a device record returned by Rust.
  factory ApprovalDevice.fromJson(Map<String, Object?> json) => ApprovalDevice(
    id: _bytes(json['id']),
    name: json['name']! as String,
    recipientPublicKey: _bytes(json['recipient_public_key']),
    transportPublicKey: _bytes(json['transport_public_key']),
    responseVerificationKey: _bytes(json['response_verification_key']),
    mailboxId: _bytes(json['mailbox_id']),
    platform: ApprovalDevicePlatform.values.byName(json['platform']! as String),
    capabilities: _strings(json['capabilities']),
    state: ApprovalEnrollmentState.values.byName(json['state']! as String),
    createdAtUnixMs: json['created_at_unix_ms']! as int,
    revokedAtUnixMs: json['revoked_at_unix_ms'] as int?,
  );

  /// Stable 16-byte identifier.
  final Uint8List id;

  /// User-assigned prompt name.
  final String name;

  /// Generic hybrid recipient public key record.
  final Uint8List recipientPublicKey;

  /// Approval-envelope transport public key record.
  final Uint8List transportPublicKey;

  /// Approval-response verification public key record.
  final Uint8List responseVerificationKey;

  /// Opaque 32-byte relay mailbox identifier.
  final Uint8List mailboxId;

  /// Phone platform.
  final ApprovalDevicePlatform platform;

  /// Versioned phone capabilities.
  final List<String> capabilities;

  /// Enrollment lifecycle state.
  final ApprovalEnrollmentState state;

  /// Creation time in Unix milliseconds.
  final int createdAtUnixMs;

  /// Revocation time in Unix milliseconds.
  final int? revokedAtUnixMs;

  /// Encodes this record for [Vault.storeApprovalDevice].
  Map<String, Object?> toJson() => {
    'version': 1,
    'id': id.toList(),
    'name': name,
    'recipient_public_key': recipientPublicKey.toList(),
    'transport_public_key': transportPublicKey.toList(),
    'response_verification_key': responseVerificationKey.toList(),
    'mailbox_id': mailboxId.toList(),
    'platform': platform.name,
    'capabilities': capabilities,
    'state': state.name,
    'created_at_unix_ms': createdAtUnixMs,
    'revoked_at_unix_ms': revokedAtUnixMs,
  };
}

/// Policy and identity for a local or CI approval source.
final class ApprovalSource {
  /// Creates a source policy after provider selection or unattended enrollment.
  const ApprovalSource({
    required this.id,
    required this.name,
    required this.mode,
    required this.identity,
    required this.allowedLockboxIds,
    required this.allowedActions,
    required this.state,
    required this.createdAtUnixMs,
    this.unattendedRecipientPublicKey,
    this.revokedAtUnixMs,
  });

  /// Decodes a source policy returned by Rust.
  factory ApprovalSource.fromJson(Map<String, Object?> json) => ApprovalSource(
    id: _bytes(json['id']),
    name: json['name']! as String,
    mode: switch (json['mode']) {
      'approval_required' => ApprovalSourceMode.approvalRequired,
      'unattended' => ApprovalSourceMode.unattended,
      final value => throw FormatException('Unknown approval mode: $value'),
    },
    identity: ApprovalSourceIdentity.fromJson(
      json['identity']! as Map<String, Object?>,
    ),
    allowedLockboxIds: (json['allowed_lockboxes']! as List<Object?>)
        .map(_bytes)
        .toList(growable: false),
    allowedActions: (json['allowed_actions']! as List<Object?>)
        .map(
          (value) => switch (value) {
            'unlock_read' => ApprovalAction.unlockRead,
            _ => throw FormatException('Unknown approval action: $value'),
          },
        )
        .toList(growable: false),
    unattendedRecipientPublicKey:
        json['unattended_recipient_public_key'] == null
        ? null
        : _bytes(json['unattended_recipient_public_key']),
    state: ApprovalEnrollmentState.values.byName(json['state']! as String),
    createdAtUnixMs: json['created_at_unix_ms']! as int,
    revokedAtUnixMs: json['revoked_at_unix_ms'] as int?,
  );

  /// Stable 16-byte identifier.
  final Uint8List id;

  /// User-assigned name displayed on the phone.
  final String name;

  /// Interactive or unattended policy.
  final ApprovalSourceMode mode;

  /// Provider-specific cryptographic identity policy.
  final ApprovalSourceIdentity identity;

  /// Stable 16-byte lockbox IDs this source may request.
  final List<Uint8List> allowedLockboxIds;

  /// Allowed operations.
  final List<ApprovalAction> allowedActions;

  /// Public key whose private half remains in CI for unattended operation.
  final Uint8List? unattendedRecipientPublicKey;

  /// Enrollment lifecycle state.
  final ApprovalEnrollmentState state;

  /// Creation time in Unix milliseconds.
  final int createdAtUnixMs;

  /// Revocation time in Unix milliseconds.
  final int? revokedAtUnixMs;

  /// Encodes this record for Vault source administration methods.
  Map<String, Object?> toJson() => {
    'version': 1,
    'id': id.toList(),
    'name': name,
    'mode': switch (mode) {
      ApprovalSourceMode.approvalRequired => 'approval_required',
      ApprovalSourceMode.unattended => 'unattended',
    },
    'identity': identity.toJson(),
    'allowed_lockboxes': allowedLockboxIds.map((id) => id.toList()).toList(),
    'allowed_actions': allowedActions
        .map(
          (action) => switch (action) {
            ApprovalAction.unlockRead => 'unlock_read',
          },
        )
        .toList(),
    'unattended_recipient_public_key': unattendedRecipientPublicKey?.toList(),
    'state': state.name,
    'created_at_unix_ms': createdAtUnixMs,
    'revoked_at_unix_ms': revokedAtUnixMs,
  };
}

Uint8List _bytes(Object? value) => Uint8List.fromList(
  (value! as List<Object?>).map((byte) => byte! as int).toList(growable: false),
);

List<String> _strings(Object? value) =>
    (value! as List<Object?>).cast<String>().toList(growable: false);
