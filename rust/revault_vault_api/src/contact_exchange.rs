//! Process-local reciprocal contact exchange. No Session Agent is used.
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use revault_publish_protocol::exchange::{
    random_token, verification, Acceptance, Bundle, ExchangeClient, LocalExchange, Offer, Request,
};
use serde::{Deserialize, Serialize};

use crate::{SecretString, VaultDirectory};
use revault_lockbox_api::Error;

/// A user-mediated operation in a full reciprocal public-key exchange.
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContactExchangeCommand {
    /// Persist a signed invitation before submitting it to the relay.
    Invite {
        /// HTTPS origin (loopback HTTP is allowed for local testing).
        relay: String,
        /// One existing local profile to share.
        profile: String,
        /// Intended recipient's claimed email.
        recipient_email: String,
        /// Invitation lifetime, between one and 168 hours.
        ttl_hours: u32,
    },
    /// Accept an invitation and return both selected public keys.
    Accept {
        /// Complete invitation URL, including its fragment.
        invitation: String,
        /// One existing local profile to share.
        profile: String,
    },
    /// List saved exchanges, optionally synchronizing and retrying acknowledgements.
    List {
        /// Avoid all relay requests when true.
        offline: bool,
    },
    /// Save both contact keys after an independent full-fingerprint comparison.
    Verify {
        /// Saved exchange ID.
        id: String,
        /// Local name for the verified contact.
        contact_name: String,
        /// Full fingerprint obtained through the independent channel.
        fingerprint: String,
        /// Independently trusted channel used for the comparison.
        channel: ComparisonChannel,
    },
    /// Cancel an unaccepted invitation owned by this Vault.
    Cancel {
        /// Saved exchange ID.
        id: String,
    },
    /// Forget local exchange state without deleting the verified contact.
    Forget {
        /// Saved exchange ID.
        id: String,
    },
    /// Authenticate an archive against a previously saved contact signing key.
    VerifyAuthor {
        /// Saved contact name.
        contact_name: String,
        /// Existing Lockbox archive file to authenticate.
        lockbox_path: String,
    },
}

/// A channel whose peer identity the user has independently established.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonChannel {
    /// Comparison with the person present.
    InPerson,
    /// Call using an independently known phone number.
    KnownPhoneCall,
    /// Independently authenticated messenger conversation.
    TrustedMessenger,
}

