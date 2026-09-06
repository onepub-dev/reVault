//! Generic server-side challenge lifecycle. Authentication and CSRF enforcement
//! belong to the embedding application, before either public method is called.
use crate::{
    crypto::{RecipientKey, UnlockMaterial},
    validation::signing_bytes,
    Error, SignedUnlockRequest, UnlockEnvelope, UnlockRequest, SCOPE, VERSION,
};
use rand_core::{OsRng, TryRngCore};
use std::collections::BTreeMap;

/// Public pairing configuration for an application using the receiver.
pub struct Application {
    /// Exact HTTPS origin, matching the local pairing.
    pub origin: String,
    /// Stable application identifier.
    pub application_id: String,
    /// Canonical lockbox ID, matching the local pairing.
    pub lockbox_id: String,
}
struct Challenge {
    session: [u8; 32],
    request: UnlockRequest,
}
/// One server process/lock epoch. Construct anew on every restart or lock.
/// No key is persisted and no server-side Session Agent is contacted.
pub struct Receiver {
    key: RecipientKey,
    boot: [u8; 32],
    pending: BTreeMap<String, Challenge>,
}
impl Receiver {
    /// Start locked with fresh recipient keys and boot identity.
    ///
    /// # Errors
    /// Returns `Internal` if fresh entropy or protected key allocation fails.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            key: RecipientKey::generate()?,
            boot: random()?,
            pending: BTreeMap::new(),
        })
    }
    /// Issue a two-minute single-use challenge to an authenticated session.
    /// `session` must be a server-derived, unguessable session binding, not data
    /// accepted directly from a request body. The callback signs with the pinned
    /// long-lived Ed25519 identity, which stays separate from content keys.
    ///
    /// # Errors
    /// Rejects invalid application metadata, clock overflow, and more than 1024
    /// outstanding challenges. Randomness and signing failures are propagated.
    pub fn issue(
        &mut self,
        app: &Application,
        session: [u8; 32],
        now: u64,
        sign: impl FnOnce(&[u8]) -> Result<[u8; 64], Error>,
    ) -> Result<SignedUnlockRequest, Error> {
        self.pending.retain(|_, c| c.request.expires_at > now);
        if self.pending.len() >= 1024 {
            return Err(Error::Busy);
        }
        let id = hex(&random()?);
        let request = UnlockRequest {
            protocol_version: VERSION,
            request_id: id.clone(),
            application_id: app.application_id.clone(),
            server_boot_id: self.boot.to_vec(),
            challenge: random()?.to_vec(),
            issued_at: now,
            expires_at: now.checked_add(120).ok_or(Error::Expired)?,
            recipient_key_id: hex(&self.boot),
            recipient_public_key: self.key.public_key().to_vec(),
            lockbox_id: app.lockbox_id.clone(),
            requested_scope: SCOPE.into(),
            origin: app.origin.clone(),
        };
        crate::validation::validate_request(&request, now)?;
        let signature = sign(&signing_bytes(&request))?;
        self.pending.insert(
            id,
            Challenge {
                session,
                request: request.clone(),
            },
        );
        Ok(SignedUnlockRequest {
            request: Some(request),
            signature: signature.to_vec(),
        })
    }
    /// Consume a session-bound challenge before decrypting. A bad envelope burns
    /// that challenge. Wrong-session submissions never consume another session's
    /// challenge. A restart has neither the old challenge nor recipient key.
    ///
    /// # Errors
    /// Returns `Replayed` for an unknown/consumed challenge, `Denied` for another
    /// session, and the decryption validation error for an invalid envelope.
    pub fn accept(
        &mut self,
        session: &[u8; 32],
        envelope: &UnlockEnvelope,
        now: u64,
    ) -> Result<UnlockMaterial, Error> {
        let id = &envelope
            .binding
            .as_ref()
            .ok_or(Error::InvalidRequest)?
            .request_id;
        let challenge = self.pending.get(id).ok_or(Error::Replayed)?;
        if challenge.session != *session {
            return Err(Error::Denied);
        }
        let challenge = self.pending.remove(id).ok_or(Error::Replayed)?;
        self.key.open(envelope, &challenge.request, now)
    }
}
fn random() -> Result<[u8; 32], Error> {
    let mut bytes = [0; 32];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| Error::Internal)?;
    Ok(bytes)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
