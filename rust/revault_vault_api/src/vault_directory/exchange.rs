use super::{
    contact_record_path, contact_signing_record_path, set_private_file_permissions,
    validate_record_name, Error, LockboxPath, Result, VaultDirectory, VaultFileLock,
};
use revault_publish_protocol::exchange::{
    decode, encode, valid_token, verification, verification_matches, LocalExchange,
};

fn exchange_path(id: &str) -> Result<LockboxPath> {
    if !valid_token(id) {
        return Err(Error::InvalidInput("invalid exchange ID".to_owned()));
    }
    LockboxPath::new(format!("/exchanges/{id}.json"))
}

fn protocol_error(error: impl std::fmt::Display) -> Error {
    Error::InvalidInput(error.to_string())
}

impl VaultDirectory {
    /// Saves private invitation state inside the encrypted vault.
    ///
    /// # Errors
    /// Rejects malformed or changed pinned state and reports vault storage errors.
    pub fn save_exchange(&self, exchange: &LocalExchange) -> Result<()> {
        if !valid_token(&exchange.token) {
            return Err(Error::InvalidInput(
                "invalid exchange capability".to_owned(),
            ));
        }
        exchange
            .offer
            .validate(exchange.offer.created_ms)
            .map_err(protocol_error)?;
        if let Some(acceptance) = &exchange.acceptance {
            acceptance
                .validate(&exchange.offer)
                .map_err(protocol_error)?;
        }
        let path = exchange_path(&exchange.offer.id)?;
        if self.lockbox.borrow().stat(&path).is_some() {
            let previous = self.load_exchange(&exchange.offer.id)?;
            if previous.offer != exchange.offer
                || previous.server != exchange.server
                || previous.token != exchange.token
                || previous.inviter != exchange.inviter
                || previous
                    .acceptance
                    .as_ref()
                    .is_some_and(|a| Some(a) != exchange.acceptance.as_ref())
                || previous.verified_contact != exchange.verified_contact
            {
                return Err(Error::InvalidInput(
                    "pinned exchange state cannot change".to_owned(),
                ));
            }
        } else if exchange.verified_contact.is_some() {
            return Err(Error::InvalidInput(
                "new exchange cannot already be verified".to_owned(),
            ));
        }
        self.put_record_replace(&path, &encode(exchange).map_err(protocol_error)?)
    }

    /// Loads a private invitation. Callers must not print its capability.
    ///
    /// # Errors
    /// Reports missing, malformed or unreadable local invitation state.
    pub fn load_exchange(&self, id: &str) -> Result<LocalExchange> {
        decode(&self.get_record(&exchange_path(id)?)?).map_err(protocol_error)
    }

    /// Lists local invitations, including those awaiting verification.
    ///
    /// # Errors
    /// Reports malformed records or vault read errors.
    pub fn list_exchanges(&self) -> Result<Vec<LocalExchange>> {
        self.list_record_names("/exchanges", ".json")?
            .into_iter()
            .map(|id| self.load_exchange(&id))
            .collect()
    }

    /// Removes local pending state without removing a previously verified contact.
    ///
    /// # Errors
    /// Reports invalid identifiers or vault mutation errors.
    pub fn forget_exchange(&self, id: &str) -> Result<()> {
        self.delete_record_if_exists(&exchange_path(id)?)
    }

    /// Atomically saves both verified keys, transcript and local verification state.
    /// Existing different contacts are refused; replacement needs explicit removal.
    ///
    /// # Errors
    /// Rejects missing/invalid replies, a mismatched fingerprint, or conflicting
    /// contact keys. Also reports vault storage errors.
    pub fn verify_exchange(&self, id: &str, name: &str, compared: &str) -> Result<()> {
        let mut exchange = self.load_exchange(id)?;
        let acceptance = exchange
            .acceptance
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("exchange has no reply".to_owned()))?;
        exchange
            .offer
            .validate(exchange.offer.created_ms)
            .map_err(protocol_error)?;
        acceptance
            .validate(&exchange.offer)
            .map_err(protocol_error)?;
        if !verification_matches(
            &verification(&exchange.offer, acceptance).map_err(protocol_error)?,
            compared,
        ) {
            return Err(Error::InvalidInput(
                "shared verification mismatch; contact was not saved".to_owned(),
            ));
        }
        if exchange
            .verified_contact
            .as_deref()
            .is_some_and(|previous| previous != name)
        {
            return Err(Error::InvalidInput(
                "exchange already verified under another name".to_owned(),
            ));
        }
        let bundle = if exchange.inviter {
            &acceptance.recipient
        } else {
            &exchange.offer.inviter
        };
        let public_path = contact_record_path(name)?;
        let signing_path = contact_signing_record_path(name)?;
        let metadata_path = LockboxPath::new(format!(
            "/contacts/{}.exchange.json",
            validate_record_name(name)?
        ))?;
        let metadata = encode(&(&exchange.offer, acceptance)).map_err(protocol_error)?;
        let encryption = bundle.encryption_key.clone();
        let signing = bundle.signing_key.clone();
        exchange.verified_contact = Some(name.to_owned());
        let state = encode(&exchange).map_err(protocol_error)?;
        let _guard = VaultFileLock::acquire(&self.path)?;
        let mut lockbox = self.lockbox.borrow_mut();
        if lockbox.stat(&public_path).is_some()
            && (lockbox.get_file(&public_path)? != encryption
                || lockbox.get_file(&signing_path)? != signing)
        {
            return Err(Error::InvalidInput("contact already exists with different keys; remove it explicitly before replacement".to_owned()));
        }
        for (path, bytes) in [
            (public_path, encryption),
            (signing_path, signing),
            (metadata_path, metadata),
            (exchange_path(id)?, state),
        ] {
            let replace = lockbox.stat(&path).is_some();
            lockbox.create_parent_dirs_for(&path)?;
            lockbox.add_file(&path, &bytes, replace)?;
        }
        lockbox.commit()?;
        set_private_file_permissions(&self.path)?;
        Ok(())
    }
}