/// Public exchange state. Contains no management capability or private key.
#[derive(Clone, Debug, Serialize)]
pub struct ContactExchangeStatus {
    /// Random exchange identifier.
    pub id: String,
    /// Pinned relay origin.
    pub relay: String,
    /// Invitation URL to share with the intended recipient.
    pub invitation: String,
    /// Claimed inviter email; not a server assertion of identity.
    pub inviter_email: String,
    /// Claimed intended recipient email.
    pub recipient_email: String,
    /// Full shared fingerprint, empty until both contributions are available.
    pub fingerprint: String,
    /// Signed expiry in Unix milliseconds.
    pub expires_ms: u64,
    /// Whether both signed contributions are locally pinned.
    pub accepted: bool,
    /// Whether this Vault independently verified and saved the contact.
    pub verified: bool,
    /// Saved contact name, empty before local verification.
    pub contact_name: String,
    /// Whether the last relay operation needs retrying.
    pub relay_pending: bool,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidInput(message.into())
}
fn protocol(error: impl std::fmt::Display) -> Error {
    invalid(error.to_string())
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn bundle(vault: &VaultDirectory, profile: &str) -> Result<Bundle, Error> {
    Ok(Bundle {
        profile: profile.to_owned(),
        email: vault
            .profile_email(profile)?
            .ok_or_else(|| invalid("set the selected profile email before exchanging keys"))?,
        generation: vault.list_profile_generations(profile)?.active_generation,
        encryption_key: vault.load_private_key(profile)?.public_key().to_bytes(),
        signing_key: vault
            .load_owner_signing_key(profile)?
            .public_key()
            .to_bytes(),
    })
}

fn status(local: &LocalExchange, relay_pending: bool) -> Result<ContactExchangeStatus, Error> {
    Ok(ContactExchangeStatus {
        id: local.offer.id.clone(),
        relay: local.server.clone(),
        invitation: format!("{}/exchange#{}", local.server, local.offer.id),
        inviter_email: local.offer.inviter.email.clone(),
        recipient_email: local.offer.recipient_email.clone(),
        fingerprint: local
            .acceptance
            .as_ref()
            .map(|a| verification(&local.offer, a))
            .transpose()
            .map_err(protocol)?
            .unwrap_or_default(),
        expires_ms: local.offer.expires_ms,
        accepted: local.acceptance.is_some(),
        verified: local.verified_contact.is_some(),
        contact_name: local.verified_contact.clone().unwrap_or_default(),
        relay_pending,
    })
}

fn acknowledge(local: &LocalExchange) -> Result<(), Error> {
    ExchangeClient::new(&local.server)
        .map_err(protocol)?
        .request(&Request::Complete {
            id: local.offer.id.clone(),
            token: local.token.clone(),
        })
        .map_err(protocol)?;
    Ok(())
}

fn synchronize(
    path: &Path,
    password: &SecretString,
    local: &mut LocalExchange,
) -> Result<bool, Error> {
    local.offer.validate(now()).map_err(protocol)?;
    let request = if local.inviter {
        Request::Create {
            offer: local.offer.clone(),
            owner_token: local.token.clone(),
        }
    } else {
        Request::Accept {
            id: local.offer.id.clone(),
            recipient_token: local.token.clone(),
            acceptance: local
                .acceptance
                .clone()
                .ok_or_else(|| invalid("missing pinned acceptance"))?,
        }
    };
    let client = ExchangeClient::new(&local.server).map_err(protocol)?;
    let response = match client.request(&request) {
        Ok(response) => response,
        Err(_) => return Ok(false),
    };
    if response.complete
        && response.offer.is_none()
        && response.acceptance.is_none()
        && local.acceptance.is_some()
    {
        return Ok(true);
    }
    if response.offer.as_ref() != Some(&local.offer) {
        return Err(invalid("relay changed the pinned invitation"));
    }
    if let Some(acceptance) = response.acceptance {
        acceptance.validate(&local.offer).map_err(protocol)?;
        if local
            .acceptance
            .as_ref()
            .is_some_and(|old| old != &acceptance)
        {
            return Err(invalid("relay changed the pinned acceptance"));
        }
        local.acceptance = Some(acceptance);
        VaultDirectory::open_file(path, password)?.save_exchange(local)?;
    }
    Ok(true)
}

/// Performs one exchange operation against an existing Vault file.
///
/// All protocol and persistence behavior runs in Rust. Vault handles are dropped
/// before HTTP requests. Applications must perform their independent comparison
/// after this call returns and submit `Verify` only after the user confirms it.
/// Relay failure after durable local preparation returns `relay_pending`, so an
/// offline status or later online list can recover the invitation and retry.
/// The optional Session Agent is never consulted or started.
///
/// # Errors
/// Reports invalid inputs, mismatched pinned state and Vault access failures.
/// Network errors before any local state can be prepared are also returned.
pub fn contact_exchange(
    path: &Path,
    password: &SecretString,
    command: ContactExchangeCommand,
) -> Result<Vec<ContactExchangeStatus>, Error> {
    match command {
        ContactExchangeCommand::Invite {
            relay,
            profile,
            recipient_email,
            ttl_hours,
        } => {
            if !(1..=168).contains(&ttl_hours) {
                return Err(invalid("ttl_hours must be between 1 and 168"));
            }
            let client = ExchangeClient::new(&relay).map_err(protocol)?;
            let vault = VaultDirectory::open_file(path, password)?;
            let created = now();
            let mut offer = Offer {
                version: 1,
                id: random_token().map_err(protocol)?,
                inviter: bundle(&vault, &profile)?,
                recipient_email: revault_publish_protocol::normalize_contact_email(
                    &recipient_email,
                )
                .map_err(protocol)?,
                created_ms: created,
                expires_ms: created + u64::from(ttl_hours) * 3_600_000,
                signature: Vec::new(),
            };
            offer
                .sign(&vault.load_owner_signing_key(&profile)?)
                .map_err(protocol)?;
            let mut local = LocalExchange {
                server: client.origin().to_owned(),
                inviter: true,
                token: random_token().map_err(protocol)?,
                offer,
                acceptance: None,
                verified_contact: None,
            };
            vault.save_exchange(&local)?;
            drop(vault);
            let pending = !synchronize(path, password, &mut local)?;
            Ok(vec![status(&local, pending)?])
        }
        ContactExchangeCommand::Accept {
            invitation,
            profile,
        } => {
            let (origin, id) = invitation
                .rsplit_once("/exchange#")
                .ok_or_else(|| invalid("invalid invitation URL"))?;
            if !revault_publish_protocol::exchange::valid_token(id) {
                return Err(invalid("invalid invitation ID"));
            }
            let client = ExchangeClient::new(origin).map_err(protocol)?;
            let locals = VaultDirectory::open_file(path, password)?.list_exchanges()?;
            let mut local =
                if let Some(local) = locals.into_iter().find(|local| local.offer.id == id) {
                    if local.inviter
                        || local.server != client.origin()
                        || local
                            .acceptance
                            .as_ref()
                            .map(|a| a.recipient.profile.as_str())
                            != Some(profile.as_str())
                    {
                        return Err(invalid("invitation conflicts with pinned local state"));
                    }
                    local
                } else {
                    let offer = client
                        .request(&Request::Inspect { id: id.to_owned() })
                        .map_err(protocol)?
                        .offer
                        .ok_or_else(|| invalid("relay omitted invitation"))?;
                    offer.validate(now()).map_err(protocol)?;
                    if offer.id != id {
                        return Err(invalid("relay substituted invitation"));
                    }
                    let vault = VaultDirectory::open_file(path, password)?;
                    let acceptance = Acceptance::new(
                        &offer,
                        bundle(&vault, &profile)?,
                        &vault.load_owner_signing_key(&profile)?,
                    )
                    .map_err(protocol)?;
                    let local = LocalExchange {
                        server: client.origin().to_owned(),
                        inviter: false,
                        token: random_token().map_err(protocol)?,
                        offer,
                        acceptance: Some(acceptance),
                        verified_contact: None,
                    };
                    vault.save_exchange(&local)?;
                    local
                };
            let pending = !synchronize(path, password, &mut local)?;
            Ok(vec![status(&local, pending)?])
        }
        ContactExchangeCommand::List { offline } => {
            let locals = VaultDirectory::open_file(path, password)?.list_exchanges()?;
            locals
                .into_iter()
                .map(|mut local| {
                    let pending = !offline
                        && if local.verified_contact.is_some() {
                            acknowledge(&local).is_err()
                        } else if now() >= local.offer.expires_ms {
                            // Expired pending records remain inspectable and can still
                            // be verified from their pinned transcript without a relay.
                            false
                        } else {
                            !synchronize(path, password, &mut local)?
                        };
                    status(&local, pending)
                })
                .collect()
        }
        ContactExchangeCommand::Verify {
            id,
            contact_name,
            fingerprint,
            channel: _,
        } => {
            let vault = VaultDirectory::open_file(path, password)?;
            vault.verify_exchange(&id, &contact_name, &fingerprint)?;
            let local = vault.load_exchange(&id)?;
            drop(vault);
            let pending = acknowledge(&local).is_err();
            Ok(vec![status(&local, pending)?])
        }
        ContactExchangeCommand::Cancel { id } => {
            let local = VaultDirectory::open_file(path, password)?.load_exchange(&id)?;
            if !local.inviter {
                return Err(invalid("only the inviter can cancel"));
            }
            ExchangeClient::new(&local.server)
                .map_err(protocol)?
                .request(&Request::Cancel {
                    id: id.clone(),
                    owner_token: local.token,
                })
                .map_err(protocol)?;
            VaultDirectory::open_file(path, password)?.forget_exchange(&id)?;
            Ok(Vec::new())
        }
        ContactExchangeCommand::Forget { id } => {
            VaultDirectory::open_file(path, password)?.forget_exchange(&id)?;
            Ok(Vec::new())
        }
        ContactExchangeCommand::VerifyAuthor {
            contact_name,
            lockbox_path,
        } => {
            let vault = VaultDirectory::open_file(path, password)?;
            let expected = vault.load_contact_signing_key(&contact_name)?;
            let keys = vault
                .list_private_keys()?
                .into_iter()
                .map(|profile| vault.load_private_key(&profile))
                .collect::<Result<Vec<_>, _>>()?;
            drop(vault);
            for key in keys {
                match revault_lockbox_api::Lockbox::open(
                    Path::new(&lockbox_path),
                    revault_lockbox_api::LockboxOpen::ContactKeyPair(key),
                ) {
                    Ok(lockbox) => {
                        if !lockbox.owner_signing_public_key_matches(&expected)? {
                            return Err(invalid(
                                "lockbox signer does not match the selected contact",
                            ));
                        }
                        return Ok(Vec::new());
                    }
                    Err(Error::InvalidKey) => (),
                    Err(error) => return Err(error),
                }
            }
            Err(invalid("no local profile can open this lockbox"))
        }
    }
}
