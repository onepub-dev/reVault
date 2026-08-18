use revault_lockbox_api::{Error, LockboxId, RecipientPublicKey, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

const ADMIN_RECORD_VERSION: u16 = 1;
const MAX_ADMIN_RECORD_BYTES: usize = 64 * 1024;

/// Stable identifier for an enrolled approval device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeviceId([u8; 16]);

impl DeviceId {
    /// Generates a cryptographically random device identifier.
    pub fn new_random() -> Result<Self> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self(bytes))
    }

    /// Creates an identifier from its stable byte representation.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the stable byte representation.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&crate::encode_hex(&self.0))
    }
}

/// Stable identifier for an enrolled local or CI approval source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ApprovalSourceId([u8; 16]);

impl ApprovalSourceId {
    /// Generates a cryptographically random source identifier.
    pub fn new_random() -> Result<Self> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self(bytes))
    }

    /// Creates an identifier from its stable byte representation.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the stable byte representation.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for ApprovalSourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&crate::encode_hex(&self.0))
    }
}

/// Mobile platform hosting an enrolled approval device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevicePlatform {
    /// Apple iOS.
    Ios,
    /// Google Android.
    Android,
}

/// Lifecycle state of an enrolled device or source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentState {
    /// The enrollment may authorize requests.
    Active,
    /// The enrollment has been revoked and must not authorize requests.
    Revoked,
}

/// Device metadata stored inside the encrypted local vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDevice {
    /// Stable device identifier.
    pub id: DeviceId,
    /// User-assigned name displayed during administration and approval.
    pub name: String,
    /// Recipient key used for generic lockbox access slots.
    pub recipient_public_key: RecipientPublicKey,
    /// Public key used to encrypt approval requests to this device.
    pub transport_public_key: Vec<u8>,
    /// Public key used to verify approval responses from this device.
    pub response_verification_key: Vec<u8>,
    /// Opaque relay mailbox identifier; this is not a push token.
    pub mailbox_id: [u8; 32],
    /// Mobile platform hosting the enrollment.
    pub platform: DevicePlatform,
    /// Versioned capability names advertised by the phone application.
    pub capabilities: Vec<String>,
    /// Current lifecycle state.
    pub state: EnrollmentState,
    /// Enrollment creation time in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Revocation time in Unix milliseconds, when revoked.
    pub revoked_at_unix_ms: Option<u64>,
}

impl StoredDevice {
    /// Encodes this public enrollment record as versioned JSON for language bindings and QR exchange.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        encode_device(self)
    }

    /// Decodes a versioned public enrollment record.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        decode_device(bytes)
    }
}

/// Source policy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalSourceMode {
    /// Every unlock requires an interactive phone approval.
    ApprovalRequired,
    /// A recipient private key held by the CI provider may unlock unattended.
    Unattended,
}

/// Action a source may request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalAction {
    /// Open a lockbox for read-only secret access.
    UnlockRead,
}

/// Provider-specific, verified identity policy for an approval source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum ApprovalSourceIdentity {
    /// An enrolled desktop. The signing key authenticates the machine, not an
    /// individual same-user process.
    LocalDesktop {
        /// Public key used to authenticate approval request envelopes.
        request_signing_public_key: Vec<u8>,
    },
    /// GitHub Actions workload identity constraints.
    #[serde(rename = "github_actions")]
    GitHubActions {
        /// Required OIDC issuer.
        issuer: String,
        /// Allowed token audiences.
        audiences: Vec<String>,
        /// Stable GitHub repository identifiers.
        repository_ids: Vec<String>,
        /// Allowed workflow references.
        workflow_refs: Vec<String>,
        /// Allowed ref patterns.
        refs: Vec<String>,
        /// Allowed deployment environments.
        environments: Vec<String>,
    },
    /// GitLab CI workload identity constraints.
    #[serde(rename = "gitlab_ci")]
    GitLabCi {
        /// Required OIDC issuer.
        issuer: String,
        /// Allowed token audiences.
        audiences: Vec<String>,
        /// Stable GitLab project identifiers.
        project_ids: Vec<String>,
        /// Allowed ref patterns.
        refs: Vec<String>,
        /// Allowed deployment environments.
        environments: Vec<String>,
    },
    /// Generic OIDC workload identity constraints.
    GenericOidc {
        /// Required OIDC issuer.
        issuer: String,
        /// Allowed token audiences.
        audiences: Vec<String>,
        /// Required values for provider-specific claims.
        claim_rules: BTreeMap<String, Vec<String>>,
    },
}

/// Local or CI source allowed to request lockbox access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredApprovalSource {
    /// Stable source identifier.
    pub id: ApprovalSourceId,
    /// User-assigned name shown prominently on approval prompts.
    pub name: String,
    /// Interactive or unattended policy.
    pub mode: ApprovalSourceMode,
    /// Cryptographically verified source identity policy.
    pub identity: ApprovalSourceIdentity,
    /// Lockboxes this source may request. An empty list authorizes none.
    pub allowed_lockboxes: Vec<LockboxId>,
    /// Operations this source may request.
    pub allowed_actions: Vec<ApprovalAction>,
    /// Public recipient key generated and retained by unattended CI, if any.
    pub unattended_recipient_public_key: Option<RecipientPublicKey>,
    /// Current lifecycle state.
    pub state: EnrollmentState,
    /// Enrollment creation time in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Revocation time in Unix milliseconds, when revoked.
    pub revoked_at_unix_ms: Option<u64>,
}

