use crate::{Error, Message, SignedUnlockRequest, UnlockRequest, SCOPE, SIGNING_DOMAIN, VERSION};
use ed25519_dalek::{Signature, VerifyingKey};

/// Reject noncanonical origins (including paths, userinfo and opaque origins).
pub fn validate_origin(origin: &str) -> Result<(), Error> {
    let parsed = url::Url::parse(origin).map_err(|_| Error::InvalidRequest)?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || parsed.origin().ascii_serialization() != origin
    {
        return Err(Error::InvalidRequest);
    }
    Ok(())
}
/// Identifiers are bounded printable ASCII without UI control characters.
pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:@".contains(&b))
}
/// Validate structural and temporal limits before signature verification or UI.
pub fn validate_request(request: &UnlockRequest, now: u64) -> Result<(), Error> {
    if request.protocol_version != VERSION {
        return Err(Error::UpgradeRequired);
    }
    validate_origin(&request.origin)?;
    if ![
        &request.request_id,
        &request.application_id,
        &request.recipient_key_id,
        &request.lockbox_id,
    ]
    .into_iter()
    .all(|s| identifier(s))
        || request.server_boot_id.len() != 32
        || request.challenge.len() != 32
        || request.recipient_public_key.len() != 32
        || request.requested_scope != SCOPE
    {
        return Err(Error::InvalidRequest);
    }
    if request.issued_at > now
        || request.expires_at <= now
        || request.expires_at <= request.issued_at
        || request.expires_at - request.issued_at > 120
    {
        return Err(Error::Expired);
    }
    Ok(())
}
/// Canonical signing bytes; unknown protobuf fields do not acquire semantics.
pub fn signing_bytes(request: &UnlockRequest) -> Vec<u8> {
    let mut bytes = SIGNING_DOMAIN.to_vec();
    bytes.extend(request.encode_to_vec());
    bytes
}
/// Verify against the key pinned locally at pairing, never a key in the request.
pub fn verify_signed<'a>(
    signed: &'a SignedUnlockRequest,
    pinned: &[u8; 32],
    now: u64,
) -> Result<&'a UnlockRequest, Error> {
    let request = signed.request.as_ref().ok_or(Error::InvalidRequest)?;
    validate_request(request, now)?;
    let key = VerifyingKey::from_bytes(pinned).map_err(|_| Error::UnpairedRecipient)?;
    let signature = Signature::from_slice(&signed.signature).map_err(|_| Error::InvalidRequest)?;
    key.verify_strict(&signing_bytes(request), &signature)
        .map_err(|_| Error::UnpairedRecipient)?;
    Ok(request)
}