impl StoredApprovalSource {
    /// Encodes this source policy as versioned JSON for language bindings.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        encode_source(self)
    }

    /// Decodes a versioned source policy record.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        decode_source(bytes)
    }
}

#[derive(Serialize, Deserialize)]
struct DeviceRecord {
    version: u16,
    id: DeviceId,
    name: String,
    recipient_public_key: Vec<u8>,
    transport_public_key: Vec<u8>,
    response_verification_key: Vec<u8>,
    mailbox_id: [u8; 32],
    platform: DevicePlatform,
    capabilities: Vec<String>,
    state: EnrollmentState,
    created_at_unix_ms: u64,
    revoked_at_unix_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SourceRecord {
    version: u16,
    id: ApprovalSourceId,
    name: String,
    mode: ApprovalSourceMode,
    identity: ApprovalSourceIdentity,
    allowed_lockboxes: Vec<[u8; 16]>,
    allowed_actions: Vec<ApprovalAction>,
    unattended_recipient_public_key: Option<Vec<u8>>,
    state: EnrollmentState,
    created_at_unix_ms: u64,
    revoked_at_unix_ms: Option<u64>,
}

pub(crate) fn encode_device(value: &StoredDevice) -> Result<Vec<u8>> {
    encode_json(&DeviceRecord {
        version: ADMIN_RECORD_VERSION,
        id: value.id,
        name: value.name.clone(),
        recipient_public_key: value.recipient_public_key.to_bytes(),
        transport_public_key: value.transport_public_key.clone(),
        response_verification_key: value.response_verification_key.clone(),
        mailbox_id: value.mailbox_id,
        platform: value.platform,
        capabilities: value.capabilities.clone(),
        state: value.state,
        created_at_unix_ms: value.created_at_unix_ms,
        revoked_at_unix_ms: value.revoked_at_unix_ms,
    })
}

pub(crate) fn decode_device(bytes: &[u8]) -> Result<StoredDevice> {
    let value: DeviceRecord = decode_json(bytes)?;
    if value.version != ADMIN_RECORD_VERSION {
        return Err(record_error("unsupported device record version"));
    }
    Ok(StoredDevice {
        id: value.id,
        name: value.name,
        recipient_public_key: RecipientPublicKey::from_bytes(&value.recipient_public_key)?,
        transport_public_key: value.transport_public_key,
        response_verification_key: value.response_verification_key,
        mailbox_id: value.mailbox_id,
        platform: value.platform,
        capabilities: value.capabilities,
        state: value.state,
        created_at_unix_ms: value.created_at_unix_ms,
        revoked_at_unix_ms: value.revoked_at_unix_ms,
    })
}

pub(crate) fn encode_source(value: &StoredApprovalSource) -> Result<Vec<u8>> {
    encode_json(&SourceRecord {
        version: ADMIN_RECORD_VERSION,
        id: value.id,
        name: value.name.clone(),
        mode: value.mode,
        identity: value.identity.clone(),
        allowed_lockboxes: value
            .allowed_lockboxes
            .iter()
            .map(|id| *id.as_bytes())
            .collect(),
        allowed_actions: value.allowed_actions.clone(),
        unattended_recipient_public_key: value
            .unattended_recipient_public_key
            .as_ref()
            .map(RecipientPublicKey::to_bytes),
        state: value.state,
        created_at_unix_ms: value.created_at_unix_ms,
        revoked_at_unix_ms: value.revoked_at_unix_ms,
    })
}

pub(crate) fn decode_source(bytes: &[u8]) -> Result<StoredApprovalSource> {
    let value: SourceRecord = decode_json(bytes)?;
    if value.version != ADMIN_RECORD_VERSION {
        return Err(record_error("unsupported approval source record version"));
    }
    let unattended_recipient_public_key = value
        .unattended_recipient_public_key
        .map(|bytes| RecipientPublicKey::from_bytes(&bytes))
        .transpose()?;
    Ok(StoredApprovalSource {
        id: value.id,
        name: value.name,
        mode: value.mode,
        identity: value.identity,
        allowed_lockboxes: value
            .allowed_lockboxes
            .into_iter()
            .map(LockboxId::from_bytes)
            .collect(),
        allowed_actions: value.allowed_actions,
        unattended_recipient_public_key,
        state: value.state,
        created_at_unix_ms: value.created_at_unix_ms,
        revoked_at_unix_ms: value.revoked_at_unix_ms,
    })
}

fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| record_error(format!("record encoding failed: {error}")))?;
    if bytes.len() > MAX_ADMIN_RECORD_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "approval administration record exceeds 64 KiB".to_string(),
        ));
    }
    Ok(bytes)
}

fn decode_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_ADMIN_RECORD_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "approval administration record exceeds 64 KiB".to_string(),
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| record_error(format!("record decoding failed: {error}")))
}

fn record_error(message: impl Into<String>) -> Error {
    Error::CorruptVaultRecord(message.into())
}
